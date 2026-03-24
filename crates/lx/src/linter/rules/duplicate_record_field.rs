use std::collections::HashSet;

use crate::ast::{Expr, ExprId, RecordField, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use crate::visitor::{AstVisitor, PatternVisitor, TypeVisitor, VisitAction, dispatch_stmt};
use miette::SourceSpan;

pub struct DuplicateRecordField {
  diagnostics: Vec<Diagnostic>,
}

impl Default for DuplicateRecordField {
  fn default() -> Self {
    Self::new()
  }
}

impl DuplicateRecordField {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new() }
  }
}

impl PatternVisitor for DuplicateRecordField {}
impl TypeVisitor for DuplicateRecordField {}
impl AstVisitor for DuplicateRecordField {
  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _arena: &crate::ast::AstArena) -> VisitAction {
    if let Expr::Record(fields) = expr {
      let mut seen = HashSet::new();
      for field in fields {
        if let RecordField::Named { name, .. } = field
          && !seen.insert(*name)
        {
          self.diagnostics.push(Diagnostic {
            level: DiagLevel::Error,
            kind: DiagnosticKind::LintWarning { rule_name: "duplicate_record_field".into(), message: format!("duplicate field '{name}' in record literal") },
            code: "L006",
            span,
            secondary: vec![],
            fix: None,
          });
        }
      }
    }
    VisitAction::Descend
  }
}

impl LintRule for DuplicateRecordField {
  fn name(&self) -> &'static str {
    "duplicate_record_field"
  }

  fn code(&self) -> &'static str {
    "L006"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn run(&mut self, stmts: &[StmtId], arena: &crate::ast::AstArena, _model: &SemanticModel) {
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
