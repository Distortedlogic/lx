use std::ops::ControlFlow;

use lx_ast::ast::ExprApply;
use lx_ast::visitor::prelude::*;
use lx_span::sym::intern;

use super::diag_helpers::{extract_field_call_parts, is_resource_action, is_resource_create, is_resource_module};
use super::{EdgeStyle, NodeKind, Walker};

enum StdlibModule {
  Pool,
  Saga,
  Plan,
  Retry,
  Cron,
  Circuit,
  Trace,
  Knowledge,
  Memory,
  Budget,
  Context,
  Tasks,
  Profile,
  User,
  Http,
  Fs,
  Git,
}

impl StdlibModule {
  fn from_str(s: &str) -> Option<Self> {
    match s {
      "pool" => Some(Self::Pool),
      "saga" => Some(Self::Saga),
      "plan" => Some(Self::Plan),
      "retry" => Some(Self::Retry),
      "cron" => Some(Self::Cron),
      "circuit" => Some(Self::Circuit),
      "trace" => Some(Self::Trace),
      "knowledge" => Some(Self::Knowledge),
      "memory" => Some(Self::Memory),
      "budget" => Some(Self::Budget),
      "context" => Some(Self::Context),
      "tasks" => Some(Self::Tasks),
      "profile" => Some(Self::Profile),
      "user" => Some(Self::User),
      "http" => Some(Self::Http),
      "fs" => Some(Self::Fs),
      "git" => Some(Self::Git),
      _ => None,
    }
  }
}

pub(super) fn visit_apply_diag(w: &mut Walker<'_>, apply: &ExprApply, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
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
        dispatch_expr(w, *a, arena)?;
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
      dispatch_expr(w, *a, arena)?;
    }
    return ControlFlow::Continue(());
  }
  dispatch_expr(w, apply.func, arena)?;
  dispatch_expr(w, apply.arg, arena)
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
  let m = StdlibModule::from_str(module)?;
  match m {
    StdlibModule::Pool | StdlibModule::Saga | StdlibModule::Plan => Some(NodeKind::Fork),
    StdlibModule::Retry | StdlibModule::Cron => Some(NodeKind::Loop),
    StdlibModule::Circuit => Some(match method {
      "check" => NodeKind::Decision,
      _ => NodeKind::Resource,
    }),
    StdlibModule::User => Some(NodeKind::User),
    StdlibModule::Trace
    | StdlibModule::Knowledge
    | StdlibModule::Memory
    | StdlibModule::Budget
    | StdlibModule::Context
    | StdlibModule::Tasks
    | StdlibModule::Profile => {
      if is_resource_create(method) {
        None
      } else {
        Some(NodeKind::Resource)
      }
    },
    StdlibModule::Http | StdlibModule::Fs | StdlibModule::Git => Some(NodeKind::Io),
  }
}

fn handle_call(w: &mut Walker<'_>, module: &str, method: &str, kind: NodeKind, args: &[ExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let ("cron", "every") = (module, method) {
    let id = w.add_node_at("loop", "cron".into(), NodeKind::Loop, Some(span));
    let ctx = w.context.clone();
    w.add_edge(&ctx, &id, String::new(), EdgeStyle::Solid);
    if let Some(last) = args.last() {
      w.context_stack.push(w.context.clone());
      w.context = id;
      dispatch_expr(w, *last, arena)?;
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
    dispatch_expr(w, *arg_id, arena)?;
  }
  ControlFlow::Continue(())
}
