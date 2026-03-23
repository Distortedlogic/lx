use crate::ast::{
  AstArena, Expr, ExprAssert, ExprCoalesce, ExprEmit, ExprFunc, ExprId, ExprMatch, ExprNamedArg, ExprSlice, ExprTernary, ExprTimeout, ExprWith, ExprYield,
  MatchArm, Param, SelArm, StmtId, WithKind,
};
use miette::SourceSpan;

use super::AstFolder;

pub fn fold_func<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, func: &ExprFunc, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let params: Vec<Param> = func
    .params
    .iter()
    .map(|p| Param { name: p.name, type_ann: p.type_ann.map(|t| f.fold_type_expr(t, arena)), default: p.default.map(|d| f.fold_expr(d, arena)) })
    .collect();
  let ret_type = func.ret_type.map(|t| f.fold_type_expr(t, arena));
  let guard = func.guard.map(|g| f.fold_expr(g, arena));
  let body = f.fold_expr(func.body, arena);
  let params_changed = params.iter().zip(func.params.iter()).any(|(a, b)| a.type_ann != b.type_ann || a.default != b.default);
  if !params_changed && ret_type == func.ret_type && guard == func.guard && body == func.body {
    return id;
  }
  arena.alloc_expr(Expr::Func(ExprFunc { params, ret_type, guard, body }), span)
}

pub fn fold_match<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, m: &ExprMatch, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let scrutinee = f.fold_expr(m.scrutinee, arena);
  let arms: Vec<MatchArm> = m
    .arms
    .iter()
    .map(|arm| MatchArm { pattern: f.fold_pattern(arm.pattern, arena), guard: arm.guard.map(|g| f.fold_expr(g, arena)), body: f.fold_expr(arm.body, arena) })
    .collect();
  let arms_changed = arms.iter().zip(m.arms.iter()).any(|(a, b)| a.pattern != b.pattern || a.guard != b.guard || a.body != b.body);
  if scrutinee == m.scrutinee && !arms_changed {
    return id;
  }
  arena.alloc_expr(Expr::Match(ExprMatch { scrutinee, arms }), span)
}

pub fn fold_ternary<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, t: ExprTernary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let cond = f.fold_expr(t.cond, arena);
  let then_ = f.fold_expr(t.then_, arena);
  let else_ = t.else_.map(|e| f.fold_expr(e, arena));
  if cond == t.cond && then_ == t.then_ && else_ == t.else_ {
    return id;
  }
  arena.alloc_expr(Expr::Ternary(ExprTernary { cond, then_, else_ }), span)
}

pub fn fold_propagate<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, inner: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = f.fold_expr(inner, arena);
  if folded == inner {
    return id;
  }
  arena.alloc_expr(Expr::Propagate(folded), span)
}

pub fn fold_coalesce<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, c: ExprCoalesce, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let expr = f.fold_expr(c.expr, arena);
  let default = f.fold_expr(c.default, arena);
  if expr == c.expr && default == c.default {
    return id;
  }
  arena.alloc_expr(Expr::Coalesce(ExprCoalesce { expr, default }), span)
}

pub fn fold_slice<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, s: ExprSlice, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let expr = f.fold_expr(s.expr, arena);
  let start = s.start.map(|st| f.fold_expr(st, arena));
  let end = s.end.map(|en| f.fold_expr(en, arena));
  if expr == s.expr && start == s.start && end == s.end {
    return id;
  }
  arena.alloc_expr(Expr::Slice(ExprSlice { expr, start, end }), span)
}

pub fn fold_named_arg<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, na: ExprNamedArg, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let value = f.fold_expr(na.value, arena);
  if value == na.value {
    return id;
  }
  arena.alloc_expr(Expr::NamedArg(ExprNamedArg { name: na.name, value }), span)
}

pub fn fold_loop<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, stmts: &[StmtId], span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded: Vec<StmtId> = stmts.iter().map(|s| f.fold_stmt(*s, arena)).collect();
  if folded.as_slice() == stmts {
    return id;
  }
  arena.alloc_expr(Expr::Loop(folded), span)
}

pub fn fold_break<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, val: Option<ExprId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = val.map(|v| f.fold_expr(v, arena));
  if folded == val {
    return id;
  }
  arena.alloc_expr(Expr::Break(folded), span)
}

pub fn fold_assert<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, a: ExprAssert, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let expr = f.fold_expr(a.expr, arena);
  let msg = a.msg.map(|m| f.fold_expr(m, arena));
  if expr == a.expr && msg == a.msg {
    return id;
  }
  arena.alloc_expr(Expr::Assert(ExprAssert { expr, msg }), span)
}

pub fn fold_par<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, stmts: &[StmtId], span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded: Vec<StmtId> = stmts.iter().map(|s| f.fold_stmt(*s, arena)).collect();
  if folded.as_slice() == stmts {
    return id;
  }
  arena.alloc_expr(Expr::Par(folded), span)
}

pub fn fold_sel<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, arms: &[SelArm], span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded: Vec<SelArm> = arms.iter().map(|arm| SelArm { expr: f.fold_expr(arm.expr, arena), handler: f.fold_expr(arm.handler, arena) }).collect();
  let changed = folded.iter().zip(arms.iter()).any(|(a, b)| a.expr != b.expr || a.handler != b.handler);
  if !changed {
    return id;
  }
  arena.alloc_expr(Expr::Sel(folded), span)
}

pub fn fold_timeout<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, t: ExprTimeout, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let ms = f.fold_expr(t.ms, arena);
  let body = f.fold_expr(t.body, arena);
  if ms == t.ms && body == t.body {
    return id;
  }
  arena.alloc_expr(Expr::Timeout(ExprTimeout { ms, body }), span)
}

pub fn fold_emit<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, e: ExprEmit, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let value = f.fold_expr(e.value, arena);
  if value == e.value {
    return id;
  }
  arena.alloc_expr(Expr::Emit(ExprEmit { value }), span)
}

pub fn fold_yield<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, y: ExprYield, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let value = f.fold_expr(y.value, arena);
  if value == y.value {
    return id;
  }
  arena.alloc_expr(Expr::Yield(ExprYield { value }), span)
}

pub fn fold_with<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, w: &ExprWith, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let (kind, kind_changed) = match &w.kind {
    WithKind::Binding { name, value, mutable } => {
      let folded_value = f.fold_expr(*value, arena);
      (WithKind::Binding { name: *name, value: folded_value, mutable: *mutable }, folded_value != *value)
    },
    WithKind::Resources { resources } => {
      let folded: Vec<_> = resources.iter().map(|(e, sym)| (f.fold_expr(*e, arena), *sym)).collect();
      let changed = folded.iter().zip(resources.iter()).any(|(a, b)| a.0 != b.0);
      (WithKind::Resources { resources: folded }, changed)
    },
    WithKind::Context { fields } => {
      let folded: Vec<_> = fields.iter().map(|(sym, e)| (*sym, f.fold_expr(*e, arena))).collect();
      let changed = folded.iter().zip(fields.iter()).any(|(a, b)| a.1 != b.1);
      (WithKind::Context { fields: folded }, changed)
    },
  };
  let body: Vec<StmtId> = w.body.iter().map(|s| f.fold_stmt(*s, arena)).collect();
  if !kind_changed && body.as_slice() == w.body.as_slice() {
    return id;
  }
  arena.alloc_expr(Expr::With(ExprWith { kind, body }), span)
}
