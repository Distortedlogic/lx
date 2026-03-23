use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

use miette::SourceSpan;

use crate::ast::{
  AstArena, BindTarget, Binding, Core, Expr, ExprApply, ExprBinary, ExprCoalesce, ExprFieldAccess, ExprFunc, ExprId, ExprMatch, ExprPipe, ExprTernary,
  ExprWith, FieldKind, Literal, MatchArm, Param, Pattern, PatternConstructor, Program, Section, Stmt, StrPart, Surface, WithKind,
};
use crate::sym::{Sym, intern};

use super::AstFolder;
use super::fold_expr::fold_literal;

static GENSYM_COUNTER: AtomicU64 = AtomicU64::new(0);

fn gensym(prefix: &str) -> Sym {
  let n = GENSYM_COUNTER.fetch_add(1, Ordering::Relaxed);
  intern(&format!("__{prefix}_{n}"))
}

fn make_lambda(name: Sym, body: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(Expr::Func(ExprFunc { params: vec![Param { name, type_ann: None, default: None }], ret_type: None, guard: None, body }), span)
}

struct Desugarer;

impl AstFolder for Desugarer {
  fn fold_pipe(&mut self, _id: ExprId, p: ExprPipe, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    let left = self.fold_expr(p.left, arena);
    let right = self.fold_expr(p.right, arena);
    arena.alloc_expr(Expr::Apply(ExprApply { func: right, arg: left }), span)
  }

  fn fold_section(&mut self, _id: ExprId, s: Section, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    match s {
      Section::Right { op, operand } => {
        let folded_operand = self.fold_expr(operand, arena);
        let p = gensym("x");
        let pi = arena.alloc_expr(Expr::Ident(p), span);
        let body = arena.alloc_expr(Expr::Binary(ExprBinary { op, left: pi, right: folded_operand }), span);
        make_lambda(p, body, span, arena)
      },
      Section::Left { operand, op } => {
        let folded_operand = self.fold_expr(operand, arena);
        let p = gensym("x");
        let pi = arena.alloc_expr(Expr::Ident(p), span);
        let body = arena.alloc_expr(Expr::Binary(ExprBinary { op, left: folded_operand, right: pi }), span);
        make_lambda(p, body, span, arena)
      },
      Section::Field(name) => {
        let p = gensym("x");
        let pi = arena.alloc_expr(Expr::Ident(p), span);
        let body = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: pi, field: FieldKind::Named(name) }), span);
        make_lambda(p, body, span, arena)
      },
      Section::Index(idx) => {
        let p = gensym("x");
        let pi = arena.alloc_expr(Expr::Ident(p), span);
        let body = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: pi, field: FieldKind::Index(idx) }), span);
        make_lambda(p, body, span, arena)
      },
      Section::BinOp(op) => {
        let a = gensym("a");
        let b = gensym("b");
        let ai = arena.alloc_expr(Expr::Ident(a), span);
        let bi = arena.alloc_expr(Expr::Ident(b), span);
        let body = arena.alloc_expr(Expr::Binary(ExprBinary { op, left: ai, right: bi }), span);
        let inner = make_lambda(b, body, span, arena);
        make_lambda(a, inner, span, arena)
      },
    }
  }

  fn fold_ternary(&mut self, _id: ExprId, t: ExprTernary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    let cond = self.fold_expr(t.cond, arena);
    let then_ = self.fold_expr(t.then_, arena);
    let else_ = t.else_.map(|e| self.fold_expr(e, arena)).unwrap_or_else(|| arena.alloc_expr(Expr::Literal(Literal::Unit), span));
    let true_pat = arena.alloc_pattern(Pattern::Literal(Literal::Bool(true)), span);
    let false_pat = arena.alloc_pattern(Pattern::Literal(Literal::Bool(false)), span);
    arena.alloc_expr(
      Expr::Match(ExprMatch {
        scrutinee: cond,
        arms: vec![MatchArm { pattern: true_pat, guard: None, body: then_ }, MatchArm { pattern: false_pat, guard: None, body: else_ }],
      }),
      span,
    )
  }

  fn fold_coalesce(&mut self, _id: ExprId, c: ExprCoalesce, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    let expr = self.fold_expr(c.expr, arena);
    let default = self.fold_expr(c.default, arena);
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
    arena.alloc_expr(
      Expr::Match(ExprMatch {
        scrutinee: expr,
        arms: vec![
          MatchArm { pattern: some_pat, guard: None, body: some_body },
          MatchArm { pattern: ok_pat, guard: None, body: ok_body },
          MatchArm { pattern: none_pat, guard: None, body: default },
          MatchArm { pattern: wildcard, guard: None, body: default },
        ],
      }),
      span,
    )
  }

  fn fold_literal(&mut self, lit: Literal, span: SourceSpan, arena: &mut AstArena) -> Literal {
    match lit {
      Literal::Str(ref parts) if parts.iter().any(|p| matches!(p, StrPart::Interp(_))) => {
        let Literal::Str(parts) = fold_literal(self, lit, span, arena) else {
          unreachable!();
        };
        Literal::Str(desugar_interp(parts, span, arena))
      },
      other => fold_literal(self, other, span, arena),
    }
  }

  fn fold_with(&mut self, _id: ExprId, w: ExprWith, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    match w.kind {
      WithKind::Binding { name, value, mutable } => {
        let folded_value = self.fold_expr(value, arena);
        let folded_body: Vec<_> = w.body.into_iter().map(|s| self.fold_stmt(s, arena)).collect();
        let binding_stmt =
          arena.alloc_stmt(Stmt::Binding(Binding { exported: false, mutable, target: BindTarget::Name(name), type_ann: None, value: folded_value }), span);
        let mut block_stmts = vec![binding_stmt];
        block_stmts.extend(folded_body);
        arena.alloc_expr(Expr::Block(block_stmts), span)
      },
      other_kind => {
        let kind = match other_kind {
          WithKind::Resources { resources } => {
            WithKind::Resources { resources: resources.into_iter().map(|(e, sym)| (self.fold_expr(e, arena), sym)).collect() }
          },
          WithKind::Context { fields } => WithKind::Context { fields: fields.into_iter().map(|(sym, e)| (sym, self.fold_expr(e, arena))).collect() },
          WithKind::Binding { .. } => unreachable!(),
        };
        let body: Vec<_> = w.body.into_iter().map(|s| self.fold_stmt(s, arena)).collect();
        arena.alloc_expr(Expr::With(ExprWith { kind, body }), span)
      },
    }
  }
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
  let folded = desugarer.fold_program(program);
  Program { stmts: folded.stmts, arena: folded.arena, _phase: PhantomData }
}
