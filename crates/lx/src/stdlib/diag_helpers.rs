use crate::ast::{Expr, FieldKind, Literal, SExpr, StrPart};

use super::Walker;

pub(super) fn extract_field_call_parts(expr: &Expr) -> Option<(&str, &str)> {
    let Expr::FieldAccess {
        expr: e,
        field: FieldKind::Named(f),
    } = expr
    else {
        return None;
    };
    let Expr::Ident(name) = &e.node else {
        return None;
    };
    Some((name.as_str(), f.as_str()))
}

pub(super) fn resolve_target(w: &Walker, expr: &Expr) -> String {
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

pub(super) fn is_field_call(expr: &Expr, obj: &str, field: &str) -> bool {
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

pub(super) fn unwrap_propagate(expr: &Expr) -> &Expr {
    match expr {
        Expr::Propagate(inner) => unwrap_propagate(&inner.node),
        other => other,
    }
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

pub(super) fn extract_spawn_label(expr: &Expr) -> String {
    if let Expr::Record(fields) = expr {
        for field in fields {
            if field.name.as_deref() == Some("command") {
                return extract_str_literal(&field.value.node).unwrap_or_else(|| "agent".into());
            }
        }
    }
    "agent".into()
}

pub(super) fn extract_msg_label(expr: &Expr) -> String {
    if let Expr::Record(fields) = expr {
        for field in fields {
            if field.name.as_deref() == Some("action") {
                return extract_str_literal(&field.value.node).unwrap_or_default();
            }
        }
    }
    String::new()
}

pub(super) fn extract_str_literal(expr: &Expr) -> Option<String> {
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
        _ => "expr".into(),
    }
}

pub(super) fn is_resource_module(module: &str) -> bool {
    matches!(
        module,
        "trace" | "knowledge" | "memory" | "budget" | "context" | "tasks" | "profile"
    )
}

pub(super) fn is_resource_create(method: &str) -> bool {
    matches!(method, "create" | "load" | "empty" | "define")
}

pub(super) fn is_resource_action(method: &str) -> bool {
    !is_resource_create(method)
}
