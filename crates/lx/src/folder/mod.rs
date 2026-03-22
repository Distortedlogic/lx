pub mod desugar;
mod fold_expr;
mod fold_expr_compound;
mod fold_pattern;
mod fold_stmt;
mod fold_type;

pub use desugar::desugar;
pub use fold_expr::*;
pub use fold_expr_compound::*;
pub use fold_pattern::*;
pub use fold_stmt::*;
pub use fold_type::*;

use crate::ast::{
  AstArena, Binding, ExprApply, ExprAssert, ExprBinary, ExprCoalesce, ExprEmit, ExprFieldAccess, ExprFunc, ExprId, ExprMatch, ExprNamedArg, ExprPipe,
  ExprSlice, ExprTernary, ExprTimeout, ExprUnary, ExprWith, ExprYield, ListElem, Literal, MapEntry, PatternConstructor, PatternId, PatternList, PatternRecord,
  Program, RecordField, Section, SelArm, StmtId, TypeExprId,
};
use crate::sym::Sym;
use miette::SourceSpan;

pub trait AstFolder {
  fn fold_program<P>(&mut self, program: Program<P>) -> Program<P> {
    fold_program(self, program)
  }

  fn fold_stmt(&mut self, id: StmtId, arena: &mut AstArena) -> StmtId {
    fold_stmt(self, id, arena)
  }

  fn fold_binding(&mut self, binding: Binding, span: SourceSpan, arena: &mut AstArena) -> StmtId {
    fold_binding(self, binding, span, arena)
  }

  fn fold_expr(&mut self, id: ExprId, arena: &mut AstArena) -> ExprId {
    fold_expr(self, id, arena)
  }

  fn fold_literal(&mut self, lit: Literal, span: SourceSpan, arena: &mut AstArena) -> Literal {
    fold_literal(self, lit, span, arena)
  }

  fn fold_ident(&mut self, name: Sym, _span: SourceSpan, _arena: &mut AstArena) -> Sym {
    name
  }

  fn fold_type_constructor(&mut self, name: Sym, _span: SourceSpan, _arena: &mut AstArena) -> Sym {
    name
  }

  fn fold_binary(&mut self, b: ExprBinary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_binary(self, b, span, arena)
  }

  fn fold_unary(&mut self, u: ExprUnary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_unary(self, u, span, arena)
  }

  fn fold_pipe(&mut self, p: ExprPipe, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_pipe(self, p, span, arena)
  }

  fn fold_apply(&mut self, a: ExprApply, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_apply(self, a, span, arena)
  }

  fn fold_section(&mut self, s: Section, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_section(self, s, span, arena)
  }

  fn fold_field_access(&mut self, fa: ExprFieldAccess, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_field_access(self, fa, span, arena)
  }

  fn fold_block(&mut self, stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_block(self, stmts, span, arena)
  }

  fn fold_tuple(&mut self, elems: Vec<ExprId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_tuple(self, elems, span, arena)
  }

  fn fold_list(&mut self, elems: Vec<ListElem>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_list(self, elems, span, arena)
  }

  fn fold_record(&mut self, fields: Vec<RecordField>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_record(self, fields, span, arena)
  }

  fn fold_map(&mut self, entries: Vec<MapEntry>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_map(self, entries, span, arena)
  }

  fn fold_func(&mut self, func: ExprFunc, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_func(self, func, span, arena)
  }

  fn fold_match(&mut self, m: ExprMatch, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_match(self, m, span, arena)
  }

  fn fold_ternary(&mut self, t: ExprTernary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_ternary(self, t, span, arena)
  }

  fn fold_propagate(&mut self, inner: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_propagate(self, inner, span, arena)
  }

  fn fold_coalesce(&mut self, c: ExprCoalesce, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_coalesce(self, c, span, arena)
  }

  fn fold_slice(&mut self, s: ExprSlice, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_slice(self, s, span, arena)
  }

  fn fold_named_arg(&mut self, na: ExprNamedArg, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_named_arg(self, na, span, arena)
  }

  fn fold_loop(&mut self, stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_loop(self, stmts, span, arena)
  }

  fn fold_break(&mut self, val: Option<ExprId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_break(self, val, span, arena)
  }

  fn fold_assert(&mut self, a: ExprAssert, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_assert(self, a, span, arena)
  }

  fn fold_par(&mut self, stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_par(self, stmts, span, arena)
  }

  fn fold_sel(&mut self, arms: Vec<SelArm>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_sel(self, arms, span, arena)
  }

  fn fold_timeout(&mut self, t: ExprTimeout, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_timeout(self, t, span, arena)
  }

  fn fold_emit(&mut self, e: ExprEmit, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_emit(self, e, span, arena)
  }

  fn fold_yield(&mut self, y: ExprYield, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_yield(self, y, span, arena)
  }

  fn fold_with(&mut self, w: ExprWith, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_with(self, w, span, arena)
  }

  fn fold_pattern(&mut self, id: PatternId, arena: &mut AstArena) -> PatternId {
    fold_pattern(self, id, arena)
  }

  fn fold_pattern_list(&mut self, pl: PatternList, span: SourceSpan, arena: &mut AstArena) -> PatternId {
    fold_pattern_list(self, pl, span, arena)
  }

  fn fold_pattern_record(&mut self, pr: PatternRecord, span: SourceSpan, arena: &mut AstArena) -> PatternId {
    fold_pattern_record(self, pr, span, arena)
  }

  fn fold_pattern_constructor(&mut self, pc: PatternConstructor, span: SourceSpan, arena: &mut AstArena) -> PatternId {
    fold_pattern_constructor(self, pc, span, arena)
  }

  fn fold_type_expr(&mut self, id: TypeExprId, arena: &mut AstArena) -> TypeExprId {
    fold_type_expr(self, id, arena)
  }
}
