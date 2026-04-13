use std::collections::HashMap;
use std::marker::PhantomData;

use lx_ast::ast::{
  AgentMethod, AstArena, BindTarget, Binding, ClassDeclData, ClassField, Expr, FieldDecl, KeywordDeclData, KeywordKind, Literal, MethodSpec, Program, Stmt,
  Surface, TraitDeclData, TraitEntry, TypeExpr,
};
use lx_fmt::format;
use lx_span::source::{CommentStore, FileId};
use lx_span::sym::intern;

fn program_with_stmt(stmt: Stmt, arena: &mut AstArena) -> Program<Surface> {
  let stmt_id = arena.alloc_stmt(stmt, (0, 0).into());
  Program {
    stmts: vec![stmt_id],
    arena: arena.clone(),
    comments: CommentStore::default(),
    comment_map: HashMap::new(),
    file: FileId::new(0),
    _phase: PhantomData,
  }
}

fn int_expr(arena: &mut AstArena, value: i64) -> lx_ast::ast::ExprId {
  arena.alloc_expr(Expr::Literal(Literal::Int(value.into())), (0, 0).into())
}

fn ident_expr(arena: &mut AstArena, name: &str) -> lx_ast::ast::ExprId {
  arena.alloc_expr(Expr::Ident(intern(name)), (0, 0).into())
}

fn named_type(arena: &mut AstArena, name: &str) -> lx_ast::ast::TypeExprId {
  arena.alloc_type_expr(TypeExpr::Named(intern(name)), (0, 0).into())
}

#[test]
fn formats_trait_declaration_with_field_defaults_and_method_signatures() {
  let mut arena = AstArena::new();
  let trait_decl = Stmt::TraitDecl(TraitDeclData {
    name: intern("Pair"),
    type_params: vec![],
    entries: vec![TraitEntry::Field(Box::new(FieldDecl {
      name: intern("left"),
      type_name: intern("Int"),
      default: Some(int_expr(&mut arena, 1)),
      constraint: None,
    }))],
    methods: vec![MethodSpec {
      name: intern("merge"),
      input: vec![
        FieldDecl { name: intern("lhs"), type_name: intern("Int"), default: None, constraint: None },
        FieldDecl { name: intern("rhs"), type_name: intern("Str"), default: None, constraint: None },
      ],
      output: intern("Bool"),
    }],
    defaults: vec![],
    requires: vec![],
    description: None,
    tags: vec![],
    exported: false,
  });

  let program = program_with_stmt(trait_decl, &mut arena);
  assert_eq!(format(&program), "Trait Pair = {\n  left: Int = 1\n  merge: Int -> Str -> Bool\n}\n");
}

#[test]
fn formats_class_declaration_regression() {
  let mut arena = AstArena::new();
  let class_decl = Stmt::ClassDecl(ClassDeclData {
    name: intern("Box"),
    type_params: vec![],
    traits: vec![],
    fields: vec![ClassField { name: intern("value"), default: int_expr(&mut arena, 1) }],
    methods: vec![AgentMethod { name: intern("get"), handler: ident_expr(&mut arena, "value") }],
    exported: false,
  });

  let program = program_with_stmt(class_decl, &mut arena);
  assert_eq!(format(&program), "Class Box = {\n  value: 1\n  get = value\n}\n");
}

#[test]
fn formats_keyword_declaration_regression() {
  let mut arena = AstArena::new();
  let keyword_decl = Stmt::KeywordDecl(KeywordDeclData {
    keyword: KeywordKind::Agent,
    name: intern("Bot"),
    type_params: vec![],
    fields: vec![ClassField { name: intern("name"), default: int_expr(&mut arena, 1) }],
    methods: vec![AgentMethod { name: intern("run"), handler: ident_expr(&mut arena, "name") }],
    trait_entries: None,
    exported: false,
    uses: vec![],
  });

  let program = program_with_stmt(keyword_decl, &mut arena);
  assert_eq!(format(&program), "Agent Bot = {\n  name: 1\n  run = name\n}\n");
}

#[test]
fn formats_binding_with_type_annotation_regression() {
  let mut arena = AstArena::new();
  let binding = Stmt::Binding(Binding {
    exported: false,
    mutable: false,
    target: BindTarget::Name(intern("answer")),
    type_ann: Some(named_type(&mut arena, "Int")),
    value: int_expr(&mut arena, 1),
  });

  let program = program_with_stmt(binding, &mut arena);
  assert_eq!(format(&program), "answer: Int = 1\n");
}

#[test]
fn formats_nested_type_expressions_through_emit_type_regression() {
  let mut arena = AstArena::new();
  let int_ty = named_type(&mut arena, "Int");
  let float_ty = named_type(&mut arena, "Float");
  let str_ty = named_type(&mut arena, "Str");
  let list_int_ty = arena.alloc_type_expr(TypeExpr::List(int_ty), (0, 0).into());
  let record_ty = arena.alloc_type_expr(
    TypeExpr::Record(vec![
      lx_ast::ast::TypeField { name: intern("left"), ty: int_ty },
      lx_ast::ast::TypeField { name: intern("right"), ty: str_ty },
    ]),
    (0, 0).into(),
  );
  let result_ty = arena.alloc_type_expr(TypeExpr::Applied(intern("Result"), vec![list_int_ty]), (0, 0).into());
  let fallible_ty = arena.alloc_type_expr(TypeExpr::Fallible { ok: result_ty, err: record_ty }, (0, 0).into());
  let list_float_ty = arena.alloc_type_expr(TypeExpr::List(float_ty), (0, 0).into());
  let map_ty = arena.alloc_type_expr(TypeExpr::Map { key: str_ty, value: list_float_ty }, (0, 0).into());
  let func_ty = arena.alloc_type_expr(TypeExpr::Func { param: fallible_ty, ret: map_ty }, (0, 0).into());
  let binding = Stmt::Binding(Binding {
    exported: false,
    mutable: false,
    target: BindTarget::Name(intern("mapper")),
    type_ann: Some(func_ty),
    value: int_expr(&mut arena, 1),
  });

  let program = program_with_stmt(binding, &mut arena);
  assert_eq!(format(&program), "mapper: Result [Int] ^ { left: Int; right: Str } -> %{Str: [Float]} = 1\n");
}
