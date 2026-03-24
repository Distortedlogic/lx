use crate::ast::{AstArena, Expr, ExprId, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use crate::visitor::{AstVisitor, PatternVisitor, TypeVisitor, VisitAction, dispatch_stmt};
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

impl PatternVisitor for BreakOutsideLoop {}
impl TypeVisitor for BreakOutsideLoop {}
impl AstVisitor for BreakOutsideLoop {
  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan) -> VisitAction {
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
    VisitAction::Descend
  }

  fn leave_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan) {
    if matches!(expr, Expr::Loop(_)) {
      self.loop_depth -= 1;
    }
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
