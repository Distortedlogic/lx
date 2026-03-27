use std::mem;

use crate::ast::{AstArena, Expr, ExprId, ExprPar};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct SingleBranchPar {
  diagnostics: Vec<Diagnostic>,
}

impl Default for SingleBranchPar {
  fn default() -> Self {
    Self::new()
  }
}

impl SingleBranchPar {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new() }
  }
}

impl LintRule for SingleBranchPar {
  fn name(&self) -> &'static str {
    "single_branch_par"
  }

  fn code(&self) -> &'static str {
    "L007"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {
    if let Expr::Par(ExprPar { stmts }) = expr
      && stmts.len() <= 1
    {
      self.diagnostics.push(Diagnostic {
        level: DiagLevel::Warning,
        kind: DiagnosticKind::LintWarning {
          rule_name: "single_branch_par".into(),
          message: "par block with a single branch has no concurrency — use the expression directly".into(),
        },
        code: "L007",
        span,
        secondary: vec![],
        fix: None,
      });
    }
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    mem::take(&mut self.diagnostics)
  }
}
