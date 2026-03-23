use crate::ast::{AstArena, Expr, ExprId, Pattern, PatternId, Stmt, StmtId};
use crate::checker::Diagnostic;
use crate::checker::semantic::SemanticModel;
use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
  Style,
  Correctness,
  Performance,
  Concurrency,
}

pub trait LintRule {
  fn name(&self) -> &'static str;
  fn category(&self) -> RuleCategory;

  fn check_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
    vec![]
  }

  fn check_stmt(&mut self, _id: StmtId, _stmt: &Stmt, _span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
    vec![]
  }

  fn check_pattern(&mut self, _id: PatternId, _pattern: &Pattern, _span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
    vec![]
  }

  fn enter_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) {}
  fn leave_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) {}
}
