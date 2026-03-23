use std::ops::ControlFlow;

use crate::ast::{AstArena, TypeExpr, TypeExprId, TypeField};
use crate::sym::Sym;
use miette::SourceSpan;

use crate::visitor::{AstVisitor, VisitAction};

pub(crate) fn walk_type_expr_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.type_expr_span(id);
  let type_expr = arena.type_expr(id);
  let action = v.visit_type_expr(id, type_expr, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_expr(id, type_expr, span, arena),
    VisitAction::Descend => walk_type_expr(v, id, type_expr, span, arena),
  }
}

pub fn walk_type_expr<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, type_expr: &TypeExpr, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match type_expr {
    TypeExpr::Named(name) => {
      let action = v.visit_type_named(id, *name, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    TypeExpr::Var(name) => {
      let action = v.visit_type_var(id, *name, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    TypeExpr::Applied(name, args) => walk_type_applied_dispatch(v, id, *name, args, span, arena)?,
    TypeExpr::List(inner) => walk_type_list_dispatch(v, id, *inner, span, arena)?,
    TypeExpr::Map { key, value } => walk_type_map_dispatch(v, id, *key, *value, span, arena)?,
    TypeExpr::Record(fields) => walk_type_record_dispatch(v, id, fields, span, arena)?,
    TypeExpr::Tuple(elems) => walk_type_tuple_dispatch(v, id, elems, span, arena)?,
    TypeExpr::Func { param, ret } => walk_type_func_dispatch(v, id, *param, *ret, span, arena)?,
    TypeExpr::Fallible { ok, err } => walk_type_fallible_dispatch(v, id, *ok, *err, span, arena)?,
  }
  v.leave_type_expr(id, type_expr, span, arena)
}

fn walk_type_applied_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: TypeExprId,
  name: Sym,
  args: &[TypeExprId],
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_type_applied(id, name, args, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_applied(id, name, args, span, arena),
    VisitAction::Descend => walk_type_applied(v, id, name, args, span, arena),
  }
}

pub fn walk_type_applied<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: TypeExprId,
  name: Sym,
  args: &[TypeExprId],
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  for &a in args {
    walk_type_expr_dispatch(v, a, arena)?;
  }
  v.leave_type_applied(id, name, args, span, arena)
}

fn walk_type_list_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, inner: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_list(id, inner, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_list(id, inner, span, arena),
    VisitAction::Descend => walk_type_list(v, id, inner, span, arena),
  }
}

pub fn walk_type_list<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, inner: TypeExprId, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  walk_type_expr_dispatch(v, inner, arena)?;
  v.leave_type_list(id, inner, span, arena)
}

fn walk_type_map_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: TypeExprId,
  key: TypeExprId,
  value: TypeExprId,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_type_map(id, key, value, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_map(id, key, value, span, arena),
    VisitAction::Descend => walk_type_map(v, id, key, value, span, arena),
  }
}

pub fn walk_type_map<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: TypeExprId,
  key: TypeExprId,
  value: TypeExprId,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  walk_type_expr_dispatch(v, key, arena)?;
  walk_type_expr_dispatch(v, value, arena)?;
  v.leave_type_map(id, key, value, span, arena)
}

fn walk_type_record_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, fields: &[TypeField], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_record(id, fields, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_record(id, fields, span, arena),
    VisitAction::Descend => walk_type_record(v, id, fields, span, arena),
  }
}

pub fn walk_type_record<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, fields: &[TypeField], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for f in fields {
    walk_type_expr_dispatch(v, f.ty, arena)?;
  }
  v.leave_type_record(id, fields, span, arena)
}

fn walk_type_tuple_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, elems: &[TypeExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_type_tuple(id, elems, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_tuple(id, elems, span, arena),
    VisitAction::Descend => walk_type_tuple(v, id, elems, span, arena),
  }
}

pub fn walk_type_tuple<V: AstVisitor + ?Sized>(v: &mut V, id: TypeExprId, elems: &[TypeExprId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &e in elems {
    walk_type_expr_dispatch(v, e, arena)?;
  }
  v.leave_type_tuple(id, elems, span, arena)
}

fn walk_type_func_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: TypeExprId,
  param: TypeExprId,
  ret: TypeExprId,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_type_func(id, param, ret, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_func(id, param, ret, span, arena),
    VisitAction::Descend => walk_type_func(v, id, param, ret, span, arena),
  }
}

pub fn walk_type_func<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: TypeExprId,
  param: TypeExprId,
  ret: TypeExprId,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  walk_type_expr_dispatch(v, param, arena)?;
  walk_type_expr_dispatch(v, ret, arena)?;
  v.leave_type_func(id, param, ret, span, arena)
}

fn walk_type_fallible_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: TypeExprId,
  ok: TypeExprId,
  err: TypeExprId,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_type_fallible(id, ok, err, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_type_fallible(id, ok, err, span, arena),
    VisitAction::Descend => walk_type_fallible(v, id, ok, err, span, arena),
  }
}

pub fn walk_type_fallible<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: TypeExprId,
  ok: TypeExprId,
  err: TypeExprId,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  walk_type_expr_dispatch(v, ok, arena)?;
  walk_type_expr_dispatch(v, err, arena)?;
  v.leave_type_fallible(id, ok, err, span, arena)
}
