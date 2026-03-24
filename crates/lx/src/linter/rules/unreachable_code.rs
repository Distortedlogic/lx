use crate::ast::{AstArena, Expr, ExprId, Stmt, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use crate::visitor::{AstVisitor, VisitAction, dispatch_stmt};
use miette::SourceSpan;

pub struct UnreachableCode {
  diagnostics: Vec<Diagnostic>,
}

impl UnreachableCode {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new() }
  }
}

impl AstVisitor for UnreachableCode {
  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, arena: &AstArena) -> VisitAction {
    let (Expr::Block(stmts) | Expr::Loop(stmts)) = expr else {
      return VisitAction::Descend;
    };

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

    VisitAction::Descend
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

  fn run(&mut self, stmts: &[StmtId], arena: &AstArena, _model: &SemanticModel) {
    for sid in stmts {
      let _ = dispatch_stmt(self, *sid, arena);
    }
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    std::mem::take(&mut self.diagnostics)
  }
}
