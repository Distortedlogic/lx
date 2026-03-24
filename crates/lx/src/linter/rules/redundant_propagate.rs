use crate::ast::{AstArena, Expr, ExprId, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::types::Type;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use crate::visitor::{AstVisitor, VisitAction, dispatch_stmt};
use miette::SourceSpan;

pub struct RedundantPropagate {
  diagnostics: Vec<Diagnostic>,
  model: Option<*const SemanticModel>,
}

impl RedundantPropagate {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new(), model: None }
  }
}

impl AstVisitor for RedundantPropagate {
  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    if let Expr::Propagate(inner_id) = expr {
      let model = unsafe { &*self.model.unwrap() };
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
              span,
              secondary: Vec::new(),
              fix: None,
            });
          },
        }
      }
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
    self.model = Some(model as *const _);
    for sid in stmts {
      let _ = dispatch_stmt(self, *sid, arena);
    }
    self.model = None;
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    std::mem::take(&mut self.diagnostics)
  }
}
