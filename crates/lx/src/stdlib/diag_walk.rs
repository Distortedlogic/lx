#[path = "diag_walk_expr.rs"]
mod diag_walk_expr;

use std::collections::HashMap;

use crate::ast::{BindTarget, Binding, Expr, McpToolDecl, SExpr, SStmt};
use crate::span::Span;
use crate::visitor::AstVisitor;

use diag_walk_expr::{extract_agent_spawn, extract_mcp_connect};

pub(crate) struct DiagNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub children: Vec<DiagNode>,
}

pub(crate) struct DiagEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub style: String,
}

pub(crate) struct Graph {
    pub nodes: Vec<DiagNode>,
    pub edges: Vec<DiagEdge>,
}

pub(crate) struct Walker {
    pub nodes: Vec<DiagNode>,
    pub edges: Vec<DiagEdge>,
    pub(super) next_id: usize,
    pub(super) agent_vars: HashMap<String, String>,
    pub(super) mcp_vars: HashMap<String, String>,
    pub(super) context: String,
}

impl Walker {
    pub fn new() -> Self {
        let main = DiagNode {
            id: "main".into(),
            label: "main".into(),
            kind: "agent".into(),
            children: vec![],
        };
        Self {
            nodes: vec![main],
            edges: vec![],
            next_id: 1,
            agent_vars: HashMap::new(),
            mcp_vars: HashMap::new(),
            context: "main".into(),
        }
    }

    pub(super) fn add_node(&mut self, prefix: &str, label: String, kind: &str) -> String {
        let id = format!("{prefix}_{}", self.next_id);
        self.next_id += 1;
        self.nodes.push(DiagNode {
            id: id.clone(),
            label,
            kind: kind.into(),
            children: vec![],
        });
        id
    }

    pub(super) fn add_edge(&mut self, from: &str, to: &str, label: String, style: &str) {
        self.edges.push(DiagEdge {
            from: from.into(),
            to: to.into(),
            label,
            style: style.into(),
        });
    }

    pub(super) fn walk_stmts(&mut self, stmts: &[SStmt]) {
        for stmt in stmts {
            self.visit_stmt(&stmt.node, stmt.span);
        }
    }

    pub fn into_graph(self) -> Graph {
        Graph {
            nodes: self.nodes,
            edges: self.edges,
        }
    }
}

impl AstVisitor for Walker {
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
        }
        self.visit_expr(&binding.value.node, binding.value.span);
    }

    fn visit_mcp_decl(&mut self, name: &str, _tools: &[McpToolDecl], _exported: bool, _span: Span) {
        let id = self.add_node("tool", name.to_string(), "tool");
        self.mcp_vars.insert(name.to_string(), id);
    }

    fn visit_agent_decl(
        &mut self,
        name: &str,
        _traits: &[String],
        _uses: &[(String, String)],
        _init: Option<&SExpr>,
        _on: Option<&SExpr>,
        _methods: &[crate::ast::AgentMethod],
        _exported: bool,
        _span: Span,
    ) {
        let id = self.add_node("agent", name.to_string(), "agent");
        self.agent_vars.insert(name.to_string(), id);
    }

    fn visit_expr(&mut self, expr: &Expr, span: Span) {
        diag_walk_expr::visit_expr_diag(self, expr, span);
    }
}
