use crate::ast::{AstArena, Expr, ExprId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::matcher::ExprMatcher;
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct EmptyMatch;

impl LintRule for EmptyMatch {
  fn name(&self) -> &'static str {
    "empty-match"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _model: &SemanticModel, arena: &AstArena) -> Vec<Diagnostic> {
    if ExprMatcher::empty_match().matches(expr, arena) {
      return vec![Diagnostic {
        level: DiagLevel::Warning,
        kind: DiagnosticKind::LintWarning { rule_name: self.name().into(), message: "match expression has zero arms".into() },
        span,
        secondary: Vec::new(),
        fix: None,
      }];
    }
    vec![]
  }
}
