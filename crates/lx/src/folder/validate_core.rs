use crate::ast::{Core, WithKind};
use crate::visitor::prelude::*;

struct CoreValidator;

impl AstVisitor for CoreValidator {
  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan) -> VisitAction {
    match expr {
      Expr::Pipe(_) => panic!("Core AST contains Expr::Pipe at offset {}. The desugarer should have converted this to Expr::Apply.", span.offset()),
      Expr::Section(_) => panic!("Core AST contains Expr::Section at offset {}. The desugarer should have converted this to a lambda.", span.offset()),
      Expr::Ternary(_) => panic!("Core AST contains Expr::Ternary at offset {}. The desugarer should have converted this to Expr::Match.", span.offset()),
      Expr::Coalesce(_) => panic!("Core AST contains Expr::Coalesce at offset {}. The desugarer should have converted this to Expr::Match.", span.offset()),
      Expr::With(w) if matches!(w.kind, WithKind::Binding { .. }) => {
        panic!("Core AST contains Expr::With(Binding) at offset {}. The desugarer should have converted this to Expr::Block.", span.offset())
      },
      _ => VisitAction::Descend,
    }
  }
}

pub(super) fn validate_core(program: &Program<Core>) {
  let mut validator = CoreValidator;
  let _ = walk_program(&mut validator, program);
}
