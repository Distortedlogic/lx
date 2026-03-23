use std::collections::HashSet;

use crate::ast::{AstArena, Expr, ExprId, RecordField};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct DuplicateRecordField;

impl LintRule for DuplicateRecordField {
  fn name(&self) -> &'static str {
    "duplicate_record_field"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
    let Expr::Record(fields) = expr else {
      return vec![];
    };

    let mut seen = HashSet::new();
    let mut diags = vec![];

    for field in fields {
      if let RecordField::Named { name, .. } = field
        && !seen.insert(*name)
      {
        diags.push(Diagnostic {
          level: DiagLevel::Error,
          kind: DiagnosticKind::LintWarning { rule_name: "duplicate_record_field".into(), message: format!("duplicate field '{name}' in record literal") },
          span,
          secondary: vec![],
          fix: None,
        });
      }
    }

    diags
  }
}
