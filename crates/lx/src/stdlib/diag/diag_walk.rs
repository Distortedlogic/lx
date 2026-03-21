#[path = "diag_types.rs"]
mod diag_types;

#[path = "diag_walk_expr.rs"]
mod diag_walk_expr;

#[path = "diag_helpers.rs"]
mod diag_helpers;

use std::collections::HashMap;

use std::collections::HashSet;

use crate::ast::{BindTarget, Binding, Expr, McpToolDecl, Program, SStmt, Stmt, UseStmt};
use crate::span::Span;
use crate::visitor::{AgentDeclCtx, AstVisitor, TraitDeclCtx, walk_program};

pub(crate) use diag_types::*;

use diag_helpers::{
    extract_agent_spawn, extract_field_call_parts, extract_mcp_connect, is_resource_create,
    is_resource_module, unwrap_propagate,
};

pub(crate) struct Walker {
    pub nodes: Vec<DiagNode>,
    pub edges: Vec<DiagEdge>,
    pub(super) next_id: usize,
    pub(super) agent_vars: HashMap<String, String>,
    pub(super) mcp_vars: HashMap<String, String>,
    pub(super) fn_nodes: HashMap<String, String>,
    pub(super) handler_maps: HashMap<String, Vec<String>>,
    pub(super) resource_vars: HashMap<String, String>,
    pub(super) imported_modules: HashSet<String>,
    pub(super) context: String,
    current_fn: Option<String>,
    subgraph_nodes: HashMap<String, Vec<String>>,
}

impl Walker {
    pub fn new() -> Self {
        let main = DiagNode {
            id: "main".into(),
            label: "main".into(),
            kind: "agent".into(),
            children: vec![],
            source_offset: None,
        };
        Self {
            nodes: vec![main],
            edges: vec![],
            next_id: 1,
            agent_vars: HashMap::new(),
            mcp_vars: HashMap::new(),
            fn_nodes: HashMap::new(),
            handler_maps: HashMap::new(),
            resource_vars: HashMap::new(),
            imported_modules: HashSet::new(),
            context: "main".into(),
            current_fn: Some("main".into()),
            subgraph_nodes: HashMap::from([("main".into(), vec!["main".into()])]),
        }
    }

    pub(super) fn add_node(&mut self, prefix: &str, label: String, kind: &str) -> String {
        self.add_node_at(prefix, label, kind, None)
    }

    pub(super) fn add_node_at(
        &mut self,
        prefix: &str,
        label: String,
        kind: &str,
        span: Option<Span>,
    ) -> String {
        let id = format!("{prefix}_{}", self.next_id);
        self.next_id += 1;
        self.nodes.push(DiagNode {
            id: id.clone(),
            label,
            kind: kind.into(),
            children: vec![],
            source_offset: span.map(|s| s.offset),
        });
        if let Some(ref fn_name) = self.current_fn {
            self.subgraph_nodes
                .entry(fn_name.clone())
                .or_default()
                .push(id.clone());
        }
        id
    }

    pub(super) fn add_edge(&mut self, from: &str, to: &str, label: String, style: &str) {
        self.add_edge_typed(from, to, label, style, "exec");
    }

    pub(super) fn add_edge_typed(
        &mut self,
        from: &str,
        to: &str,
        label: String,
        style: &str,
        edge_type: &str,
    ) {
        self.edges.push(DiagEdge {
            from: from.into(),
            to: to.into(),
            label,
            style: style.into(),
            edge_type: edge_type.into(),
        });
    }

    pub(super) fn walk_stmts(&mut self, stmts: &[SStmt]) {
        for stmt in stmts {
            self.visit_stmt(&stmt.node, stmt.span);
        }
    }

    pub fn into_graph(self) -> Graph {
        let subgraphs = self
            .subgraph_nodes
            .into_iter()
            .map(|(label, node_ids)| Subgraph { label, node_ids })
            .collect();
        Graph {
            nodes: self.nodes,
            edges: self.edges,
            subgraphs,
        }
    }
}

impl AstVisitor for Walker {
    fn visit_program(&mut self, program: &Program) {
        let saved_fn = self.current_fn.take();
        for stmt in &program.stmts {
            if let Stmt::Binding(b) = &stmt.node
                && let BindTarget::Name(name) | BindTarget::Reassign(name) = &b.target
                && let Expr::Func { .. } = &b.value.node
                && name != "main"
                && !self.fn_nodes.contains_key(name)
            {
                let id = self.add_node("agent", name.clone(), "agent");
                self.fn_nodes.insert(name.clone(), id);
            }
        }
        self.current_fn = saved_fn;
        walk_program(self, program);
    }

    fn visit_binding(&mut self, binding: &Binding, _span: Span) {
        let name = match &binding.target {
            BindTarget::Name(n) | BindTarget::Reassign(n) => Some(n.clone()),
            _ => None,
        };
        if let Some(ref var_name) = name {
            if let Some(spawn_label) = extract_agent_spawn(&binding.value) {
                let id = self.add_node("agent", spawn_label, "agent");
                self.agent_vars.insert(var_name.clone(), id);
                return;
            }
            if let Some(mcp_label) = extract_mcp_connect(&binding.value) {
                let id = self.add_node("tool", mcp_label, "tool");
                self.mcp_vars.insert(var_name.clone(), id);
                return;
            }
            if let Expr::Func { body, .. } = &binding.value.node
                && var_name != "main"
            {
                let id = if let Some(existing) = self.fn_nodes.get(var_name) {
                    existing.clone()
                } else {
                    let id = self.add_node("agent", var_name.clone(), "agent");
                    self.fn_nodes.insert(var_name.clone(), id.clone());
                    id
                };
                let saved_ctx = self.context.clone();
                let saved_fn = self.current_fn.clone();
                self.context = id.clone();
                self.current_fn = Some(var_name.clone());
                self.subgraph_nodes
                    .entry(var_name.clone())
                    .or_default()
                    .push(id);
                self.visit_expr(&body.node, body.span);
                self.context = saved_ctx;
                self.current_fn = saved_fn;
                return;
            }
            let inner = unwrap_propagate(&binding.value.node);
            if let Expr::Apply { func, .. } = inner
                && let Some((module, method)) = extract_field_call_parts(&func.node)
                && is_resource_create(method)
                && is_resource_module(module)
            {
                let id = self.add_node("resource", module.to_string(), "resource");
                let ctx = self.context.clone();
                self.add_edge(&ctx, &id, "create".into(), "solid");
                self.resource_vars.insert(var_name.clone(), id);
            }
            if let Expr::Map(entries) = &binding.value.node {
                let idents: Vec<String> = entries
                    .iter()
                    .filter_map(|e| {
                        if let Expr::Ident(n) = &e.value.node {
                            Some(n.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                if !idents.is_empty() {
                    self.handler_maps.insert(var_name.clone(), idents);
                }
            }
        }
        self.visit_expr(&binding.value.node, binding.value.span);
    }

    fn visit_use(&mut self, stmt: &UseStmt, _span: Span) {
        let is_base_std = stmt.path.first().is_some_and(|p| p == "std") && stmt.path.len() == 2;
        if !is_base_std && let Some(last) = stmt.path.last() {
            self.imported_modules.insert(last.clone());
        }
    }

    fn visit_type_def(
        &mut self,
        name: &str,
        _variants: &[(String, usize)],
        _exported: bool,
        _span: Span,
    ) {
        self.add_node("type", name.to_string(), "type");
    }

    fn visit_trait_decl(&mut self, ctx: &TraitDeclCtx<'_>, _span: Span) {
        self.add_node("type", ctx.name.to_string(), "type");
    }

    fn visit_mcp_decl(&mut self, name: &str, _tools: &[McpToolDecl], _exported: bool, _span: Span) {
        let id = self.add_node("tool", name.to_string(), "tool");
        self.mcp_vars.insert(name.to_string(), id);
    }

    fn visit_agent_decl(&mut self, ctx: &AgentDeclCtx<'_>, _span: Span) {
        let id = self.add_node("agent", ctx.name.to_string(), "agent");
        self.agent_vars.insert(ctx.name.to_string(), id.clone());
        let saved = self.context.clone();
        self.context = id;
        if let Some(i) = ctx.init {
            self.visit_expr(&i.node, i.span);
        }
        if let Some(o) = ctx.on {
            self.visit_expr(&o.node, o.span);
        }
        for m in ctx.methods {
            self.visit_expr(&m.handler.node, m.handler.span);
        }
        self.context = saved;
    }

    fn visit_expr(&mut self, expr: &Expr, span: Span) {
        diag_walk_expr::visit_expr_diag(self, expr, span);
    }
}
