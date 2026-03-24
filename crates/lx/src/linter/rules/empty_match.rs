use crate::ast::{AstArena, Expr, ExprId, ExprMatch, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use crate::visitor::{AstVisitor, PatternVisitor, TypeVisitor, VisitAction, dispatch_stmt};
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

impl PatternVisitor for EmptyMatch {}
impl TypeVisitor for EmptyMatch {}
impl AstVisitor for EmptyMatch {
  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan) -> VisitAction {
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
    VisitAction::Descend
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

  fn run(&mut self, stmts: &[StmtId], arena: &AstArena, _model: &SemanticModel) {
    for sid in stmts {
      if dispatch_stmt(self, *sid, arena).is_break() {
        break;
      }
    }
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    std::mem::take(&mut self.diagnostics)
  }
}
