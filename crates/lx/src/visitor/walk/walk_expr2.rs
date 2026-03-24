use std::ops::ControlFlow;

use crate::ast::{
  AstArena, ExprAssert, ExprBreak, ExprCoalesce, ExprEmit, ExprId, ExprLoop, ExprNamedArg, ExprPar, ExprPropagate, ExprSlice, ExprTernary, ExprTimeout,
  ExprWith, ExprYield, SelArm,
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

walk_dispatch_id!(walk_loop_dispatch, walk_loop, visit_loop, leave_loop, ExprLoop, ExprId);
walk_dispatch_id!(walk_par_dispatch, walk_par, visit_par, leave_par, ExprPar, ExprId);
walk_dispatch_id_slice!(walk_sel_dispatch, walk_sel, visit_sel, leave_sel, SelArm, ExprId);
walk_dispatch_id!(walk_propagate_dispatch, walk_propagate, visit_propagate, leave_propagate, ExprPropagate, ExprId);
walk_dispatch_id!(walk_break_dispatch, walk_break, visit_break, leave_break, ExprBreak, ExprId);

pub fn walk_ternary<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, ternary: &ExprTernary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  ternary.walk_children(v, arena)?;
  v.leave_ternary(id, ternary, span);
  ControlFlow::Continue(())
}

pub fn walk_propagate<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, propagate: &ExprPropagate, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  propagate.walk_children(v, arena)?;
  v.leave_propagate(id, propagate, span);
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

pub fn walk_loop<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, loop_node: &ExprLoop, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  loop_node.walk_children(v, arena)?;
  v.leave_loop(id, loop_node, span);
  ControlFlow::Continue(())
}

pub fn walk_break<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, brk: &ExprBreak, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  brk.walk_children(v, arena)?;
  v.leave_break(id, brk, span);
  ControlFlow::Continue(())
}

pub fn walk_assert<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, assert: &ExprAssert, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  assert.walk_children(v, arena)?;
  v.leave_assert(id, assert, span);
  ControlFlow::Continue(())
}

pub fn walk_par<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, par: &ExprPar, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  par.walk_children(v, arena)?;
  v.leave_par(id, par, span);
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
