use std::mem;

use crate::ast::{AstArena, Expr, ExprId, ExprMatch};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct EmptyMatch {
  diagnostics: Vec<Diagnostic>,
}

impl Default for EmptyMatch {
  fn default() -> Self {
    Self::new()
  }
}

impl EmptyMatch {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new() }
  }
}

impl LintRule for EmptyMatch {
  fn name(&self) -> &'static str {
    "empty-match"
  }

  fn code(&self) -> &'static str {
    "L002"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {
    if let Expr::Match(ExprMatch { arms, .. }) = expr
      && arms.is_empty()
    {
      self.diagnostics.push(Diagnostic {
        level: DiagLevel::Warning,
        kind: DiagnosticKind::LintWarning { rule_name: "empty-match".into(), message: "match expression has zero arms".into() },
        code: "L002",
        span,
        secondary: Vec::new(),
        fix: None,
      });
    }
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    mem::take(&mut self.diagnostics)
  }
}
