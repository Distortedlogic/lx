use crate::ast::{Expr, ExprBinary, ExprFieldAccess, FieldKind, Literal, StrPart};

pub(super) fn extract_field_call_parts(expr: &Expr) -> Option<(&str, &str)> {
  let Expr::FieldAccess(ExprFieldAccess { expr: e, field: FieldKind::Named(f) }) = expr else {
    return None;
  };
  let Expr::Ident(name) = &e.node else {
    return None;
  };
  Some((name.as_str(), f.as_str()))
}

pub(super) fn unwrap_propagate(expr: &Expr) -> &Expr {
  match expr {
    Expr::Propagate(inner) => unwrap_propagate(&inner.node),
    other => other,
  }
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
    Expr::Ident(name) => name.to_string(),
    Expr::FieldAccess(ExprFieldAccess { expr: e, field: FieldKind::Named(f) }) => {
      format!("{}.{f}", expr_label(&e.node))
    },
    Expr::Binary(ExprBinary { op, left, right }) => {
      format!("{} {op} {}", expr_label(&left.node), expr_label(&right.node))
    },
    Expr::Literal(Literal::Int(n)) => n.to_string(),
    Expr::Literal(Literal::Bool(b)) => b.to_string(),
    Expr::Literal(Literal::Str(_)) => extract_str_literal(expr).unwrap_or_else(|| "str".into()),
    _ => "expr".into(),
  }
}

pub(super) fn is_resource_module(module: &str) -> bool {
  matches!(module, "trace" | "knowledge" | "memory" | "budget" | "context" | "tasks" | "profile")
}

pub(super) fn is_resource_create(method: &str) -> bool {
  matches!(method, "create" | "load" | "empty" | "define")
}

pub(super) fn is_resource_action(method: &str) -> bool {
  !is_resource_create(method)
}
