use std::ops::ControlFlow;

use crate::ast::{
  BinOp, FieldKind, ListElem, Literal, MapEntry, MatchArm, Param, RecordField, SExpr, SStmt, SType, Section, SelArm, StrPart, UnaryOp,
};
use crate::sym::Sym;
use miette::SourceSpan;

use super::super::AstVisitor;

pub fn walk_literal<V: AstVisitor + ?Sized>(v: &mut V, lit: &Literal, span: SourceSpan) -> ControlFlow<()> {
  if let Literal::Str(parts) = lit {
    for part in parts {
      if let StrPart::Interp(e) = part {
        v.visit_expr(&e.node, e.span)?;
      }
    }
  }
  v.leave_literal(lit, span)
}

pub fn walk_binary<V: AstVisitor + ?Sized>(v: &mut V, op: BinOp, left: &SExpr, right: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&left.node, left.span)?;
  v.visit_expr(&right.node, right.span)?;
  v.leave_binary(op, left, right, span)
}

pub fn walk_unary<V: AstVisitor + ?Sized>(v: &mut V, op: UnaryOp, operand: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&operand.node, operand.span)?;
  v.leave_unary(op, operand, span)
}

pub fn walk_pipe<V: AstVisitor + ?Sized>(v: &mut V, left: &SExpr, right: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&left.node, left.span)?;
  v.visit_expr(&right.node, right.span)?;
  v.leave_pipe(left, right, span)
}

pub fn walk_apply<V: AstVisitor + ?Sized>(v: &mut V, func: &SExpr, arg: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&func.node, func.span)?;
  v.visit_expr(&arg.node, arg.span)?;
  v.leave_apply(func, arg, span)
}

pub fn walk_section<V: AstVisitor + ?Sized>(v: &mut V, section: &Section, span: SourceSpan) -> ControlFlow<()> {
  match section {
    Section::Right { operand, .. } | Section::Left { operand, .. } => {
      v.visit_expr(&operand.node, operand.span)?;
    },
    _ => {},
  }
  v.leave_section(section, span)
}

pub fn walk_field_access<V: AstVisitor + ?Sized>(v: &mut V, expr: &SExpr, field: &FieldKind, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&expr.node, expr.span)?;
  if let FieldKind::Computed(c) = field {
    v.visit_expr(&c.node, c.span)?;
  }
  v.leave_field_access(expr, field, span)
}

pub fn walk_block<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
  for s in stmts {
    v.visit_stmt(&s.node, s.span)?;
  }
  v.leave_block(stmts, span)
}

pub fn walk_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SExpr], span: SourceSpan) -> ControlFlow<()> {
  for e in elems {
    v.visit_expr(&e.node, e.span)?;
  }
  v.leave_tuple(elems, span)
}

pub fn walk_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[ListElem], span: SourceSpan) -> ControlFlow<()> {
  for e in elems {
    match e {
      ListElem::Single(se) | ListElem::Spread(se) => v.visit_expr(&se.node, se.span)?,
    };
  }
  v.leave_list(elems, span)
}

pub fn walk_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[RecordField], span: SourceSpan) -> ControlFlow<()> {
  for f in fields {
    v.visit_expr(&f.value.node, f.value.span)?;
  }
  v.leave_record(fields, span)
}

pub fn walk_map<V: AstVisitor + ?Sized>(v: &mut V, entries: &[MapEntry], span: SourceSpan) -> ControlFlow<()> {
  for e in entries {
    if let Some(ref k) = e.key {
      v.visit_expr(&k.node, k.span)?;
    }
    v.visit_expr(&e.value.node, e.value.span)?;
  }
  v.leave_map(entries, span)
}

pub fn walk_func<V: AstVisitor + ?Sized>(v: &mut V, params: &[Param], ret_type: Option<&SType>, guard: Option<&SExpr>, body: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  for p in params {
    if let Some(ref d) = p.default {
      v.visit_expr(&d.node, d.span)?;
    }
    if let Some(ref ty) = p.type_ann {
      v.visit_type_expr(&ty.node, ty.span)?;
    }
  }
  if let Some(rt) = ret_type {
    v.visit_type_expr(&rt.node, rt.span)?;
  }
  if let Some(g) = guard {
    v.visit_expr(&g.node, g.span)?;
  }
  v.visit_expr(&body.node, body.span)?;
  v.leave_func(params, ret_type, guard, body, span)
}

pub fn walk_match<V: AstVisitor + ?Sized>(v: &mut V, scrutinee: &SExpr, arms: &[MatchArm], span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&scrutinee.node, scrutinee.span)?;
  for arm in arms {
    v.visit_pattern(&arm.pattern.node, arm.pattern.span)?;
    if let Some(ref g) = arm.guard {
      v.visit_expr(&g.node, g.span)?;
    }
    v.visit_expr(&arm.body.node, arm.body.span)?;
  }
  v.leave_match(scrutinee, arms, span)
}

pub fn walk_ternary<V: AstVisitor + ?Sized>(v: &mut V, cond: &SExpr, then_: &SExpr, else_: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&cond.node, cond.span)?;
  v.visit_expr(&then_.node, then_.span)?;
  if let Some(e) = else_ {
    v.visit_expr(&e.node, e.span)?;
  }
  v.leave_ternary(cond, then_, else_, span)
}

pub fn walk_propagate<V: AstVisitor + ?Sized>(v: &mut V, inner: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&inner.node, inner.span)?;
  v.leave_propagate(inner, span)
}

pub fn walk_coalesce<V: AstVisitor + ?Sized>(v: &mut V, expr: &SExpr, default: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&expr.node, expr.span)?;
  v.visit_expr(&default.node, default.span)?;
  v.leave_coalesce(expr, default, span)
}

pub fn walk_slice<V: AstVisitor + ?Sized>(v: &mut V, expr: &SExpr, start: Option<&SExpr>, end: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&expr.node, expr.span)?;
  if let Some(s) = start {
    v.visit_expr(&s.node, s.span)?;
  }
  if let Some(e) = end {
    v.visit_expr(&e.node, e.span)?;
  }
  v.leave_slice(expr, start, end, span)
}

pub fn walk_named_arg<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, value: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&value.node, value.span)?;
  v.leave_named_arg(name, value, span)
}

pub fn walk_loop<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
  for s in stmts {
    v.visit_stmt(&s.node, s.span)?;
  }
  v.leave_loop(stmts, span)
}

pub fn walk_break<V: AstVisitor + ?Sized>(v: &mut V, value: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
  if let Some(val) = value {
    v.visit_expr(&val.node, val.span)?;
  }
  v.leave_break(value, span)
}

pub fn walk_assert<V: AstVisitor + ?Sized>(v: &mut V, expr: &SExpr, msg: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&expr.node, expr.span)?;
  if let Some(m) = msg {
    v.visit_expr(&m.node, m.span)?;
  }
  v.leave_assert(expr, msg, span)
}

pub fn walk_par<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
  for s in stmts {
    v.visit_stmt(&s.node, s.span)?;
  }
  v.leave_par(stmts, span)
}

pub fn walk_timeout<V: AstVisitor + ?Sized>(v: &mut V, ms: &SExpr, body: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&ms.node, ms.span)?;
  v.visit_expr(&body.node, body.span)?;
  v.leave_timeout(ms, body, span)
}

pub fn walk_sel<V: AstVisitor + ?Sized>(v: &mut V, arms: &[SelArm], span: SourceSpan) -> ControlFlow<()> {
  for arm in arms {
    v.visit_expr(&arm.expr.node, arm.expr.span)?;
    v.visit_expr(&arm.handler.node, arm.handler.span)?;
  }
  v.leave_sel(arms, span)
}

pub fn walk_emit<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&value.node, value.span)?;
  v.leave_emit(value, span)
}

pub fn walk_yield<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&value.node, value.span)?;
  v.leave_yield(value, span)
}

pub fn walk_with<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, value: &SExpr, body: &[SStmt], mutable: bool, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&value.node, value.span)?;
  for s in body {
    v.visit_stmt(&s.node, s.span)?;
  }
  v.leave_with(name, value, body, mutable, span)
}

pub fn walk_with_resource<V: AstVisitor + ?Sized>(v: &mut V, resources: &[(SExpr, Sym)], body: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
  for (r, _) in resources {
    v.visit_expr(&r.node, r.span)?;
  }
  for s in body {
    v.visit_stmt(&s.node, s.span)?;
  }
  v.leave_with_resource(resources, body, span)
}

pub fn walk_with_context<V: AstVisitor + ?Sized>(v: &mut V, fields: &[(Sym, SExpr)], body: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
  for (_, expr) in fields {
    v.visit_expr(&expr.node, expr.span)?;
  }
  for s in body {
    v.visit_stmt(&s.node, s.span)?;
  }
  v.leave_with_context(fields, body, span)
}
