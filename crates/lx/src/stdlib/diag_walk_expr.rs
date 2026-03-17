use crate::ast::{Expr, FieldKind, Literal, SExpr, StrPart};
use crate::span::Span;
use crate::visitor::{AstVisitor, walk_expr};

use super::Walker;

pub(super) fn visit_expr_diag(w: &mut Walker, expr: &Expr, span: Span) {
    match expr {
        Expr::AgentSend { target, msg } => {
            let to = resolve_target(w, &target.node);
            w.add_edge(
                &w.context.clone(),
                &to,
                extract_msg_label(&msg.node),
                "dashed",
            );
        }
        Expr::AgentAsk { target, msg } => {
            let to = resolve_target(w, &target.node);
            w.add_edge(
                &w.context.clone(),
                &to,
                extract_msg_label(&msg.node),
                "solid",
            );
        }
        Expr::Par(stmts) => {
            let fork_id = w.add_node("fork", "par".into(), "fork");
            w.add_edge(&w.context.clone(), &fork_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = fork_id;
            w.walk_stmts(stmts);
            w.context = saved;
        }
        Expr::Sel(arms) => {
            let dec_id = w.add_node("sel", "sel".into(), "decision");
            w.add_edge(&w.context.clone(), &dec_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = dec_id;
            for arm in arms {
                w.visit_expr(&arm.handler.node, arm.handler.span);
            }
            w.context = saved;
        }
        Expr::Match { scrutinee, arms } => {
            let label = format!("{}?", expr_label(&scrutinee.node));
            let dec_id = w.add_node("match", label, "decision");
            w.add_edge(&w.context.clone(), &dec_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = dec_id;
            for arm in arms {
                w.visit_expr(&arm.body.node, arm.body.span);
            }
            w.context = saved;
        }
        Expr::Apply { func, arg } => {
            if let Some(target_id) = extract_mcp_call(w, expr) {
                w.add_edge(&w.context.clone(), &target_id, "mcp.call".into(), "solid");
                return;
            }
            w.visit_expr(&func.node, func.span);
            w.visit_expr(&arg.node, arg.span);
        }
        Expr::Refine {
            initial,
            grade,
            revise,
            ..
        } => {
            w.visit_expr(&initial.node, initial.span);
            w.visit_expr(&grade.node, grade.span);
            w.visit_expr(&revise.node, revise.span);
        }
        _ => walk_expr(w, expr, span),
    }
}

fn resolve_target(w: &Walker, expr: &Expr) -> String {
    if let Expr::Ident(name) = expr {
        if let Some(id) = w.agent_vars.get(name) {
            return id.clone();
        }
        if let Some(id) = w.mcp_vars.get(name) {
            return id.clone();
        }
        return name.clone();
    }
    "unknown".into()
}

fn extract_mcp_call(w: &Walker, expr: &Expr) -> Option<String> {
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
    w.mcp_vars.get(var).cloned()
}

pub(super) fn extract_agent_spawn(sexpr: &SExpr) -> Option<String> {
    let expr = unwrap_propagate(&sexpr.node);
    let Expr::Apply { func, arg } = expr else {
        return None;
    };
    if !is_field_call(&func.node, "agent", "spawn") {
        return None;
    }
    Some(extract_spawn_label(&arg.node))
}

pub(super) fn extract_mcp_connect(sexpr: &SExpr) -> Option<String> {
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

pub(super) fn expr_label(expr: &Expr) -> String {
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
