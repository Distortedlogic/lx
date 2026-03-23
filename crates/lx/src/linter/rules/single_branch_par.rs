use crate::ast::{AstArena, Expr, ExprId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct SingleBranchPar;

impl LintRule for SingleBranchPar {
  fn name(&self) -> &'static str {
    "single_branch_par"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
    if let Expr::Par(stmts) = expr
      && stmts.len() <= 1
    {
      return vec![Diagnostic {
        level: DiagLevel::Warning,
        kind: DiagnosticKind::LintWarning {
          rule_name: "single_branch_par".into(),
          message: "par block with a single branch has no concurrency — use the expression directly".into(),
        },
        span,
        secondary: vec![],
        fix: None,
      }];
    }
    vec![]
  }
}
