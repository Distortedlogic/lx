#[path = "diag_walk_expr.rs"]
mod diag_walk_expr;

use std::collections::HashMap;

use crate::ast::{BindTarget, Program, SStmt, Stmt};

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

    pub fn walk_program(&mut self, program: &Program) {
        for stmt in &program.stmts {
            self.walk_stmt(&stmt.node);
        }
    }

    pub(super) fn walk_stmts(&mut self, stmts: &[SStmt]) {
        for stmt in stmts {
            self.walk_stmt(&stmt.node);
        }
    }

    fn walk_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Binding(binding) => {
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
                self.walk_expr(&binding.value.node);
            }
            Stmt::McpDecl { name, .. } => {
                let id = self.add_node("tool", name.clone(), "tool");
                self.mcp_vars.insert(name.clone(), id);
            }
            Stmt::AgentDecl { name, .. } => {
                let id = self.add_node("agent", name.clone(), "agent");
                self.agent_vars.insert(name.clone(), id);
            }
            Stmt::Expr(sexpr) => self.walk_expr(&sexpr.node),
            _ => {}
        }
    }

    pub fn into_graph(self) -> Graph {
        Graph {
            nodes: self.nodes,
            edges: self.edges,
        }
    }
}
