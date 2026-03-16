use std::collections::HashMap;

use crate::ast::{BindTarget, Expr, FieldKind, Literal, Program, SExpr, SStmt, Stmt, StrPart};

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
    next_id: usize,
    agent_vars: HashMap<String, String>,
    mcp_vars: HashMap<String, String>,
    context: String,
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

    fn add_node(&mut self, prefix: &str, label: String, kind: &str) -> String {
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

    fn add_edge(&mut self, from: &str, to: &str, label: String, style: &str) {
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

    fn walk_stmts(&mut self, stmts: &[SStmt]) {
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
            Stmt::Expr(sexpr) => self.walk_expr(&sexpr.node),
            _ => {}
        }
    }

    fn walk_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::AgentSend { target, msg } => {
                let to = self.resolve_target(&target.node);
                self.add_edge(
                    &self.context.clone(),
                    &to,
                    extract_msg_label(&msg.node),
                    "dashed",
                );
            }
            Expr::AgentAsk { target, msg } => {
                let to = self.resolve_target(&target.node);
                self.add_edge(
                    &self.context.clone(),
                    &to,
                    extract_msg_label(&msg.node),
                    "solid",
                );
            }
            Expr::Par(stmts) => {
                let fork_id = self.add_node("fork", "par".into(), "fork");
                self.add_edge(&self.context.clone(), &fork_id, String::new(), "solid");
                let saved = self.context.clone();
                self.context = fork_id;
                self.walk_stmts(stmts);
                self.context = saved;
            }
            Expr::Sel(arms) => {
                let dec_id = self.add_node("sel", "sel".into(), "decision");
                self.add_edge(&self.context.clone(), &dec_id, String::new(), "solid");
                let saved = self.context.clone();
                self.context = dec_id;
                for arm in arms {
                    self.walk_expr(&arm.handler.node);
                }
                self.context = saved;
            }
            Expr::Match { scrutinee, arms } => {
                let label = format!("{}?", expr_label(&scrutinee.node));
                let dec_id = self.add_node("match", label, "decision");
                self.add_edge(&self.context.clone(), &dec_id, String::new(), "solid");
                let saved = self.context.clone();
                self.context = dec_id;
                for arm in arms {
                    self.walk_expr(&arm.body.node);
                }
                self.context = saved;
            }
            Expr::Loop(stmts) | Expr::Block(stmts) => self.walk_stmts(stmts),
            Expr::With { body, .. } => self.walk_stmts(body),
            Expr::Refine {
                initial,
                grade,
                revise,
                ..
            } => {
                self.walk_expr(&initial.node);
                self.walk_expr(&grade.node);
                self.walk_expr(&revise.node);
            }
            Expr::Pipe { left, right } => {
                self.walk_expr(&left.node);
                self.walk_expr(&right.node);
            }
            Expr::Propagate(inner) => self.walk_expr(&inner.node),
            Expr::Apply { func, arg } => {
                if let Some(target_id) = self.extract_mcp_call(expr) {
                    self.add_edge(
                        &self.context.clone(),
                        &target_id,
                        "mcp.call".into(),
                        "solid",
                    );
                    return;
                }
                self.walk_expr(&func.node);
                self.walk_expr(&arg.node);
            }
            Expr::Ternary { then_, else_, .. } => {
                self.walk_expr(&then_.node);
                if let Some(e) = else_ {
                    self.walk_expr(&e.node);
                }
            }
            Expr::Func { body, .. } => self.walk_expr(&body.node),
            Expr::Coalesce { expr, default } => {
                self.walk_expr(&expr.node);
                self.walk_expr(&default.node);
            }
            _ => {}
        }
    }

    fn resolve_target(&self, expr: &Expr) -> String {
        if let Expr::Ident(name) = expr {
            if let Some(id) = self.agent_vars.get(name) {
                return id.clone();
            }
            if let Some(id) = self.mcp_vars.get(name) {
                return id.clone();
            }
            return name.clone();
        }
        "unknown".into()
    }

    fn extract_mcp_call(&self, expr: &Expr) -> Option<String> {
        let Expr::Apply { func, .. } = expr else {
            return None;
        };
        let Expr::Apply { func: f2, .. } = &func.node else {
            return None;
        };
        let Expr::Apply {
            func: f3,
            arg: conn,
        } = &f2.node
        else {
            return None;
        };
        if !is_field_call(&f3.node, "mcp", "call") {
            return None;
        }
        let Expr::Ident(var) = &conn.node else {
            return None;
        };
        self.mcp_vars.get(var).cloned()
    }

    pub fn into_graph(self) -> Graph {
        Graph {
            nodes: self.nodes,
            edges: self.edges,
        }
    }
}

fn extract_agent_spawn(sexpr: &SExpr) -> Option<String> {
    let expr = unwrap_propagate(&sexpr.node);
    let Expr::Apply { func, arg } = expr else {
        return None;
    };
    if !is_field_call(&func.node, "agent", "spawn") {
        return None;
    }
    Some(extract_spawn_label(&arg.node))
}

fn extract_mcp_connect(sexpr: &SExpr) -> Option<String> {
    let expr = unwrap_propagate(&sexpr.node);
    let Expr::Apply { func, arg } = expr else {
        return None;
    };
    if !is_field_call(&func.node, "mcp", "connect") {
        return None;
    }
    Some(extract_spawn_label(&arg.node))
}

fn unwrap_propagate(expr: &Expr) -> &Expr {
    match expr {
        Expr::Propagate(inner) => unwrap_propagate(&inner.node),
        other => other,
    }
}

fn is_field_call(expr: &Expr, obj: &str, field: &str) -> bool {
    let Expr::FieldAccess {
        expr: e,
        field: FieldKind::Named(f),
    } = expr
    else {
        return false;
    };
    let Expr::Ident(name) = &e.node else {
        return false;
    };
    name == obj && f == field
}

fn extract_spawn_label(expr: &Expr) -> String {
    if let Expr::Record(fields) = expr {
        for field in fields {
            if field.name.as_deref() == Some("command") {
                return extract_str_literal(&field.value.node).unwrap_or_else(|| "agent".into());
            }
        }
    }
    "agent".into()
}

fn extract_msg_label(expr: &Expr) -> String {
    if let Expr::Record(fields) = expr {
        for field in fields {
            if field.name.as_deref() == Some("action") {
                return extract_str_literal(&field.value.node).unwrap_or_default();
            }
        }
    }
    String::new()
}

fn extract_str_literal(expr: &Expr) -> Option<String> {
    let Expr::Literal(Literal::Str(parts)) = expr else {
        return None;
    };
    if parts.len() != 1 {
        return None;
    }
    let StrPart::Text(t) = &parts[0] else {
        return None;
    };
    Some(t.clone())
}

fn expr_label(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => name.clone(),
        Expr::FieldAccess {
            expr: e,
            field: FieldKind::Named(f),
        } => {
            format!("{}.{f}", expr_label(&e.node))
        }
        Expr::Binary { op, left, right } => {
            format!(
                "{} {op} {}",
                expr_label(&left.node),
                expr_label(&right.node)
            )
        }
        Expr::Literal(Literal::Int(n)) => n.to_string(),
        Expr::Literal(Literal::Bool(b)) => b.to_string(),
        Expr::Literal(Literal::Str(_)) => extract_str_literal(expr).unwrap_or_else(|| "str".into()),
        _ => "match".into(),
    }
}
