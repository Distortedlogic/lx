use crate::ast::{AstArena, Expr, ExprId, Pattern, PatternId, Program, Stmt, StmtId, TypeExpr, TypeExprId};
use miette::SourceSpan;

pub enum TransformOp<T> {
  Continue,
  Replace(T),
  Stop,
}

pub trait AstTransformer {
  fn transform_program<P>(&mut self, program: Program<P>) -> Program<P> {
    super::walk_transform::walk_transform_program(self, program)
  }

  fn transform_stmt(&mut self, _id: StmtId, _stmt: &Stmt, _span: SourceSpan, _arena: &AstArena) -> TransformOp<Stmt> {
    TransformOp::Continue
  }

  fn transform_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) -> TransformOp<Expr> {
    TransformOp::Continue
  }

  fn transform_pattern(&mut self, _id: PatternId, _pattern: &Pattern, _span: SourceSpan, _arena: &AstArena) -> TransformOp<Pattern> {
    TransformOp::Continue
  }

  fn transform_type_expr(&mut self, _id: TypeExprId, _te: &TypeExpr, _span: SourceSpan, _arena: &AstArena) -> TransformOp<TypeExpr> {
    TransformOp::Continue
  }

  fn leave_stmt(&mut self, _id: StmtId, stmt: Stmt, span: SourceSpan, _arena: &mut AstArena) -> (Stmt, SourceSpan) {
    (stmt, span)
  }

  fn leave_expr(&mut self, _id: ExprId, expr: Expr, span: SourceSpan, _arena: &mut AstArena) -> (Expr, SourceSpan) {
    (expr, span)
  }

  fn leave_pattern(&mut self, _id: PatternId, pattern: Pattern, span: SourceSpan, _arena: &mut AstArena) -> (Pattern, SourceSpan) {
    (pattern, span)
  }

  fn leave_type_expr(&mut self, _id: TypeExprId, te: TypeExpr, span: SourceSpan, _arena: &mut AstArena) -> (TypeExpr, SourceSpan) {
    (te, span)
  }

  fn transform_stmts(&mut self, stmts: Vec<StmtId>, arena: &mut AstArena) -> Vec<StmtId> {
    stmts.into_iter().map(|s| super::walk_transform::walk_transform_stmt(self, s, arena)).collect()
  }
}
