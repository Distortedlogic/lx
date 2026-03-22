use crate::ast::{
  AstArena, Expr, ExprAssert, ExprCoalesce, ExprEmit, ExprFunc, ExprId, ExprMatch, ExprNamedArg, ExprSlice, ExprTernary, ExprTimeout, ExprWith, ExprYield,
  MatchArm, Param, SelArm, StmtId, WithKind,
};
use miette::SourceSpan;

use super::AstFolder;
use super::fold_expr::fold_stmts;

pub fn fold_func<F: AstFolder + ?Sized>(f: &mut F, func: ExprFunc, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let params = func
    .params
    .into_iter()
    .map(|p| Param { name: p.name, type_ann: p.type_ann.map(|t| f.fold_type_expr(t, arena)), default: p.default.map(|d| f.fold_expr(d, arena)) })
    .collect();
  let ret_type = func.ret_type.map(|t| f.fold_type_expr(t, arena));
  let guard = func.guard.map(|g| f.fold_expr(g, arena));
  let body = f.fold_expr(func.body, arena);
  arena.alloc_expr(Expr::Func(ExprFunc { params, ret_type, guard, body }), span)
}

pub fn fold_match<F: AstFolder + ?Sized>(f: &mut F, m: ExprMatch, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let scrutinee = f.fold_expr(m.scrutinee, arena);
  let arms = m
    .arms
    .into_iter()
    .map(|arm| MatchArm { pattern: f.fold_pattern(arm.pattern, arena), guard: arm.guard.map(|g| f.fold_expr(g, arena)), body: f.fold_expr(arm.body, arena) })
    .collect();
  arena.alloc_expr(Expr::Match(ExprMatch { scrutinee, arms }), span)
}

pub fn fold_ternary<F: AstFolder + ?Sized>(f: &mut F, t: ExprTernary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let cond = f.fold_expr(t.cond, arena);
  let then_ = f.fold_expr(t.then_, arena);
  let else_ = t.else_.map(|e| f.fold_expr(e, arena));
  arena.alloc_expr(Expr::Ternary(ExprTernary { cond, then_, else_ }), span)
}

pub fn fold_propagate<F: AstFolder + ?Sized>(f: &mut F, inner: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = f.fold_expr(inner, arena);
  arena.alloc_expr(Expr::Propagate(folded), span)
}

pub fn fold_coalesce<F: AstFolder + ?Sized>(f: &mut F, c: ExprCoalesce, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let expr = f.fold_expr(c.expr, arena);
  let default = f.fold_expr(c.default, arena);
  arena.alloc_expr(Expr::Coalesce(ExprCoalesce { expr, default }), span)
}

pub fn fold_slice<F: AstFolder + ?Sized>(f: &mut F, s: ExprSlice, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let expr = f.fold_expr(s.expr, arena);
  let start = s.start.map(|st| f.fold_expr(st, arena));
  let end = s.end.map(|en| f.fold_expr(en, arena));
  arena.alloc_expr(Expr::Slice(ExprSlice { expr, start, end }), span)
}

pub fn fold_named_arg<F: AstFolder + ?Sized>(f: &mut F, na: ExprNamedArg, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let value = f.fold_expr(na.value, arena);
  arena.alloc_expr(Expr::NamedArg(ExprNamedArg { name: na.name, value }), span)
}

pub fn fold_loop<F: AstFolder + ?Sized>(f: &mut F, stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = fold_stmts(f, stmts, arena);
  arena.alloc_expr(Expr::Loop(folded), span)
}

pub fn fold_break<F: AstFolder + ?Sized>(f: &mut F, val: Option<ExprId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = val.map(|v| f.fold_expr(v, arena));
  arena.alloc_expr(Expr::Break(folded), span)
}

pub fn fold_assert<F: AstFolder + ?Sized>(f: &mut F, a: ExprAssert, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let expr = f.fold_expr(a.expr, arena);
  let msg = a.msg.map(|m| f.fold_expr(m, arena));
  arena.alloc_expr(Expr::Assert(ExprAssert { expr, msg }), span)
}

pub fn fold_par<F: AstFolder + ?Sized>(f: &mut F, stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = fold_stmts(f, stmts, arena);
  arena.alloc_expr(Expr::Par(folded), span)
}

pub fn fold_sel<F: AstFolder + ?Sized>(f: &mut F, arms: Vec<SelArm>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = arms.into_iter().map(|arm| SelArm { expr: f.fold_expr(arm.expr, arena), handler: f.fold_expr(arm.handler, arena) }).collect();
  arena.alloc_expr(Expr::Sel(folded), span)
}

pub fn fold_timeout<F: AstFolder + ?Sized>(f: &mut F, t: ExprTimeout, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let ms = f.fold_expr(t.ms, arena);
  let body = f.fold_expr(t.body, arena);
  arena.alloc_expr(Expr::Timeout(ExprTimeout { ms, body }), span)
}

pub fn fold_emit<F: AstFolder + ?Sized>(f: &mut F, e: ExprEmit, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let value = f.fold_expr(e.value, arena);
  arena.alloc_expr(Expr::Emit(ExprEmit { value }), span)
}

pub fn fold_yield<F: AstFolder + ?Sized>(f: &mut F, y: ExprYield, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let value = f.fold_expr(y.value, arena);
  arena.alloc_expr(Expr::Yield(ExprYield { value }), span)
}

pub fn fold_with<F: AstFolder + ?Sized>(f: &mut F, w: ExprWith, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let kind = match w.kind {
    WithKind::Binding { name, value, mutable } => {
      let folded_value = f.fold_expr(value, arena);
      WithKind::Binding { name, value: folded_value, mutable }
    },
    WithKind::Resources { resources } => {
      let folded = resources.into_iter().map(|(e, sym)| (f.fold_expr(e, arena), sym)).collect();
      WithKind::Resources { resources: folded }
    },
    WithKind::Context { fields } => {
      let folded = fields.into_iter().map(|(sym, e)| (sym, f.fold_expr(e, arena))).collect();
      WithKind::Context { fields: folded }
    },
  };
  let body = fold_stmts(f, w.body, arena);
  arena.alloc_expr(Expr::With(ExprWith { kind, body }), span)
}
