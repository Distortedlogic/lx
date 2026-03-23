use std::ops::ControlFlow;

use crate::ast::{
  AstArena, ExprApply, ExprBinary, ExprFieldAccess, ExprFunc, ExprId, ExprMatch, ExprPipe, ExprUnary, FieldKind, ListElem, Literal, MapEntry, RecordField,
  Section, StmtId, StrPart,
};
use miette::SourceSpan;

use super::super::{AstVisitor, VisitAction};
use super::dispatch_expr;
use super::walk_pattern::walk_pattern_dispatch;
use super::walk_type::walk_type_expr_dispatch;

walk_dispatch_id!(walk_literal_dispatch, walk_literal, visit_literal, leave_literal, Literal, ExprId);
walk_dispatch_id!(walk_binary_dispatch, walk_binary, visit_binary, leave_binary, ExprBinary, ExprId);
walk_dispatch_id!(walk_unary_dispatch, walk_unary, visit_unary, leave_unary, ExprUnary, ExprId);
walk_dispatch_id!(walk_pipe_dispatch, walk_pipe, visit_pipe, leave_pipe, ExprPipe, ExprId);
walk_dispatch_id!(walk_apply_dispatch, walk_apply, visit_apply, leave_apply, ExprApply, ExprId);
walk_dispatch_id!(walk_section_dispatch, walk_section, visit_section, leave_section, Section, ExprId);
walk_dispatch_id!(walk_field_access_dispatch, walk_field_access, visit_field_access, leave_field_access, ExprFieldAccess, ExprId);
walk_dispatch_id!(walk_func_dispatch, walk_func, visit_func, leave_func, ExprFunc, ExprId);
walk_dispatch_id!(walk_match_dispatch, walk_match, visit_match, leave_match, ExprMatch, ExprId);

walk_dispatch_id_slice!(walk_block_dispatch, walk_block, visit_block, leave_block, StmtId, ExprId);
walk_dispatch_id_slice!(walk_tuple_dispatch, walk_tuple, visit_tuple, leave_tuple, ExprId, ExprId);
walk_dispatch_id_slice!(walk_list_dispatch, walk_list, visit_list, leave_list, ListElem, ExprId);
walk_dispatch_id_slice!(walk_record_dispatch, walk_record, visit_record, leave_record, RecordField, ExprId);
walk_dispatch_id_slice!(walk_map_dispatch, walk_map, visit_map, leave_map, MapEntry, ExprId);

pub fn walk_literal<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, lit: &Literal, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let Literal::Str(parts) = lit {
    for part in parts {
      if let StrPart::Interp(eid) = part {
        dispatch_expr(v, *eid, arena)?;
      }
    }
  }
  v.leave_literal(id, lit, span, arena)
}

pub fn walk_binary<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, binary: &ExprBinary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, binary.left, arena)?;
  dispatch_expr(v, binary.right, arena)?;
  v.leave_binary(id, binary, span, arena)
}

pub fn walk_unary<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, unary: &ExprUnary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, unary.operand, arena)?;
  v.leave_unary(id, unary, span, arena)
}

pub fn walk_pipe<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, pipe: &ExprPipe, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, pipe.left, arena)?;
  dispatch_expr(v, pipe.right, arena)?;
  v.leave_pipe(id, pipe, span, arena)
}

pub fn walk_apply<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, apply: &ExprApply, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, apply.func, arena)?;
  dispatch_expr(v, apply.arg, arena)?;
  v.leave_apply(id, apply, span, arena)
}

pub fn walk_section<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, section: &Section, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match section {
    Section::Right { operand, .. } | Section::Left { operand, .. } => {
      dispatch_expr(v, *operand, arena)?;
    },
    _ => {},
  }
  v.leave_section(id, section, span, arena)
}

pub fn walk_field_access<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, fa: &ExprFieldAccess, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, fa.expr, arena)?;
  if let FieldKind::Computed(c) = &fa.field {
    dispatch_expr(v, *c, arena)?;
  }
  v.leave_field_access(id, fa, span, arena)
}

pub fn walk_block<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, stmts: &[StmtId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &s in stmts {
    super::dispatch_stmt(v, s, arena)?;
  }
  v.leave_block(id, stmts, span, arena)
}

pub fn walk_tuple<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, elems: &[ExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &e in elems {
    dispatch_expr(v, e, arena)?;
  }
  v.leave_tuple(id, elems, span, arena)
}

pub fn walk_list<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, elems: &[ListElem], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for e in elems {
    let eid = match e {
      ListElem::Single(eid) | ListElem::Spread(eid) => *eid,
    };
    dispatch_expr(v, eid, arena)?;
  }
  v.leave_list(id, elems, span, arena)
}

pub fn walk_record<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, fields: &[RecordField], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for f in fields {
    let eid = match f {
      RecordField::Named { value, .. } | RecordField::Spread(value) => *value,
    };
    dispatch_expr(v, eid, arena)?;
  }
  v.leave_record(id, fields, span, arena)
}

pub fn walk_map<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, entries: &[MapEntry], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for entry in entries {
    match entry {
      MapEntry::Keyed { key, value } => {
        dispatch_expr(v, *key, arena)?;
        dispatch_expr(v, *value, arena)?;
      },
      MapEntry::Spread(value) => {
        dispatch_expr(v, *value, arena)?;
      },
    }
  }
  v.leave_map(id, entries, span, arena)
}

pub fn walk_func<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, func: &ExprFunc, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for p in &func.params {
    if let Some(d) = p.default {
      dispatch_expr(v, d, arena)?;
    }
    if let Some(ty) = p.type_ann {
      walk_type_expr_dispatch(v, ty, arena)?;
    }
  }
  if let Some(rt) = func.ret_type {
    walk_type_expr_dispatch(v, rt, arena)?;
  }
  if let Some(g) = func.guard {
    dispatch_expr(v, g, arena)?;
  }
  dispatch_expr(v, func.body, arena)?;
  v.leave_func(id, func, span, arena)
}

pub fn walk_match<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, m: &ExprMatch, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, m.scrutinee, arena)?;
  for arm in &m.arms {
    walk_pattern_dispatch(v, arm.pattern, arena)?;
    if let Some(g) = arm.guard {
      dispatch_expr(v, g, arena)?;
    }
    dispatch_expr(v, arm.body, arena)?;
  }
  v.leave_match(id, m, span, arena)
}
