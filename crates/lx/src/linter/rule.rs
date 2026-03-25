use crate::ast::{AstArena, Core, Expr, ExprId, Program, Stmt, StmtId};
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
  fn code(&self) -> &'static str;
  fn category(&self) -> RuleCategory;
  fn run(&mut self, _program: &Program<Core>, _model: &SemanticModel) {}
  fn check_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {}
  fn leave_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {}
  fn check_stmt(&mut self, _id: StmtId, _stmt: &Stmt, _span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {}
  fn take_diagnostics(&mut self) -> Vec<Diagnostic>;
}
