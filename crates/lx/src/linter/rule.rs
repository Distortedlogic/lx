use crate::ast::{AstArena, StmtId};
use crate::checker::Diagnostic;
use crate::checker::semantic::SemanticModel;

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
  fn run(&mut self, stmts: &[StmtId], arena: &AstArena, model: &SemanticModel);
  fn take_diagnostics(&mut self) -> Vec<Diagnostic>;
}
