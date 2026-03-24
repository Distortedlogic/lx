use super::registry::RuleRegistry;
use crate::ast::Program;
use crate::checker::Diagnostic;
use crate::checker::semantic::SemanticModel;
use crate::linter::rules::mut_never_mutated::check_unused_mut;

pub fn lint<P>(program: &Program<P>, model: &SemanticModel, registry: &mut RuleRegistry) -> Vec<Diagnostic> {
  let mut all_diags = Vec::new();
  for rule in registry.rules_mut() {
    rule.run(&program.stmts, &program.arena, model);
    all_diags.extend(rule.take_diagnostics());
  }
  all_diags.extend(check_unused_mut(program, model, &program.arena));
  all_diags
}
