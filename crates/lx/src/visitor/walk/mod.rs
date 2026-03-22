mod walk_expr;
mod walk_pattern;
mod walk_type;

pub use walk_expr::*;
pub use walk_pattern::*;
pub use walk_type::*;

use crate::ast::{BindTarget, Binding, ClassDeclData, Expr, Program, SExpr, Stmt};
use miette::SourceSpan;

use super::AstVisitor;

pub fn walk_program<V: AstVisitor + ?Sized>(v: &mut V, program: &Program) {
  for stmt in &program.stmts {
    v.visit_stmt(&stmt.node, stmt.span);
  }
}

pub fn walk_stmt<V: AstVisitor + ?Sized>(v: &mut V, stmt: &Stmt, span: SourceSpan) {
  match stmt {
    Stmt::Binding(binding) => v.visit_binding(binding, span),
    Stmt::TypeDef { name, variants, exported } => {
      v.visit_type_def(name, variants, *exported, span);
    },
    Stmt::TraitUnion(def) => v.visit_trait_union(def, span),
    Stmt::TraitDecl(data) => v.visit_trait_decl(data, span),
    Stmt::ClassDecl(data) => v.visit_class_decl(data, span),
    Stmt::FieldUpdate { name, fields, value } => {
      v.visit_field_update(name, fields, value, span);
    },
    Stmt::Use(use_stmt) => v.visit_use(use_stmt, span),
    Stmt::Expr(sexpr) => v.visit_expr(&sexpr.node, sexpr.span),
  }
}

pub fn walk_binding<V: AstVisitor + ?Sized>(v: &mut V, binding: &Binding, _span: SourceSpan) {
  if let Some(ref ty) = binding.type_ann {
    v.visit_type_expr(&ty.node, ty.span);
  }
  if let BindTarget::Pattern(pat) = &binding.target {
    v.visit_pattern(&pat.node, pat.span);
  }
  v.visit_expr(&binding.value.node, binding.value.span);
}

pub fn walk_class_decl<V: AstVisitor + ?Sized>(v: &mut V, data: &ClassDeclData, span: SourceSpan) {
  for f in &data.fields {
    v.visit_expr(&f.default.node, f.default.span);
  }
  for m in &data.methods {
    v.visit_expr(&m.handler.node, m.handler.span);
  }
  v.visit_class_decl_post(data, span);
}

pub fn walk_field_update<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, _span: SourceSpan) {
  v.visit_expr(&value.node, value.span);
}

pub fn walk_expr<V: AstVisitor + ?Sized>(v: &mut V, expr: &Expr, span: SourceSpan) {
  match expr {
    Expr::Literal(lit) => v.visit_literal(lit, span),
    Expr::Ident(name) => v.visit_ident(*name, span),
    Expr::TypeConstructor(name) => v.visit_type_constructor(*name, span),
    Expr::Binary { op, left, right } => v.visit_binary(*op, left, right, span),
    Expr::Unary { op, operand } => v.visit_unary(*op, operand, span),
    Expr::Pipe { left, right } => v.visit_pipe(left, right, span),
    Expr::Apply { func, arg } => v.visit_apply(func, arg, span),
    Expr::Section(section) => v.visit_section(section, span),
    Expr::FieldAccess { expr: e, field } => v.visit_field_access(e, field, span),
    Expr::Block(stmts) => v.visit_block(stmts, span),
    Expr::Tuple(elems) => v.visit_tuple(elems, span),
    Expr::List(elems) => v.visit_list(elems, span),
    Expr::Record(fields) => v.visit_record(fields, span),
    Expr::Map(entries) => v.visit_map(entries, span),
    Expr::Func { params, ret_type, body } => {
      v.visit_func(params, ret_type.as_ref(), body, span);
    },
    Expr::Match { scrutinee, arms } => v.visit_match(scrutinee, arms, span),
    Expr::Ternary { cond, then_, else_ } => {
      v.visit_ternary(cond, then_, else_.as_deref(), span);
    },
    Expr::Propagate(inner) => v.visit_propagate(inner, span),
    Expr::Coalesce { expr: e, default } => v.visit_coalesce(e, default, span),
    Expr::Slice { expr: e, start, end } => {
      v.visit_slice(e, start.as_deref(), end.as_deref(), span);
    },
    Expr::NamedArg { name, value } => v.visit_named_arg(name, value, span),
    Expr::Loop(stmts) => v.visit_loop(stmts, span),
    Expr::Break(val) => v.visit_break(val.as_deref(), span),
    Expr::Assert { expr: e, msg } => v.visit_assert(e, msg.as_deref(), span),
    Expr::Par(stmts) => v.visit_par(stmts, span),
    Expr::Sel(arms) => v.visit_sel(arms, span),
    Expr::Emit { value } => v.visit_emit(value, span),
    Expr::Yield { value } => v.visit_yield(value, span),
    Expr::With { name, value, body, mutable } => {
      v.visit_with(name, value, body, *mutable, span);
    },
    Expr::WithResource { resources, body } => {
      v.visit_with_resource(resources, body, span);
    },
    Expr::WithContext { fields, body } => {
      v.visit_with_context(fields, body, span);
    },
  }
}
