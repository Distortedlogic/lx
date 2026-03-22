use std::ops::ControlFlow;

use crate::ast::{AstArena, BindTarget, Binding, ClassDeclData, Expr, Program, Stmt, StmtFieldUpdate, StmtId, TraitDeclData, TraitEntry};
use miette::SourceSpan;

use super::{AstVisitor, VisitAction};

macro_rules! walk_dispatch {
  ($dispatch_name:ident, $walk_name:ident, $visit:ident, $leave:ident, $node_ty:ty) => {
    pub(crate) fn $dispatch_name<V: AstVisitor + ?Sized>(v: &mut V, node: &$node_ty, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
      let action = v.$visit(node, span, arena);
      match action {
        VisitAction::Stop => ControlFlow::Break(()),
        VisitAction::Skip => v.$leave(node, span, arena),
        VisitAction::Descend => $walk_name(v, node, span, arena),
      }
    }
  };
}

macro_rules! walk_dispatch_slice {
  ($dispatch_name:ident, $walk_name:ident, $visit:ident, $leave:ident, $elem_ty:ty) => {
    pub(crate) fn $dispatch_name<V: AstVisitor + ?Sized>(v: &mut V, elems: &[$elem_ty], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
      let action = v.$visit(elems, span, arena);
      match action {
        VisitAction::Stop => ControlFlow::Break(()),
        VisitAction::Skip => v.$leave(elems, span, arena),
        VisitAction::Descend => $walk_name(v, elems, span, arena),
      }
    }
  };
}

mod walk_expr;
mod walk_expr2;
mod walk_pattern;
mod walk_type;

pub use walk_expr::*;
pub use walk_expr2::*;
pub use walk_pattern::*;
pub use walk_type::*;

walk_dispatch!(walk_trait_decl_dispatch, walk_trait_decl, visit_trait_decl, leave_trait_decl, TraitDeclData);
walk_dispatch!(walk_class_decl_dispatch, walk_class_decl, visit_class_decl, leave_class_decl, ClassDeclData);
walk_dispatch!(walk_field_update_dispatch, walk_field_update, visit_field_update, leave_field_update, StmtFieldUpdate);

pub fn walk_program<V: AstVisitor + ?Sized, P>(v: &mut V, program: &Program<P>) -> ControlFlow<()> {
  let arena = &program.arena;
  for &sid in &program.stmts {
    dispatch_stmt(v, sid, arena)?;
  }
  v.leave_program(program)
}

pub fn dispatch_stmt<V: AstVisitor + ?Sized>(v: &mut V, id: StmtId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.stmt_span(id);
  let stmt = arena.stmt(id);
  let action = v.on_stmt(stmt, span, arena);
  match action {
    VisitAction::Stop => return ControlFlow::Break(()),
    VisitAction::Skip => return v.leave_stmt(stmt, span, arena),
    VisitAction::Descend => {},
  }
  walk_stmt(v, id, arena)?;
  v.leave_stmt(stmt, span, arena)
}

pub fn walk_stmt<V: AstVisitor + ?Sized>(v: &mut V, id: StmtId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.stmt_span(id);
  let stmt = arena.stmt(id);
  match stmt {
    Stmt::Binding(binding) => {
      let action = v.visit_binding(binding, span, arena);
      match action {
        VisitAction::Stop => return ControlFlow::Break(()),
        VisitAction::Skip => {},
        VisitAction::Descend => {
          walk_binding(v, binding, span, arena)?;
        },
      }
    },
    Stmt::TypeDef(def) => {
      let action = v.visit_type_def(def, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Stmt::TraitUnion(def) => {
      let action = v.visit_trait_union(def, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Stmt::TraitDecl(data) => walk_trait_decl_dispatch(v, data, span, arena)?,
    Stmt::ClassDecl(data) => walk_class_decl_dispatch(v, data, span, arena)?,
    Stmt::FieldUpdate(fu) => walk_field_update_dispatch(v, fu, span, arena)?,
    Stmt::Use(use_stmt) => {
      let action = v.visit_use(use_stmt, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Stmt::Expr(eid) => {
      let espan = arena.expr_span(*eid);
      dispatch_expr(v, arena.expr(*eid), espan, arena)?;
    },
  }
  ControlFlow::Continue(())
}

pub fn walk_binding<V: AstVisitor + ?Sized>(v: &mut V, binding: &Binding, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let Some(ty_id) = binding.type_ann {
    walk_type_expr_dispatch(v, ty_id, arena)?;
  }
  if let BindTarget::Pattern(pid) = &binding.target {
    walk_pattern_dispatch(v, *pid, arena)?;
  }
  let val_span = arena.expr_span(binding.value);
  dispatch_expr(v, arena.expr(binding.value), val_span, arena)?;
  v.leave_binding(binding, span, arena)
}

pub fn dispatch_expr<V: AstVisitor + ?Sized>(v: &mut V, expr: &Expr, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.on_expr(expr, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_expr(expr, span, arena),
    VisitAction::Descend => walk_expr(v, expr, span, arena),
  }
}

pub fn walk_expr<V: AstVisitor + ?Sized>(v: &mut V, expr: &Expr, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match expr {
    Expr::Literal(lit) => walk_literal_dispatch(v, lit, span, arena)?,
    Expr::Ident(name) => {
      let action = v.visit_ident(*name, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Expr::TypeConstructor(name) => {
      let action = v.visit_type_constructor(*name, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Expr::Binary(binary) => walk_binary_dispatch(v, binary, span, arena)?,
    Expr::Unary(unary) => walk_unary_dispatch(v, unary, span, arena)?,
    Expr::Pipe(pipe) => walk_pipe_dispatch(v, pipe, span, arena)?,
    Expr::Apply(apply) => walk_apply_dispatch(v, apply, span, arena)?,
    Expr::Section(section) => walk_section_dispatch(v, section, span, arena)?,
    Expr::FieldAccess(fa) => walk_field_access_dispatch(v, fa, span, arena)?,
    Expr::Block(stmts) => walk_block_dispatch(v, stmts, span, arena)?,
    Expr::Tuple(elems) => walk_tuple_dispatch(v, elems, span, arena)?,
    Expr::List(elems) => walk_list_dispatch(v, elems, span, arena)?,
    Expr::Record(fields) => walk_record_dispatch(v, fields, span, arena)?,
    Expr::Map(entries) => walk_map_dispatch(v, entries, span, arena)?,
    Expr::Func(func) => walk_func_dispatch(v, func, span, arena)?,
    Expr::Match(m) => walk_match_dispatch(v, m, span, arena)?,
    Expr::Ternary(ternary) => walk_ternary_dispatch(v, ternary, span, arena)?,
    Expr::Propagate(inner) => walk_propagate_dispatch(v, *inner, span, arena)?,
    Expr::Coalesce(coalesce) => walk_coalesce_dispatch(v, coalesce, span, arena)?,
    Expr::Slice(slice) => walk_slice_dispatch(v, slice, span, arena)?,
    Expr::NamedArg(na) => walk_named_arg_dispatch(v, na, span, arena)?,
    Expr::Loop(stmts) => walk_loop_dispatch(v, stmts, span, arena)?,
    Expr::Break(val) => walk_break_dispatch(v, *val, span, arena)?,
    Expr::Assert(assert) => walk_assert_dispatch(v, assert, span, arena)?,
    Expr::Par(stmts) => walk_par_dispatch(v, stmts, span, arena)?,
    Expr::Sel(arms) => walk_sel_dispatch(v, arms, span, arena)?,
    Expr::Timeout(timeout) => walk_timeout_dispatch(v, timeout, span, arena)?,
    Expr::Emit(emit) => walk_emit_dispatch(v, emit, span, arena)?,
    Expr::Yield(yld) => walk_yield_dispatch(v, yld, span, arena)?,
    Expr::With(with) => walk_with_dispatch(v, with, span, arena)?,
  }
  v.leave_expr(expr, span, arena)
}

pub fn walk_trait_decl<V: AstVisitor + ?Sized>(v: &mut V, data: &TraitDeclData, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for entry in &data.entries {
    if let TraitEntry::Field(field) = entry {
      if let Some(default) = field.default {
        dispatch_expr(v, arena.expr(default), arena.expr_span(default), arena)?;
      }
      if let Some(constraint) = field.constraint {
        dispatch_expr(v, arena.expr(constraint), arena.expr_span(constraint), arena)?;
      }
    }
  }
  for method in &data.methods {
    for input in &method.input {
      if let Some(d) = input.default {
        dispatch_expr(v, arena.expr(d), arena.expr_span(d), arena)?;
      }
      if let Some(c) = input.constraint {
        dispatch_expr(v, arena.expr(c), arena.expr_span(c), arena)?;
      }
    }
  }
  for method in &data.defaults {
    dispatch_expr(v, arena.expr(method.handler), arena.expr_span(method.handler), arena)?;
  }
  v.leave_trait_decl(data, span, arena)
}

pub fn walk_class_decl<V: AstVisitor + ?Sized>(v: &mut V, data: &ClassDeclData, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for f in &data.fields {
    dispatch_expr(v, arena.expr(f.default), arena.expr_span(f.default), arena)?;
  }
  for m in &data.methods {
    dispatch_expr(v, arena.expr(m.handler), arena.expr_span(m.handler), arena)?;
  }
  v.leave_class_decl(data, span, arena)
}

pub fn walk_field_update<V: AstVisitor + ?Sized>(v: &mut V, fu: &StmtFieldUpdate, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  dispatch_expr(v, arena.expr(fu.value), arena.expr_span(fu.value), arena)?;
  v.leave_field_update(fu, span, arena)
}
