use std::marker::PhantomData;

use lx_ast::ast::{attach_comments, AstArena, Expr, NodeId, Program, Stmt, StmtId, Surface, TraitEntry, TypeExpr};
use lx_parser::{lexer::lex, parser::parse};
use lx_span::source::{Comment, CommentStore, FileId};
use lx_span::sym::intern;

fn parse_surface(source: &str) -> Program<Surface> {
  let (tokens, comments) = lex(source).unwrap_or_else(|err| panic!("lex failed for fixture:\n{source}\n{err}"));
  let result = parse(tokens, FileId::new(0), comments, source);
  assert!(result.errors.is_empty(), "parse failed for fixture:\n{source}\n{:?}", result.errors);
  result.program.expect("parser returned no program")
}

fn first_stmt_id(program: &Program<Surface>) -> StmtId {
  program.stmts[0]
}

#[test]
fn parses_keyword_declaration_regression() {
  let program = parse_surface("Agent Bot = { run = 1 }");
  assert!(matches!(program.arena.stmt(first_stmt_id(&program)), Stmt::KeywordDecl(_)));
}

#[test]
fn parses_trait_field_entries_regression() {
  let program = parse_surface("Trait Pair = { left: Int; right: Int }");
  let Stmt::TraitDecl(data) = program.arena.stmt(first_stmt_id(&program)) else {
    panic!("expected trait declaration");
  };
  assert_eq!(data.entries.len(), 2);
  assert!(matches!(data.entries[0], TraitEntry::Field(_)));
  assert!(matches!(data.entries[1], TraitEntry::Field(_)));
}

#[test]
fn parses_class_fields_and_methods_regression() {
  let program = parse_surface("Class Box = { value: 1; get = value }");
  let Stmt::ClassDecl(data) = program.arena.stmt(first_stmt_id(&program)) else {
    panic!("expected class declaration");
  };
  assert_eq!(data.fields.len(), 1);
  assert_eq!(data.methods.len(), 1);
}

#[test]
fn parses_binding_with_type_annotation_regression() {
  let program = parse_surface("answer: Int = 1");
  let Stmt::Binding(binding) = program.arena.stmt(first_stmt_id(&program)) else {
    panic!("expected binding");
  };
  let type_ann = binding.type_ann.expect("expected type annotation");
  assert!(matches!(program.arena.type_expr(type_ann), TypeExpr::Named(name) if name.as_str() == "Int"));
}

#[test]
fn parses_nested_type_expressions_regression() {
  let source = "mapper: Result [Int] ^ { left: Int; right: Str } -> %{Str: [Float]} = 1";
  let program = parse_surface(source);
  let Stmt::Binding(binding) = program.arena.stmt(first_stmt_id(&program)) else {
    panic!("expected binding");
  };
  let type_ann = binding.type_ann.expect("expected nested type annotation");
  let TypeExpr::Func { param, ret } = program.arena.type_expr(type_ann) else {
    panic!("expected top-level function type");
  };
  let TypeExpr::Fallible { ok, err } = program.arena.type_expr(*param) else {
    panic!("expected fallible parameter type");
  };
  assert!(matches!(program.arena.type_expr(*ok), TypeExpr::Applied(name, args) if name.as_str() == "Result" && args.len() == 1));
  assert!(matches!(program.arena.type_expr(*err), TypeExpr::Record(fields) if fields.len() == 2));
  assert!(matches!(program.arena.type_expr(*ret), TypeExpr::Map { .. }));
}

#[test]
fn comment_attachments_assign_leading_and_trailing_to_first_stmt() {
  let source = "-- leading on stmt\nx = 1 -- trailing on stmt\n";
  let program = parse_surface(source);

  assert_eq!(program.leading_comments(NodeId::Stmt(first_stmt_id(&program))).len(), 1);
  assert_eq!(program.leading_comments(NodeId::Stmt(first_stmt_id(&program)))[0].text, "-- leading on stmt");
  assert_eq!(program.trailing_comments(NodeId::Stmt(first_stmt_id(&program))).len(), 1);
  assert_eq!(program.trailing_comments(NodeId::Stmt(first_stmt_id(&program)))[0].text, "-- trailing on stmt");
}

#[test]
fn attach_comments_can_classify_dangling_for_a_synthetic_node_span() {
  let source = "--slot;";
  let mut arena = AstArena::new();
  let expr_id = arena.alloc_expr(Expr::Ident(intern("slot")), (0, 6).into());
  let stmt_id = arena.alloc_stmt(Stmt::Expr(expr_id), (0, 7).into());
  let comments = CommentStore::from_vec(vec![Comment { span: (0, 2).into(), text: "--".into() }]);
  let comment_map = attach_comments(&[stmt_id], &arena, &comments, source);
  let program: Program<Surface> = Program {
    stmts: vec![stmt_id],
    arena,
    comments,
    comment_map,
    file: FileId::new(0),
    _phase: PhantomData,
  };

  assert_eq!(program.dangling_comments(NodeId::Expr(expr_id)).len(), 1);
  assert!(program.leading_comments(NodeId::Expr(expr_id)).is_empty());
  assert!(program.trailing_comments(NodeId::Expr(expr_id)).is_empty());
}
