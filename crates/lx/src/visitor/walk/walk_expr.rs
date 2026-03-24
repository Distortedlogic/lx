use std::ops::ControlFlow;

use crate::ast::{
  AstArena, ExprApply, ExprBinary, ExprBlock, ExprFieldAccess, ExprFunc, ExprId, ExprMatch, ExprPipe, ExprTuple, ExprUnary, ListElem, Literal, MapEntry,
  RecordField, Section,
};
use miette::SourceSpan;

use super::super::{AstVisitor, VisitAction};

walk_dispatch_id!(walk_literal_dispatch, walk_literal, visit_literal, leave_literal, Literal, ExprId);
walk_dispatch_id!(walk_binary_dispatch, walk_binary, visit_binary, leave_binary, ExprBinary, ExprId);
walk_dispatch_id!(walk_unary_dispatch, walk_unary, visit_unary, leave_unary, ExprUnary, ExprId);
walk_dispatch_id!(walk_pipe_dispatch, walk_pipe, visit_pipe, leave_pipe, ExprPipe, ExprId);
walk_dispatch_id!(walk_apply_dispatch, walk_apply, visit_apply, leave_apply, ExprApply, ExprId);
walk_dispatch_id!(walk_section_dispatch, walk_section, visit_section, leave_section, Section, ExprId);
walk_dispatch_id!(walk_field_access_dispatch, walk_field_access, visit_field_access, leave_field_access, ExprFieldAccess, ExprId);
walk_dispatch_id!(walk_func_dispatch, walk_func, visit_func, leave_func, ExprFunc, ExprId);
walk_dispatch_id!(walk_match_dispatch, walk_match, visit_match, leave_match, ExprMatch, ExprId);

walk_dispatch_id!(walk_block_dispatch, walk_block, visit_block, leave_block, ExprBlock, ExprId);
walk_dispatch_id!(walk_tuple_dispatch, walk_tuple, visit_tuple, leave_tuple, ExprTuple, ExprId);
walk_dispatch_id_slice!(walk_list_dispatch, walk_list, visit_list, leave_list, ListElem, ExprId);
walk_dispatch_id_slice!(walk_record_dispatch, walk_record, visit_record, leave_record, RecordField, ExprId);
walk_dispatch_id_slice!(walk_map_dispatch, walk_map, visit_map, leave_map, MapEntry, ExprId);

pub fn walk_literal<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, lit: &Literal, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  lit.walk_children(v, arena)?;
  v.leave_literal(id, lit, span);
  ControlFlow::Continue(())
}

pub fn walk_binary<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, binary: &ExprBinary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  binary.walk_children(v, arena)?;
  v.leave_binary(id, binary, span);
  ControlFlow::Continue(())
}

pub fn walk_unary<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, unary: &ExprUnary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  unary.walk_children(v, arena)?;
  v.leave_unary(id, unary, span);
  ControlFlow::Continue(())
}

pub fn walk_pipe<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, pipe: &ExprPipe, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  pipe.walk_children(v, arena)?;
  v.leave_pipe(id, pipe, span);
  ControlFlow::Continue(())
}

pub fn walk_apply<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, apply: &ExprApply, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  apply.walk_children(v, arena)?;
  v.leave_apply(id, apply, span);
  ControlFlow::Continue(())
}

pub fn walk_section<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, section: &Section, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  section.walk_children(v, arena)?;
  v.leave_section(id, section, span);
  ControlFlow::Continue(())
}

pub fn walk_field_access<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, fa: &ExprFieldAccess, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  fa.walk_children(v, arena)?;
  v.leave_field_access(id, fa, span);
  ControlFlow::Continue(())
}

pub fn walk_block<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, block: &ExprBlock, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  block.walk_children(v, arena)?;
  v.leave_block(id, block, span);
  ControlFlow::Continue(())
}

pub fn walk_tuple<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, tuple: &ExprTuple, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  tuple.walk_children(v, arena)?;
  v.leave_tuple(id, tuple, span);
  ControlFlow::Continue(())
}

pub fn walk_list<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, elems: &[ListElem], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for elem in elems {
    elem.walk_children(v, arena)?;
  }
  v.leave_list(id, elems, span);
  ControlFlow::Continue(())
}

pub fn walk_record<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, fields: &[RecordField], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for field in fields {
    field.walk_children(v, arena)?;
  }
  v.leave_record(id, fields, span);
  ControlFlow::Continue(())
}

pub fn walk_map<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, entries: &[MapEntry], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for entry in entries {
    entry.walk_children(v, arena)?;
  }
  v.leave_map(id, entries, span);
  ControlFlow::Continue(())
}

pub fn walk_func<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, func: &ExprFunc, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  func.walk_children(v, arena)?;
  v.leave_func(id, func, span);
  ControlFlow::Continue(())
}

pub fn walk_match<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, m: &ExprMatch, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  m.walk_children(v, arena)?;
  v.leave_match(id, m, span);
  ControlFlow::Continue(())
}
