mod walk_expr;
mod walk_pattern;
mod walk_type;

pub use walk_expr::*;
pub use walk_pattern::*;
pub use walk_type::*;

use std::ops::ControlFlow;

use crate::ast::{
  BindTarget, Binding, ClassDeclData, Expr, ExprApply, ExprAssert, ExprBinary, ExprCoalesce, ExprEmit, ExprFieldAccess, ExprFunc, ExprMatch, ExprNamedArg,
  ExprPipe, ExprSlice, ExprTernary, ExprTimeout, ExprUnary, ExprWith, ExprYield, Program, SExpr, Stmt, StmtFieldUpdate, StmtTypeDef, TraitDeclData,
  TraitEntry, WithKind,
};
use crate::sym::Sym;
use miette::SourceSpan;

use super::AstVisitor;

pub fn walk_program<V: AstVisitor + ?Sized>(v: &mut V, program: &Program) -> ControlFlow<()> {
  for stmt in &program.stmts {
    v.visit_stmt(&stmt.node, stmt.span)?;
  }
  v.leave_program(program)
}

pub fn walk_stmt<V: AstVisitor + ?Sized>(v: &mut V, stmt: &Stmt, span: SourceSpan) -> ControlFlow<()> {
  match stmt {
    Stmt::Binding(binding) => v.visit_binding(binding, span)?,
    Stmt::TypeDef(StmtTypeDef { name, variants, exported }) => {
      v.visit_type_def(*name, variants, *exported, span)?;
    },
    Stmt::TraitUnion(def) => v.visit_trait_union(def, span)?,
    Stmt::TraitDecl(data) => v.visit_trait_decl(data, span)?,
    Stmt::ClassDecl(data) => v.visit_class_decl(data, span)?,
    Stmt::FieldUpdate(StmtFieldUpdate { name, fields, value }) => {
      v.visit_field_update(*name, fields, value, span)?;
    },
    Stmt::Use(use_stmt) => v.visit_use(use_stmt, span)?,
    Stmt::Expr(sexpr) => v.visit_expr(&sexpr.node, sexpr.span)?,
  }
  v.leave_stmt(stmt, span)
}

pub fn walk_binding<V: AstVisitor + ?Sized>(v: &mut V, binding: &Binding, span: SourceSpan) -> ControlFlow<()> {
  if let Some(ref ty) = binding.type_ann {
    v.visit_type_expr(&ty.node, ty.span)?;
  }
  if let BindTarget::Pattern(pat) = &binding.target {
    v.visit_pattern(&pat.node, pat.span)?;
  }
  v.visit_expr(&binding.value.node, binding.value.span)?;
  v.leave_binding(binding, span)
}

pub fn walk_trait_decl<V: AstVisitor + ?Sized>(v: &mut V, data: &TraitDeclData, span: SourceSpan) -> ControlFlow<()> {
  for entry in &data.entries {
    if let TraitEntry::Field(field) = entry {
      if let Some(ref default) = field.default {
        v.visit_expr(&default.node, default.span)?;
      }
      if let Some(ref constraint) = field.constraint {
        v.visit_expr(&constraint.node, constraint.span)?;
      }
    }
  }
  for method in &data.methods {
    for input in &method.input {
      if let Some(ref d) = input.default {
        v.visit_expr(&d.node, d.span)?;
      }
      if let Some(ref c) = input.constraint {
        v.visit_expr(&c.node, c.span)?;
      }
    }
  }
  for method in &data.defaults {
    v.visit_expr(&method.handler.node, method.handler.span)?;
  }
  v.leave_trait_decl(data, span)
}

pub fn walk_class_decl<V: AstVisitor + ?Sized>(v: &mut V, data: &ClassDeclData, span: SourceSpan) -> ControlFlow<()> {
  for f in &data.fields {
    v.visit_expr(&f.default.node, f.default.span)?;
  }
  for m in &data.methods {
    v.visit_expr(&m.handler.node, m.handler.span)?;
  }
  v.leave_class_decl(data, span)
}

pub fn walk_field_update<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, fields: &[Sym], value: &SExpr, span: SourceSpan) -> ControlFlow<()> {
  v.visit_expr(&value.node, value.span)?;
  v.leave_field_update(name, fields, value, span)
}

pub fn walk_expr<V: AstVisitor + ?Sized>(v: &mut V, expr: &Expr, span: SourceSpan) -> ControlFlow<()> {
  match expr {
    Expr::Literal(lit) => v.visit_literal(lit, span)?,
    Expr::Ident(name) => v.visit_ident(*name, span)?,
    Expr::TypeConstructor(name) => v.visit_type_constructor(*name, span)?,
    Expr::Binary(ExprBinary { op, left, right }) => v.visit_binary(*op, left, right, span)?,
    Expr::Unary(ExprUnary { op, operand }) => v.visit_unary(*op, operand, span)?,
    Expr::Pipe(ExprPipe { left, right }) => v.visit_pipe(left, right, span)?,
    Expr::Apply(ExprApply { func, arg }) => v.visit_apply(func, arg, span)?,
    Expr::Section(section) => v.visit_section(section, span)?,
    Expr::FieldAccess(ExprFieldAccess { expr: e, field }) => v.visit_field_access(e, field, span)?,
    Expr::Block(stmts) => v.visit_block(stmts, span)?,
    Expr::Tuple(elems) => v.visit_tuple(elems, span)?,
    Expr::List(elems) => v.visit_list(elems, span)?,
    Expr::Record(fields) => v.visit_record(fields, span)?,
    Expr::Map(entries) => v.visit_map(entries, span)?,
    Expr::Func(ExprFunc { params, ret_type, guard, body }) => {
      v.visit_func(params, ret_type.as_ref(), guard.as_deref(), body, span)?;
    },
    Expr::Match(ExprMatch { scrutinee, arms }) => v.visit_match(scrutinee, arms, span)?,
    Expr::Ternary(ExprTernary { cond, then_, else_ }) => {
      v.visit_ternary(cond, then_, else_.as_deref(), span)?;
    },
    Expr::Propagate(inner) => v.visit_propagate(inner, span)?,
    Expr::Coalesce(ExprCoalesce { expr: e, default }) => v.visit_coalesce(e, default, span)?,
    Expr::Slice(ExprSlice { expr: e, start, end }) => {
      v.visit_slice(e, start.as_deref(), end.as_deref(), span)?;
    },
    Expr::NamedArg(ExprNamedArg { name, value }) => v.visit_named_arg(*name, value, span)?,
    Expr::Loop(stmts) => v.visit_loop(stmts, span)?,
    Expr::Break(val) => v.visit_break(val.as_deref(), span)?,
    Expr::Assert(ExprAssert { expr: e, msg }) => v.visit_assert(e, msg.as_deref(), span)?,
    Expr::Par(stmts) => v.visit_par(stmts, span)?,
    Expr::Sel(arms) => v.visit_sel(arms, span)?,
    Expr::Timeout(ExprTimeout { ms, body }) => v.visit_timeout(ms, body, span)?,
    Expr::Emit(ExprEmit { value }) => v.visit_emit(value, span)?,
    Expr::Yield(ExprYield { value }) => v.visit_yield(value, span)?,
    Expr::With(ExprWith { kind, body }) => match kind {
      WithKind::Binding { name, value, mutable } => {
        v.visit_with(*name, value, body, *mutable, span)?;
      },
      WithKind::Resources { resources } => {
        v.visit_with_resource(resources, body, span)?;
      },
      WithKind::Context { fields } => {
        v.visit_with_context(fields, body, span)?;
      },
    },
  }
  v.leave_expr(expr, span)
}
