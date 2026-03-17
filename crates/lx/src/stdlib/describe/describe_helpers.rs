use crate::ast::{Expr, FieldKind, Literal, SExpr, StrPart};

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

pub(super) fn extract_ai_call(sexpr: &SExpr) -> Option<String> {
    extract_ai_call_from_apply(unwrap_propagate(&sexpr.node))
}

pub(super) fn extract_ai_call_from_apply(expr: &Expr) -> Option<String> {
    let Expr::Apply { func, .. } = expr else {
        return None;
    };
    if is_field_call(&func.node, "ai", "prompt") || is_field_call(&func.node, "ai", "prompt_with") {
        return Some("ai.prompt".into());
    }
    None
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

pub(super) fn expr_name(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => name.clone(),
        Expr::FieldAccess {
            expr: e,
            field: FieldKind::Named(f),
        } => {
            format!("{}.{f}", expr_name(&e.node))
        }
        _ => "expr".into(),
    }
}
