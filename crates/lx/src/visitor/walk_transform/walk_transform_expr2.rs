use crate::ast::{
  AstArena, Expr, ExprAssert, ExprCoalesce, ExprEmit, ExprNamedArg, ExprSlice, ExprTernary, ExprTimeout, ExprWith, ExprYield, SelArm, StmtId, WithKind,
};

use super::walk_transform_expr::walk_transform_expr;
use super::walk_transform_stmt;
use crate::visitor::transformer::AstTransformer;

pub(super) fn recurse_expr_children2<T: AstTransformer + ?Sized>(t: &mut T, expr: Expr, arena: &mut AstArena) -> Expr {
  match expr {
    Expr::Ternary(ternary) => {
      let cond = walk_transform_expr(t, ternary.cond, arena);
      let then_ = walk_transform_expr(t, ternary.then_, arena);
      let else_ = ternary.else_.map(|e| walk_transform_expr(t, e, arena));
      Expr::Ternary(ExprTernary { cond, then_, else_ })
    },
    Expr::Propagate(inner) => Expr::Propagate(walk_transform_expr(t, inner, arena)),
    Expr::Coalesce(c) => {
      let expr = walk_transform_expr(t, c.expr, arena);
      let default = walk_transform_expr(t, c.default, arena);
      Expr::Coalesce(ExprCoalesce { expr, default })
    },
    Expr::Slice(s) => {
      let expr = walk_transform_expr(t, s.expr, arena);
      let start = s.start.map(|st| walk_transform_expr(t, st, arena));
      let end = s.end.map(|en| walk_transform_expr(t, en, arena));
      Expr::Slice(ExprSlice { expr, start, end })
    },
    Expr::NamedArg(na) => {
      let value = walk_transform_expr(t, na.value, arena);
      Expr::NamedArg(ExprNamedArg { name: na.name, value })
    },
    Expr::Loop(stmts) => Expr::Loop(recurse_stmts(t, stmts, arena)),
    Expr::Break(val) => Expr::Break(val.map(|v| walk_transform_expr(t, v, arena))),
    Expr::Assert(a) => {
      let expr = walk_transform_expr(t, a.expr, arena);
      let msg = a.msg.map(|m| walk_transform_expr(t, m, arena));
      Expr::Assert(ExprAssert { expr, msg })
    },
    Expr::Par(stmts) => Expr::Par(recurse_stmts(t, stmts, arena)),
    Expr::Sel(arms) => {
      let folded: Vec<SelArm> =
        arms.into_iter().map(|arm| SelArm { expr: walk_transform_expr(t, arm.expr, arena), handler: walk_transform_expr(t, arm.handler, arena) }).collect();
      Expr::Sel(folded)
    },
    Expr::Timeout(timeout) => {
      let ms = walk_transform_expr(t, timeout.ms, arena);
      let body = walk_transform_expr(t, timeout.body, arena);
      Expr::Timeout(ExprTimeout { ms, body })
    },
    Expr::Emit(e) => Expr::Emit(ExprEmit { value: walk_transform_expr(t, e.value, arena) }),
    Expr::Yield(y) => Expr::Yield(ExprYield { value: walk_transform_expr(t, y.value, arena) }),
    Expr::With(w) => recurse_with(t, w, arena),
    other => other,
  }
}

fn recurse_stmts<T: AstTransformer + ?Sized>(t: &mut T, stmts: Vec<StmtId>, arena: &mut AstArena) -> Vec<StmtId> {
  stmts.into_iter().map(|s| walk_transform_stmt(t, s, arena)).collect()
}

fn recurse_with<T: AstTransformer + ?Sized>(t: &mut T, w: ExprWith, arena: &mut AstArena) -> Expr {
  let kind = match w.kind {
    WithKind::Binding { name, value, mutable } => {
      let folded_value = walk_transform_expr(t, value, arena);
      WithKind::Binding { name, value: folded_value, mutable }
    },
    WithKind::Resources { resources } => {
      let folded: Vec<_> = resources.into_iter().map(|(e, sym)| (walk_transform_expr(t, e, arena), sym)).collect();
      WithKind::Resources { resources: folded }
    },
    WithKind::Context { fields } => {
      let folded: Vec<_> = fields.into_iter().map(|(sym, e)| (sym, walk_transform_expr(t, e, arena))).collect();
      WithKind::Context { fields: folded }
    },
  };
  let body: Vec<StmtId> = w.body.into_iter().map(|s| walk_transform_stmt(t, s, arena)).collect();
  Expr::With(ExprWith { kind, body })
}
