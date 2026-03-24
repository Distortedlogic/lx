use std::ops::ControlFlow;

use crate::ast::{
  AstArena, ExprAssert, ExprCoalesce, ExprEmit, ExprId, ExprNamedArg, ExprSlice, ExprTernary, ExprTimeout, ExprWith, ExprYield, SelArm, StmtId,
};
use miette::SourceSpan;

use super::super::{AstVisitor, VisitAction};

walk_dispatch_id!(walk_ternary_dispatch, walk_ternary, visit_ternary, leave_ternary, ExprTernary, ExprId);
walk_dispatch_id!(walk_coalesce_dispatch, walk_coalesce, visit_coalesce, leave_coalesce, ExprCoalesce, ExprId);
walk_dispatch_id!(walk_slice_dispatch, walk_slice, visit_slice, leave_slice, ExprSlice, ExprId);
walk_dispatch_id!(walk_named_arg_dispatch, walk_named_arg, visit_named_arg, leave_named_arg, ExprNamedArg, ExprId);
walk_dispatch_id!(walk_assert_dispatch, walk_assert, visit_assert, leave_assert, ExprAssert, ExprId);
walk_dispatch_id!(walk_timeout_dispatch, walk_timeout, visit_timeout, leave_timeout, ExprTimeout, ExprId);
walk_dispatch_id!(walk_emit_dispatch, walk_emit, visit_emit, leave_emit, ExprEmit, ExprId);
walk_dispatch_id!(walk_yield_dispatch, walk_yield, visit_yield, leave_yield, ExprYield, ExprId);
walk_dispatch_id!(walk_with_dispatch, walk_with, visit_with, leave_with, ExprWith, ExprId);

walk_dispatch_id_slice!(walk_loop_dispatch, walk_loop, visit_loop, leave_loop, StmtId, ExprId);
walk_dispatch_id_slice!(walk_par_dispatch, walk_par, visit_par, leave_par, StmtId, ExprId);
walk_dispatch_id_slice!(walk_sel_dispatch, walk_sel, visit_sel, leave_sel, SelArm, ExprId);

pub(crate) fn walk_propagate_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, inner: ExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_propagate(id, inner, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_propagate(id, inner, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => walk_propagate(v, id, inner, span, arena),
  }
}

pub(crate) fn walk_break_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, value: Option<ExprId>, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_break(id, value, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_break(id, value, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => walk_break(v, id, value, span, arena),
  }
}

pub fn walk_ternary<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, ternary: &ExprTernary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  ternary.walk_children(v, arena)?;
  v.leave_ternary(id, ternary, span);
  ControlFlow::Continue(())
}

pub fn walk_propagate<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, inner: ExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  super::dispatch_expr(v, inner, arena)?;
  v.leave_propagate(id, inner, span);
  ControlFlow::Continue(())
}

pub fn walk_coalesce<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, coalesce: &ExprCoalesce, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  coalesce.walk_children(v, arena)?;
  v.leave_coalesce(id, coalesce, span);
  ControlFlow::Continue(())
}

pub fn walk_slice<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, slice: &ExprSlice, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  slice.walk_children(v, arena)?;
  v.leave_slice(id, slice, span);
  ControlFlow::Continue(())
}

pub fn walk_named_arg<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, na: &ExprNamedArg, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  na.walk_children(v, arena)?;
  v.leave_named_arg(id, na, span);
  ControlFlow::Continue(())
}

pub fn walk_loop<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, stmts: &[StmtId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &s in stmts {
    super::dispatch_stmt(v, s, arena)?;
  }
  v.leave_loop(id, stmts, span);
  ControlFlow::Continue(())
}

pub fn walk_break<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, value: Option<ExprId>, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let Some(val) = value {
    super::dispatch_expr(v, val, arena)?;
  }
  v.leave_break(id, value, span);
  ControlFlow::Continue(())
}

pub fn walk_assert<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, assert: &ExprAssert, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  assert.walk_children(v, arena)?;
  v.leave_assert(id, assert, span);
  ControlFlow::Continue(())
}

pub fn walk_par<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, stmts: &[StmtId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &s in stmts {
    super::dispatch_stmt(v, s, arena)?;
  }
  v.leave_par(id, stmts, span);
  ControlFlow::Continue(())
}

pub fn walk_sel<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, arms: &[SelArm], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for arm in arms {
    arm.walk_children(v, arena)?;
  }
  v.leave_sel(id, arms, span);
  ControlFlow::Continue(())
}

pub fn walk_timeout<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, timeout: &ExprTimeout, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  timeout.walk_children(v, arena)?;
  v.leave_timeout(id, timeout, span);
  ControlFlow::Continue(())
}

pub fn walk_emit<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, emit: &ExprEmit, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  emit.walk_children(v, arena)?;
  v.leave_emit(id, emit, span);
  ControlFlow::Continue(())
}

pub fn walk_yield<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, yld: &ExprYield, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  yld.walk_children(v, arena)?;
  v.leave_yield(id, yld, span);
  ControlFlow::Continue(())
}

pub fn walk_with<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, with: &ExprWith, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  with.walk_children(v, arena)?;
  v.leave_with(id, with, span);
  ControlFlow::Continue(())
}
