use std::sync::Arc;

use lx_ast::ast::{Core, Program};
use lx_checker::{check, Diagnostic};
use lx_desugar::desugar;
use lx_fmt::format;
use lx_linter::{RuleRegistry, lint};
use lx_parser::{lexer::lex, parser::parse};
use lx_span::source::FileId;

fn parse_core(source: &str) -> Program<Core> {
  let (tokens, comments) = lex(source).unwrap_or_else(|err| panic!("lex failed for fixture:\n{source}\n{err}"));
  let parsed = parse(tokens, FileId::new(0), comments, source);
  assert!(parsed.errors.is_empty(), "parse failed for fixture:\n{source}\n{:?}", parsed.errors);
  desugar(parsed.program.expect("parser returned no program"))
}

fn run_pipeline(source: &str) -> (String, Vec<Diagnostic>, Vec<Diagnostic>) {
  let program = parse_core(source);
  let formatted = format(&program);
  let check_result = check(&program, Arc::<str>::from(source));
  let mut registry = RuleRegistry::default_rules();
  let lint_diagnostics = lint(&program, &check_result.semantic, &mut registry);
  (formatted, check_result.diagnostics, lint_diagnostics)
}

#[test]
fn full_pipeline_smoke_regression() {
  let source = "use std/time\nvalue = time\n";
  let (formatted, checker_diags, lint_diags) = run_pipeline(source);

  assert!(!formatted.is_empty(), "formatted output should be non-empty");
  assert_eq!(checker_diags.iter().map(|diag| diag.code).collect::<Vec<_>>(), Vec::<&str>::new());
  assert_eq!(lint_diags.iter().map(|diag| diag.code).collect::<Vec<_>>(), Vec::<&str>::new());
}
