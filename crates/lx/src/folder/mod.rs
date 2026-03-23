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
  Program, RecordField, Section, SelArm, StmtId, StmtTypeDef, TraitUnionDef, TypeExprId, UseStmt,
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

  fn fold_binding(&mut self, id: StmtId, binding: Binding, span: SourceSpan, arena: &mut AstArena) -> StmtId {
    fold_binding(self, id, binding, span, arena)
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

  fn fold_binary(&mut self, id: ExprId, b: ExprBinary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_binary(self, id, b, span, arena)
  }

  fn fold_unary(&mut self, id: ExprId, u: ExprUnary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_unary(self, id, u, span, arena)
  }

  fn fold_pipe(&mut self, id: ExprId, p: ExprPipe, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_pipe(self, id, p, span, arena)
  }

  fn fold_apply(&mut self, id: ExprId, a: ExprApply, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_apply(self, id, a, span, arena)
  }

  fn fold_section(&mut self, id: ExprId, s: Section, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_section(self, id, s, span, arena)
  }

  fn fold_field_access(&mut self, id: ExprId, fa: ExprFieldAccess, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_field_access(self, id, fa, span, arena)
  }

  fn fold_block(&mut self, id: ExprId, stmts: &[StmtId], span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_block(self, id, stmts, span, arena)
  }

  fn fold_tuple(&mut self, id: ExprId, elems: &[ExprId], span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_tuple(self, id, elems, span, arena)
  }

  fn fold_list(&mut self, id: ExprId, elems: Vec<ListElem>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_list(self, id, elems, span, arena)
  }

  fn fold_record(&mut self, id: ExprId, fields: Vec<RecordField>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_record(self, id, fields, span, arena)
  }

  fn fold_map(&mut self, id: ExprId, entries: Vec<MapEntry>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_map(self, id, entries, span, arena)
  }

  fn fold_func(&mut self, id: ExprId, func: ExprFunc, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_func(self, id, func, span, arena)
  }

  fn fold_match(&mut self, id: ExprId, m: ExprMatch, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_match(self, id, m, span, arena)
  }

  fn fold_ternary(&mut self, id: ExprId, t: ExprTernary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_ternary(self, id, t, span, arena)
  }

  fn fold_propagate(&mut self, id: ExprId, inner: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_propagate(self, id, inner, span, arena)
  }

  fn fold_coalesce(&mut self, id: ExprId, c: ExprCoalesce, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_coalesce(self, id, c, span, arena)
  }

  fn fold_slice(&mut self, id: ExprId, s: ExprSlice, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_slice(self, id, s, span, arena)
  }

  fn fold_named_arg(&mut self, id: ExprId, na: ExprNamedArg, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_named_arg(self, id, na, span, arena)
  }

  fn fold_loop(&mut self, id: ExprId, stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_loop(self, id, stmts, span, arena)
  }

  fn fold_break(&mut self, id: ExprId, val: Option<ExprId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_break(self, id, val, span, arena)
  }

  fn fold_assert(&mut self, id: ExprId, a: ExprAssert, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_assert(self, id, a, span, arena)
  }

  fn fold_par(&mut self, id: ExprId, stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_par(self, id, stmts, span, arena)
  }

  fn fold_sel(&mut self, id: ExprId, arms: Vec<SelArm>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_sel(self, id, arms, span, arena)
  }

  fn fold_timeout(&mut self, id: ExprId, t: ExprTimeout, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_timeout(self, id, t, span, arena)
  }

  fn fold_emit(&mut self, id: ExprId, e: ExprEmit, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_emit(self, id, e, span, arena)
  }

  fn fold_yield(&mut self, id: ExprId, y: ExprYield, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_yield(self, id, y, span, arena)
  }

  fn fold_with(&mut self, id: ExprId, w: ExprWith, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    fold_with(self, id, w, span, arena)
  }

  fn fold_type_def(&mut self, id: StmtId, _def: StmtTypeDef, _span: SourceSpan, _arena: &mut AstArena) -> StmtId {
    id
  }
  fn fold_trait_union(&mut self, id: StmtId, _def: TraitUnionDef, _span: SourceSpan, _arena: &mut AstArena) -> StmtId {
    id
  }
  fn fold_use(&mut self, id: StmtId, _stmt: UseStmt, _span: SourceSpan, _arena: &mut AstArena) -> StmtId {
    id
  }

  fn fold_pattern(&mut self, id: PatternId, arena: &mut AstArena) -> PatternId {
    fold_pattern(self, id, arena)
  }

  fn fold_pattern_list(&mut self, id: PatternId, pl: &PatternList, span: SourceSpan, arena: &mut AstArena) -> PatternId {
    fold_pattern_list(self, id, pl, span, arena)
  }

  fn fold_pattern_record(&mut self, id: PatternId, pr: &PatternRecord, span: SourceSpan, arena: &mut AstArena) -> PatternId {
    fold_pattern_record(self, id, pr, span, arena)
  }

  fn fold_pattern_constructor(&mut self, id: PatternId, pc: &PatternConstructor, span: SourceSpan, arena: &mut AstArena) -> PatternId {
    fold_pattern_constructor(self, id, pc, span, arena)
  }

  fn fold_type_expr(&mut self, id: TypeExprId, arena: &mut AstArena) -> TypeExprId {
    fold_type_expr(self, id, arena)
  }
}
