use crate::ast::{AstArena, Expr, ExprId, Pattern, PatternId, Program, Stmt, StmtId, TypeExpr, TypeExprId};
use miette::SourceSpan;

pub enum TransformOp<T> {
  Continue(T),
  Skip(T),
  Stop,
}

pub trait AstTransformer {
  fn transform_program<P>(&mut self, program: Program<P>) -> Program<P> {
    super::walk_transform::walk_transform_program(self, program)
  }

  fn transform_stmt(&mut self, _id: StmtId, stmt: Stmt, _span: SourceSpan, _arena: &mut AstArena) -> TransformOp<Stmt> {
    TransformOp::Continue(stmt)
  }

  fn transform_expr(&mut self, _id: ExprId, expr: Expr, _span: SourceSpan, _arena: &mut AstArena) -> TransformOp<Expr> {
    TransformOp::Continue(expr)
  }

  fn transform_pattern(&mut self, _id: PatternId, pattern: Pattern, _span: SourceSpan, _arena: &mut AstArena) -> TransformOp<Pattern> {
    TransformOp::Continue(pattern)
  }

  fn transform_type_expr(&mut self, _id: TypeExprId, te: TypeExpr, _span: SourceSpan, _arena: &mut AstArena) -> TransformOp<TypeExpr> {
    TransformOp::Continue(te)
  }

  fn leave_stmt(&mut self, _id: StmtId, stmt: Stmt, _span: SourceSpan, _arena: &mut AstArena) -> Stmt {
    stmt
  }

  fn leave_expr(&mut self, _id: ExprId, expr: Expr, _span: SourceSpan, _arena: &mut AstArena) -> Expr {
    expr
  }

  fn leave_pattern(&mut self, _id: PatternId, pattern: Pattern, _span: SourceSpan, _arena: &mut AstArena) -> Pattern {
    pattern
  }

  fn leave_type_expr(&mut self, _id: TypeExprId, te: TypeExpr, _span: SourceSpan, _arena: &mut AstArena) -> TypeExpr {
    te
  }
}
