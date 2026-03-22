use crate::ast::{Expr, SExpr};
use crate::visitor::{AstVisitor, walk_expr};
use miette::SourceSpan;

use super::diag_helpers::{extract_field_call_parts, is_resource_action, is_resource_create, is_resource_module};
use super::{EdgeStyle, NodeKind, Walker};

pub(super) fn visit_expr_diag(w: &mut Walker, expr: &Expr, span: SourceSpan) {
  match expr {
    Expr::Apply { func, arg } => {
      if let Some((module, method, args)) = uncurry_call(expr) {
        if let Some(kind) = classify_call(module, method) {
          handle_call(w, module, method, kind, &args, span);
          return;
        }
        if w.imported_modules.contains(&crate::sym::intern(module)) {
          let label = format!("{module}.{method}");
          let id = w.add_node_at("tool", label, NodeKind::Tool, Some(span));
          let ctx = w.context.clone();
          w.add_edge(&ctx, &id, String::new(), EdgeStyle::Solid);
          for a in &args {
            w.visit_expr(&a.node, a.span);
          }
          return;
        }
      }
      if let Some((name, fn_args)) = uncurry_fn_call(expr)
        && let Some(node_id) = w.fn_nodes.get(&crate::sym::intern(name)).cloned()
      {
        let ctx = w.context.clone();
        w.add_edge(&ctx, &node_id, String::new(), EdgeStyle::Solid);
        for a in &fn_args {
          w.visit_expr(&a.node, a.span);
        }
        return;
      }
      w.visit_expr(&func.node, func.span);
      w.visit_expr(&arg.node, arg.span);
    },
    _ => walk_expr(w, expr, span),
  }
}

fn uncurry_call(expr: &Expr) -> Option<(&str, &str, Vec<&SExpr>)> {
  let mut args = Vec::new();
  let mut current = expr;
  while let Expr::Apply { func, arg } = current {
    args.push(arg.as_ref());
    current = &func.node;
  }
  let (module, method) = extract_field_call_parts(current)?;
  args.reverse();
  Some((module, method, args))
}

fn uncurry_fn_call(expr: &Expr) -> Option<(&str, Vec<&SExpr>)> {
  let mut args = Vec::new();
  let mut current = expr;
  while let Expr::Apply { func, arg } = current {
    args.push(arg.as_ref());
    current = &func.node;
  }
  let Expr::Ident(name) = current else {
    return None;
  };
  args.reverse();
  Some((name.as_str(), args))
}

fn classify_call(module: &str, method: &str) -> Option<NodeKind> {
  match module {
    "pool" => Some(NodeKind::Fork),
    "saga" => Some(NodeKind::Fork),
    "plan" => Some(NodeKind::Fork),
    "retry" => Some(NodeKind::Loop),
    "cron" => Some(NodeKind::Loop),
    "circuit" => Some(match method {
      "check" => NodeKind::Decision,
      _ => NodeKind::Resource,
    }),
    "user" => Some(NodeKind::User),
    "trace" | "knowledge" | "memory" | "budget" | "context" | "tasks" | "profile" => {
      if is_resource_create(method) {
        None
      } else {
        Some(NodeKind::Resource)
      }
    },
    "http" | "fs" | "git" => Some(NodeKind::Io),
    _ => None,
  }
}

fn handle_call(w: &mut Walker, module: &str, method: &str, kind: NodeKind, args: &[&SExpr], span: SourceSpan) {
  if let ("cron", "every") = (module, method) {
    let id = w.add_node_at("loop", "cron".into(), NodeKind::Loop, Some(span));
    let ctx = w.context.clone();
    w.add_edge(&ctx, &id, String::new(), EdgeStyle::Solid);
    if let Some(last) = args.last() {
      w.context_stack.push(w.context.clone());
      w.context = id;
      w.visit_expr(&last.node, last.span);
      w.context = w.context_stack.pop().expect("diag: context_stack underflow");
    }
    return;
  }
  if is_resource_module(module) && is_resource_action(method) {
    for arg in args {
      if let Expr::Ident(var) = &arg.node
        && let Some(node_id) = w.resource_vars.get(var).cloned()
      {
        let ctx = w.context.clone();
        w.add_edge(&ctx, &node_id, method.to_string(), EdgeStyle::Dashed);
        return;
      }
    }
  }

  let label = format!("{module}.{method}");
  let id = w.add_node_at(kind.as_str(), label, kind, Some(span));
  let ctx = w.context.clone();
  w.add_edge(&ctx, &id, String::new(), EdgeStyle::Solid);
  for arg in args {
    w.visit_expr(&arg.node, arg.span);
  }
}
