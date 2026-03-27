use std::mem;

use crate::ast::{AstArena, Expr, ExprBlock, ExprId, ExprLoop, ExprPar, Stmt, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct UnreachableCode {
  diagnostics: Vec<Diagnostic>,
}

impl Default for UnreachableCode {
  fn default() -> Self {
    Self::new()
  }
}

impl UnreachableCode {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new() }
  }

  fn scan_stmts(&mut self, stmts: &[StmtId], arena: &AstArena) {
    let mut found_break = false;
    for &sid in stmts {
      let stmt_span = arena.stmt_span(sid);
      if found_break {
        self.diagnostics.push(Diagnostic {
          level: DiagLevel::Warning,
          kind: DiagnosticKind::LintWarning { rule_name: "unreachable_code".into(), message: "unreachable code after break".into() },
          code: "L004",
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
  }
}

impl LintRule for UnreachableCode {
  fn name(&self) -> &'static str {
    "unreachable_code"
  }

  fn code(&self) -> &'static str {
    "L004"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, arena: &AstArena, _model: &SemanticModel) {
    match expr {
      Expr::Block(ExprBlock { stmts }) | Expr::Loop(ExprLoop { stmts }) | Expr::Par(ExprPar { stmts }) => {
        self.scan_stmts(stmts, arena);
      },
      _ => {},
    }
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    mem::take(&mut self.diagnostics)
  }
}
