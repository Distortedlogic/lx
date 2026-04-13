use std::collections::HashMap;
use std::sync::Arc;

use lx_ast::ast::{BindTarget, Core, Program, Stmt};
use lx_checker::module_graph::ModuleSignature;
use lx_checker::type_arena::TypeArena;
use lx_checker::{CheckResult, check, check_with_imports};
use lx_desugar::desugar;
use lx_parser::{lexer::lex, parser::parse};
use lx_span::source::FileId;
use lx_span::sym::{Sym, intern};

fn parse_core(source: &str) -> Program<Core> {
  let (tokens, comments) = lex(source).unwrap_or_else(|err| panic!("lex failed for fixture:\n{source}\n{err}"));
  let parsed = parse(tokens, FileId::new(0), comments, source);
  assert!(parsed.errors.is_empty(), "parse failed for fixture:\n{source}\n{:?}", parsed.errors);
  desugar(parsed.program.expect("parser returned no program"))
}

fn check_source(source: &str) -> (Program<Core>, CheckResult) {
  let program = parse_core(source);
  let result = check(&program, Arc::<str>::from(source));
  (program, result)
}

fn check_source_with_imports(source: &str, imports: HashMap<Sym, ModuleSignature>) -> (Program<Core>, CheckResult) {
  let program = parse_core(source);
  let result = check_with_imports(&program, Arc::<str>::from(source), imports);
  (program, result)
}

fn binding_value(program: &Program<Core>, name: &str) -> lx_ast::ast::ExprId {
  let target = intern(name);
  for stmt_id in &program.stmts {
    if let Stmt::Binding(binding) = program.arena.stmt(*stmt_id)
      && let BindTarget::Name(name) = binding.target
      && name == target
    {
      return binding.value;
    }
  }
  panic!("binding not found: {name}");
}

fn diagnostic_codes(result: &CheckResult) -> Vec<&'static str> {
  result.diagnostics.iter().map(|diag| diag.code).collect()
}

#[test]
fn binding_annotation_mismatch_regression() {
  let (_, result) = check_source("answer: Str = 1");
  assert_eq!(diagnostic_codes(&result), vec!["E009"]);
}

#[test]
fn function_parameter_annotation_resolution_regression() {
  let (program, result) = check_source("id = (x: Int) x\nvalue = id 1");
  assert!(result.diagnostics.is_empty(), "unexpected diagnostics: {:?}", diagnostic_codes(&result));
  let value_expr = binding_value(&program, "value");
  let ty = result.semantic.type_of_expr(value_expr).expect("missing type for value binding expression");
  assert_eq!(result.semantic.display_type(ty), "Int");
}

#[test]
fn match_exhaustiveness_warning_regression() {
  let (_, result) = check_source("value = true ? { true -> 1 }");
  assert_eq!(diagnostic_codes(&result), vec!["E007"]);
}

#[test]
fn imported_names_resolve_through_check_with_imports_regression() {
  let type_arena = TypeArena::new();
  let mut bindings = HashMap::new();
  bindings.insert(intern("answer"), type_arena.int());
  let imports = HashMap::from([(intern("math"), ModuleSignature { file: None, bindings, types: HashMap::new(), traits: HashMap::new(), type_arena })]);

  let (program, result) = check_source_with_imports("use dep/math { answer }\nvalue = answer", imports);
  assert!(result.diagnostics.is_empty(), "unexpected diagnostics: {:?}", diagnostic_codes(&result));
  let value_expr = binding_value(&program, "value");
  let ty = result.semantic.type_of_expr(value_expr).expect("missing type for imported binding reference");
  assert_eq!(result.semantic.display_type(ty), "Int");
}

#[test]
fn pattern_bindings_enter_scope_regression() {
  let (program, result) = check_source("value = (1, 2) ? { (x, y) -> x }");
  assert!(result.diagnostics.is_empty(), "unexpected diagnostics: {:?}", diagnostic_codes(&result));
  let value_expr = binding_value(&program, "value");
  let ty = result.semantic.type_of_expr(value_expr).expect("missing type for match expression");
  assert_eq!(result.semantic.display_type(ty), "Int");
}
