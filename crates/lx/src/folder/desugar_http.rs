use miette::SourceSpan;

use crate::ast::{AstArena, BinOp, ClassDeclData, ClassField, Expr, ExprBinary, ExprFieldAccess, FieldKind, KeywordDeclData, Stmt, StmtId, UseKind, UseStmt};
use crate::folder::gen_ast::{gen_block, gen_field_call, gen_func, gen_ident, gen_list, gen_literal_str, gen_method, gen_ok_unit, gen_record, gen_self_field};
use crate::sym::{Sym, intern};

fn has_user_method(methods: &[crate::ast::AgentMethod], name: &str) -> bool {
  let sym = intern(name);
  methods.iter().any(|m| m.name == sym)
}

fn has_user_field(fields: &[ClassField], name: &str) -> bool {
  let sym = intern(name);
  fields.iter().any(|f| f.name == sym)
}

pub(super) fn desugar_http(data: KeywordDeclData, span: SourceSpan, arena: &mut AstArena) -> Vec<StmtId> {
  let connector_sym = intern("Connector");
  let connector_path: Vec<Sym> = vec![intern("std"), intern("connector")];
  let use_connector = arena.alloc_stmt(Stmt::Use(UseStmt { path: connector_path, kind: UseKind::Selective(vec![connector_sym]) }), span);

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

  if !has_user_method(&methods, "connect") {
    let ok = gen_ok_unit(span, arena);
    methods.push(gen_method("connect", gen_func(&[], ok, span, arena)));
  }
  if !has_user_method(&methods, "disconnect") {
    let ok = gen_ok_unit(span, arena);
    methods.push(gen_method("disconnect", gen_func(&[], ok, span, arena)));
  }
  if !has_user_method(&methods, "call") {
    methods.push(gen_method("call", build_http_call(span, arena)));
  }
  if !has_user_method(&methods, "tools") {
    let self_endpoints = gen_self_field("endpoints", span, arena);
    methods.push(gen_method("tools", gen_func(&[], self_endpoints, span, arena)));
  }

  let class_stmt = arena.alloc_stmt(
    Stmt::ClassDecl(ClassDeclData { name: data.name, type_params: data.type_params, traits: vec![connector_sym], fields, methods, exported: data.exported }),
    span,
  );

  vec![use_connector, use_http, class_stmt]
}

fn build_http_call(span: SourceSpan, arena: &mut AstArena) -> crate::ast::ExprId {
  let self_base_url = gen_self_field("base_url", span, arena);
  let req_tool = {
    let req = gen_ident("req", span, arena);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: req, field: FieldKind::Named(intern("tool")) }), span)
  };
  let url_expr = arena.alloc_expr(Expr::Binary(ExprBinary { op: BinOp::Concat, left: self_base_url, right: req_tool }), span);
  let url_assign = arena.alloc_stmt(
    Stmt::Binding(crate::ast::Binding {
      exported: false,
      mutable: false,
      target: crate::ast::BindTarget::Name(intern("url")),
      type_ann: None,
      value: url_expr,
    }),
    span,
  );

  let req_args_method = {
    let req = gen_ident("req", span, arena);
    let args = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: req, field: FieldKind::Named(intern("args")) }), span);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: args, field: FieldKind::Named(intern("method")) }), span)
  };
  let default_get = gen_literal_str("GET", span, arena);
  let coalesce_expr = super::desugar::desugar_coalesce(req_args_method, default_get, span, arena);
  let method_coalesced = arena.alloc_expr(coalesce_expr, span);

  let url_ref = gen_ident("url", span, arena);

  let req_args_body = {
    let req = gen_ident("req", span, arena);
    let args = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: req, field: FieldKind::Named(intern("args")) }), span);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: args, field: FieldKind::Named(intern("body")) }), span)
  };

  let self_headers = gen_self_field("headers", span, arena);

  let opts_record = gen_record(
    vec![(intern("method"), method_coalesced), (intern("url"), url_ref), (intern("body"), req_args_body), (intern("headers"), self_headers)],
    span,
    arena,
  );

  let http_request_call = gen_field_call("http", "request", &[opts_record], span, arena);
  let http_stmt = arena.alloc_stmt(Stmt::Expr(http_request_call), span);

  let body = gen_block(vec![url_assign, http_stmt], span, arena);
  gen_func(&["req"], body, span, arena)
}
