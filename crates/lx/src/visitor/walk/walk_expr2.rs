use std::ops::ControlFlow;

use crate::ast::{
  AstArena, ExprAssert, ExprCoalesce, ExprEmit, ExprId, ExprNamedArg, ExprSlice, ExprTernary, ExprTimeout, ExprWith, ExprYield, SelArm, StmtId, WithKind,
};
use miette::SourceSpan;

use super::super::{AstVisitor, VisitAction};
use super::dispatch_expr;

walk_dispatch!(walk_ternary_dispatch, walk_ternary, visit_ternary, leave_ternary, ExprTernary);
walk_dispatch!(walk_coalesce_dispatch, walk_coalesce, visit_coalesce, leave_coalesce, ExprCoalesce);
walk_dispatch!(walk_slice_dispatch, walk_slice, visit_slice, leave_slice, ExprSlice);
walk_dispatch!(walk_named_arg_dispatch, walk_named_arg, visit_named_arg, leave_named_arg, ExprNamedArg);
walk_dispatch!(walk_assert_dispatch, walk_assert, visit_assert, leave_assert, ExprAssert);
walk_dispatch!(walk_timeout_dispatch, walk_timeout, visit_timeout, leave_timeout, ExprTimeout);
walk_dispatch!(walk_emit_dispatch, walk_emit, visit_emit, leave_emit, ExprEmit);
walk_dispatch!(walk_yield_dispatch, walk_yield, visit_yield, leave_yield, ExprYield);
walk_dispatch!(walk_with_dispatch, walk_with, visit_with, leave_with, ExprWith);

walk_dispatch_slice!(walk_loop_dispatch, walk_loop, visit_loop, leave_loop, StmtId);
walk_dispatch_slice!(walk_par_dispatch, walk_par, visit_par, leave_par, StmtId);
walk_dispatch_slice!(walk_sel_dispatch, walk_sel, visit_sel, leave_sel, SelArm);

pub(crate) fn walk_propagate_dispatch<V: AstVisitor + ?Sized>(v: &mut V, inner: ExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_propagate(inner, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_propagate(inner, span, arena),
    VisitAction::Descend => walk_propagate(v, inner, span, arena),
  }
}

pub(crate) fn walk_break_dispatch<V: AstVisitor + ?Sized>(v: &mut V, value: Option<ExprId>, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_break(value, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_break(value, span, arena),
    VisitAction::Descend => walk_break(v, value, span, arena),
  }
}

pub fn walk_ternary<V: AstVisitor + ?Sized>(v: &mut V, ternary: &ExprTernary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(ternary.cond), arena.expr_span(ternary.cond), arena)?;
  dispatch_expr(v, arena.expr(ternary.then_), arena.expr_span(ternary.then_), arena)?;
  if let Some(e) = ternary.else_ {
    dispatch_expr(v, arena.expr(e), arena.expr_span(e), arena)?;
  }
  v.leave_ternary(ternary, span, arena)
}

pub fn walk_propagate<V: AstVisitor + ?Sized>(v: &mut V, inner: ExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(inner), arena.expr_span(inner), arena)?;
  v.leave_propagate(inner, span, arena)
}

pub fn walk_coalesce<V: AstVisitor + ?Sized>(v: &mut V, coalesce: &ExprCoalesce, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(coalesce.expr), arena.expr_span(coalesce.expr), arena)?;
  dispatch_expr(v, arena.expr(coalesce.default), arena.expr_span(coalesce.default), arena)?;
  v.leave_coalesce(coalesce, span, arena)
}

pub fn walk_slice<V: AstVisitor + ?Sized>(v: &mut V, slice: &ExprSlice, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(slice.expr), arena.expr_span(slice.expr), arena)?;
  if let Some(s) = slice.start {
    dispatch_expr(v, arena.expr(s), arena.expr_span(s), arena)?;
  }
  if let Some(e) = slice.end {
    dispatch_expr(v, arena.expr(e), arena.expr_span(e), arena)?;
  }
  v.leave_slice(slice, span, arena)
}

pub fn walk_named_arg<V: AstVisitor + ?Sized>(v: &mut V, na: &ExprNamedArg, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(na.value), arena.expr_span(na.value), arena)?;
  v.leave_named_arg(na, span, arena)
}

pub fn walk_loop<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[StmtId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &s in stmts {
    super::dispatch_stmt(v, s, arena)?;
  }
  v.leave_loop(stmts, span, arena)
}

pub fn walk_break<V: AstVisitor + ?Sized>(v: &mut V, value: Option<ExprId>, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let Some(val) = value {
    dispatch_expr(v, arena.expr(val), arena.expr_span(val), arena)?;
  }
  v.leave_break(value, span, arena)
}

pub fn walk_assert<V: AstVisitor + ?Sized>(v: &mut V, assert: &ExprAssert, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(assert.expr), arena.expr_span(assert.expr), arena)?;
  if let Some(m) = assert.msg {
    dispatch_expr(v, arena.expr(m), arena.expr_span(m), arena)?;
  }
  v.leave_assert(assert, span, arena)
}

pub fn walk_par<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[StmtId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &s in stmts {
    super::dispatch_stmt(v, s, arena)?;
  }
  v.leave_par(stmts, span, arena)
}

pub fn walk_sel<V: AstVisitor + ?Sized>(v: &mut V, arms: &[SelArm], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for arm in arms {
    dispatch_expr(v, arena.expr(arm.expr), arena.expr_span(arm.expr), arena)?;
    dispatch_expr(v, arena.expr(arm.handler), arena.expr_span(arm.handler), arena)?;
  }
  v.leave_sel(arms, span, arena)
}

pub fn walk_timeout<V: AstVisitor + ?Sized>(v: &mut V, timeout: &ExprTimeout, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(timeout.ms), arena.expr_span(timeout.ms), arena)?;
  dispatch_expr(v, arena.expr(timeout.body), arena.expr_span(timeout.body), arena)?;
  v.leave_timeout(timeout, span, arena)
}

pub fn walk_emit<V: AstVisitor + ?Sized>(v: &mut V, emit: &ExprEmit, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(emit.value), arena.expr_span(emit.value), arena)?;
  v.leave_emit(emit, span, arena)
}

pub fn walk_yield<V: AstVisitor + ?Sized>(v: &mut V, yld: &ExprYield, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(yld.value), arena.expr_span(yld.value), arena)?;
  v.leave_yield(yld, span, arena)
}

pub fn walk_with<V: AstVisitor + ?Sized>(v: &mut V, with: &ExprWith, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match &with.kind {
    WithKind::Binding { value, .. } => {
      dispatch_expr(v, arena.expr(*value), arena.expr_span(*value), arena)?;
    },
    WithKind::Resources { resources } => {
      for &(r, _) in resources {
        dispatch_expr(v, arena.expr(r), arena.expr_span(r), arena)?;
      }
    },
    WithKind::Context { fields } => {
      for &(_, eid) in fields {
        dispatch_expr(v, arena.expr(eid), arena.expr_span(eid), arena)?;
      }
    },
  }
  for &s in &with.body {
    super::dispatch_stmt(v, s, arena)?;
  }
  v.leave_with(with, span, arena)
}
