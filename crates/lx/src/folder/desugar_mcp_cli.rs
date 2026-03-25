use miette::SourceSpan;

use crate::ast::{
  AgentMethod, AstArena, BinOp, ClassDeclData, ClassField, Expr, ExprBinary, ExprFieldAccess,
  ExprTernary, FieldKind, KeywordDeclData, Stmt, StmtId, UseKind, UseStmt,
};
use crate::folder::gen_ast::*;
use crate::sym::{Sym, intern};

fn has_user_method(methods: &[AgentMethod], name: &str) -> bool {
  let sym = intern(name);
  methods.iter().any(|m| m.name == sym)
}

fn has_user_field(fields: &[ClassField], name: &str) -> bool {
  let sym = intern(name);
  fields.iter().any(|f| f.name == sym)
}

pub(super) fn desugar_mcp(
  data: KeywordDeclData,
  span: SourceSpan,
  arena: &mut AstArena,
) -> Vec<StmtId> {
  let connector_sym = intern("Connector");
  let connector_path: Vec<Sym> = vec![intern("std"), intern("connector")];
  let use_connector = arena.alloc_stmt(
    Stmt::Use(UseStmt { path: connector_path, kind: UseKind::Selective(vec![connector_sym]) }),
    span,
  );

  let mcp_path: Vec<Sym> = vec![intern("std"), intern("mcp")];
  let use_mcp =
    arena.alloc_stmt(Stmt::Use(UseStmt { path: mcp_path, kind: UseKind::Whole }), span);

  let mut fields = data.fields;
  let mut methods = data.methods;

  if !has_user_field(&fields, "session") {
    let none_val = gen_none(span, arena);
    fields.push(ClassField { name: intern("session"), default: none_val });
  }

  if !has_user_method(&methods, "connect") {
    methods.push(gen_method("connect", build_mcp_connect(span, arena)));
  }
  if !has_user_method(&methods, "disconnect") {
    methods.push(gen_method("disconnect", build_mcp_disconnect(span, arena)));
  }
  if !has_user_method(&methods, "call") {
    methods.push(gen_method("call", build_mcp_call(span, arena)));
  }
  if !has_user_method(&methods, "tools") {
    methods.push(gen_method("tools", build_mcp_tools(span, arena)));
  }

  let class_stmt = arena.alloc_stmt(
    Stmt::ClassDecl(ClassDeclData {
      name: data.name,
      type_params: data.type_params,
      traits: vec![connector_sym],
      fields,
      methods,
      exported: data.exported,
    }),
    span,
  );

  vec![use_connector, use_mcp, class_stmt]
}

fn build_mcp_connect(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_cmd = gen_self_field("command", span, arena);
  let self_args = gen_self_field("args", span, arena);
  let config = gen_record(
    vec![(intern("command"), self_cmd), (intern("args"), self_args)],
    span,
    arena,
  );
  let mcp_connect = gen_field_call("mcp", "connect", &[config], span, arena);
  let propagated = gen_propagate(mcp_connect, span, arena);
  let assign = gen_field_update("self", "session", propagated, span, arena);
  let ok = gen_ok_unit(span, arena);
  let ok_stmt = arena.alloc_stmt(Stmt::Expr(ok), span);
  let body = gen_block(vec![assign, ok_stmt], span, arena);
  gen_func(&[], body, span, arena)
}

fn build_mcp_disconnect(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_session = gen_self_field("session", span, arena);
  let none = gen_none(span, arena);
  let cond = arena.alloc_expr(
    Expr::Binary(ExprBinary { op: BinOp::Eq, left: self_session, right: none }),
    span,
  );
  let ok_branch = gen_ok_unit(span, arena);
  let self_session2 = gen_self_field("session", span, arena);
  let mcp_close = gen_field_call("mcp", "close", &[self_session2], span, arena);
  let close_stmt = arena.alloc_stmt(Stmt::Expr(mcp_close), span);
  let ok2 = gen_ok_unit(span, arena);
  let ok2_stmt = arena.alloc_stmt(Stmt::Expr(ok2), span);
  let else_body = gen_block(vec![close_stmt, ok2_stmt], span, arena);
  let ternary = arena.alloc_expr(
    Expr::Ternary(ExprTernary { cond, then_: ok_branch, else_: Some(else_body) }),
    span,
  );
  gen_func(&[], ternary, span, arena)
}

fn build_mcp_call(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_session = gen_self_field("session", span, arena);
  let req_tool = {
    let req = gen_ident("req", span, arena);
    arena.alloc_expr(
      Expr::FieldAccess(ExprFieldAccess {
        expr: req,
        field: FieldKind::Named(intern("tool")),
      }),
      span,
    )
  };
  let req_args = {
    let req = gen_ident("req", span, arena);
    arena.alloc_expr(
      Expr::FieldAccess(ExprFieldAccess {
        expr: req,
        field: FieldKind::Named(intern("args")),
      }),
      span,
    )
  };
  let mcp_call =
    gen_field_call("mcp", "call", &[self_session, req_tool, req_args], span, arena);
  let propagated = gen_propagate(mcp_call, span, arena);
  gen_func(&["req"], propagated, span, arena)
}

fn build_mcp_tools(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_session = gen_self_field("session", span, arena);
  let none = gen_none(span, arena);
  let cond = arena.alloc_expr(
    Expr::Binary(ExprBinary { op: BinOp::Eq, left: self_session, right: none }),
    span,
  );
  let empty_list = gen_list(vec![], span, arena);
  let self_session2 = gen_self_field("session", span, arena);
  let list_tools = gen_field_call("mcp", "list_tools", &[self_session2], span, arena);
  let propagated = gen_propagate(list_tools, span, arena);
  let ternary = arena.alloc_expr(
    Expr::Ternary(ExprTernary { cond, then_: empty_list, else_: Some(propagated) }),
    span,
  );
  gen_func(&[], ternary, span, arena)
}

pub(super) fn desugar_cli(
  data: KeywordDeclData,
  span: SourceSpan,
  arena: &mut AstArena,
) -> Vec<StmtId> {
  let connector_sym = intern("Connector");
  let connector_path: Vec<Sym> = vec![intern("std"), intern("connector")];
  let use_connector = arena.alloc_stmt(
    Stmt::Use(UseStmt { path: connector_path, kind: UseKind::Selective(vec![connector_sym]) }),
    span,
  );

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

  if !has_user_method(&methods, "connect") {
    let ok = gen_ok_unit(span, arena);
    methods.push(gen_method("connect", gen_func(&[], ok, span, arena)));
  }
  if !has_user_method(&methods, "disconnect") {
    let ok = gen_ok_unit(span, arena);
    methods.push(gen_method("disconnect", gen_func(&[], ok, span, arena)));
  }
  if !has_user_method(&methods, "call") {
    methods.push(gen_method("call", build_cli_call(span, arena)));
  }
  if !has_user_method(&methods, "tools") {
    let self_tool_defs = gen_self_field("tool_defs", span, arena);
    methods.push(gen_method("tools", gen_func(&[], self_tool_defs, span, arena)));
  }

  let class_stmt = arena.alloc_stmt(
    Stmt::ClassDecl(ClassDeclData {
      name: data.name,
      type_params: data.type_params,
      traits: vec![connector_sym],
      fields,
      methods,
      exported: data.exported,
    }),
    span,
  );

  vec![use_connector, class_stmt]
}

fn build_cli_call(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_command = gen_self_field("command", span, arena);
  let space = gen_literal_str(" ", span, arena);
  let req_tool = {
    let req = gen_ident("req", span, arena);
    arena.alloc_expr(
      Expr::FieldAccess(ExprFieldAccess {
        expr: req,
        field: FieldKind::Named(intern("tool")),
      }),
      span,
    )
  };
  let cmd_with_space = arena.alloc_expr(
    Expr::Binary(ExprBinary { op: BinOp::Concat, left: self_command, right: space }),
    span,
  );
  let cmd_str = arena.alloc_expr(
    Expr::Binary(ExprBinary { op: BinOp::Concat, left: cmd_with_space, right: req_tool }),
    span,
  );
  let bash_fn = gen_ident("bash", span, arena);
  let bash_call = gen_apply(bash_fn, cmd_str, span, arena);
  let propagated = gen_propagate(bash_call, span, arena);
  gen_func(&["req"], propagated, span, arena)
}
