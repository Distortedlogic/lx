use crate::ast::{AstArena, Expr, ExprId, ExprPropagate, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::types::Type;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use crate::visitor::{AstVisitor, PatternVisitor, TypeVisitor, VisitAction, dispatch_stmt};
use miette::SourceSpan;

pub struct RedundantPropagate {
  diagnostics: Vec<Diagnostic>,
  candidates: Vec<(ExprId, SourceSpan)>,
}

impl Default for RedundantPropagate {
  fn default() -> Self {
    Self::new()
  }
}

impl RedundantPropagate {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new(), candidates: Vec::new() }
  }
}

impl PatternVisitor for RedundantPropagate {}
impl TypeVisitor for RedundantPropagate {}
impl AstVisitor for RedundantPropagate {
  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan) -> VisitAction {
    if let Expr::Propagate(ExprPropagate { inner: inner_id }) = expr {
      self.candidates.push((*inner_id, span));
    }
    VisitAction::Descend
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

  fn run(&mut self, stmts: &[StmtId], arena: &AstArena, model: &SemanticModel) {
    for sid in stmts {
      if dispatch_stmt(self, *sid, arena).is_break() {
        break;
      }
    }

    for (inner_id, span) in &self.candidates {
      if let Some(type_id) = model.type_of_expr(*inner_id) {
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
              span: *span,
              secondary: Vec::new(),
              fix: None,
            });
          },
        }
      }
    }

    self.candidates.clear();
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    std::mem::take(&mut self.diagnostics)
  }
}
