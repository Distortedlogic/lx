use lx_ast::ast::{Core, Expr, ExprId, KeywordKind, Program, Stmt, WithKind};
use lx_ast::visitor::prelude::*;
use lx_desugar::desugar;
use lx_parser::{lexer::lex, parser::parse};
use lx_span::source::FileId;

fn desugar_source(source: &str) -> Program<Core> {
  let (tokens, comments) = lex(source).unwrap_or_else(|err| panic!("lex failed for fixture:\n{source}\n{err}"));
  let parsed = parse(tokens, FileId::new(0), comments, source);
  assert!(parsed.errors.is_empty(), "parse failed for fixture:\n{source}\n{:?}", parsed.errors);
  desugar(parsed.program.expect("parser returned no program"))
}

fn parse_source_errors(source: &str) -> Vec<String> {
  let (tokens, comments) = lex(source).unwrap_or_else(|err| panic!("lex failed for fixture:\n{source}\n{err}"));
  let parsed = parse(tokens, FileId::new(0), comments, source);
  parsed.errors.into_iter().map(|err| format!("{err:?}")).collect()
}

#[derive(Default)]
struct SurfaceOnlyDetector {
  saw_keyword_decl: bool,
  saw_pipe: bool,
  saw_section: bool,
  saw_ternary: bool,
  saw_coalesce: bool,
  saw_with_binding: bool,
}

impl AstVisitor for SurfaceOnlyDetector {
  fn visit_stmt(&mut self, _id: StmtId, stmt: &Stmt, _span: SourceSpan) -> VisitAction {
    if let Stmt::KeywordDecl(data) = stmt {
      match data.keyword {
        KeywordKind::Agent
        | KeywordKind::Tool
        | KeywordKind::Prompt
        | KeywordKind::Store
        | KeywordKind::Session
        | KeywordKind::Guard
        | KeywordKind::Workflow
        | KeywordKind::Schema
        | KeywordKind::Http => self.saw_keyword_decl = true,
      }
    }
    VisitAction::Descend
  }

  fn visit_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan) -> VisitAction {
    match expr {
      Expr::Pipe(_) => self.saw_pipe = true,
      Expr::Section(_) => self.saw_section = true,
      Expr::Ternary(_) => self.saw_ternary = true,
      Expr::Coalesce(_) => self.saw_coalesce = true,
      Expr::With(with) if matches!(with.kind, WithKind::Binding { .. }) => self.saw_with_binding = true,
      _ => {},
    }
    VisitAction::Descend
  }
}

fn assert_no_surface_only_nodes(program: &Program<Core>) {
  let mut detector = SurfaceOnlyDetector::default();
  let _ = walk_program(&mut detector, program);
  assert!(!detector.saw_keyword_decl, "core program still contains KeywordDecl");
  assert!(!detector.saw_pipe, "core program still contains Expr::Pipe");
  assert!(!detector.saw_section, "core program still contains Expr::Section");
  assert!(!detector.saw_ternary, "core program still contains Expr::Ternary");
  assert!(!detector.saw_coalesce, "core program still contains Expr::Coalesce");
  assert!(!detector.saw_with_binding, "core program still contains Expr::With(Binding)");
}

#[test]
fn desugars_keyword_decl_regression() {
  let program = desugar_source("Agent Bot = { run = 1 }");
  assert_no_surface_only_nodes(&program);
}

#[test]
fn desugars_pipe_regression() {
  let program = desugar_source("value = 1 | f");
  assert_no_surface_only_nodes(&program);
}

#[test]
fn desugars_section_regression() {
  let program = desugar_source("value = (.name)");
  assert_no_surface_only_nodes(&program);
}

#[test]
fn desugars_ternary_regression() {
  let program = desugar_source("value = true ? 1 : 2");
  assert_no_surface_only_nodes(&program);
}

#[test]
fn desugars_coalesce_regression() {
  let program = desugar_source("maybe = None\nvalue = maybe ?? 1");
  assert_no_surface_only_nodes(&program);
}

#[test]
fn with_binding_source_syntax_is_currently_parser_blocked() {
  for source in ["value = with x = 1 { x }", "value = (with x = 1 { x })", "value = with mut x = 1 { x }"] {
    let errors = parse_source_errors(source);
    assert!(
      !errors.is_empty(),
      "source unexpectedly parsed end-to-end and should replace this blocker test:\n{source}"
    );
  }
}
