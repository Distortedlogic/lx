use crate::ast::{AstArena, Expr, ExprId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct BreakOutsideLoop {
  loop_depth: usize,
  diagnostics: Vec<Diagnostic>,
}

impl Default for BreakOutsideLoop {
  fn default() -> Self {
    Self::new()
  }
}

impl BreakOutsideLoop {
  pub fn new() -> Self {
    Self { loop_depth: 0, diagnostics: Vec::new() }
  }
}

impl LintRule for BreakOutsideLoop {
  fn name(&self) -> &'static str {
    "break_outside_loop"
  }

  fn code(&self) -> &'static str {
    "L001"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {
    if matches!(expr, Expr::Loop(_)) {
      self.loop_depth += 1;
    }
    if matches!(expr, Expr::Break(_)) && self.loop_depth == 0 {
      self.diagnostics.push(Diagnostic {
        level: DiagLevel::Error,
        kind: DiagnosticKind::LintWarning { rule_name: "break_outside_loop".into(), message: "break used outside of a loop".into() },
        code: "L001",
        span,
        secondary: vec![],
        fix: None,
      });
    }
  }

  fn leave_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {
    if matches!(expr, Expr::Loop(_)) {
      self.loop_depth -= 1;
    }
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    std::mem::take(&mut self.diagnostics)
  }
}
