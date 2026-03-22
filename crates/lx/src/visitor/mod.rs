use std::ops::ControlFlow;

use crate::ast::{
  BinOp, Binding, ClassDeclData, Expr, FieldKind, FieldPattern, ListElem, Literal, MapEntry, MatchArm, Param, Pattern, Program, RecordField, SExpr, SPattern,
  SStmt, SType, Section, SelArm, Stmt, TraitDeclData, TraitUnionDef, TypeExpr, TypeField, UnaryOp, UseStmt,
};
use crate::sym::Sym;
use miette::SourceSpan;

mod walk;
pub use walk::*;

pub trait AstVisitor {
  fn visit_program(&mut self, program: &Program) -> ControlFlow<()> {
    walk_program(self, program)
  }
  fn visit_stmt(&mut self, stmt: &Stmt, span: SourceSpan) -> ControlFlow<()> {
    walk_stmt(self, stmt, span)
  }
  fn visit_binding(&mut self, binding: &Binding, span: SourceSpan) -> ControlFlow<()> {
    walk_binding(self, binding, span)
  }
  fn visit_type_def(&mut self, _name: Sym, _variants: &[(Sym, usize)], _exported: bool, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn visit_trait_decl(&mut self, data: &TraitDeclData, span: SourceSpan) -> ControlFlow<()> {
    walk_trait_decl(self, data, span)
  }
  fn visit_class_decl(&mut self, data: &ClassDeclData, span: SourceSpan) -> ControlFlow<()> {
    walk_class_decl(self, data, span)
  }
  fn visit_trait_union(&mut self, _def: &TraitUnionDef, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn visit_field_update(&mut self, name: Sym, fields: &[Sym], value: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_field_update(self, name, fields, value, span)
  }
  fn visit_use(&mut self, _stmt: &UseStmt, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn visit_expr(&mut self, expr: &Expr, span: SourceSpan) -> ControlFlow<()> {
    walk_expr(self, expr, span)
  }
  fn visit_literal(&mut self, lit: &Literal, span: SourceSpan) -> ControlFlow<()> {
    walk_literal(self, lit, span)
  }
  fn visit_ident(&mut self, _name: Sym, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_constructor(&mut self, _name: Sym, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_binary(&mut self, op: BinOp, left: &SExpr, right: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_binary(self, op, left, right, span)
  }
  fn visit_unary(&mut self, op: UnaryOp, operand: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_unary(self, op, operand, span)
  }
  fn visit_pipe(&mut self, left: &SExpr, right: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_pipe(self, left, right, span)
  }
  fn visit_apply(&mut self, func: &SExpr, arg: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_apply(self, func, arg, span)
  }
  fn visit_section(&mut self, section: &Section, span: SourceSpan) -> ControlFlow<()> {
    walk_section(self, section, span)
  }
  fn visit_field_access(&mut self, expr: &SExpr, field: &FieldKind, span: SourceSpan) -> ControlFlow<()> {
    walk_field_access(self, expr, field, span)
  }
  fn visit_block(&mut self, stmts: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
    walk_block(self, stmts, span)
  }
  fn visit_tuple(&mut self, elems: &[SExpr], span: SourceSpan) -> ControlFlow<()> {
    walk_tuple(self, elems, span)
  }
  fn visit_list(&mut self, elems: &[ListElem], span: SourceSpan) -> ControlFlow<()> {
    walk_list(self, elems, span)
  }
  fn visit_record(&mut self, fields: &[RecordField], span: SourceSpan) -> ControlFlow<()> {
    walk_record(self, fields, span)
  }
  fn visit_map(&mut self, entries: &[MapEntry], span: SourceSpan) -> ControlFlow<()> {
    walk_map(self, entries, span)
  }
  fn visit_func(&mut self, params: &[Param], ret_type: Option<&SType>, guard: Option<&SExpr>, body: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_func(self, params, ret_type, guard, body, span)
  }
  fn visit_match(&mut self, scrutinee: &SExpr, arms: &[MatchArm], span: SourceSpan) -> ControlFlow<()> {
    walk_match(self, scrutinee, arms, span)
  }
  fn visit_ternary(&mut self, cond: &SExpr, then_: &SExpr, else_: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
    walk_ternary(self, cond, then_, else_, span)
  }
  fn visit_propagate(&mut self, inner: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_propagate(self, inner, span)
  }
  fn visit_coalesce(&mut self, expr: &SExpr, default: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_coalesce(self, expr, default, span)
  }
  fn visit_slice(&mut self, expr: &SExpr, start: Option<&SExpr>, end: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
    walk_slice(self, expr, start, end, span)
  }
  fn visit_named_arg(&mut self, name: Sym, value: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_named_arg(self, name, value, span)
  }
  fn visit_loop(&mut self, stmts: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
    walk_loop(self, stmts, span)
  }
  fn visit_break(&mut self, value: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
    walk_break(self, value, span)
  }
  fn visit_assert(&mut self, expr: &SExpr, msg: Option<&SExpr>, span: SourceSpan) -> ControlFlow<()> {
    walk_assert(self, expr, msg, span)
  }
  fn visit_par(&mut self, stmts: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
    walk_par(self, stmts, span)
  }
  fn visit_sel(&mut self, arms: &[SelArm], span: SourceSpan) -> ControlFlow<()> {
    walk_sel(self, arms, span)
  }
  fn visit_timeout(&mut self, ms: &SExpr, body: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_timeout(self, ms, body, span)
  }
  fn visit_emit(&mut self, value: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_emit(self, value, span)
  }
  fn visit_yield(&mut self, value: &SExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_yield(self, value, span)
  }
  fn visit_with(&mut self, name: Sym, value: &SExpr, body: &[SStmt], mutable: bool, span: SourceSpan) -> ControlFlow<()> {
    walk_with(self, name, value, body, mutable, span)
  }
  fn visit_with_resource(&mut self, resources: &[(SExpr, Sym)], body: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
    walk_with_resource(self, resources, body, span)
  }
  fn visit_with_context(&mut self, fields: &[(Sym, SExpr)], body: &[SStmt], span: SourceSpan) -> ControlFlow<()> {
    walk_with_context(self, fields, body, span)
  }
  fn visit_pattern(&mut self, pattern: &Pattern, span: SourceSpan) -> ControlFlow<()> {
    walk_pattern(self, pattern, span)
  }
  fn visit_pattern_literal(&mut self, _lit: &Literal, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pattern_bind(&mut self, _name: Sym, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pattern_wildcard(&mut self, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pattern_tuple(&mut self, elems: &[SPattern], span: SourceSpan) -> ControlFlow<()> {
    walk_pattern_tuple(self, elems, span)
  }
  fn visit_pattern_list(&mut self, elems: &[SPattern], rest: Option<Sym>, span: SourceSpan) -> ControlFlow<()> {
    walk_pattern_list(self, elems, rest, span)
  }
  fn visit_pattern_record(&mut self, fields: &[FieldPattern], rest: Option<Sym>, span: SourceSpan) -> ControlFlow<()> {
    walk_pattern_record(self, fields, rest, span)
  }
  fn visit_pattern_constructor(&mut self, name: Sym, args: &[SPattern], span: SourceSpan) -> ControlFlow<()> {
    walk_pattern_constructor(self, name, args, span)
  }
  fn visit_type_expr(&mut self, type_expr: &TypeExpr, span: SourceSpan) -> ControlFlow<()> {
    walk_type_expr(self, type_expr, span)
  }
  fn visit_type_named(&mut self, _name: Sym, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_var(&mut self, _name: Sym, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_applied(&mut self, name: Sym, args: &[SType], span: SourceSpan) -> ControlFlow<()> {
    walk_type_applied(self, name, args, span)
  }
  fn visit_type_list(&mut self, inner: &SType, span: SourceSpan) -> ControlFlow<()> {
    walk_type_list(self, inner, span)
  }
  fn visit_type_map(&mut self, key: &SType, value: &SType, span: SourceSpan) -> ControlFlow<()> {
    walk_type_map(self, key, value, span)
  }
  fn visit_type_record(&mut self, fields: &[TypeField], span: SourceSpan) -> ControlFlow<()> {
    walk_type_record(self, fields, span)
  }
  fn visit_type_tuple(&mut self, elems: &[SType], span: SourceSpan) -> ControlFlow<()> {
    walk_type_tuple(self, elems, span)
  }
  fn visit_type_func(&mut self, param: &SType, ret: &SType, span: SourceSpan) -> ControlFlow<()> {
    walk_type_func(self, param, ret, span)
  }
  fn visit_type_fallible(&mut self, ok: &SType, err: &SType, span: SourceSpan) -> ControlFlow<()> {
    walk_type_fallible(self, ok, err, span)
  }

  fn leave_program(&mut self, _program: &Program) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_stmt(&mut self, _stmt: &Stmt, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_binding(&mut self, _binding: &Binding, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_trait_decl(&mut self, _data: &TraitDeclData, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_class_decl(&mut self, _data: &ClassDeclData, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_expr(&mut self, _expr: &Expr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_literal(&mut self, _lit: &Literal, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_binary(&mut self, _op: BinOp, _left: &SExpr, _right: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_unary(&mut self, _op: UnaryOp, _operand: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_pipe(&mut self, _left: &SExpr, _right: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_apply(&mut self, _func: &SExpr, _arg: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_section(&mut self, _section: &Section, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_field_access(&mut self, _expr: &SExpr, _field: &FieldKind, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_block(&mut self, _stmts: &[SStmt], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_tuple(&mut self, _elems: &[SExpr], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_list(&mut self, _elems: &[ListElem], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_record(&mut self, _fields: &[RecordField], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_map(&mut self, _entries: &[MapEntry], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_func(&mut self, _params: &[Param], _ret_type: Option<&SType>, _guard: Option<&SExpr>, _body: &SExpr, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_match(&mut self, _scrutinee: &SExpr, _arms: &[MatchArm], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_ternary(&mut self, _cond: &SExpr, _then_: &SExpr, _else_: Option<&SExpr>, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_propagate(&mut self, _inner: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_coalesce(&mut self, _expr: &SExpr, _default: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_slice(&mut self, _expr: &SExpr, _start: Option<&SExpr>, _end: Option<&SExpr>, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_named_arg(&mut self, _name: Sym, _value: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_loop(&mut self, _stmts: &[SStmt], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_break(&mut self, _value: Option<&SExpr>, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_assert(&mut self, _expr: &SExpr, _msg: Option<&SExpr>, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_par(&mut self, _stmts: &[SStmt], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_sel(&mut self, _arms: &[SelArm], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_timeout(&mut self, _ms: &SExpr, _body: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_emit(&mut self, _value: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_yield(&mut self, _value: &SExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_with(&mut self, _name: Sym, _value: &SExpr, _body: &[SStmt], _mutable: bool, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_with_resource(&mut self, _resources: &[(SExpr, Sym)], _body: &[SStmt], _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_with_context(&mut self, _fields: &[(Sym, SExpr)], _body: &[SStmt], _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pattern(&mut self, _pattern: &Pattern, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_pattern_tuple(&mut self, _elems: &[SPattern], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_pattern_list(&mut self, _elems: &[SPattern], _rest: Option<Sym>, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pattern_record(&mut self, _fields: &[FieldPattern], _rest: Option<Sym>, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pattern_constructor(&mut self, _name: Sym, _args: &[SPattern], _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_expr(&mut self, _type_expr: &TypeExpr, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_type_applied(&mut self, _name: Sym, _args: &[SType], _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_list(&mut self, _inner: &SType, _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_type_map(&mut self, _key: &SType, _value: &SType, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_record(&mut self, _fields: &[TypeField], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_type_tuple(&mut self, _elems: &[SType], _span: SourceSpan) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn leave_type_func(&mut self, _param: &SType, _ret: &SType, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_fallible(&mut self, _ok: &SType, _err: &SType, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_field_update(&mut self, _name: Sym, _fields: &[Sym], _value: &SExpr, _span: SourceSpan) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
}
