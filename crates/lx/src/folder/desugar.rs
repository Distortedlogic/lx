use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

use miette::SourceSpan;

use crate::ast::{
  AstArena, BindTarget, Binding, Core, Expr, ExprApply, ExprBinary, ExprFieldAccess, ExprFunc, ExprId, ExprMatch, ExprWith, FieldKind, Literal, MatchArm,
  Param, Pattern, PatternConstructor, Program, Section, Stmt, StrPart, Surface, WithKind,
};
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

fn desugar_ternary(cond: ExprId, then_: ExprId, else_: Option<ExprId>, span: SourceSpan, arena: &mut AstArena) -> Expr {
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
  Expr::Block(block_stmts)
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
