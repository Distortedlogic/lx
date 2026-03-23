use crate::ast::{AstArena, Expr, ExprId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct BreakOutsideLoop {
  loop_depth: usize,
}

impl Default for BreakOutsideLoop {
  fn default() -> Self {
    Self::new()
  }
}

impl BreakOutsideLoop {
  pub fn new() -> Self {
    Self { loop_depth: 0 }
  }
}

impl LintRule for BreakOutsideLoop {
  fn name(&self) -> &'static str {
    "break_outside_loop"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn enter_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, _arena: &AstArena) {
    if matches!(expr, Expr::Loop(_)) {
      self.loop_depth += 1;
    }
  }

  fn leave_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, _arena: &AstArena) {
    if matches!(expr, Expr::Loop(_)) {
      self.loop_depth -= 1;
    }
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
    if matches!(expr, Expr::Break(_)) && self.loop_depth == 0 {
      return vec![Diagnostic {
        level: DiagLevel::Error,
        kind: DiagnosticKind::LintWarning { rule_name: "break_outside_loop".into(), message: "break used outside of a loop".into() },
        span,
        secondary: vec![],
        fix: None,
      }];
    }
    vec![]
  }
}
