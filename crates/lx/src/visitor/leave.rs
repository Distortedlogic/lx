use std::ops::ControlFlow;

use crate::ast::{
  AstArena, Binding, ClassDeclData, Expr, ExprApply, ExprAssert, ExprBinary, ExprCoalesce, ExprEmit, ExprFieldAccess, ExprFunc, ExprId, ExprMatch,
  ExprNamedArg, ExprPipe, ExprSlice, ExprTernary, ExprTimeout, ExprUnary, ExprWith, ExprYield, FieldPattern, ListElem, Literal, MapEntry, Pattern, PatternId,
  Program, RecordField, Section, SelArm, Stmt, StmtFieldUpdate, StmtId, TraitDeclData, TypeExpr, TypeExprId, TypeField,
};

use crate::sym::Sym;
use miette::SourceSpan;

pub trait AstLeave {
  fn leave_program<P>(&mut self, _program: &Program<P>) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_stmt(&mut self, _stmt: &Stmt, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_binding(&mut self, _binding: &Binding, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_trait_decl(&mut self, _data: &TraitDeclData, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_class_decl(&mut self, _data: &ClassDeclData, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_field_update(&mut self, _update: &StmtFieldUpdate, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_expr(&mut self, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_literal(&mut self, _lit: &Literal, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_binary(&mut self, _binary: &ExprBinary, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_unary(&mut self, _unary: &ExprUnary, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pipe(&mut self, _pipe: &ExprPipe, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_apply(&mut self, _apply: &ExprApply, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_section(&mut self, _section: &Section, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_field_access(&mut self, _fa: &ExprFieldAccess, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_block(&mut self, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_tuple(&mut self, _elems: &[ExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_list(&mut self, _elems: &[ListElem], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_record(&mut self, _fields: &[RecordField], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_map(&mut self, _entries: &[MapEntry], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_func(&mut self, _func: &ExprFunc, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_match(&mut self, _m: &ExprMatch, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_ternary(&mut self, _ternary: &ExprTernary, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_propagate(&mut self, _inner: ExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_coalesce(&mut self, _coalesce: &ExprCoalesce, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_slice(&mut self, _slice: &ExprSlice, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_named_arg(&mut self, _na: &ExprNamedArg, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_loop(&mut self, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_break(&mut self, _value: Option<ExprId>, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_assert(&mut self, _assert: &ExprAssert, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_par(&mut self, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_sel(&mut self, _arms: &[SelArm], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_timeout(&mut self, _timeout: &ExprTimeout, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_emit(&mut self, _emit: &ExprEmit, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_yield(&mut self, _yld: &ExprYield, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_with(&mut self, _with: &ExprWith, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pattern(&mut self, _pattern: &Pattern, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pattern_tuple(&mut self, _elems: &[PatternId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pattern_list(&mut self, _elems: &[PatternId], _rest: Option<Sym>, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pattern_record(&mut self, _fields: &[FieldPattern], _rest: Option<Sym>, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_pattern_constructor(&mut self, _name: Sym, _args: &[PatternId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_expr(&mut self, _type_expr: &TypeExpr, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_applied(&mut self, _name: Sym, _args: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_list(&mut self, _inner: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_map(&mut self, _key: TypeExprId, _value: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_record(&mut self, _fields: &[TypeField], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_tuple(&mut self, _elems: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_func(&mut self, _param: TypeExprId, _ret: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
  fn leave_type_fallible(&mut self, _ok: TypeExprId, _err: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    ControlFlow::Continue(())
  }
}
