#[path = "diag_types.rs"]
mod diag_types;

#[path = "diag_walk_expr.rs"]
mod diag_walk_expr;

#[path = "diag_helpers.rs"]
mod diag_helpers;

use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;

use crate::ast::{
  AstArena, BindTarget, Binding, Expr, ExprApply, ExprFunc, ExprId, ExprMatch, ExprTernary, MapEntry, Program, SelArm, Stmt, StmtId, StmtTypeDef,
  TraitDeclData, UseStmt,
};
use crate::sym::{Sym, intern};
use crate::visitor::{AstVisitor, PatternVisitor, TypeVisitor, VisitAction, dispatch_expr, walk_loop, walk_par};
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
  arena: *const AstArena,
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
      current_fn: Some(intern("main")),
      subgraph_nodes: HashMap::from([("main".into(), vec!["main".into()])]),
      arena: std::ptr::null(),
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

impl PatternVisitor for Walker {}
impl TypeVisitor for Walker {}
impl AstVisitor for Walker {
  fn leave_par(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan) {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
  }

  fn leave_sel(&mut self, _id: ExprId, _arms: &[SelArm], _span: SourceSpan) {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
  }

  fn leave_match(&mut self, _id: ExprId, _m: &ExprMatch, _span: SourceSpan) {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
  }

  fn leave_ternary(&mut self, _id: ExprId, _ternary: &ExprTernary, _span: SourceSpan) {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
  }

  fn leave_loop(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan) {
    self.context = self.context_stack.pop().expect("diag: context_stack underflow");
  }

  fn visit_program<P>(&mut self, program: &Program<P>) -> VisitAction {
    self.arena = &program.arena as *const AstArena;
    let arena = unsafe { &*self.arena };
    let saved_fn = self.current_fn.take();
    for &sid in &program.stmts {
      let stmt = arena.stmt(sid);
      if let Stmt::Binding(b) = stmt
        && let BindTarget::Name(name) | BindTarget::Reassign(name) = &b.target
        && matches!(arena.expr(b.value), Expr::Func(_))
        && *name != "main"
        && !self.fn_nodes.contains_key(name)
      {
        let id = self.add_node("agent", name.to_string(), NodeKind::Agent);
        self.fn_nodes.insert(*name, id);
      }
    }
    self.current_fn = saved_fn;
    VisitAction::Descend
  }

  fn visit_binding(&mut self, _id: StmtId, binding: &Binding, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arena = unsafe { &*self.arena };
    let name = match &binding.target {
      BindTarget::Name(n) | BindTarget::Reassign(n) => Some(*n),
      _ => None,
    };
    if let Some(var_name) = name {
      if let Expr::Func(ExprFunc { body, .. }) = arena.expr(binding.value)
        && var_name != "main"
      {
        let body = *body;
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
        if dispatch_expr(self, body, arena).is_break() {
          self.context = saved_ctx;
          self.current_fn = saved_fn;
          return VisitAction::Stop;
        }
        self.context = saved_ctx;
        self.current_fn = saved_fn;
        return VisitAction::Skip;
      }
      let inner = unwrap_propagate(arena.expr(binding.value), arena);
      if let Expr::Apply(ExprApply { func, .. }) = inner
        && let Some((module, method)) = extract_field_call_parts(arena.expr(*func), arena)
        && is_resource_create(method)
        && is_resource_module(module)
      {
        let id = self.add_node("resource", module.to_string(), NodeKind::Resource);
        let ctx = self.context.clone();
        self.add_edge(&ctx, &id, "create".into(), EdgeStyle::Solid);
        self.resource_vars.insert(var_name, id);
      }
      if let Expr::Map(entries) = arena.expr(binding.value) {
        let idents: Vec<Sym> = entries
          .iter()
          .filter_map(|e| {
            let val = match e {
              MapEntry::Keyed { value, .. } | MapEntry::Spread(value) => *value,
            };
            if let Expr::Ident(n) = arena.expr(val) { Some(*n) } else { None }
          })
          .collect();
        if !idents.is_empty() {
          self.handler_maps.insert(var_name, idents);
        }
      }
    }
    if dispatch_expr(self, binding.value, arena).is_break() {
      return VisitAction::Stop;
    }
    VisitAction::Skip
  }

  fn visit_use(&mut self, _id: StmtId, stmt: &UseStmt, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let is_base_std = stmt.path.first().is_some_and(|p| *p == "std") && stmt.path.len() == 2;
    if !is_base_std && let Some(last) = stmt.path.last() {
      self.imported_modules.insert(*last);
    }
    VisitAction::Descend
  }

  fn visit_type_def(&mut self, _id: StmtId, def: &StmtTypeDef, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.add_node("type", def.name.to_string(), NodeKind::Type);
    VisitAction::Descend
  }

  fn visit_trait_decl(&mut self, _id: StmtId, data: &TraitDeclData, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.add_node("type", data.name.to_string(), NodeKind::Type);
    VisitAction::Skip
  }

  fn visit_par(&mut self, _id: ExprId, stmts: &[StmtId], span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arena = unsafe { &*self.arena };
    let fork_id = self.add_node_at("fork", "par".into(), NodeKind::Fork, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &fork_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = fork_id;
    if walk_par(self, _id, stmts, span, arena).is_break() {
      return VisitAction::Stop;
    }
    VisitAction::Skip
  }

  fn visit_sel(&mut self, _id: ExprId, arms: &[SelArm], span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arena = unsafe { &*self.arena };
    let dec_id = self.add_node_at("sel", "sel".into(), NodeKind::Decision, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &dec_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = dec_id;
    for arm in arms {
      if dispatch_expr(self, arm.handler, arena).is_break() {
        return VisitAction::Stop;
      }
    }
    VisitAction::Skip
  }

  fn visit_match(&mut self, _id: ExprId, m: &ExprMatch, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arena = unsafe { &*self.arena };
    let label = format!("{}?", diag_helpers::expr_label(arena.expr(m.scrutinee), arena));
    let dec_id = self.add_node_at("match", label, NodeKind::Decision, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &dec_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = dec_id;
    for arm in &m.arms {
      if dispatch_expr(self, arm.body, arena).is_break() {
        return VisitAction::Stop;
      }
    }
    VisitAction::Skip
  }

  fn visit_ternary(&mut self, _id: ExprId, ternary: &ExprTernary, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arena = unsafe { &*self.arena };
    let label = format!("{}?", diag_helpers::expr_label(arena.expr(ternary.cond), arena));
    let dec_id = self.add_node_at("cond", label, NodeKind::Decision, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &dec_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = dec_id.clone();
    if dispatch_expr(self, ternary.then_, arena).is_break() {
      return VisitAction::Stop;
    }
    if let Some(e) = ternary.else_ {
      self.context = dec_id;
      if dispatch_expr(self, e, arena).is_break() {
        return VisitAction::Stop;
      }
    }
    VisitAction::Skip
  }

  fn visit_loop(&mut self, _id: ExprId, stmts: &[StmtId], span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arena = unsafe { &*self.arena };
    let loop_id = self.add_node_at("loop", "loop".into(), NodeKind::Loop, Some(span));
    let ctx = self.context.clone();
    self.add_edge(&ctx, &loop_id, String::new(), EdgeStyle::Solid);
    self.context_stack.push(self.context.clone());
    self.context = loop_id;
    if walk_loop(self, _id, stmts, span, arena).is_break() {
      return VisitAction::Stop;
    }
    VisitAction::Skip
  }

  fn visit_apply(&mut self, _id: ExprId, apply: &ExprApply, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arena = unsafe { &*self.arena };
    match diag_walk_expr::visit_apply_diag(self, apply, span, arena) {
      ControlFlow::Continue(()) => VisitAction::Skip,
      ControlFlow::Break(()) => VisitAction::Stop,
    }
  }
}
