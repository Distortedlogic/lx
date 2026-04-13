use std::sync::Arc;

use lx_ast::ast::{BindTarget, Core, Program, Stmt};
use lx_checker::check;
use lx_checker::module_graph::extract_signature;
use lx_checker::semantic::{DefKind, DefinitionId};
use lx_desugar::desugar;
use lx_parser::{lexer::lex, parser::parse};
use lx_span::source::FileId;
use lx_span::sym::intern;

fn parse_core(source: &str) -> Program<Core> {
  let (tokens, comments) = lex(source).unwrap_or_else(|err| panic!("lex failed for fixture:\n{source}\n{err}"));
  let parsed = parse(tokens, FileId::new(0), comments, source);
  assert!(parsed.errors.is_empty(), "parse failed for fixture:\n{source}\n{:?}", parsed.errors);
  desugar(parsed.program.expect("parser returned no program"))
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

#[test]
fn semantic_model_baseline_for_import_and_binding_resolution() {
  let source = "use std/time\nanswer = 1\nvalue = answer\n";
  let program = parse_core(source);
  let result = check(&program, Arc::<str>::from(source));
  let model = &result.semantic;

  assert!(result.diagnostics.is_empty(), "unexpected diagnostics present");
  assert_eq!(model.definitions.len(), 3);
  assert!(matches!(model.definitions[0].kind, DefKind::Import));
  assert_eq!(model.definitions[0].name.as_str(), "time");
  assert!(matches!(model.definitions[1].kind, DefKind::Binding));
  assert_eq!(model.definitions[1].name.as_str(), "answer");
  assert!(matches!(model.definitions[2].kind, DefKind::Binding));
  assert_eq!(model.definitions[2].name.as_str(), "value");

  let answer_def = DefinitionId::new(1);
  assert_eq!(model.references_to(answer_def).len(), 1);
  assert_eq!(model.display_type(model.type_of_def(answer_def).unwrap()), "Int");

  let value_expr_id = binding_value(&program, "value");
  assert_eq!(model.display_type(model.type_of_expr(value_expr_id).unwrap()), "Int");
}

#[test]
fn module_signature_baseline_for_exported_binding_and_trait() {
  let source = "+answer = 1\n+Trait Pair = { left: Int; right: Int }\n";
  let program = parse_core(source);
  let result = check(&program, Arc::<str>::from(source));
  let signature = extract_signature(&program, &result.semantic);

  assert!(result.diagnostics.is_empty(), "unexpected diagnostics present");
  assert_eq!(signature.bindings.len(), 1);
  assert!(signature.bindings.contains_key(&intern("answer")));
  assert_eq!(signature.traits.len(), 1);
  assert!(signature.traits.contains_key(&intern("Pair")));
  assert!(signature.types.is_empty());
  assert_eq!(signature.traits[&intern("Pair")].len(), 2);
}
