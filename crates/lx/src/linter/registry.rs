use super::rule::LintRule;

pub struct RuleRegistry {
  rules: Vec<Box<dyn LintRule>>,
}

impl Default for RuleRegistry {
  fn default() -> Self {
    Self::new()
  }
}

impl RuleRegistry {
  pub fn new() -> Self {
    Self { rules: Vec::new() }
  }

  pub fn register(&mut self, rule: Box<dyn LintRule>) {
    self.rules.push(rule);
  }

  pub fn rules_mut(&mut self) -> &mut [Box<dyn LintRule>] {
    &mut self.rules
  }

  pub fn default_rules() -> Self {
    let mut registry = Self::new();
    registry.register(Box::new(super::rules::empty_match::EmptyMatch));
    registry.register(Box::new(super::rules::redundant_propagate::RedundantPropagate));
    registry.register(Box::new(super::rules::break_outside_loop::BreakOutsideLoop::new()));
    registry.register(Box::new(super::rules::unreachable_code::UnreachableCode));
    registry.register(Box::new(super::rules::unused_import::UnusedImport));
    registry.register(Box::new(super::rules::duplicate_record_field::DuplicateRecordField));
    registry.register(Box::new(super::rules::single_branch_par::SingleBranchPar));
    registry
  }
}
