use std::ops::ControlFlow;

use crate::ast::{
  AstArena, Binding, ClassDeclData, Expr, ExprApply, ExprAssert, ExprBinary, ExprCoalesce, ExprEmit, ExprFieldAccess, ExprFunc, ExprId, ExprMatch,
  ExprNamedArg, ExprPipe, ExprSlice, ExprTernary, ExprTimeout, ExprUnary, ExprWith, ExprYield, FieldPattern, ListElem, Literal, MapEntry, Pattern, PatternId,
  Program, RecordField, Section, SelArm, Stmt, StmtFieldUpdate, StmtId, StmtTypeDef, TraitDeclData, TraitUnionDef, TypeExpr, TypeExprId, TypeField, UseStmt,
};
use crate::sym::Sym;
use miette::SourceSpan;

mod leave;
mod walk;
pub use leave::*;
pub use walk::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitAction {
  Descend,
  Skip,
  Stop,
}

impl VisitAction {
  pub fn is_stop(self) -> bool {
    self == VisitAction::Stop
  }

  pub fn to_control_flow(self) -> ControlFlow<()> {
    match self {
      VisitAction::Stop => ControlFlow::Break(()),
      _ => ControlFlow::Continue(()),
    }
  }
}

pub fn cf_to_action(cf: ControlFlow<()>) -> VisitAction {
  match cf {
    ControlFlow::Continue(()) => VisitAction::Skip,
    ControlFlow::Break(()) => VisitAction::Stop,
  }
}

pub trait AstVisitor: AstLeave {
  fn visit_program<P>(&mut self, program: &Program<P>) -> VisitAction {
    cf_to_action(walk_program(self, program))
  }
  fn on_stmt(&mut self, _stmt: &Stmt, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_binding(&mut self, binding: &Binding, span: SourceSpan, arena: &AstArena) -> VisitAction {
    cf_to_action(walk_binding(self, binding, span, arena))
  }
  fn visit_type_def(&mut self, _def: &StmtTypeDef, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_trait_decl(&mut self, _data: &TraitDeclData, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_class_decl(&mut self, _data: &ClassDeclData, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_trait_union(&mut self, _def: &TraitUnionDef, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_field_update(&mut self, _update: &StmtFieldUpdate, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_use(&mut self, _stmt: &UseStmt, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn on_expr(&mut self, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_literal(&mut self, _lit: &Literal, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_ident(&mut self, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_constructor(&mut self, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_binary(&mut self, _binary: &ExprBinary, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_unary(&mut self, _unary: &ExprUnary, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pipe(&mut self, _pipe: &ExprPipe, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_apply(&mut self, _apply: &ExprApply, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_section(&mut self, _section: &Section, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_field_access(&mut self, _fa: &ExprFieldAccess, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_block(&mut self, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_tuple(&mut self, _elems: &[ExprId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_list(&mut self, _elems: &[ListElem], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_record(&mut self, _fields: &[RecordField], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_map(&mut self, _entries: &[MapEntry], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_func(&mut self, _func: &ExprFunc, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_match(&mut self, _m: &ExprMatch, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_ternary(&mut self, _ternary: &ExprTernary, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_propagate(&mut self, _inner: ExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_coalesce(&mut self, _coalesce: &ExprCoalesce, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_slice(&mut self, _slice: &ExprSlice, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_named_arg(&mut self, _na: &ExprNamedArg, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_loop(&mut self, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_break(&mut self, _value: Option<ExprId>, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_assert(&mut self, _assert: &ExprAssert, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_par(&mut self, _stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_sel(&mut self, _arms: &[SelArm], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_timeout(&mut self, _timeout: &ExprTimeout, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_emit(&mut self, _emit: &ExprEmit, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_yield(&mut self, _yld: &ExprYield, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_with(&mut self, _with: &ExprWith, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern(&mut self, _pattern: &Pattern, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_literal(&mut self, _lit: &Literal, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_bind(&mut self, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_wildcard(&mut self, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_tuple(&mut self, _elems: &[PatternId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_list(&mut self, _elems: &[PatternId], _rest: Option<Sym>, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_record(&mut self, _fields: &[FieldPattern], _rest: Option<Sym>, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_constructor(&mut self, _name: Sym, _args: &[PatternId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_expr(&mut self, _type_expr: &TypeExpr, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_named(&mut self, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_var(&mut self, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_applied(&mut self, _name: Sym, _args: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_list(&mut self, _inner: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_map(&mut self, _key: TypeExprId, _value: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_record(&mut self, _fields: &[TypeField], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_tuple(&mut self, _elems: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_func(&mut self, _param: TypeExprId, _ret: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_type_fallible(&mut self, _ok: TypeExprId, _err: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
  }
}
