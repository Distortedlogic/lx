use std::sync::Arc;

use lx_ast::ast::{Core, Program};
use lx_checker::diagnostics::DiagnosticKind;
use lx_checker::{check, Diagnostic};
use lx_desugar::desugar;
use lx_linter::{RuleRegistry, lint};
use lx_parser::{lexer::lex, parser::parse};
use lx_span::source::FileId;

fn parse_core(source: &str) -> Program<Core> {
  let (tokens, comments) = lex(source).unwrap_or_else(|err| panic!("lex failed for fixture:\n{source}\n{err}"));
  let parsed = parse(tokens, FileId::new(0), comments, source);
  assert!(parsed.errors.is_empty(), "parse failed for fixture:\n{source}\n{:?}", parsed.errors);
  desugar(parsed.program.expect("parser returned no program"))
}

fn lint_source(source: &str) -> Vec<Diagnostic> {
  let program = parse_core(source);
  let check_result = check(&program, Arc::<str>::from(source));
  let mut registry = RuleRegistry::default_rules();
  lint(&program, &check_result.semantic, &mut registry)
}

fn assert_has_lint(source: &str, expected_code: &str, expected_rule_name: &str) {
  let diagnostics = lint_source(source);
  assert!(
    diagnostics.iter().any(|diag| {
      diag.code == expected_code
        && matches!(&diag.kind, DiagnosticKind::LintWarning { rule_name, .. } if rule_name.contains(expected_rule_name))
    }),
    "missing lint {expected_code} / {expected_rule_name} for fixture:\n{source}\nactual codes: {:?}",
    diagnostics.iter().map(|diag| diag.code).collect::<Vec<_>>()
  );
}

#[test]
fn break_outside_loop_regression() {
  assert_has_lint("break", "L001", "break_outside_loop");
}

#[test]
fn redundant_propagate_regression() {
  assert_has_lint("value = 1^", "L003", "redundant-propagate");
}

#[test]
fn duplicate_record_field_regression() {
  assert_has_lint("value = { a: 1; a: 2 }", "L006", "duplicate_record_field");
}

#[test]
fn unused_import_regression() {
  assert_has_lint("use std/time", "L005", "unused_import");
}

#[test]
fn empty_match_regression() {
  assert_has_lint("value = true ? {}", "L002", "empty-match");
}

#[test]
fn single_branch_par_regression() {
  assert_has_lint("par { 1 }", "L007", "single_branch_par");
}

#[test]
fn mut_never_mutated_regression() {
  assert_has_lint("value := 1", "L008", "mut_never_mutated");
}

#[test]
fn unreachable_code_regression() {
  assert_has_lint("loop { break; 1 }", "L004", "unreachable_code");
}
