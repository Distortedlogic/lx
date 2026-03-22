use crate::ast::{
  BinOp, Binding, ClassDeclData, Expr, FieldKind, FieldPattern, ListElem, Literal, MapEntry, MatchArm, Param, Pattern, Program, RecordField, SExpr, SPattern,
  SType, Section, SelArm, Stmt, TraitDeclData, TraitUnionDef, TypeExpr, TypeField, UnaryOp, UseStmt,
};
use miette::SourceSpan;

mod walk;
pub use walk::*;

pub trait AstVisitor {
  fn visit_program(&mut self, program: &Program) {
    walk_program(self, program);
  }
  fn visit_stmt(&mut self, stmt: &Stmt, span: SourceSpan) {
    walk_stmt(self, stmt, span);
  }
  fn visit_binding(&mut self, binding: &Binding, span: SourceSpan) {
    walk_binding(self, binding, span);
  }
  fn visit_type_def(&mut self, _name: &str, _variants: &[(String, usize)], _exported: bool, _span: SourceSpan) {}
  fn visit_trait_decl(&mut self, _data: &TraitDeclData, _span: SourceSpan) {}
  fn visit_class_decl(&mut self, data: &ClassDeclData, span: SourceSpan) {
    walk_class_decl(self, data, span);
  }
  fn visit_class_decl_post(&mut self, _data: &ClassDeclData, _span: SourceSpan) {}
  fn visit_trait_union(&mut self, _def: &TraitUnionDef, _span: SourceSpan) {}
  fn visit_field_update(&mut self, _name: &str, _fields: &[String], value: &SExpr, span: SourceSpan) {
    walk_field_update(self, value, span);
  }
  fn visit_use(&mut self, _stmt: &UseStmt, _span: SourceSpan) {}
  fn visit_expr(&mut self, expr: &Expr, span: SourceSpan) {
    walk_expr(self, expr, span);
  }
  fn visit_literal(&mut self, lit: &Literal, span: SourceSpan) {
    walk_literal(self, lit, span);
  }
  fn visit_ident(&mut self, _name: &str, _span: SourceSpan) {}
  fn visit_type_constructor(&mut self, _name: &str, _span: SourceSpan) {}
  fn visit_binary(&mut self, _op: BinOp, left: &SExpr, right: &SExpr, span: SourceSpan) {
    walk_binary(self, left, right, span);
  }
  fn visit_unary(&mut self, _op: UnaryOp, operand: &SExpr, span: SourceSpan) {
    walk_unary(self, operand, span);
  }
  fn visit_pipe(&mut self, left: &SExpr, right: &SExpr, span: SourceSpan) {
    walk_pipe(self, left, right, span);
  }
  fn visit_apply(&mut self, func: &SExpr, arg: &SExpr, span: SourceSpan) {
    walk_apply(self, func, arg, span);
  }
  fn visit_section(&mut self, section: &Section, span: SourceSpan) {
    walk_section(self, section, span);
  }
  fn visit_field_access(&mut self, expr: &SExpr, field: &FieldKind, span: SourceSpan) {
    walk_field_access(self, expr, field, span);
  }
  fn visit_block(&mut self, stmts: &[crate::ast::SStmt], span: SourceSpan) {
    walk_block(self, stmts, span);
  }
  fn visit_block_post(&mut self, _stmts: &[crate::ast::SStmt], _span: SourceSpan) {}
  fn visit_tuple(&mut self, elems: &[SExpr], span: SourceSpan) {
    walk_tuple(self, elems, span);
  }
  fn visit_list(&mut self, elems: &[ListElem], span: SourceSpan) {
    walk_list(self, elems, span);
  }
  fn visit_record(&mut self, fields: &[RecordField], span: SourceSpan) {
    walk_record(self, fields, span);
  }
  fn visit_map(&mut self, entries: &[MapEntry], span: SourceSpan) {
    walk_map(self, entries, span);
  }
  fn visit_func(&mut self, params: &[Param], ret_type: Option<&SType>, body: &SExpr, span: SourceSpan) {
    walk_func(self, params, ret_type, body, span);
  }
  fn visit_func_post(&mut self, _params: &[Param], _ret_type: Option<&SType>, _body: &SExpr, _span: SourceSpan) {}
  fn visit_match(&mut self, scrutinee: &SExpr, arms: &[MatchArm], span: SourceSpan) {
    walk_match(self, scrutinee, arms, span);
  }
  fn visit_match_post(&mut self, _scrutinee: &SExpr, _arms: &[MatchArm], _span: SourceSpan) {}
  fn visit_ternary(&mut self, cond: &SExpr, then_: &SExpr, else_: Option<&SExpr>, span: SourceSpan) {
    walk_ternary(self, cond, then_, else_, span);
  }
  fn visit_ternary_post(&mut self, _cond: &SExpr, _then_: &SExpr, _else_: Option<&SExpr>, _span: SourceSpan) {}
  fn visit_propagate(&mut self, inner: &SExpr, span: SourceSpan) {
    walk_propagate(self, inner, span);
  }
  fn visit_coalesce(&mut self, expr: &SExpr, default: &SExpr, span: SourceSpan) {
    walk_coalesce(self, expr, default, span);
  }
  fn visit_slice(&mut self, expr: &SExpr, start: Option<&SExpr>, end: Option<&SExpr>, span: SourceSpan) {
    walk_slice(self, expr, start, end, span);
  }
  fn visit_named_arg(&mut self, _name: &str, value: &SExpr, span: SourceSpan) {
    walk_named_arg(self, value, span);
  }
  fn visit_loop(&mut self, stmts: &[crate::ast::SStmt], span: SourceSpan) {
    walk_loop(self, stmts, span);
  }
  fn visit_loop_post(&mut self, _stmts: &[crate::ast::SStmt], _span: SourceSpan) {}
  fn visit_break(&mut self, value: Option<&SExpr>, span: SourceSpan) {
    walk_break(self, value, span);
  }
  fn visit_assert(&mut self, expr: &SExpr, msg: Option<&SExpr>, span: SourceSpan) {
    walk_assert(self, expr, msg, span);
  }
  fn visit_par(&mut self, stmts: &[crate::ast::SStmt], span: SourceSpan) {
    walk_par(self, stmts, span);
  }
  fn visit_par_post(&mut self, _stmts: &[crate::ast::SStmt], _span: SourceSpan) {}
  fn visit_sel(&mut self, arms: &[SelArm], span: SourceSpan) {
    walk_sel(self, arms, span);
  }
  fn visit_sel_post(&mut self, _arms: &[SelArm], _span: SourceSpan) {}
  fn visit_emit(&mut self, value: &SExpr, span: SourceSpan) {
    walk_emit(self, value, span);
  }
  fn visit_yield(&mut self, value: &SExpr, span: SourceSpan) {
    walk_yield(self, value, span);
  }
  fn visit_with(&mut self, _name: &str, value: &SExpr, body: &[crate::ast::SStmt], _mutable: bool, span: SourceSpan) {
    walk_with(self, value, body, span);
  }
  fn visit_with_post(&mut self, _name: &str, _value: &SExpr, _body: &[crate::ast::SStmt], _mutable: bool, _span: SourceSpan) {}
  fn visit_with_resource(&mut self, resources: &[(SExpr, String)], body: &[crate::ast::SStmt], span: SourceSpan) {
    walk_with_resource(self, resources, body, span);
  }
  fn visit_with_resource_post(&mut self, _resources: &[(SExpr, String)], _body: &[crate::ast::SStmt], _span: SourceSpan) {}
  fn visit_with_context(&mut self, fields: &[(String, SExpr)], body: &[crate::ast::SStmt], span: SourceSpan) {
    walk_with_context(self, fields, body, span);
  }
  fn visit_with_context_post(&mut self, _fields: &[(String, SExpr)], _body: &[crate::ast::SStmt], _span: SourceSpan) {}
  fn visit_pattern(&mut self, pattern: &Pattern, span: SourceSpan) {
    walk_pattern(self, pattern, span);
  }
  fn visit_pattern_literal(&mut self, _lit: &Literal, _span: SourceSpan) {}
  fn visit_pattern_bind(&mut self, _name: &str, _span: SourceSpan) {}
  fn visit_pattern_wildcard(&mut self, _span: SourceSpan) {}
  fn visit_pattern_tuple(&mut self, elems: &[SPattern], span: SourceSpan) {
    walk_pattern_tuple(self, elems, span);
  }
  fn visit_pattern_list(&mut self, elems: &[SPattern], _rest: Option<&str>, span: SourceSpan) {
    walk_pattern_list(self, elems, span);
  }
  fn visit_pattern_record(&mut self, fields: &[FieldPattern], _rest: Option<&str>, span: SourceSpan) {
    walk_pattern_record(self, fields, span);
  }
  fn visit_pattern_constructor(&mut self, _name: &str, args: &[SPattern], span: SourceSpan) {
    walk_pattern_constructor(self, args, span);
  }
  fn visit_type_expr(&mut self, type_expr: &TypeExpr, span: SourceSpan) {
    walk_type_expr(self, type_expr, span);
  }
  fn visit_type_named(&mut self, _name: &str, _span: SourceSpan) {}
  fn visit_type_var(&mut self, _name: &str, _span: SourceSpan) {}
  fn visit_type_applied(&mut self, _name: &str, args: &[SType], span: SourceSpan) {
    walk_type_applied(self, args, span);
  }
  fn visit_type_list(&mut self, inner: &SType, span: SourceSpan) {
    walk_type_list(self, inner, span);
  }
  fn visit_type_map(&mut self, key: &SType, value: &SType, span: SourceSpan) {
    walk_type_map(self, key, value, span);
  }
  fn visit_type_record(&mut self, fields: &[TypeField], span: SourceSpan) {
    walk_type_record(self, fields, span);
  }
  fn visit_type_tuple(&mut self, elems: &[SType], span: SourceSpan) {
    walk_type_tuple(self, elems, span);
  }
  fn visit_type_func(&mut self, param: &SType, ret: &SType, span: SourceSpan) {
    walk_type_func(self, param, ret, span);
  }
  fn visit_type_fallible(&mut self, ok: &SType, err: &SType, span: SourceSpan) {
    walk_type_fallible(self, ok, err, span);
  }
}
