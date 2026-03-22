use std::ops::ControlFlow;

use crate::ast::{SType, TypeExpr, TypeField};
use crate::sym::Sym;
use miette::SourceSpan;

use crate::visitor::AstVisitor;

pub fn walk_type_expr<V: AstVisitor + ?Sized>(v: &mut V, type_expr: &TypeExpr, span: SourceSpan) -> ControlFlow<()> {
  match type_expr {
    TypeExpr::Named(name) => v.visit_type_named(*name, span)?,
    TypeExpr::Var(name) => v.visit_type_var(*name, span)?,
    TypeExpr::Applied(name, args) => v.visit_type_applied(*name, args, span)?,
    TypeExpr::List(inner) => v.visit_type_list(inner, span)?,
    TypeExpr::Map { key, value } => v.visit_type_map(key, value, span)?,
    TypeExpr::Record(fields) => v.visit_type_record(fields, span)?,
    TypeExpr::Tuple(elems) => v.visit_type_tuple(elems, span)?,
    TypeExpr::Func { param, ret } => v.visit_type_func(param, ret, span)?,
    TypeExpr::Fallible { ok, err } => v.visit_type_fallible(ok, err, span)?,
  }
  v.leave_type_expr(type_expr, span)
}

pub fn walk_type_applied<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, args: &[SType], span: SourceSpan) -> ControlFlow<()> {
  for a in args {
    v.visit_type_expr(&a.node, a.span)?;
  }
  v.leave_type_applied(name, args, span)
}

pub fn walk_type_list<V: AstVisitor + ?Sized>(v: &mut V, inner: &SType, span: SourceSpan) -> ControlFlow<()> {
  v.visit_type_expr(&inner.node, inner.span)?;
  v.leave_type_list(inner, span)
}

pub fn walk_type_map<V: AstVisitor + ?Sized>(v: &mut V, key: &SType, value: &SType, span: SourceSpan) -> ControlFlow<()> {
  v.visit_type_expr(&key.node, key.span)?;
  v.visit_type_expr(&value.node, value.span)?;
  v.leave_type_map(key, value, span)
}

pub fn walk_type_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[TypeField], span: SourceSpan) -> ControlFlow<()> {
  for f in fields {
    v.visit_type_expr(&f.ty.node, f.ty.span)?;
  }
  v.leave_type_record(fields, span)
}

pub fn walk_type_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SType], span: SourceSpan) -> ControlFlow<()> {
  for e in elems {
    v.visit_type_expr(&e.node, e.span)?;
  }
  v.leave_type_tuple(elems, span)
}

pub fn walk_type_func<V: AstVisitor + ?Sized>(v: &mut V, param: &SType, ret: &SType, span: SourceSpan) -> ControlFlow<()> {
  v.visit_type_expr(&param.node, param.span)?;
  v.visit_type_expr(&ret.node, ret.span)?;
  v.leave_type_func(param, ret, span)
}

pub fn walk_type_fallible<V: AstVisitor + ?Sized>(v: &mut V, ok: &SType, err: &SType, span: SourceSpan) -> ControlFlow<()> {
  v.visit_type_expr(&ok.node, ok.span)?;
  v.visit_type_expr(&err.node, err.span)?;
  v.leave_type_fallible(ok, err, span)
}
