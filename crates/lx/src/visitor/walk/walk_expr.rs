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

walk_dispatch!(walk_literal_dispatch, walk_literal, visit_literal, leave_literal, Literal);
walk_dispatch!(walk_binary_dispatch, walk_binary, visit_binary, leave_binary, ExprBinary);
walk_dispatch!(walk_unary_dispatch, walk_unary, visit_unary, leave_unary, ExprUnary);
walk_dispatch!(walk_pipe_dispatch, walk_pipe, visit_pipe, leave_pipe, ExprPipe);
walk_dispatch!(walk_apply_dispatch, walk_apply, visit_apply, leave_apply, ExprApply);
walk_dispatch!(walk_section_dispatch, walk_section, visit_section, leave_section, Section);
walk_dispatch!(walk_field_access_dispatch, walk_field_access, visit_field_access, leave_field_access, ExprFieldAccess);
walk_dispatch!(walk_func_dispatch, walk_func, visit_func, leave_func, ExprFunc);
walk_dispatch!(walk_match_dispatch, walk_match, visit_match, leave_match, ExprMatch);

walk_dispatch_slice!(walk_block_dispatch, walk_block, visit_block, leave_block, StmtId);
walk_dispatch_slice!(walk_tuple_dispatch, walk_tuple, visit_tuple, leave_tuple, ExprId);
walk_dispatch_slice!(walk_list_dispatch, walk_list, visit_list, leave_list, ListElem);
walk_dispatch_slice!(walk_record_dispatch, walk_record, visit_record, leave_record, RecordField);
walk_dispatch_slice!(walk_map_dispatch, walk_map, visit_map, leave_map, MapEntry);

pub fn walk_literal<V: AstVisitor + ?Sized>(v: &mut V, lit: &Literal, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let Literal::Str(parts) = lit {
    for part in parts {
      if let StrPart::Interp(eid) = part {
        dispatch_expr(v, arena.expr(*eid), arena.expr_span(*eid), arena)?;
      }
    }
  }
  v.leave_literal(lit, span, arena)
}

pub fn walk_binary<V: AstVisitor + ?Sized>(v: &mut V, binary: &ExprBinary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(binary.left), arena.expr_span(binary.left), arena)?;
  dispatch_expr(v, arena.expr(binary.right), arena.expr_span(binary.right), arena)?;
  v.leave_binary(binary, span, arena)
}

pub fn walk_unary<V: AstVisitor + ?Sized>(v: &mut V, unary: &ExprUnary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(unary.operand), arena.expr_span(unary.operand), arena)?;
  v.leave_unary(unary, span, arena)
}

pub fn walk_pipe<V: AstVisitor + ?Sized>(v: &mut V, pipe: &ExprPipe, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(pipe.left), arena.expr_span(pipe.left), arena)?;
  dispatch_expr(v, arena.expr(pipe.right), arena.expr_span(pipe.right), arena)?;
  v.leave_pipe(pipe, span, arena)
}

pub fn walk_apply<V: AstVisitor + ?Sized>(v: &mut V, apply: &ExprApply, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(apply.func), arena.expr_span(apply.func), arena)?;
  dispatch_expr(v, arena.expr(apply.arg), arena.expr_span(apply.arg), arena)?;
  v.leave_apply(apply, span, arena)
}

pub fn walk_section<V: AstVisitor + ?Sized>(v: &mut V, section: &Section, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match section {
    Section::Right { operand, .. } | Section::Left { operand, .. } => {
      dispatch_expr(v, arena.expr(*operand), arena.expr_span(*operand), arena)?;
    },
    _ => {},
  }
  v.leave_section(section, span, arena)
}

pub fn walk_field_access<V: AstVisitor + ?Sized>(v: &mut V, fa: &ExprFieldAccess, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(fa.expr), arena.expr_span(fa.expr), arena)?;
  if let FieldKind::Computed(c) = &fa.field {
    dispatch_expr(v, arena.expr(*c), arena.expr_span(*c), arena)?;
  }
  v.leave_field_access(fa, span, arena)
}

pub fn walk_block<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[StmtId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &s in stmts {
    super::dispatch_stmt(v, s, arena)?;
  }
  v.leave_block(stmts, span, arena)
}

pub fn walk_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[ExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &e in elems {
    dispatch_expr(v, arena.expr(e), arena.expr_span(e), arena)?;
  }
  v.leave_tuple(elems, span, arena)
}

pub fn walk_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[ListElem], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for e in elems {
    let eid = match e {
      ListElem::Single(eid) | ListElem::Spread(eid) => *eid,
    };
    dispatch_expr(v, arena.expr(eid), arena.expr_span(eid), arena)?;
  }
  v.leave_list(elems, span, arena)
}

pub fn walk_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[RecordField], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for f in fields {
    let eid = match f {
      RecordField::Named { value, .. } | RecordField::Spread(value) => *value,
    };
    dispatch_expr(v, arena.expr(eid), arena.expr_span(eid), arena)?;
  }
  v.leave_record(fields, span, arena)
}

pub fn walk_map<V: AstVisitor + ?Sized>(v: &mut V, entries: &[MapEntry], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for entry in entries {
    match entry {
      MapEntry::Keyed { key, value } => {
        dispatch_expr(v, arena.expr(*key), arena.expr_span(*key), arena)?;
        dispatch_expr(v, arena.expr(*value), arena.expr_span(*value), arena)?;
      },
      MapEntry::Spread(value) => {
        dispatch_expr(v, arena.expr(*value), arena.expr_span(*value), arena)?;
      },
    }
  }
  v.leave_map(entries, span, arena)
}

pub fn walk_func<V: AstVisitor + ?Sized>(v: &mut V, func: &ExprFunc, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for p in &func.params {
    if let Some(d) = p.default {
      dispatch_expr(v, arena.expr(d), arena.expr_span(d), arena)?;
    }
    if let Some(ty) = p.type_ann {
      walk_type_expr_dispatch(v, ty, arena)?;
    }
  }
  if let Some(rt) = func.ret_type {
    walk_type_expr_dispatch(v, rt, arena)?;
  }
  if let Some(g) = func.guard {
    dispatch_expr(v, arena.expr(g), arena.expr_span(g), arena)?;
  }
  dispatch_expr(v, arena.expr(func.body), arena.expr_span(func.body), arena)?;
  v.leave_func(func, span, arena)
}

pub fn walk_match<V: AstVisitor + ?Sized>(v: &mut V, m: &ExprMatch, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(m.scrutinee), arena.expr_span(m.scrutinee), arena)?;
  for arm in &m.arms {
    walk_pattern_dispatch(v, arm.pattern, arena)?;
    if let Some(g) = arm.guard {
      dispatch_expr(v, arena.expr(g), arena.expr_span(g), arena)?;
    }
    dispatch_expr(v, arena.expr(arm.body), arena.expr_span(arm.body), arena)?;
  }
  v.leave_match(m, span, arena)
}
