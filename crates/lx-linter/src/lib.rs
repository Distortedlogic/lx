mod registry;
mod rule;
pub mod rules;
mod runner;

pub use registry::RuleRegistry;
pub use rule::{LintRule, RuleCategory};
pub use runner::lint;
