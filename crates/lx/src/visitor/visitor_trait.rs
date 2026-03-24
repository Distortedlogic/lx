use std::ops::ControlFlow;

use crate::ast::{
  AstArena, Binding, ClassDeclData, Expr, ExprApply, ExprAssert, ExprBinary, ExprCoalesce, ExprEmit, ExprFieldAccess, ExprFunc, ExprId, ExprMatch,
  ExprNamedArg, ExprPipe, ExprSlice, ExprTernary, ExprTimeout, ExprUnary, ExprWith, ExprYield, FieldPattern, ListElem, Literal, MapEntry, Pattern, PatternId,
  Program, RecordField, Section, SelArm, Stmt, StmtFieldUpdate, StmtId, StmtTypeDef, TraitDeclData, TraitUnionDef, TypeExpr, TypeExprId, TypeField, UseStmt,
};
use crate::sym::Sym;
use miette::SourceSpan;

use super::{VisitAction, walk_program};

#[rustfmt::skip]
pub trait AstVisitor {
  fn visit_program<P>(&mut self, program: &Program<P>) -> VisitAction {
    match walk_program(self, program) {
      ControlFlow::Continue(()) => VisitAction::Descend,
      ControlFlow::Break(()) => VisitAction::Stop,
    }
  }
  fn leave_program<P>(&mut self, _program: &Program<P>) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_stmt(&mut self, _id: StmtId, _stmt: &Stmt, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_stmt(&mut self, _id: StmtId, _stmt: &Stmt, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_binding(&mut self, _id: StmtId, _binding: &Binding, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_binding(&mut self, _id: StmtId, _binding: &Binding, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_def(&mut self, _id: StmtId, _def: &StmtTypeDef, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_trait_decl(&mut self, _id: StmtId, _data: &TraitDeclData, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_trait_decl(&mut self, _id: StmtId, _data: &TraitDeclData, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_class_decl(&mut self, _id: StmtId, _data: &ClassDeclData, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_class_decl(&mut self, _id: StmtId, _data: &ClassDeclData, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_trait_union(&mut self, _id: StmtId, _def: &TraitUnionDef, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_field_update(&mut self, _id: StmtId, _update: &StmtFieldUpdate, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_field_update(&mut self, _id: StmtId, _update: &StmtFieldUpdate, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_use(&mut self, _id: StmtId, _stmt: &UseStmt, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_literal(&mut self, _id: ExprId, _lit: &Literal, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_literal(&mut self, _id: ExprId, _lit: &Literal, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_ident(&mut self, _id: ExprId, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_type_constructor(&mut self, _id: ExprId, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_binary(&mut self, _id: ExprId, _binary: &ExprBinary, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_binary(&mut self, _id: ExprId, _binary: &ExprBinary, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_unary(&mut self, _id: ExprId, _unary: &ExprUnary, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_unary(&mut self, _id: ExprId, _unary: &ExprUnary, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pipe(&mut self, _id: ExprId, _pipe: &ExprPipe, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_pipe(&mut self, _id: ExprId, _pipe: &ExprPipe, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_apply(&mut self, _id: ExprId, _apply: &ExprApply, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_apply(&mut self, _id: ExprId, _apply: &ExprApply, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_section(&mut self, _id: ExprId, _section: &Section, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_section(&mut self, _id: ExprId, _section: &Section, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_field_access(&mut self, _id: ExprId, _fa: &ExprFieldAccess, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_field_access(&mut self, _id: ExprId, _fa: &ExprFieldAccess, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_block(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_block(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_tuple(&mut self, _id: ExprId, _elems: &[ExprId], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_tuple(&mut self, _id: ExprId, _elems: &[ExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_list(&mut self, _id: ExprId, _elems: &[ListElem], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_list(&mut self, _id: ExprId, _elems: &[ListElem], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_record(&mut self, _id: ExprId, _fields: &[RecordField], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_record(&mut self, _id: ExprId, _fields: &[RecordField], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_map(&mut self, _id: ExprId, _entries: &[MapEntry], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_map(&mut self, _id: ExprId, _entries: &[MapEntry], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_func(&mut self, _id: ExprId, _func: &ExprFunc, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_func(&mut self, _id: ExprId, _func: &ExprFunc, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_match(&mut self, _id: ExprId, _m: &ExprMatch, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_match(&mut self, _id: ExprId, _m: &ExprMatch, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_ternary(&mut self, _id: ExprId, _ternary: &ExprTernary, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_ternary(&mut self, _id: ExprId, _ternary: &ExprTernary, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_propagate(&mut self, _id: ExprId, _inner: ExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_propagate(&mut self, _id: ExprId, _inner: ExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_coalesce(&mut self, _id: ExprId, _coalesce: &ExprCoalesce, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_coalesce(&mut self, _id: ExprId, _coalesce: &ExprCoalesce, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_slice(&mut self, _id: ExprId, _slice: &ExprSlice, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_slice(&mut self, _id: ExprId, _slice: &ExprSlice, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_named_arg(&mut self, _id: ExprId, _na: &ExprNamedArg, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_named_arg(&mut self, _id: ExprId, _na: &ExprNamedArg, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_loop(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_loop(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_break(&mut self, _id: ExprId, _value: Option<ExprId>, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_break(&mut self, _id: ExprId, _value: Option<ExprId>, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_assert(&mut self, _id: ExprId, _assert: &ExprAssert, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_assert(&mut self, _id: ExprId, _assert: &ExprAssert, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_par(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_par(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_sel(&mut self, _id: ExprId, _arms: &[SelArm], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_sel(&mut self, _id: ExprId, _arms: &[SelArm], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_timeout(&mut self, _id: ExprId, _timeout: &ExprTimeout, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_timeout(&mut self, _id: ExprId, _timeout: &ExprTimeout, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_emit(&mut self, _id: ExprId, _emit: &ExprEmit, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_emit(&mut self, _id: ExprId, _emit: &ExprEmit, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_yield(&mut self, _id: ExprId, _yld: &ExprYield, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_yield(&mut self, _id: ExprId, _yld: &ExprYield, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_with(&mut self, _id: ExprId, _with: &ExprWith, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_with(&mut self, _id: ExprId, _with: &ExprWith, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pattern(&mut self, _id: PatternId, _pattern: &Pattern, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_pattern(&mut self, _id: PatternId, _pattern: &Pattern, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pattern_literal(&mut self, _id: PatternId, _lit: &Literal, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_pattern_bind(&mut self, _id: PatternId, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_pattern_wildcard(&mut self, _id: PatternId, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_pattern_tuple(&mut self, _id: PatternId, _elems: &[PatternId], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_pattern_tuple(&mut self, _id: PatternId, _elems: &[PatternId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pattern_list(&mut self, _id: PatternId, _elems: &[PatternId], _rest: Option<Sym>, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_pattern_list(&mut self, _id: PatternId, _elems: &[PatternId], _rest: Option<Sym>, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pattern_record(&mut self, _id: PatternId, _fields: &[FieldPattern], _rest: Option<Sym>, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_pattern_record(&mut self, _id: PatternId, _fields: &[FieldPattern], _rest: Option<Sym>, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_pattern_constructor(&mut self, _id: PatternId, _name: Sym, _args: &[PatternId], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_pattern_constructor(&mut self, _id: PatternId, _name: Sym, _args: &[PatternId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_expr(&mut self, _id: TypeExprId, _type_expr: &TypeExpr, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_type_expr(&mut self, _id: TypeExprId, _type_expr: &TypeExpr, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_named(&mut self, _id: TypeExprId, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_type_var(&mut self, _id: TypeExprId, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn visit_type_applied(&mut self, _id: TypeExprId, _name: Sym, _args: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_type_applied(&mut self, _id: TypeExprId, _name: Sym, _args: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_list(&mut self, _id: TypeExprId, _inner: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_type_list(&mut self, _id: TypeExprId, _inner: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_map(&mut self, _id: TypeExprId, _key: TypeExprId, _value: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_type_map(&mut self, _id: TypeExprId, _key: TypeExprId, _value: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_record(&mut self, _id: TypeExprId, _fields: &[TypeField], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_type_record(&mut self, _id: TypeExprId, _fields: &[TypeField], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_tuple(&mut self, _id: TypeExprId, _elems: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_type_tuple(&mut self, _id: TypeExprId, _elems: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_func(&mut self, _id: TypeExprId, _param: TypeExprId, _ret: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_type_func(&mut self, _id: TypeExprId, _param: TypeExprId, _ret: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
  fn visit_type_fallible(&mut self, _id: TypeExprId, _ok: TypeExprId, _err: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction { VisitAction::Descend }
  fn leave_type_fallible(&mut self, _id: TypeExprId, _ok: TypeExprId, _err: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> { ControlFlow::Continue(()) }
}
