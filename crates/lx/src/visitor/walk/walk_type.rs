use std::ops::ControlFlow;

use crate::ast::{AstArena, TypeExpr, TypeExprId, TypeField};
use crate::sym::Sym;
use miette::SourceSpan;

use crate::visitor::{AstVisitor, VisitAction};

pub(crate) fn walk_type_expr_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.type_expr_span(id);
  let type_expr = arena.type_expr(id);
  let action = v.visit_type_expr(type_expr, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_expr(type_expr, span, arena),
    VisitAction::Descend => walk_type_expr(v, type_expr, span, arena),
  }
}

pub fn walk_type_expr<V: AstVisitor + ?Sized>(v: &mut V, type_expr: &TypeExpr, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match type_expr {
    TypeExpr::Named(name) => {
      let action = v.visit_type_named(*name, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    TypeExpr::Var(name) => {
      let action = v.visit_type_var(*name, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    TypeExpr::Applied(name, args) => walk_type_applied_dispatch(v, *name, args, span, arena)?,
    TypeExpr::List(inner) => walk_type_list_dispatch(v, *inner, span, arena)?,
    TypeExpr::Map { key, value } => walk_type_map_dispatch(v, *key, *value, span, arena)?,
    TypeExpr::Record(fields) => walk_type_record_dispatch(v, fields, span, arena)?,
    TypeExpr::Tuple(elems) => walk_type_tuple_dispatch(v, elems, span, arena)?,
    TypeExpr::Func { param, ret } => walk_type_func_dispatch(v, *param, *ret, span, arena)?,
    TypeExpr::Fallible { ok, err } => walk_type_fallible_dispatch(v, *ok, *err, span, arena)?,
  }
  v.leave_type_expr(type_expr, span, arena)
}

fn walk_type_applied_dispatch<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, args: &[TypeExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_applied(name, args, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_applied(name, args, span, arena),
    VisitAction::Descend => walk_type_applied(v, name, args, span, arena),
  }
}

pub fn walk_type_applied<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, args: &[TypeExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &a in args {
    walk_type_expr_dispatch(v, a, arena)?;
  }
  v.leave_type_applied(name, args, span, arena)
}

fn walk_type_list_dispatch<V: AstVisitor + ?Sized>(v: &mut V, inner: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_list(inner, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_list(inner, span, arena),
    VisitAction::Descend => walk_type_list(v, inner, span, arena),
  }
}

pub fn walk_type_list<V: AstVisitor + ?Sized>(v: &mut V, inner: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  walk_type_expr_dispatch(v, inner, arena)?;
  v.leave_type_list(inner, span, arena)
}

fn walk_type_map_dispatch<V: AstVisitor + ?Sized>(v: &mut V, key: TypeExprId, value: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_map(key, value, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_map(key, value, span, arena),
    VisitAction::Descend => walk_type_map(v, key, value, span, arena),
  }
}

pub fn walk_type_map<V: AstVisitor + ?Sized>(v: &mut V, key: TypeExprId, value: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  walk_type_expr_dispatch(v, key, arena)?;
  walk_type_expr_dispatch(v, value, arena)?;
  v.leave_type_map(key, value, span, arena)
}

fn walk_type_record_dispatch<V: AstVisitor + ?Sized>(v: &mut V, fields: &[TypeField], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_record(fields, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_record(fields, span, arena),
    VisitAction::Descend => walk_type_record(v, fields, span, arena),
  }
}

pub fn walk_type_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[TypeField], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for f in fields {
    walk_type_expr_dispatch(v, f.ty, arena)?;
  }
  v.leave_type_record(fields, span, arena)
}

fn walk_type_tuple_dispatch<V: AstVisitor + ?Sized>(v: &mut V, elems: &[TypeExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_tuple(elems, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_tuple(elems, span, arena),
    VisitAction::Descend => walk_type_tuple(v, elems, span, arena),
  }
}

pub fn walk_type_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[TypeExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &e in elems {
    walk_type_expr_dispatch(v, e, arena)?;
  }
  v.leave_type_tuple(elems, span, arena)
}

fn walk_type_func_dispatch<V: AstVisitor + ?Sized>(v: &mut V, param: TypeExprId, ret: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_func(param, ret, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_func(param, ret, span, arena),
    VisitAction::Descend => walk_type_func(v, param, ret, span, arena),
  }
}

pub fn walk_type_func<V: AstVisitor + ?Sized>(v: &mut V, param: TypeExprId, ret: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  walk_type_expr_dispatch(v, param, arena)?;
  walk_type_expr_dispatch(v, ret, arena)?;
  v.leave_type_func(param, ret, span, arena)
}

fn walk_type_fallible_dispatch<V: AstVisitor + ?Sized>(v: &mut V, ok: TypeExprId, err: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_fallible(ok, err, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_fallible(ok, err, span, arena),
    VisitAction::Descend => walk_type_fallible(v, ok, err, span, arena),
  }
}

pub fn walk_type_fallible<V: AstVisitor + ?Sized>(v: &mut V, ok: TypeExprId, err: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  walk_type_expr_dispatch(v, ok, arena)?;
  walk_type_expr_dispatch(v, err, arena)?;
  v.leave_type_fallible(ok, err, span, arena)
}
