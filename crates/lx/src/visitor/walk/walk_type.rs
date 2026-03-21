use crate::ast::{SType, TypeExpr, TypeField};
use crate::span::Span;

use crate::visitor::AstVisitor;

pub fn walk_type_expr<V: AstVisitor + ?Sized>(v: &mut V, type_expr: &TypeExpr, span: Span) {
  match type_expr {
    TypeExpr::Named(name) => v.visit_type_named(name, span),
    TypeExpr::Var(name) => v.visit_type_var(name, span),
    TypeExpr::Applied(name, args) => v.visit_type_applied(name, args, span),
    TypeExpr::List(inner) => v.visit_type_list(inner, span),
    TypeExpr::Map { key, value } => v.visit_type_map(key, value, span),
    TypeExpr::Record(fields) => v.visit_type_record(fields, span),
    TypeExpr::Tuple(elems) => v.visit_type_tuple(elems, span),
    TypeExpr::Func { param, ret } => v.visit_type_func(param, ret, span),
    TypeExpr::Fallible { ok, err } => v.visit_type_fallible(ok, err, span),
  }
}

pub fn walk_type_applied<V: AstVisitor + ?Sized>(v: &mut V, args: &[SType], _span: Span) {
  for a in args {
    v.visit_type_expr(&a.node, a.span);
  }
}

pub fn walk_type_list<V: AstVisitor + ?Sized>(v: &mut V, inner: &SType, _span: Span) {
  v.visit_type_expr(&inner.node, inner.span);
}

pub fn walk_type_map<V: AstVisitor + ?Sized>(v: &mut V, key: &SType, value: &SType, _span: Span) {
  v.visit_type_expr(&key.node, key.span);
  v.visit_type_expr(&value.node, value.span);
}

pub fn walk_type_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[TypeField], _span: Span) {
  for f in fields {
    v.visit_type_expr(&f.ty.node, f.ty.span);
  }
}

pub fn walk_type_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SType], _span: Span) {
  for e in elems {
    v.visit_type_expr(&e.node, e.span);
  }
}

pub fn walk_type_func<V: AstVisitor + ?Sized>(v: &mut V, param: &SType, ret: &SType, _span: Span) {
  v.visit_type_expr(&param.node, param.span);
  v.visit_type_expr(&ret.node, ret.span);
}

pub fn walk_type_fallible<V: AstVisitor + ?Sized>(v: &mut V, ok: &SType, err: &SType, _span: Span) {
  v.visit_type_expr(&ok.node, ok.span);
  v.visit_type_expr(&err.node, err.span);
}
