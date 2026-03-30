use miette::SourceSpan;

use crate::folder::gen_ast::{gen_block, gen_field_call, gen_func, gen_ident, gen_list, gen_literal_str, gen_method, gen_record, gen_self_field};
use lx_ast::ast::{
  AgentMethod, AstArena, BinOp, BindTarget, Binding, ClassDeclData, ClassField, Expr, ExprBinary, ExprFieldAccess, ExprId, FieldKind, KeywordDeclData, Stmt,
  StmtId, UseKind, UseStmt,
};
use lx_span::sym::{Sym, intern};

fn has_user_method(methods: &[AgentMethod], name: &str) -> bool {
  let sym = intern(name);
  methods.iter().any(|m| m.name == sym)
}

fn has_user_field(fields: &[ClassField], name: &str) -> bool {
  let sym = intern(name);
  fields.iter().any(|f| f.name == sym)
}

pub(super) fn desugar_http(data: KeywordDeclData, span: SourceSpan, arena: &mut AstArena) -> Vec<StmtId> {
  let tool_sym = intern("Tool");
  let tool_path: Vec<Sym> = vec![intern("std"), intern("tool")];
  let use_tool = arena.alloc_stmt(Stmt::Use(UseStmt { path: tool_path, kind: UseKind::Selective(vec![tool_sym]) }), span);

  let http_path: Vec<Sym> = vec![intern("std"), intern("http")];
  let use_http = arena.alloc_stmt(Stmt::Use(UseStmt { path: http_path, kind: UseKind::Whole }), span);

  let mut fields = data.fields;
  let mut methods = data.methods;

  if !has_user_field(&fields, "base_url") {
    let empty_str = gen_literal_str("", span, arena);
    fields.push(ClassField { name: intern("base_url"), default: empty_str });
  }
  if !has_user_field(&fields, "headers") {
    let empty_rec = gen_record(vec![], span, arena);
    fields.push(ClassField { name: intern("headers"), default: empty_rec });
  }
  if !has_user_field(&fields, "endpoints") {
    let empty_list = gen_list(vec![], span, arena);
    fields.push(ClassField { name: intern("endpoints"), default: empty_list });
  }

  if !has_user_method(&methods, "run") {
    methods.push(gen_method("run", build_http_run(span, arena)));
  }

  let class_stmt = arena.alloc_stmt(
    Stmt::ClassDecl(ClassDeclData { name: data.name, type_params: data.type_params, traits: vec![tool_sym], fields, methods, exported: data.exported }),
    span,
  );

  vec![use_tool, use_http, class_stmt]
}

fn build_http_run(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_base_url = gen_self_field("base_url", span, arena);
  let args_tool = {
    let args = gen_ident("args", span, arena);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: args, field: FieldKind::Named(intern("tool")) }), span)
  };
  let url_expr = arena.alloc_expr(Expr::Binary(ExprBinary { op: BinOp::Concat, left: self_base_url, right: args_tool }), span);
  let url_assign = arena
    .alloc_stmt(Stmt::Binding(Binding { exported: false, mutable: false, target: BindTarget::Name(intern("url")), type_ann: None, value: url_expr }), span);

  let args_method = {
    let args = gen_ident("args", span, arena);
    let a = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: args, field: FieldKind::Named(intern("args")) }), span);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: a, field: FieldKind::Named(intern("method")) }), span)
  };
  let default_get = gen_literal_str("GET", span, arena);
  let coalesce_expr = super::desugar::desugar_coalesce(args_method, default_get, span, arena);
  let method_coalesced = arena.alloc_expr(coalesce_expr, span);

  let url_ref = gen_ident("url", span, arena);

  let args_body = {
    let args = gen_ident("args", span, arena);
    let a = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: args, field: FieldKind::Named(intern("args")) }), span);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: a, field: FieldKind::Named(intern("body")) }), span)
  };

  let self_headers = gen_self_field("headers", span, arena);

  let opts_record = gen_record(
    vec![(intern("method"), method_coalesced), (intern("url"), url_ref), (intern("body"), args_body), (intern("headers"), self_headers)],
    span,
    arena,
  );

  let http_request_call = gen_field_call("http", "request", &[opts_record], span, arena);
  let http_stmt = arena.alloc_stmt(Stmt::Expr(http_request_call), span);

  let body = gen_block(vec![url_assign, http_stmt], span, arena);
  gen_func(&["args"], body, span, arena)
}
