use std::ops::ControlFlow;

use crate::ast::{AstArena, Expr, ExprApply, ExprId};
use crate::sym::intern;
use crate::visitor::dispatch_expr;
use miette::SourceSpan;

use super::diag_helpers::{extract_field_call_parts, is_resource_action, is_resource_create, is_resource_module};
use super::{EdgeStyle, NodeKind, Walker};

pub(super) fn visit_apply_diag(w: &mut Walker, apply: &ExprApply, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let Some((module, method, args)) = uncurry_call(apply, arena) {
    if let Some(kind) = classify_call(module, method) {
      return handle_call(w, module, method, kind, &args, span, arena);
    }
    if w.imported_modules.contains(&intern(module)) {
      let label = format!("{module}.{method}");
      let id = w.add_node_at("tool", label, NodeKind::Tool, Some(span));
      let ctx = w.context.clone();
      w.add_edge(&ctx, &id, String::new(), EdgeStyle::Solid);
      for a in &args {
        let a_expr = arena.expr(*a);
        let a_span = arena.expr_span(*a);
        dispatch_expr(w, a_expr, a_span, arena)?;
      }
      return ControlFlow::Continue(());
    }
  }
  if let Some((name, fn_args)) = uncurry_fn_call(apply, arena)
    && let Some(node_id) = w.fn_nodes.get(&intern(name)).cloned()
  {
    let ctx = w.context.clone();
    w.add_edge(&ctx, &node_id, String::new(), EdgeStyle::Solid);
    for a in &fn_args {
      let a_expr = arena.expr(*a);
      let a_span = arena.expr_span(*a);
      dispatch_expr(w, a_expr, a_span, arena)?;
    }
    return ControlFlow::Continue(());
  }
  let func_expr = arena.expr(apply.func);
  let func_span = arena.expr_span(apply.func);
  dispatch_expr(w, func_expr, func_span, arena)?;
  let arg_expr = arena.expr(apply.arg);
  let arg_span = arena.expr_span(apply.arg);
  dispatch_expr(w, arg_expr, arg_span, arena)
}

fn uncurry_call<'a>(apply: &'a ExprApply, arena: &'a AstArena) -> Option<(&'a str, &'a str, Vec<ExprId>)> {
  let mut args = vec![apply.arg];
  let mut current = arena.expr(apply.func);
  while let Expr::Apply(ExprApply { func, arg }) = current {
    args.push(*arg);
    current = arena.expr(*func);
  }
  let (module, method) = extract_field_call_parts(current, arena)?;
  args.reverse();
  Some((module, method, args))
}

fn uncurry_fn_call<'a>(apply: &'a ExprApply, arena: &'a AstArena) -> Option<(&'a str, Vec<ExprId>)> {
  let mut args = vec![apply.arg];
  let mut current = arena.expr(apply.func);
  while let Expr::Apply(ExprApply { func, arg }) = current {
    args.push(*arg);
    current = arena.expr(*func);
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

fn handle_call(w: &mut Walker, module: &str, method: &str, kind: NodeKind, args: &[ExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let ("cron", "every") = (module, method) {
    let id = w.add_node_at("loop", "cron".into(), NodeKind::Loop, Some(span));
    let ctx = w.context.clone();
    w.add_edge(&ctx, &id, String::new(), EdgeStyle::Solid);
    if let Some(last) = args.last() {
      w.context_stack.push(w.context.clone());
      w.context = id;
      let l_expr = arena.expr(*last);
      let l_span = arena.expr_span(*last);
      dispatch_expr(w, l_expr, l_span, arena)?;
      w.context = w.context_stack.pop().expect("diag: context_stack underflow");
    }
    return ControlFlow::Continue(());
  }
  if is_resource_module(module) && is_resource_action(method) {
    for arg_id in args {
      if let Expr::Ident(var) = arena.expr(*arg_id)
        && let Some(node_id) = w.resource_vars.get(var).cloned()
      {
        let ctx = w.context.clone();
        w.add_edge(&ctx, &node_id, method.to_string(), EdgeStyle::Dashed);
        return ControlFlow::Continue(());
      }
    }
  }

  let label = format!("{module}.{method}");
  let id = w.add_node_at(kind.as_str(), label, kind, Some(span));
  let ctx = w.context.clone();
  w.add_edge(&ctx, &id, String::new(), EdgeStyle::Solid);
  for arg_id in args {
    let a_expr = arena.expr(*arg_id);
    let a_span = arena.expr_span(*arg_id);
    dispatch_expr(w, a_expr, a_span, arena)?;
  }
  ControlFlow::Continue(())
}
