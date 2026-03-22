#[path = "diag_types.rs"]
mod diag_types;

#[path = "diag_walk_expr.rs"]
mod diag_walk_expr;

#[path = "diag_helpers.rs"]
mod diag_helpers;

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::ControlFlow;

use crate::ast::{BindTarget, Binding, Expr, ExprApply, ExprFunc, MatchArm, Program, SExpr, SStmt, SelArm, Stmt, TraitDeclData, UseStmt};
use crate::sym::Sym;
use crate::visitor::{AstVisitor, walk_loop, walk_par, walk_program};
use miette::SourceSpan;

pub(crate) use diag_types::*;

use diag_helpers::{extract_field_call_parts, is_resource_create, is_resource_module, unwrap_propagate};

pub(crate) struct Walker {
  pub nodes: Vec<DiagNode>,
  pub edges: Vec<DiagEdge>,
  pub(super) next_id: usize,
  pub(super) fn_nodes: HashMap<Sym, String>,
  pub(super) handler_maps: HashMap<Sym, Vec<Sym>>,
  pub(super) resource_vars: HashMap<Sym, String>,
  pub(super) imported_modules: HashSet<Sym>,
  pub(super) context: String,
  pub(super) context_stack: Vec<String>,
  current_fn: Option<Sym>,
  subgraph_nodes: HashMap<String, Vec<String>>,
}

impl Walker {
  pub fn new() -> Self {
    let main = DiagNode { id: "main".into(), label: "main".into(), kind: NodeKind::Agent, children: vec![], source_offset: None };
    Self {
      nodes: vec![main],
      edges: vec![],
      next_id: 1,
      fn_nodes: HashMap::new(),
      handler_maps: HashMap::new(),
      resource_vars: HashMap::new(),
      imported_modules: HashSet::new(),
      context: "main".into(),
      context_stack: Vec::new(),
      current_fn: Some(crate::sym::intern("main")),
      subgraph_nodes: HashMap::from([("main".into(), vec!["main".into()])]),
    }
  }

  pub(super) fn add_node(&mut self, prefix: &str, label: String, kind: NodeKind) -> String {
    self.add_node_at(prefix, label, kind, None)
  }

  pub(super) fn add_node_at(&mut self, prefix: &str, label: String, kind: NodeKind, span: Option<SourceSpan>) -> String {
    let id = format!("{prefix}_{}", self.next_id);
    self.next_id += 1;
    self.nodes.push(DiagNode { id: id.clone(), label, kind, children: vec![], source_offset: span.map(|s| s.offset() as u32) });
    if let Some(ref fn_name) = self.current_fn {
      self.subgraph_nodes.entry(fn_name.to_string()).or_default().push(id.clone());
    }
    id
  }

  pub(super) fn add_edge(&mut self, from: &str, to: &str, label: String, style: EdgeStyle) {
    self.add_edge_typed(from, to, label, style, EdgeType::Exec);
  }

  pub(super) fn add_edge_typed(&mut self, from: &str, to: &str, label: String, style: EdgeStyle, edge_type: EdgeType) {
    self.edges.push(DiagEdge { from: from.into(), to: to.into(), label, style, edge_type });
  }

  pub fn into_graph(self) -> Graph {
    let subgraphs = self.subgraph_nodes.into_iter().map(|(label, node_ids)| Subgraph { label, node_ids }).collect();
    Graph { nodes: self.nodes, edges: self.edges, subgraphs }
  }
}

impl AstVisitor for Walker {
  fn visit_program(&mut self, program: &Program) -> ControlFlow<()> {
    let saved_fn = self.current_fn.take();
    for stmt in &program.stmts {
      if let Stmt::Binding(b) = &stmt.node
        && let BindTarget::Name(name) | BindTarget::Reassign(name) = &b.target
        && let Expr::Func(_) = &b.value.node
        && *name != "main"
        && !self.fn_nodes.contains_key(name)
      {
        let id = self.add_node("agent", name.to_string(), NodeKind::Agent);
        self.fn_nodes.insert(*name, id);
      }
    }
    self.current_fn = saved_fn;
    walk_program(self, program)
  }

  fn visit_binding(&mut self, binding: &Binding, _span: SourceSpan) -> ControlFlow<()> {
    let name = match &binding.target {
      BindTarget::Name(n) | BindTarget::Reassign(n) => Some(*n),
      _ => None,
    };
    if let Some(var_name) = name {
      if let Expr::Func(ExprFunc { body, .. }) = &binding.value.node
        && var_name != "main"
      {
        let id = if let Some(existing) = self.fn_nodes.get(&var_name) {
          existing.clone()
        } else {
          let id = self.add_node("agent", var_name.to_string(), NodeKind::Agent);
          self.fn_nodes.insert(var_name, id.clone());
          id
        };
        let saved_ctx = self.context.clone();
        let saved_fn = self.current_fn;
        self.context = id.clone();
        self.current_fn = Some(var_name);
        self.subgraph_nodes.entry(var_name.to_string()).or_default().push(id);
        self.visit_expr(&body.node, body.span)?;
        self.context = saved_ctx;
        self.current_fn = saved_fn;
        return ControlFlow::Continue(());
      }
      let inner = unwrap_propagate(&binding.value.node);
      if let Expr::Apply(ExprApply { func, .. }) = inner
        && let Some((module, method)) = extract_field_call_parts(&func.node)
        && is_resource_create(method)
        && is_resource_module(module)
      {
        let id = self.add_node("resource", module.to_string(), NodeKind::Resource);
        let ctx = self.context.clone();
        self.add_edge(&ctx, &id, "create".into(), EdgeStyle::Solid);
        self.resource_vars.insert(var_name, id);
      }
      if let Expr::Map(entries) = &binding.value.node {
        let idents: Vec<Sym> = entries.iter().filter_map(|e| if let Expr::Ident(n) = &e.value.node { Some(*n) } else { None }).collect();
        if !idents.is_empty() {
          self.handler_maps.insert(var_name, idents);
        }
      }
    }
    self.visit_expr(&binding.value.node, binding.value.span)
  }

  fn visit_use(&mut self, stmt: &UseStmt, _span: SourceSpan) -> ControlFlow<()> {
    let is_base_std = stmt.path.first().is_some_and(|p| *p == "std") && stmt.path.len() == 2;
    if !is_base_std && let Some(last) = stmt.path.last() {
      self.imported_modules.insert(*last);
    }
    ControlFlow::Continue(())
  }

  fn visit_type_def(&mut self, name: Sym, _variants: &[(Sym, usize)], _exported: bool, _span: SourceSpan) -> ControlFlow<()> {
    self.add_node("type", name.to_string(), NodeKind::Type);
    ControlFlow::Continue(())
  }

  fn visit_trait_decl(&mut self, data: &TraitDeclData, _span: SourceSpan) -> ControlFlow<()> {
    self.add_node("type", data.name.to_string(), NodeKind::Type);
    ControlFlow::Continue(())
  }

  fn visit_par(&mut self, stmts: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
    let fork_id = self.add_node_at("fork", "par".into(), NodeKind::Fork, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &fork_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = fork_id;
    walk_par(self, stmts, span)
  }

  fn leave_par(&mut self, _stmts: &[SStmt], _span: SourceSpan) -> ControlFlow<()> {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
    ControlFlow::Continue(())
  }

  fn visit_sel(&mut self, arms: &[SelArm], span: SourceSpan) -> ControlFlow<()> {
    let dec_id = self.add_node_at("sel", "sel".into(), NodeKind::Decision, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &dec_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = dec_id;
    for arm in arms {
      self.visit_expr(&arm.handler.node, arm.handler.span)?;
    }
    self.leave_sel(arms, span)
  }

  fn leave_sel(&mut self, _arms: &[SelArm], _span: SourceSpan) -> ControlFlow<()> {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
    ControlFlow::Continue(())
  }

  fn visit_match(&mut self, scrutinee: &SExpr, arms: &[MatchArm], span: SourceSpan) -> ControlFlow<()> {
    let label = format!("{}?", diag_helpers::expr_label(&scrutinee.node));
    let dec_id = self.add_node_at("match", label, NodeKind::Decision, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &dec_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = dec_id;
    for arm in arms {
      self.visit_expr(&arm.body.node, arm.body.span)?;
    }
    self.leave_match(scrutinee, arms, span)
  }

  fn leave_match(&mut self, _scrutinee: &SExpr, _arms: &[MatchArm], _span: SourceSpan) -> ControlFlow<()> {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
    ControlFlow::Continue(())
  }

  fn visit_ternary(&mut self, cond: &SExpr, then_: &SExpr, else_: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
    let label = format!("{}?", diag_helpers::expr_label(&cond.node));
    let dec_id = self.add_node_at("cond", label, NodeKind::Decision, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &dec_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = dec_id.clone();
    self.visit_expr(&then_.node, then_.span)?;
    if let Some(e) = else_ {
      self.context = dec_id;
      self.visit_expr(&e.node, e.span)?;
    }
    self.leave_ternary(cond, then_, else_, span)
  }

  fn leave_ternary(&mut self, _cond: &SExpr, _then_: &SExpr, _else_: Option<&SExpr>, _span: SourceSpan) -> ControlFlow<()> {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
    ControlFlow::Continue(())
  }

  fn visit_loop(&mut self, stmts: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
    let loop_id = self.add_node_at("loop", "loop".into(), NodeKind::Loop, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &loop_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = loop_id;
    walk_loop(self, stmts, span)
  }

  fn leave_loop(&mut self, _stmts: &[SStmt], _span: SourceSpan) -> ControlFlow<()> {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
    ControlFlow::Continue(())
  }

  fn visit_expr(&mut self, expr: &Expr, span: SourceSpan) -> ControlFlow<()> {
    diag_walk_expr::visit_expr_diag(self, expr, span)
  }
}
