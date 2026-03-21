use crate::ast::{Expr, SExpr};
use crate::span::Span;
use crate::visitor::{AstVisitor, walk_expr};

use super::Walker;
use super::diag_helpers::{extract_field_call_parts, is_resource_action, is_resource_create, is_resource_module};

pub(super) fn visit_expr_diag(w: &mut Walker, expr: &Expr, span: Span) {
  match expr {
    Expr::Shell { .. } => {
      let id = w.add_node_at("io", "shell".into(), "io", Some(span));
      let ctx = w.context.clone();
      w.add_edge_typed(&ctx, &id, String::new(), "solid", "io");
    },
    Expr::Apply { func, arg } => {
      if let Some((module, method, args)) = uncurry_call(expr) {
        if let Some(kind) = classify_call(module, method) {
          handle_call(w, module, method, kind, &args, span);
          return;
        }
        if w.imported_modules.contains(module) {
          let label = format!("{module}.{method}");
          let id = w.add_node_at("tool", label, "tool", Some(span));
          let ctx = w.context.clone();
          w.add_edge(&ctx, &id, String::new(), "solid");
          for a in &args {
            w.visit_expr(&a.node, a.span);
          }
          return;
        }
      }
      if let Some((name, fn_args)) = uncurry_fn_call(expr)
        && let Some(node_id) = w.fn_nodes.get(name).cloned()
      {
        let ctx = w.context.clone();
        w.add_edge(&ctx, &node_id, String::new(), "solid");
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

fn classify_call(module: &str, method: &str) -> Option<&'static str> {
  match module {
    "pool" => Some("fork"),
    "saga" => Some("fork"),
    "plan" => Some("fork"),
    "retry" => Some("loop"),
    "cron" => Some("loop"),
    "circuit" => Some(match method {
      "check" => "decision",
      _ => "resource",
    }),
    "user" => Some("user"),
    "trace" | "knowledge" | "memory" | "budget" | "context" | "tasks" | "profile" => {
      if is_resource_create(method) {
        None
      } else {
        Some("resource")
      }
    },
    "http" | "fs" | "git" => Some("io"),
    _ => None,
  }
}

fn handle_call(w: &mut Walker, module: &str, method: &str, kind: &str, args: &[&SExpr], span: Span) {
  if let ("cron", "every") = (module, method) {
    let id = w.add_node_at("loop", "cron".into(), "loop", Some(span));
    let ctx = w.context.clone();
    w.add_edge(&ctx, &id, String::new(), "solid");
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
        w.add_edge(&ctx, &node_id, method.to_string(), "dashed");
        return;
      }
    }
  }

  let label = format!("{module}.{method}");
  let id = w.add_node_at(kind, label, kind, Some(span));
  let ctx = w.context.clone();
  w.add_edge(&ctx, &id, String::new(), "solid");
  for arg in args {
    w.visit_expr(&arg.node, arg.span);
  }
}
