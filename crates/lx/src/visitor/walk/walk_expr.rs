use crate::ast::{FieldKind, ListElem, Literal, MapEntry, MatchArm, Param, RecordField, SExpr, SStmt, SType, Section, SelArm, StrPart};
use miette::SourceSpan;

use super::super::AstVisitor;

pub fn walk_literal<V: AstVisitor + ?Sized>(v: &mut V, lit: &Literal, _span: SourceSpan) {
  if let Literal::Str(parts) = lit {
    for part in parts {
      if let StrPart::Interp(e) = part {
        v.visit_expr(&e.node, e.span);
      }
    }
  }
}

pub fn walk_binary<V: AstVisitor + ?Sized>(v: &mut V, left: &SExpr, right: &SExpr, _span: SourceSpan) {
  v.visit_expr(&left.node, left.span);
  v.visit_expr(&right.node, right.span);
}

pub fn walk_unary<V: AstVisitor + ?Sized>(v: &mut V, operand: &SExpr, _span: SourceSpan) {
  v.visit_expr(&operand.node, operand.span);
}

pub fn walk_pipe<V: AstVisitor + ?Sized>(v: &mut V, left: &SExpr, right: &SExpr, _span: SourceSpan) {
  v.visit_expr(&left.node, left.span);
  v.visit_expr(&right.node, right.span);
}

pub fn walk_apply<V: AstVisitor + ?Sized>(v: &mut V, func: &SExpr, arg: &SExpr, _span: SourceSpan) {
  v.visit_expr(&func.node, func.span);
  v.visit_expr(&arg.node, arg.span);
}

pub fn walk_section<V: AstVisitor + ?Sized>(v: &mut V, section: &Section, _span: SourceSpan) {
  match section {
    Section::Right { operand, .. } | Section::Left { operand, .. } => {
      v.visit_expr(&operand.node, operand.span);
    },
    _ => {},
  }
}

pub fn walk_field_access<V: AstVisitor + ?Sized>(v: &mut V, expr: &SExpr, field: &FieldKind, _span: SourceSpan) {
  v.visit_expr(&expr.node, expr.span);
  if let FieldKind::Computed(c) = field {
    v.visit_expr(&c.node, c.span);
  }
}

pub fn walk_block<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], span: SourceSpan) {
  for s in stmts {
    v.visit_stmt(&s.node, s.span);
  }
  v.visit_block_post(stmts, span);
}

pub fn walk_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SExpr], _span: SourceSpan) {
  for e in elems {
    v.visit_expr(&e.node, e.span);
  }
}

pub fn walk_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[ListElem], _span: SourceSpan) {
  for e in elems {
    match e {
      ListElem::Single(se) | ListElem::Spread(se) => v.visit_expr(&se.node, se.span),
    }
  }
}

pub fn walk_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[RecordField], _span: SourceSpan) {
  for f in fields {
    v.visit_expr(&f.value.node, f.value.span);
  }
}

pub fn walk_map<V: AstVisitor + ?Sized>(v: &mut V, entries: &[MapEntry], _span: SourceSpan) {
  for e in entries {
    if let Some(ref k) = e.key {
      v.visit_expr(&k.node, k.span);
    }
    v.visit_expr(&e.value.node, e.value.span);
  }
}

pub fn walk_func<V: AstVisitor + ?Sized>(v: &mut V, params: &[Param], ret_type: Option<&SType>, body: &SExpr, span: SourceSpan) {
  for p in params {
    if let Some(ref d) = p.default {
      v.visit_expr(&d.node, d.span);
    }
    if let Some(ref ty) = p.type_ann {
      v.visit_type_expr(&ty.node, ty.span);
    }
  }
  if let Some(rt) = ret_type {
    v.visit_type_expr(&rt.node, rt.span);
  }
  v.visit_expr(&body.node, body.span);
  v.visit_func_post(params, ret_type, body, span);
}

pub fn walk_match<V: AstVisitor + ?Sized>(v: &mut V, scrutinee: &SExpr, arms: &[MatchArm], span: SourceSpan) {
  v.visit_expr(&scrutinee.node, scrutinee.span);
  for arm in arms {
    v.visit_pattern(&arm.pattern.node, arm.pattern.span);
    if let Some(ref g) = arm.guard {
      v.visit_expr(&g.node, g.span);
    }
    v.visit_expr(&arm.body.node, arm.body.span);
  }
  v.visit_match_post(scrutinee, arms, span);
}

pub fn walk_ternary<V: AstVisitor + ?Sized>(v: &mut V, cond: &SExpr, then_: &SExpr, else_: Option<&SExpr>, span: SourceSpan) {
  v.visit_expr(&cond.node, cond.span);
  v.visit_expr(&then_.node, then_.span);
  if let Some(e) = else_ {
    v.visit_expr(&e.node, e.span);
  }
  v.visit_ternary_post(cond, then_, else_, span);
}

pub fn walk_propagate<V: AstVisitor + ?Sized>(v: &mut V, inner: &SExpr, _span: SourceSpan) {
  v.visit_expr(&inner.node, inner.span);
}

pub fn walk_coalesce<V: AstVisitor + ?Sized>(v: &mut V, expr: &SExpr, default: &SExpr, _span: SourceSpan) {
  v.visit_expr(&expr.node, expr.span);
  v.visit_expr(&default.node, default.span);
}

pub fn walk_slice<V: AstVisitor + ?Sized>(v: &mut V, expr: &SExpr, start: Option<&SExpr>, end: Option<&SExpr>, _span: SourceSpan) {
  v.visit_expr(&expr.node, expr.span);
  if let Some(s) = start {
    v.visit_expr(&s.node, s.span);
  }
  if let Some(e) = end {
    v.visit_expr(&e.node, e.span);
  }
}

pub fn walk_named_arg<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, _span: SourceSpan) {
  v.visit_expr(&value.node, value.span);
}

pub fn walk_loop<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], span: SourceSpan) {
  for s in stmts {
    v.visit_stmt(&s.node, s.span);
  }
  v.visit_loop_post(stmts, span);
}

pub fn walk_break<V: AstVisitor + ?Sized>(v: &mut V, value: Option<&SExpr>, _span: SourceSpan) {
  if let Some(val) = value {
    v.visit_expr(&val.node, val.span);
  }
}

pub fn walk_assert<V: AstVisitor + ?Sized>(v: &mut V, expr: &SExpr, msg: Option<&SExpr>, _span: SourceSpan) {
  v.visit_expr(&expr.node, expr.span);
  if let Some(m) = msg {
    v.visit_expr(&m.node, m.span);
  }
}

pub fn walk_par<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], span: SourceSpan) {
  for s in stmts {
    v.visit_stmt(&s.node, s.span);
  }
  v.visit_par_post(stmts, span);
}

pub fn walk_sel<V: AstVisitor + ?Sized>(v: &mut V, arms: &[SelArm], span: SourceSpan) {
  for arm in arms {
    v.visit_expr(&arm.expr.node, arm.expr.span);
    v.visit_expr(&arm.handler.node, arm.handler.span);
  }
  v.visit_sel_post(arms, span);
}

pub fn walk_emit<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, _span: SourceSpan) {
  v.visit_expr(&value.node, value.span);
}

pub fn walk_yield<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, _span: SourceSpan) {
  v.visit_expr(&value.node, value.span);
}

pub fn walk_with<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, body: &[SStmt], _span: SourceSpan) {
  v.visit_expr(&value.node, value.span);
  for s in body {
    v.visit_stmt(&s.node, s.span);
  }
}

pub fn walk_with_resource<V: AstVisitor + ?Sized>(v: &mut V, resources: &[(SExpr, String)], body: &[SStmt], _span: SourceSpan) {
  for (r, _) in resources {
    v.visit_expr(&r.node, r.span);
  }
  for s in body {
    v.visit_stmt(&s.node, s.span);
  }
}

pub fn walk_with_context<V: AstVisitor + ?Sized>(v: &mut V, fields: &[(String, SExpr)], body: &[SStmt], _span: SourceSpan) {
  for (_, expr) in fields {
    v.visit_expr(&expr.node, expr.span);
  }
  for s in body {
    v.visit_stmt(&s.node, s.span);
  }
}
