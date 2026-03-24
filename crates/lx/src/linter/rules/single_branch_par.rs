use crate::ast::{AstArena, Expr, ExprId, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use crate::visitor::{AstVisitor, VisitAction, dispatch_stmt};
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

impl AstVisitor for SingleBranchPar {
  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    if let Expr::Par(stmts) = expr
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
    VisitAction::Descend
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
