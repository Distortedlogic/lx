use super::registry::RuleRegistry;
use super::rule::LintRule;
use crate::rules::mut_never_mutated::check_unused_mut;
use lx_ast::ast::Core;
use lx_ast::visitor::prelude::*;
use lx_checker::Diagnostic;
use lx_checker::semantic::SemanticModel;

struct LintWalker<'a> {
  rules: &'a mut [Box<dyn LintRule>],
  arena: &'a AstArena,
  model: &'a SemanticModel,
}

impl AstVisitor for LintWalker<'_> {
  fn visit_expr(&mut self, id: ExprId, expr: &Expr, span: SourceSpan) -> VisitAction {
    for rule in self.rules.iter_mut() {
      rule.check_expr(id, expr, span, self.arena, self.model);
    }
    VisitAction::Descend
  }

  fn leave_expr(&mut self, id: ExprId, expr: &Expr, span: SourceSpan) {
    for rule in self.rules.iter_mut() {
      rule.leave_expr(id, expr, span, self.arena, self.model);
    }
  }

  fn visit_stmt(&mut self, id: StmtId, stmt: &Stmt, span: SourceSpan) -> VisitAction {
    for rule in self.rules.iter_mut() {
      rule.check_stmt(id, stmt, span, self.arena, self.model);
    }
    VisitAction::Descend
  }
}

pub fn lint(program: &Program<Core>, model: &SemanticModel, registry: &mut RuleRegistry) -> Vec<Diagnostic> {
  let mut all_diags = Vec::new();

  for rule in registry.rules_mut() {
    rule.run(program, model);
  }

  let mut walker = LintWalker { rules: registry.rules_mut(), arena: &program.arena, model };
  let _ = walk_program(&mut walker, program);

  for rule in registry.rules_mut() {
    all_diags.extend(rule.take_diagnostics());
  }

  all_diags.extend(check_unused_mut(program, model, &program.arena));
  all_diags
}
