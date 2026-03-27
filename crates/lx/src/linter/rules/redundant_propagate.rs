use std::mem;

use crate::ast::{AstArena, Expr, ExprId, ExprPropagate};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::types::Type;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct RedundantPropagate {
  diagnostics: Vec<Diagnostic>,
}

impl Default for RedundantPropagate {
  fn default() -> Self {
    Self::new()
  }
}

impl RedundantPropagate {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new() }
  }
}

impl LintRule for RedundantPropagate {
  fn name(&self) -> &'static str {
    "redundant-propagate"
  }

  fn code(&self) -> &'static str {
    "L003"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _arena: &AstArena, model: &SemanticModel) {
    if let Expr::Propagate(ExprPropagate { inner: inner_id }) = expr
      && let Some(type_id) = model.type_of_expr(*inner_id)
    {
      let ty = model.type_arena.get(type_id);
      match ty {
        Type::Result { .. } | Type::Maybe(_) => {},
        _ => {
          self.diagnostics.push(Diagnostic {
            level: DiagLevel::Warning,
            kind: DiagnosticKind::LintWarning {
              rule_name: "redundant-propagate".into(),
              message: format!("propagate (^) on non-Result/Maybe type `{}`", model.type_arena.display(type_id)),
            },
            code: "L003",
            span,
            secondary: Vec::new(),
            fix: None,
          });
        },
      }
    }
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    mem::take(&mut self.diagnostics)
  }
}
