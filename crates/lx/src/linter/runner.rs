use std::ops::ControlFlow;

use crate::ast::{AstArena, Expr, ExprId, Pattern, PatternId, Program, Stmt, StmtId};
use crate::checker::Diagnostic;
use crate::checker::semantic::SemanticModel;
use crate::linter::rules::mut_never_mutated::check_unused_mut;
use crate::visitor::{AstVisitor, VisitAction};
use miette::SourceSpan;

use super::registry::RuleRegistry;
use super::rule::LintRule;

pub struct LintRunner<'a> {
  rules: &'a mut [Box<dyn LintRule>],
  model: &'a SemanticModel,
  diagnostics: Vec<Diagnostic>,
}

impl<'a> LintRunner<'a> {
  fn new(rules: &'a mut [Box<dyn LintRule>], model: &'a SemanticModel) -> Self {
    Self { rules, model, diagnostics: Vec::new() }
  }
}

impl AstVisitor for LintRunner<'_> {
  fn visit_expr(&mut self, id: ExprId, expr: &Expr, span: SourceSpan, arena: &AstArena) -> VisitAction {
    for rule in self.rules.iter_mut() {
      rule.enter_expr(id, expr, span, arena);
      let mut diags = rule.check_expr(id, expr, span, self.model, arena);
      self.diagnostics.append(&mut diags);
    }
    VisitAction::Descend
  }

  fn leave_expr(&mut self, id: ExprId, expr: &Expr, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
    for rule in self.rules.iter_mut() {
      rule.leave_expr(id, expr, span, arena);
    }
    ControlFlow::Continue(())
  }

  fn visit_stmt(&mut self, id: StmtId, stmt: &Stmt, span: SourceSpan, arena: &AstArena) -> VisitAction {
    for rule in self.rules.iter_mut() {
      let mut diags = rule.check_stmt(id, stmt, span, self.model, arena);
      self.diagnostics.append(&mut diags);
    }
    VisitAction::Descend
  }

  fn visit_pattern(&mut self, id: PatternId, pattern: &Pattern, span: SourceSpan, arena: &AstArena) -> VisitAction {
    for rule in self.rules.iter_mut() {
      let mut diags = rule.check_pattern(id, pattern, span, self.model, arena);
      self.diagnostics.append(&mut diags);
    }
    VisitAction::Descend
  }
}

pub fn lint<P>(program: &Program<P>, model: &SemanticModel, registry: &mut RuleRegistry) -> Vec<Diagnostic> {
  let mut runner = LintRunner::new(registry.rules_mut(), model);
  runner.visit_program(program);
  let mut diags = runner.diagnostics;
  diags.extend(check_unused_mut(program, model, &program.arena));
  diags
}
