use miette::SourceSpan;

use crate::ast::{
  AgentMethod, AstArena, BinOp, ClassDeclData, ClassField, Expr, ExprBinary, ExprFieldAccess, ExprId, ExprMatch, FieldKind, KeywordDeclData, Literal, MatchArm,
  Pattern, Stmt, StmtId, UseKind, UseStmt,
};
use crate::folder::gen_ast::{
  gen_apply, gen_block, gen_field_call, gen_field_update, gen_func, gen_ident, gen_literal_str, gen_method, gen_none, gen_propagate, gen_record, gen_self_field,
};
use crate::sym::{Sym, intern};

fn has_user_method(methods: &[AgentMethod], name: &str) -> bool {
  let sym = intern(name);
  methods.iter().any(|m| m.name == sym)
}

fn has_user_field(fields: &[ClassField], name: &str) -> bool {
  let sym = intern(name);
  fields.iter().any(|f| f.name == sym)
}

pub(super) fn desugar_mcp(data: KeywordDeclData, span: SourceSpan, arena: &mut AstArena) -> Vec<StmtId> {
  let tool_sym = intern("Tool");
  let tool_path: Vec<Sym> = vec![intern("std"), intern("tool")];
  let use_tool = arena.alloc_stmt(Stmt::Use(UseStmt { path: tool_path, kind: UseKind::Selective(vec![tool_sym]) }), span);

  let mut fields = data.fields;
  let mut methods = data.methods;

  if !has_user_field(&fields, "session") {
    let none_val = gen_none(span, arena);
    fields.push(ClassField { name: intern("session"), default: none_val });
  }

  if !has_user_method(&methods, "run") {
    methods.push(gen_method("run", build_mcp_run(span, arena)));
  }

  let class_stmt = arena.alloc_stmt(
    Stmt::ClassDecl(ClassDeclData { name: data.name, type_params: data.type_params, traits: vec![tool_sym], fields, methods, exported: data.exported }),
    span,
  );

  vec![use_tool, class_stmt]
}

fn build_mcp_run(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_session = gen_self_field("session", span, arena);
  let none = gen_none(span, arena);
  let cond = arena.alloc_expr(Expr::Binary(ExprBinary { op: BinOp::Eq, left: self_session, right: none }), span);

  let self_cmd = gen_self_field("command", span, arena);
  let self_args = gen_self_field("args", span, arena);
  let config = gen_record(vec![(intern("command"), self_cmd), (intern("args"), self_args)], span, arena);
  let mcp_connect = gen_field_call("mcp", "connect", &[config], span, arena);
  let propagated_connect = gen_propagate(mcp_connect, span, arena);
  let assign = gen_field_update("self", "session", propagated_connect, span, arena);
  let unit = arena.alloc_expr(Expr::Literal(Literal::Unit), span);
  let unit_stmt = arena.alloc_stmt(Stmt::Expr(unit), span);
  let connect_block = gen_block(vec![assign, unit_stmt], span, arena);

  let noop = arena.alloc_expr(Expr::Literal(Literal::Unit), span);
  let true_pat = arena.alloc_pattern(Pattern::Literal(Literal::Bool(true)), span);
  let false_pat = arena.alloc_pattern(Pattern::Literal(Literal::Bool(false)), span);
  let connect_if_needed = arena.alloc_expr(
    Expr::Match(ExprMatch {
      scrutinee: cond,
      arms: vec![MatchArm { pattern: true_pat, guard: None, body: connect_block }, MatchArm { pattern: false_pat, guard: None, body: noop }],
    }),
    span,
  );
  let connect_stmt = arena.alloc_stmt(Stmt::Expr(connect_if_needed), span);

  let self_session2 = gen_self_field("session", span, arena);
  let args_tool = {
    let args = gen_ident("args", span, arena);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: args, field: FieldKind::Named(intern("tool")) }), span)
  };
  let args_args = {
    let args = gen_ident("args", span, arena);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: args, field: FieldKind::Named(intern("args")) }), span)
  };
  let mcp_call = gen_field_call("mcp", "call", &[self_session2, args_tool, args_args], span, arena);
  let propagated_call = gen_propagate(mcp_call, span, arena);
  let call_stmt = arena.alloc_stmt(Stmt::Expr(propagated_call), span);

  let body = gen_block(vec![connect_stmt, call_stmt], span, arena);
  gen_func(&["args"], body, span, arena)
}

pub(super) fn desugar_cli(data: KeywordDeclData, span: SourceSpan, arena: &mut AstArena) -> Vec<StmtId> {
  let tool_sym = intern("Tool");
  let tool_path: Vec<Sym> = vec![intern("std"), intern("tool")];
  let use_tool = arena.alloc_stmt(Stmt::Use(UseStmt { path: tool_path, kind: UseKind::Selective(vec![tool_sym]) }), span);

  let mut fields = data.fields;
  let mut methods = data.methods;

  if !has_user_field(&fields, "tool_defs") {
    let empty = gen_list(vec![], span, arena);
    fields.push(ClassField { name: intern("tool_defs"), default: empty });
  }
  if !has_user_field(&fields, "env") {
    let empty = gen_record(vec![], span, arena);
    fields.push(ClassField { name: intern("env"), default: empty });
  }

  if !has_user_method(&methods, "run") {
    methods.push(gen_method("run", build_cli_run(span, arena)));
  }

  let class_stmt = arena.alloc_stmt(
    Stmt::ClassDecl(ClassDeclData { name: data.name, type_params: data.type_params, traits: vec![tool_sym], fields, methods, exported: data.exported }),
    span,
  );

  vec![use_tool, class_stmt]
}

fn build_cli_run(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_command = gen_self_field("command", span, arena);
  let space = gen_literal_str(" ", span, arena);
  let args_command = {
    let args = gen_ident("args", span, arena);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: args, field: FieldKind::Named(intern("command")) }), span)
  };
  let default_empty = gen_literal_str("", span, arena);
  let coalesce_expr = super::desugar::desugar_coalesce(args_command, default_empty, span, arena);
  let args_cmd = arena.alloc_expr(coalesce_expr, span);
  let cmd_with_space = arena.alloc_expr(Expr::Binary(ExprBinary { op: BinOp::Concat, left: self_command, right: space }), span);
  let cmd_str = arena.alloc_expr(Expr::Binary(ExprBinary { op: BinOp::Concat, left: cmd_with_space, right: args_cmd }), span);
  let bash_fn = gen_ident("bash", span, arena);
  let bash_call = gen_apply(bash_fn, cmd_str, span, arena);
  let propagated = gen_propagate(bash_call, span, arena);
  gen_func(&["args"], propagated, span, arena)
}

use crate::folder::gen_ast::gen_list;
