use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

use miette::SourceSpan;

use crate::ast::{
  AstArena, BinOp, BindTarget, Binding, ClassDeclData, ClassField, Core, Expr, ExprApply,
  ExprBinary, ExprBlock, ExprFieldAccess, ExprFunc, ExprId, ExprMatch, ExprTernary, ExprWith,
  FieldKind, KeywordDeclData, KeywordKind, Literal, MatchArm, Param, Pattern, PatternConstructor,
  Program, Section, Stmt, StmtId, StrPart, Surface, UseKind, UseStmt, WithKind,
};
use crate::folder::gen_ast::*;
use crate::sym::{Sym, intern};
use crate::visitor::transformer::AstTransformer;

static GENSYM_COUNTER: AtomicU64 = AtomicU64::new(0);

fn gensym(prefix: &str) -> Sym {
  let n = GENSYM_COUNTER.fetch_add(1, Ordering::Relaxed);
  intern(&format!("__{prefix}_{n}"))
}

fn make_lambda_expr(name: Sym, body: ExprId) -> Expr {
  Expr::Func(ExprFunc { params: vec![Param { name, type_ann: None, default: None }], type_params: vec![], ret_type: None, guard: None, body })
}

fn alloc_lambda(name: Sym, body: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(make_lambda_expr(name, body), span)
}

struct Desugarer;

impl AstTransformer for Desugarer {
  fn transform_stmts(&mut self, stmts: Vec<StmtId>, arena: &mut AstArena) -> Vec<StmtId> {
    let mut result = Vec::new();
    for sid in stmts {
      let span = arena.stmt_span(sid);
      let stmt = arena.stmt(sid).clone();
      match stmt {
        Stmt::KeywordDecl(data) => {
          let desugared = desugar_keyword(data, span, arena);
          result.extend(desugared);
        },
        _ => {
          let transformed = crate::visitor::walk_transform::walk_transform_stmt(self, sid, arena);
          result.push(transformed);
        },
      }
    }
    result
  }

  fn leave_expr(&mut self, _id: ExprId, expr: Expr, span: SourceSpan, arena: &mut AstArena) -> (Expr, SourceSpan) {
    let result = match expr {
      Expr::Pipe(p) => Expr::Apply(ExprApply { func: p.right, arg: p.left }),
      Expr::Section(s) => desugar_section(s, span, arena),
      Expr::Ternary(t) => desugar_ternary(t.cond, t.then_, t.else_, span, arena),
      Expr::Coalesce(c) => desugar_coalesce(c.expr, c.default, span, arena),
      Expr::Literal(ref lit) if has_interp(lit) => {
        let Expr::Literal(Literal::Str(parts)) = expr else { unreachable!() };
        Expr::Literal(Literal::Str(desugar_interp(parts, span, arena)))
      },
      Expr::With(ref w) if matches!(w.kind, WithKind::Binding { .. }) => {
        let Expr::With(w) = expr else { unreachable!() };
        desugar_with_binding(w, span, arena)
      },
      other => other,
    };
    (result, span)
  }
}

fn has_interp(lit: &Literal) -> bool {
  matches!(lit, Literal::Str(parts) if parts.iter().any(|p| matches!(p, StrPart::Interp(_))))
}

fn desugar_section(s: Section, span: SourceSpan, arena: &mut AstArena) -> Expr {
  match s {
    Section::Right { op, operand } => {
      let p = gensym("x");
      let pi = arena.alloc_expr(Expr::Ident(p), span);
      let body = arena.alloc_expr(Expr::Binary(ExprBinary { op, left: pi, right: operand }), span);
      make_lambda_expr(p, body)
    },
    Section::Left { operand, op } => {
      let p = gensym("x");
      let pi = arena.alloc_expr(Expr::Ident(p), span);
      let body = arena.alloc_expr(Expr::Binary(ExprBinary { op, left: operand, right: pi }), span);
      make_lambda_expr(p, body)
    },
    Section::Field(name) => {
      let p = gensym("x");
      let pi = arena.alloc_expr(Expr::Ident(p), span);
      let body = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: pi, field: FieldKind::Named(name) }), span);
      make_lambda_expr(p, body)
    },
    Section::Index(idx) => {
      let p = gensym("x");
      let pi = arena.alloc_expr(Expr::Ident(p), span);
      let body = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: pi, field: FieldKind::Index(idx) }), span);
      make_lambda_expr(p, body)
    },
    Section::BinOp(op) => {
      let a = gensym("a");
      let b = gensym("b");
      let ai = arena.alloc_expr(Expr::Ident(a), span);
      let bi = arena.alloc_expr(Expr::Ident(b), span);
      let body = arena.alloc_expr(Expr::Binary(ExprBinary { op, left: ai, right: bi }), span);
      let inner = alloc_lambda(b, body, span, arena);
      make_lambda_expr(a, inner)
    },
  }
}

pub(super) fn desugar_ternary(cond: ExprId, then_: ExprId, else_: Option<ExprId>, span: SourceSpan, arena: &mut AstArena) -> Expr {
  let else_body = else_.unwrap_or_else(|| arena.alloc_expr(Expr::Literal(Literal::Unit), span));
  let true_pat = arena.alloc_pattern(Pattern::Literal(Literal::Bool(true)), span);
  let false_pat = arena.alloc_pattern(Pattern::Literal(Literal::Bool(false)), span);
  Expr::Match(ExprMatch {
    scrutinee: cond,
    arms: vec![MatchArm { pattern: true_pat, guard: None, body: then_ }, MatchArm { pattern: false_pat, guard: None, body: else_body }],
  })
}

fn desugar_coalesce(expr: ExprId, default: ExprId, span: SourceSpan, arena: &mut AstArena) -> Expr {
  let v = gensym("v");
  let v_bind = |arena: &mut AstArena| arena.alloc_pattern(Pattern::Bind(v), span);
  let v_ref = |arena: &mut AstArena| arena.alloc_expr(Expr::Ident(v), span);
  let ctor_pat =
    |name: &str, args: Vec<_>, arena: &mut AstArena| arena.alloc_pattern(Pattern::Constructor(PatternConstructor { name: intern(name), args }), span);
  let some_bind = v_bind(arena);
  let some_pat = ctor_pat("Some", vec![some_bind], arena);
  let some_body = v_ref(arena);
  let ok_bind = v_bind(arena);
  let ok_pat = ctor_pat("Ok", vec![ok_bind], arena);
  let ok_body = v_ref(arena);
  let none_pat = ctor_pat("None", vec![], arena);
  let wildcard = arena.alloc_pattern(Pattern::Wildcard, span);
  Expr::Match(ExprMatch {
    scrutinee: expr,
    arms: vec![
      MatchArm { pattern: some_pat, guard: None, body: some_body },
      MatchArm { pattern: ok_pat, guard: None, body: ok_body },
      MatchArm { pattern: none_pat, guard: None, body: default },
      MatchArm { pattern: wildcard, guard: None, body: default },
    ],
  })
}

fn desugar_with_binding(w: ExprWith, span: SourceSpan, arena: &mut AstArena) -> Expr {
  let WithKind::Binding { name, value, mutable } = w.kind else { unreachable!() };
  let binding_stmt = arena.alloc_stmt(Stmt::Binding(Binding { exported: false, mutable, target: BindTarget::Name(name), type_ann: None, value }), span);
  let mut block_stmts = vec![binding_stmt];
  block_stmts.extend(w.body);
  Expr::Block(ExprBlock { stmts: block_stmts })
}

fn desugar_interp(parts: Vec<StrPart>, span: SourceSpan, arena: &mut AstArena) -> Vec<StrPart> {
  let mut result = Vec::new();
  let mut pending = Vec::new();
  for part in parts {
    match part {
      StrPart::Text(s) => pending.push(s),
      StrPart::Interp(eid) => {
        if !pending.is_empty() {
          result.push(StrPart::Text(pending.join("")));
          pending.clear();
        }
        let to_str = arena.alloc_expr(Expr::Ident(intern("to_str")), span);
        let stringified = arena.alloc_expr(Expr::Apply(ExprApply { func: to_str, arg: eid }), span);
        result.push(StrPart::Interp(stringified));
      },
    }
  }
  if !pending.is_empty() {
    result.push(StrPart::Text(pending.join("")));
  }
  result
}

fn desugar_keyword(data: KeywordDeclData, span: SourceSpan, arena: &mut AstArena) -> Vec<StmtId> {
  if data.keyword == KeywordKind::Schema {
    return super::desugar_schema::desugar_schema(data, span, arena);
  }
  if data.keyword == KeywordKind::Mcp {
    return desugar_mcp(data, span, arena);
  }
  if data.keyword == KeywordKind::Cli {
    return desugar_cli(data, span, arena);
  }

  let (import_path, trait_name) = match data.keyword {
    KeywordKind::Agent => (vec!["std", "agent"], "Agent"),
    KeywordKind::Tool => (vec!["std", "tool"], "Tool"),
    KeywordKind::Prompt => (vec!["std", "prompt"], "Prompt"),
    KeywordKind::Connector => (vec!["std", "connector"], "Connector"),
    KeywordKind::Store => (vec!["std", "collection"], "Collection"),
    KeywordKind::Session => (vec!["std", "session"], "Session"),
    KeywordKind::Guard => (vec!["std", "guard"], "Guard"),
    KeywordKind::Workflow => (vec!["std", "workflow"], "Workflow"),
    _ => return vec![arena.alloc_stmt(Stmt::KeywordDecl(data), span)],
  };

  let trait_sym = intern(trait_name);
  let path: Vec<Sym> = import_path.iter().map(|s| intern(s)).collect();

  let use_stmt = arena.alloc_stmt(
    Stmt::Use(UseStmt { path, kind: UseKind::Selective(vec![trait_sym]) }),
    span,
  );

  let fields = data.fields;
  let methods = data.methods;

  let class_stmt = arena.alloc_stmt(
    Stmt::ClassDecl(ClassDeclData {
      name: data.name,
      type_params: data.type_params,
      traits: vec![trait_sym],
      fields,
      methods,
      exported: data.exported,
    }),
    span,
  );

  vec![use_stmt, class_stmt]
}

fn has_user_method(methods: &[crate::ast::AgentMethod], name: &str) -> bool {
  let sym = intern(name);
  methods.iter().any(|m| m.name == sym)
}

fn has_user_field(fields: &[ClassField], name: &str) -> bool {
  let sym = intern(name);
  fields.iter().any(|f| f.name == sym)
}

fn desugar_mcp(data: KeywordDeclData, span: SourceSpan, arena: &mut AstArena) -> Vec<StmtId> {
  let connector_sym = intern("Connector");
  let connector_path: Vec<Sym> = vec![intern("std"), intern("connector")];
  let use_connector = arena.alloc_stmt(
    Stmt::Use(UseStmt { path: connector_path, kind: UseKind::Selective(vec![connector_sym]) }),
    span,
  );

  let mcp_path: Vec<Sym> = vec![intern("std"), intern("mcp")];
  let use_mcp = arena.alloc_stmt(Stmt::Use(UseStmt { path: mcp_path, kind: UseKind::Whole }), span);

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
      Expr::FieldAccess(ExprFieldAccess { expr: req, field: FieldKind::Named(intern("tool")) }),
      span,
    )
  };
  let req_args = {
    let req = gen_ident("req", span, arena);
    arena.alloc_expr(
      Expr::FieldAccess(ExprFieldAccess { expr: req, field: FieldKind::Named(intern("args")) }),
      span,
    )
  };
  let mcp_call = gen_field_call("mcp", "call", &[self_session, req_tool, req_args], span, arena);
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

pub fn desugar(program: Program<Surface>) -> Program<Core> {
  let mut desugarer = Desugarer;
  let folded = desugarer.transform_program(program);
  let core =
    Program { stmts: folded.stmts, arena: folded.arena, comments: folded.comments, comment_map: folded.comment_map, file: folded.file, _phase: PhantomData };
  if cfg!(debug_assertions) {
    super::validate_core::validate_core(&core);
  }
  core
}
