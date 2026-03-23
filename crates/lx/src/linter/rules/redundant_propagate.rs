use crate::ast::{AstArena, Expr, ExprId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::types::Type;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct RedundantPropagate;

impl LintRule for RedundantPropagate {
  fn name(&self) -> &'static str {
    "redundant-propagate"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
    if let Expr::Propagate(inner_id) = expr
      && let Some(type_id) = model.type_of_expr(*inner_id)
    {
      let ty = model.type_arena.get(type_id);
      match ty {
        Type::Result { .. } | Type::Maybe(_) => {},
        _ => {
          return vec![Diagnostic {
            level: DiagLevel::Warning,
            kind: DiagnosticKind::LintWarning {
              rule_name: self.name().into(),
              message: format!("propagate (^) on non-Result/Maybe type `{}`", model.type_arena.display(type_id)),
            },
            span,
            secondary: Vec::new(),
            fix: None,
          }];
        },
      }
    }
    vec![]
  }
}
