use std::collections::HashSet;
use std::mem;

use crate::rule::{LintRule, RuleCategory};
use lx_ast::ast::{AstArena, Expr, ExprId, RecordField};
use lx_checker::diagnostics::DiagnosticKind;
use lx_checker::semantic::SemanticModel;
use lx_checker::{DiagLevel, Diagnostic};
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

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {
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
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    mem::take(&mut self.diagnostics)
  }
}
