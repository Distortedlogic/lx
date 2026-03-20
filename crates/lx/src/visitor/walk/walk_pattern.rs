use crate::ast::{
    FieldPattern, MatchArm, Param, Pattern, SExpr, SPattern, SStmt, SType, SelArm, StrPart,
};
use crate::span::Span;

use crate::visitor::{AstVisitor, MetaCtx, RefineCtx};

pub fn walk_func<V: AstVisitor + ?Sized>(
    v: &mut V,
    params: &[Param],
    ret_type: Option<&SType>,
    body: &SExpr,
    _span: Span,
) {
    for p in params {
        if let Some(ref d) = p.default {
            v.visit_expr(&d.node, d.span);
        }
        if let Some(ref ty) = p.type_ann {
            v.visit_type_expr(&ty.node, ty.span);
        }
    }
    if let Some(rt) = ret_type {
        v.visit_type_expr(&rt.node, rt.span);
    }
    v.visit_expr(&body.node, body.span);
}

pub fn walk_match<V: AstVisitor + ?Sized>(
    v: &mut V,
    scrutinee: &SExpr,
    arms: &[MatchArm],
    _span: Span,
) {
    v.visit_expr(&scrutinee.node, scrutinee.span);
    for arm in arms {
        v.visit_pattern(&arm.pattern.node, arm.pattern.span);
        if let Some(ref g) = arm.guard {
            v.visit_expr(&g.node, g.span);
        }
        v.visit_expr(&arm.body.node, arm.body.span);
    }
}

pub fn walk_ternary<V: AstVisitor + ?Sized>(
    v: &mut V,
    cond: &SExpr,
    then_: &SExpr,
    else_: Option<&SExpr>,
    _span: Span,
) {
    v.visit_expr(&cond.node, cond.span);
    v.visit_expr(&then_.node, then_.span);
    if let Some(e) = else_ {
        v.visit_expr(&e.node, e.span);
    }
}

pub fn walk_propagate<V: AstVisitor + ?Sized>(v: &mut V, inner: &SExpr, _span: Span) {
    v.visit_expr(&inner.node, inner.span);
}

pub fn walk_coalesce<V: AstVisitor + ?Sized>(
    v: &mut V,
    expr: &SExpr,
    default: &SExpr,
    _span: Span,
) {
    v.visit_expr(&expr.node, expr.span);
    v.visit_expr(&default.node, default.span);
}

pub fn walk_slice<V: AstVisitor + ?Sized>(
    v: &mut V,
    expr: &SExpr,
    start: Option<&SExpr>,
    end: Option<&SExpr>,
    _span: Span,
) {
    v.visit_expr(&expr.node, expr.span);
    if let Some(s) = start {
        v.visit_expr(&s.node, s.span);
    }
    if let Some(e) = end {
        v.visit_expr(&e.node, e.span);
    }
}

pub fn walk_named_arg<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, _span: Span) {
    v.visit_expr(&value.node, value.span);
}

pub fn walk_loop<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], _span: Span) {
    for s in stmts {
        v.visit_stmt(&s.node, s.span);
    }
}

pub fn walk_break<V: AstVisitor + ?Sized>(v: &mut V, value: Option<&SExpr>, _span: Span) {
    if let Some(val) = value {
        v.visit_expr(&val.node, val.span);
    }
}

pub fn walk_assert<V: AstVisitor + ?Sized>(
    v: &mut V,
    expr: &SExpr,
    msg: Option<&SExpr>,
    _span: Span,
) {
    v.visit_expr(&expr.node, expr.span);
    if let Some(m) = msg {
        v.visit_expr(&m.node, m.span);
    }
}

pub fn walk_par<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], _span: Span) {
    for s in stmts {
        v.visit_stmt(&s.node, s.span);
    }
}

pub fn walk_sel<V: AstVisitor + ?Sized>(v: &mut V, arms: &[SelArm], _span: Span) {
    for arm in arms {
        v.visit_expr(&arm.expr.node, arm.expr.span);
        v.visit_expr(&arm.handler.node, arm.handler.span);
    }
}

pub fn walk_agent_send<V: AstVisitor + ?Sized>(
    v: &mut V,
    target: &SExpr,
    msg: &SExpr,
    _span: Span,
) {
    v.visit_expr(&target.node, target.span);
    v.visit_expr(&msg.node, msg.span);
}

pub fn walk_agent_ask<V: AstVisitor + ?Sized>(v: &mut V, target: &SExpr, msg: &SExpr, _span: Span) {
    v.visit_expr(&target.node, target.span);
    v.visit_expr(&msg.node, msg.span);
}

pub fn walk_emit<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, _span: Span) {
    v.visit_expr(&value.node, value.span);
}

pub fn walk_yield<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, _span: Span) {
    v.visit_expr(&value.node, value.span);
}

pub fn walk_with<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, body: &[SStmt], _span: Span) {
    v.visit_expr(&value.node, value.span);
    for s in body {
        v.visit_stmt(&s.node, s.span);
    }
}

pub fn walk_with_resource<V: AstVisitor + ?Sized>(
    v: &mut V,
    resources: &[(SExpr, String)],
    body: &[SStmt],
    _span: Span,
) {
    for (r, _) in resources {
        v.visit_expr(&r.node, r.span);
    }
    for s in body {
        v.visit_stmt(&s.node, s.span);
    }
}

pub fn walk_with_context<V: AstVisitor + ?Sized>(
    v: &mut V,
    fields: &[(String, SExpr)],
    body: &[SStmt],
    _span: Span,
) {
    for (_, expr) in fields {
        v.visit_expr(&expr.node, expr.span);
    }
    for s in body {
        v.visit_stmt(&s.node, s.span);
    }
}

pub fn walk_refine<V: AstVisitor + ?Sized>(v: &mut V, ctx: &RefineCtx<'_>, _span: Span) {
    v.visit_expr(&ctx.initial.node, ctx.initial.span);
    v.visit_expr(&ctx.grade.node, ctx.grade.span);
    v.visit_expr(&ctx.revise.node, ctx.revise.span);
    v.visit_expr(&ctx.threshold.node, ctx.threshold.span);
    v.visit_expr(&ctx.max_rounds.node, ctx.max_rounds.span);
    if let Some(o) = ctx.on_round {
        v.visit_expr(&o.node, o.span);
    }
}

pub fn walk_meta<V: AstVisitor + ?Sized>(v: &mut V, ctx: &MetaCtx<'_>, _span: Span) {
    v.visit_expr(&ctx.task.node, ctx.task.span);
    v.visit_expr(&ctx.strategies.node, ctx.strategies.span);
    v.visit_expr(&ctx.attempt.node, ctx.attempt.span);
    v.visit_expr(&ctx.evaluate.node, ctx.evaluate.span);
    if let Some(s) = ctx.select {
        v.visit_expr(&s.node, s.span);
    }
    if let Some(o) = ctx.on_switch {
        v.visit_expr(&o.node, o.span);
    }
}

pub fn walk_shell<V: AstVisitor + ?Sized>(v: &mut V, parts: &[StrPart], _span: Span) {
    for part in parts {
        if let StrPart::Interp(e) = part {
            v.visit_expr(&e.node, e.span);
        }
    }
}

pub fn walk_pattern<V: AstVisitor + ?Sized>(v: &mut V, pattern: &Pattern, span: Span) {
    match pattern {
        Pattern::Literal(lit) => v.visit_pattern_literal(lit, span),
        Pattern::Bind(name) => v.visit_pattern_bind(name, span),
        Pattern::Wildcard => v.visit_pattern_wildcard(span),
        Pattern::Tuple(elems) => v.visit_pattern_tuple(elems, span),
        Pattern::List { elems, rest } => {
            v.visit_pattern_list(elems, rest.as_deref(), span);
        }
        Pattern::Record { fields, rest } => {
            v.visit_pattern_record(fields, rest.as_deref(), span);
        }
        Pattern::Constructor { name, args } => {
            v.visit_pattern_constructor(name, args, span);
        }
    }
}

pub fn walk_pattern_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SPattern], _span: Span) {
    for e in elems {
        v.visit_pattern(&e.node, e.span);
    }
}

pub fn walk_pattern_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SPattern], _span: Span) {
    for e in elems {
        v.visit_pattern(&e.node, e.span);
    }
}

pub fn walk_pattern_record<V: AstVisitor + ?Sized>(
    v: &mut V,
    fields: &[FieldPattern],
    _span: Span,
) {
    for f in fields {
        if let Some(ref p) = f.pattern {
            v.visit_pattern(&p.node, p.span);
        }
    }
}

pub fn walk_pattern_constructor<V: AstVisitor + ?Sized>(v: &mut V, args: &[SPattern], _span: Span) {
    for a in args {
        v.visit_pattern(&a.node, a.span);
    }
}
