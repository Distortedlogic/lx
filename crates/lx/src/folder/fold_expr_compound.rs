use crate::ast::{
    Expr, ExprAssert, ExprCoalesce, ExprEmit, ExprFunc, ExprMatch,
    ExprNamedArg, ExprSlice, ExprTernary, ExprTimeout, ExprWith, ExprYield,
    MatchArm, Param, SExpr, SStmt, SelArm, WithKind,
};
use miette::SourceSpan;

use super::fold_expr::fold_stmts;
use super::AstFolder;

pub fn fold_func<F: AstFolder + ?Sized>(
    f: &mut F,
    func: ExprFunc,
    span: SourceSpan,
) -> SExpr {
    let params = func
        .params
        .into_iter()
        .map(|p| Param {
            name: p.name,
            type_ann: p.type_ann.map(|t| f.fold_type_expr(t.node, t.span)),
            default: p.default.map(|d| f.fold_expr(d.node, d.span)),
        })
        .collect();
    let ret_type = func.ret_type.map(|t| f.fold_type_expr(t.node, t.span));
    let guard = func.guard.map(|g| Box::new(f.fold_expr(g.node, g.span)));
    let body = Box::new(f.fold_expr(func.body.node, func.body.span));
    SExpr::new(Expr::Func(ExprFunc { params, ret_type, guard, body }), span)
}

pub fn fold_match<F: AstFolder + ?Sized>(
    f: &mut F,
    m: ExprMatch,
    span: SourceSpan,
) -> SExpr {
    let scrutinee = Box::new(f.fold_expr(m.scrutinee.node, m.scrutinee.span));
    let arms = m
        .arms
        .into_iter()
        .map(|arm| MatchArm {
            pattern: f.fold_pattern(arm.pattern.node, arm.pattern.span),
            guard: arm.guard.map(|g| f.fold_expr(g.node, g.span)),
            body: f.fold_expr(arm.body.node, arm.body.span),
        })
        .collect();
    SExpr::new(Expr::Match(ExprMatch { scrutinee, arms }), span)
}

pub fn fold_ternary<F: AstFolder + ?Sized>(
    f: &mut F,
    t: ExprTernary,
    span: SourceSpan,
) -> SExpr {
    let cond = Box::new(f.fold_expr(t.cond.node, t.cond.span));
    let then_ = Box::new(f.fold_expr(t.then_.node, t.then_.span));
    let else_ = t.else_.map(|e| Box::new(f.fold_expr(e.node, e.span)));
    SExpr::new(Expr::Ternary(ExprTernary { cond, then_, else_ }), span)
}

pub fn fold_propagate<F: AstFolder + ?Sized>(
    f: &mut F,
    inner: Box<SExpr>,
    span: SourceSpan,
) -> SExpr {
    let folded = Box::new(f.fold_expr(inner.node, inner.span));
    SExpr::new(Expr::Propagate(folded), span)
}

pub fn fold_coalesce<F: AstFolder + ?Sized>(
    f: &mut F,
    c: ExprCoalesce,
    span: SourceSpan,
) -> SExpr {
    let expr = Box::new(f.fold_expr(c.expr.node, c.expr.span));
    let default = Box::new(f.fold_expr(c.default.node, c.default.span));
    SExpr::new(Expr::Coalesce(ExprCoalesce { expr, default }), span)
}

pub fn fold_slice<F: AstFolder + ?Sized>(
    f: &mut F,
    s: ExprSlice,
    span: SourceSpan,
) -> SExpr {
    let expr = Box::new(f.fold_expr(s.expr.node, s.expr.span));
    let start = s.start.map(|st| Box::new(f.fold_expr(st.node, st.span)));
    let end = s.end.map(|en| Box::new(f.fold_expr(en.node, en.span)));
    SExpr::new(Expr::Slice(ExprSlice { expr, start, end }), span)
}

pub fn fold_named_arg<F: AstFolder + ?Sized>(
    f: &mut F,
    na: ExprNamedArg,
    span: SourceSpan,
) -> SExpr {
    let value = Box::new(f.fold_expr(na.value.node, na.value.span));
    SExpr::new(Expr::NamedArg(ExprNamedArg { name: na.name, value }), span)
}

pub fn fold_loop<F: AstFolder + ?Sized>(f: &mut F, stmts: Vec<SStmt>, span: SourceSpan) -> SExpr {
    let folded = fold_stmts(f, stmts);
    SExpr::new(Expr::Loop(folded), span)
}

pub fn fold_break<F: AstFolder + ?Sized>(
    f: &mut F,
    val: Option<Box<SExpr>>,
    span: SourceSpan,
) -> SExpr {
    let folded = val.map(|v| Box::new(f.fold_expr(v.node, v.span)));
    SExpr::new(Expr::Break(folded), span)
}

pub fn fold_assert<F: AstFolder + ?Sized>(
    f: &mut F,
    a: ExprAssert,
    span: SourceSpan,
) -> SExpr {
    let expr = Box::new(f.fold_expr(a.expr.node, a.expr.span));
    let msg = a.msg.map(|m| Box::new(f.fold_expr(m.node, m.span)));
    SExpr::new(Expr::Assert(ExprAssert { expr, msg }), span)
}

pub fn fold_par<F: AstFolder + ?Sized>(f: &mut F, stmts: Vec<SStmt>, span: SourceSpan) -> SExpr {
    let folded = fold_stmts(f, stmts);
    SExpr::new(Expr::Par(folded), span)
}

pub fn fold_sel<F: AstFolder + ?Sized>(
    f: &mut F,
    arms: Vec<SelArm>,
    span: SourceSpan,
) -> SExpr {
    let folded = arms
        .into_iter()
        .map(|arm| SelArm {
            expr: f.fold_expr(arm.expr.node, arm.expr.span),
            handler: f.fold_expr(arm.handler.node, arm.handler.span),
        })
        .collect();
    SExpr::new(Expr::Sel(folded), span)
}

pub fn fold_timeout<F: AstFolder + ?Sized>(
    f: &mut F,
    t: ExprTimeout,
    span: SourceSpan,
) -> SExpr {
    let ms = Box::new(f.fold_expr(t.ms.node, t.ms.span));
    let body = Box::new(f.fold_expr(t.body.node, t.body.span));
    SExpr::new(Expr::Timeout(ExprTimeout { ms, body }), span)
}

pub fn fold_emit<F: AstFolder + ?Sized>(f: &mut F, e: ExprEmit, span: SourceSpan) -> SExpr {
    let value = Box::new(f.fold_expr(e.value.node, e.value.span));
    SExpr::new(Expr::Emit(ExprEmit { value }), span)
}

pub fn fold_yield<F: AstFolder + ?Sized>(f: &mut F, y: ExprYield, span: SourceSpan) -> SExpr {
    let value = Box::new(f.fold_expr(y.value.node, y.value.span));
    SExpr::new(Expr::Yield(ExprYield { value }), span)
}

pub fn fold_with<F: AstFolder + ?Sized>(f: &mut F, w: ExprWith, span: SourceSpan) -> SExpr {
    let kind = match w.kind {
        WithKind::Binding { name, value, mutable } => {
            let folded_value = Box::new(f.fold_expr(value.node, value.span));
            WithKind::Binding { name, value: folded_value, mutable }
        },
        WithKind::Resources { resources } => {
            let folded = resources
                .into_iter()
                .map(|(e, sym)| (f.fold_expr(e.node, e.span), sym))
                .collect();
            WithKind::Resources { resources: folded }
        },
        WithKind::Context { fields } => {
            let folded = fields
                .into_iter()
                .map(|(sym, e)| (sym, f.fold_expr(e.node, e.span)))
                .collect();
            WithKind::Context { fields: folded }
        },
    };
    let body = fold_stmts(f, w.body);
    SExpr::new(Expr::With(ExprWith { kind, body }), span)
}
