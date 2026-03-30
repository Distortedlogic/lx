use std::ops::ControlFlow;

use crate::ast::{
  AstArena, ExprApply, ExprAsk, ExprAssert, ExprBinary, ExprBlock, ExprBreak, ExprCoalesce, ExprEmit, ExprFieldAccess, ExprFunc, ExprId, ExprLoop, ExprMatch,
  ExprNamedArg, ExprPar, ExprPipe, ExprPropagate, ExprSlice, ExprTell, ExprTernary, ExprTimeout, ExprTuple, ExprUnary, ExprWith, ExprYield, ListElem, Literal,
  MapEntry, RecordField, Section, SelArm,
};
use miette::SourceSpan;

use super::super::{AstVisitor, VisitAction};

define_walk_and_dispatch!(walk_literal_dispatch, walk_literal, visit_literal, leave_literal, Literal, ExprId);
define_walk_and_dispatch!(walk_binary_dispatch, walk_binary, visit_binary, leave_binary, ExprBinary, ExprId);
define_walk_and_dispatch!(walk_unary_dispatch, walk_unary, visit_unary, leave_unary, ExprUnary, ExprId);
define_walk_and_dispatch!(walk_pipe_dispatch, walk_pipe, visit_pipe, leave_pipe, ExprPipe, ExprId);
define_walk_and_dispatch!(walk_tell_dispatch, walk_tell, visit_tell, leave_tell, ExprTell, ExprId);
define_walk_and_dispatch!(walk_ask_dispatch, walk_ask, visit_ask, leave_ask, ExprAsk, ExprId);
define_walk_and_dispatch!(walk_apply_dispatch, walk_apply, visit_apply, leave_apply, ExprApply, ExprId);
define_walk_and_dispatch!(walk_section_dispatch, walk_section, visit_section, leave_section, Section, ExprId);
define_walk_and_dispatch!(walk_field_access_dispatch, walk_field_access, visit_field_access, leave_field_access, ExprFieldAccess, ExprId);
define_walk_and_dispatch!(walk_block_dispatch, walk_block, visit_block, leave_block, ExprBlock, ExprId);
define_walk_and_dispatch!(walk_tuple_dispatch, walk_tuple, visit_tuple, leave_tuple, ExprTuple, ExprId);
define_walk_and_dispatch!(walk_func_dispatch, walk_func, visit_func, leave_func, ExprFunc, ExprId);
define_walk_and_dispatch!(walk_match_dispatch, walk_match, visit_match, leave_match, ExprMatch, ExprId);
define_walk_and_dispatch!(walk_ternary_dispatch, walk_ternary, visit_ternary, leave_ternary, ExprTernary, ExprId);
define_walk_and_dispatch!(walk_coalesce_dispatch, walk_coalesce, visit_coalesce, leave_coalesce, ExprCoalesce, ExprId);
define_walk_and_dispatch!(walk_slice_dispatch, walk_slice, visit_slice, leave_slice, ExprSlice, ExprId);
define_walk_and_dispatch!(walk_named_arg_dispatch, walk_named_arg, visit_named_arg, leave_named_arg, ExprNamedArg, ExprId);
define_walk_and_dispatch!(walk_assert_dispatch, walk_assert, visit_assert, leave_assert, ExprAssert, ExprId);
define_walk_and_dispatch!(walk_timeout_dispatch, walk_timeout, visit_timeout, leave_timeout, ExprTimeout, ExprId);
define_walk_and_dispatch!(walk_emit_dispatch, walk_emit, visit_emit, leave_emit, ExprEmit, ExprId);
define_walk_and_dispatch!(walk_yield_dispatch, walk_yield, visit_yield, leave_yield, ExprYield, ExprId);
define_walk_and_dispatch!(walk_with_dispatch, walk_with, visit_with, leave_with, ExprWith, ExprId);
define_walk_and_dispatch!(walk_loop_dispatch, walk_loop, visit_loop, leave_loop, ExprLoop, ExprId);
define_walk_and_dispatch!(walk_par_dispatch, walk_par, visit_par, leave_par, ExprPar, ExprId);
define_walk_and_dispatch!(walk_propagate_dispatch, walk_propagate, visit_propagate, leave_propagate, ExprPropagate, ExprId);
define_walk_and_dispatch!(walk_break_dispatch, walk_break, visit_break, leave_break, ExprBreak, ExprId);

pub fn walk_list_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, elems: &[ListElem], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_list(id, elems, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_list(id, elems, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_list(v, id, elems, span, arena)?;
      v.leave_list(id, elems, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_list<V: AstVisitor + ?Sized>(v: &mut V, _id: ExprId, elems: &[ListElem], _span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for elem in elems {
    elem.walk_children(v, arena)?;
  }
  ControlFlow::Continue(())
}

pub fn walk_record_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, fields: &[RecordField], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_record(id, fields, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_record(id, fields, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_record(v, id, fields, span, arena)?;
      v.leave_record(id, fields, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_record<V: AstVisitor + ?Sized>(v: &mut V, _id: ExprId, fields: &[RecordField], _span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for field in fields {
    field.walk_children(v, arena)?;
  }
  ControlFlow::Continue(())
}

pub fn walk_map_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, entries: &[MapEntry], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_map(id, entries, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_map(id, entries, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_map(v, id, entries, span, arena)?;
      v.leave_map(id, entries, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_map<V: AstVisitor + ?Sized>(v: &mut V, _id: ExprId, entries: &[MapEntry], _span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for entry in entries {
    entry.walk_children(v, arena)?;
  }
  ControlFlow::Continue(())
}

pub fn walk_sel_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, arms: &[SelArm], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_sel(id, arms, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_sel(id, arms, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_sel(v, id, arms, span, arena)?;
      v.leave_sel(id, arms, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_sel<V: AstVisitor + ?Sized>(v: &mut V, _id: ExprId, arms: &[SelArm], _span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for arm in arms {
    arm.walk_children(v, arena)?;
  }
  ControlFlow::Continue(())
}
