use crate::ast::{AstArena, Expr, ExprId, Stmt};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct UnreachableCode;

impl LintRule for UnreachableCode {
  fn name(&self) -> &'static str {
    "unreachable_code"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, _model: &SemanticModel, arena: &AstArena) -> Vec<Diagnostic> {
    let (Expr::Block(stmts) | Expr::Loop(stmts)) = expr else {
      return vec![];
    };

    let mut found_break = false;
    let mut diags = vec![];

    for &sid in stmts {
      let stmt_span = arena.stmt_span(sid);
      if found_break {
        diags.push(Diagnostic {
          level: DiagLevel::Warning,
          kind: DiagnosticKind::LintWarning { rule_name: "unreachable_code".into(), message: "unreachable code after break".into() },
          span: stmt_span,
          secondary: vec![],
          fix: None,
        });
        break;
      }

      let stmt = arena.stmt(sid);
      if let Stmt::Expr(eid) = stmt
        && matches!(arena.expr(*eid), Expr::Break(_))
      {
        found_break = true;
      }
    }

    diags
  }
}
