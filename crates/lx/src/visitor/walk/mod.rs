use std::ops::ControlFlow;

use crate::ast::{AstArena, BindTarget, Binding, ClassDeclData, Expr, ExprId, NodeId, Program, Stmt, StmtFieldUpdate, StmtId, TraitDeclData};
use miette::SourceSpan;

use super::{AstVisitor, VisitAction};

macro_rules! define_walk_and_dispatch {
  ($dispatch_name:ident, $walk_name:ident, $visit:ident, $leave:ident, $node_ty:ty, $id_ty:ty) => {
    pub fn $dispatch_name<V: AstVisitor + ?Sized>(v: &mut V, id: $id_ty, node: &$node_ty, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
      let action = v.$visit(id, node, span);
      match action {
        VisitAction::Stop => ControlFlow::Break(()),
        VisitAction::Skip => {
          v.$leave(id, node, span);
          ControlFlow::Continue(())
        },
        VisitAction::Descend => {
          $walk_name(v, id, node, span, arena)?;
          v.$leave(id, node, span);
          ControlFlow::Continue(())
        },
      }
    }

    pub fn $walk_name<V: AstVisitor + ?Sized>(v: &mut V, _id: $id_ty, node: &$node_ty, _span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
      node.walk_children(v, arena)?;
      ControlFlow::Continue(())
    }
  };
}

mod generated;
mod walk_pattern;
mod walk_type;

pub use generated::*;
pub use walk_pattern::*;
pub use walk_type::*;

define_walk_and_dispatch!(walk_trait_decl_dispatch, walk_trait_decl, visit_trait_decl, leave_trait_decl, TraitDeclData, StmtId);
define_walk_and_dispatch!(walk_class_decl_dispatch, walk_class_decl, visit_class_decl, leave_class_decl, ClassDeclData, StmtId);
define_walk_and_dispatch!(walk_field_update_dispatch, walk_field_update, visit_field_update, leave_field_update, StmtFieldUpdate, StmtId);

fn dispatch_child<V: AstVisitor + ?Sized>(v: &mut V, child: NodeId, arena: &AstArena) -> ControlFlow<()> {
  match child {
    NodeId::Expr(id) => dispatch_expr(v, id, arena),
    NodeId::Stmt(id) => dispatch_stmt(v, id, arena),
    NodeId::Pattern(id) => walk_pattern_dispatch(v, id, arena),
    NodeId::TypeExpr(id) => walk_type_expr_dispatch(v, id, arena),
  }
}

pub fn dispatch_children<V: AstVisitor + ?Sized>(v: &mut V, children: &[NodeId], arena: &AstArena) -> ControlFlow<()> {
  for &child in children {
    dispatch_child(v, child, arena)?;
  }
  ControlFlow::Continue(())
}

pub fn walk_program<V: AstVisitor + ?Sized, P>(v: &mut V, program: &Program<P>) -> ControlFlow<()> {
  let action = v.visit_program(program);
  match action {
    VisitAction::Stop => return ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_program(program);
      return ControlFlow::Continue(());
    },
    VisitAction::Descend => {},
  }
  let arena = &program.arena;
  for &sid in &program.stmts {
    dispatch_stmt(v, sid, arena)?;
  }
  v.leave_program(program);
  ControlFlow::Continue(())
}

pub fn dispatch_stmt<V: AstVisitor + ?Sized>(v: &mut V, id: StmtId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.stmt_span(id);
  let stmt = arena.stmt(id);
  let action = v.visit_stmt(id, stmt, span);
  match action {
    VisitAction::Stop => return ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_stmt(id, stmt, span);
      return ControlFlow::Continue(());
    },
    VisitAction::Descend => {},
  }
  walk_stmt(v, id, arena)?;
  v.leave_stmt(id, stmt, span);
  ControlFlow::Continue(())
}

pub fn walk_stmt<V: AstVisitor + ?Sized>(v: &mut V, id: StmtId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.stmt_span(id);
  let stmt = arena.stmt(id);
  match stmt {
    Stmt::Binding(binding) => {
      let action = v.visit_binding(id, binding, span);
      match action {
        VisitAction::Stop => return ControlFlow::Break(()),
        VisitAction::Skip => {},
        VisitAction::Descend => {
          walk_binding(v, id, binding, span, arena)?;
        },
      }
    },
    Stmt::TypeDef(def) => {
      let action = v.visit_type_def(id, def, span);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Stmt::TraitUnion(def) => {
      let action = v.visit_trait_union(id, def, span);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Stmt::TraitDecl(data) => walk_trait_decl_dispatch(v, id, data, span, arena)?,
    Stmt::ClassDecl(data) => walk_class_decl_dispatch(v, id, data, span, arena)?,
    Stmt::FieldUpdate(fu) => walk_field_update_dispatch(v, id, fu, span, arena)?,
    Stmt::Use(use_stmt) => {
      let action = v.visit_use(id, use_stmt, span);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Stmt::Expr(eid) => {
      dispatch_expr(v, *eid, arena)?;
    },
  }
  ControlFlow::Continue(())
}

pub fn walk_binding<V: AstVisitor + ?Sized>(v: &mut V, id: StmtId, binding: &Binding, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  if let Some(ty_id) = binding.type_ann {
    walk_type_expr_dispatch(v, ty_id, arena)?;
  }
  if let BindTarget::Pattern(pid) = &binding.target {
    walk_pattern_dispatch(v, *pid, arena)?;
  }
  dispatch_expr(v, binding.value, arena)?;
  v.leave_binding(id, binding, span);
  ControlFlow::Continue(())
}

pub fn dispatch_expr<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, arena: &AstArena) -> ControlFlow<()> {
  let expr = arena.expr(id);
  let span = arena.expr_span(id);
  let action = v.visit_expr(id, expr, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_expr(id, expr, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_expr(v, id, arena)?;
      let expr = arena.expr(id);
      let span = arena.expr_span(id);
      v.leave_expr(id, expr, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_expr<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, arena: &AstArena) -> ControlFlow<()> {
  let expr = arena.expr(id);
  let span = arena.expr_span(id);
  match expr {
    Expr::Literal(lit) => walk_literal_dispatch(v, id, lit, span, arena)?,
    Expr::Ident(name) => {
      let action = v.visit_ident(id, *name, span);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Expr::TypeConstructor(name) => {
      let action = v.visit_type_constructor(id, *name, span);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Expr::Binary(binary) => walk_binary_dispatch(v, id, binary, span, arena)?,
    Expr::Unary(unary) => walk_unary_dispatch(v, id, unary, span, arena)?,
    Expr::Pipe(pipe) => walk_pipe_dispatch(v, id, pipe, span, arena)?,
    Expr::Apply(apply) => walk_apply_dispatch(v, id, apply, span, arena)?,
    Expr::Section(section) => walk_section_dispatch(v, id, section, span, arena)?,
    Expr::FieldAccess(fa) => walk_field_access_dispatch(v, id, fa, span, arena)?,
    Expr::Block(block) => walk_block_dispatch(v, id, block, span, arena)?,
    Expr::Tuple(tuple) => walk_tuple_dispatch(v, id, tuple, span, arena)?,
    Expr::List(elems) => walk_list_dispatch(v, id, elems, span, arena)?,
    Expr::Record(fields) => walk_record_dispatch(v, id, fields, span, arena)?,
    Expr::Map(entries) => walk_map_dispatch(v, id, entries, span, arena)?,
    Expr::Func(func) => walk_func_dispatch(v, id, func, span, arena)?,
    Expr::Match(m) => walk_match_dispatch(v, id, m, span, arena)?,
    Expr::Ternary(ternary) => walk_ternary_dispatch(v, id, ternary, span, arena)?,
    Expr::Propagate(propagate) => walk_propagate_dispatch(v, id, propagate, span, arena)?,
    Expr::Coalesce(coalesce) => walk_coalesce_dispatch(v, id, coalesce, span, arena)?,
    Expr::Slice(slice) => walk_slice_dispatch(v, id, slice, span, arena)?,
    Expr::NamedArg(na) => walk_named_arg_dispatch(v, id, na, span, arena)?,
    Expr::Loop(loop_node) => walk_loop_dispatch(v, id, loop_node, span, arena)?,
    Expr::Break(brk) => walk_break_dispatch(v, id, brk, span, arena)?,
    Expr::Assert(assert) => walk_assert_dispatch(v, id, assert, span, arena)?,
    Expr::Par(par) => walk_par_dispatch(v, id, par, span, arena)?,
    Expr::Sel(arms) => walk_sel_dispatch(v, id, arms, span, arena)?,
    Expr::Timeout(timeout) => walk_timeout_dispatch(v, id, timeout, span, arena)?,
    Expr::Emit(emit) => walk_emit_dispatch(v, id, emit, span, arena)?,
    Expr::Yield(yld) => walk_yield_dispatch(v, id, yld, span, arena)?,
    Expr::With(with) => walk_with_dispatch(v, id, with, span, arena)?,
  }
  ControlFlow::Continue(())
}
