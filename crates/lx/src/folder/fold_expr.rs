use crate::ast::{
    Expr, ExprApply, ExprBinary, ExprFieldAccess, ExprPipe, ExprUnary,
    FieldKind, ListElem, Literal, MapEntry, RecordField, SExpr, SStmt,
    Section, StrPart,
};
use miette::SourceSpan;

use super::AstFolder;

pub fn fold_expr<F: AstFolder + ?Sized>(f: &mut F, expr: Expr, span: SourceSpan) -> SExpr {
    match expr {
        Expr::Literal(lit) => {
            let folded = f.fold_literal(lit, span);
            SExpr::new(Expr::Literal(folded), span)
        },
        Expr::Ident(name) => {
            let folded = f.fold_ident(name, span);
            SExpr::new(Expr::Ident(folded), span)
        },
        Expr::TypeConstructor(name) => {
            let folded = f.fold_type_constructor(name, span);
            SExpr::new(Expr::TypeConstructor(folded), span)
        },
        Expr::Binary(b) => f.fold_binary(b, span),
        Expr::Unary(u) => f.fold_unary(u, span),
        Expr::Pipe(p) => f.fold_pipe(p, span),
        Expr::Apply(a) => f.fold_apply(a, span),
        Expr::Section(s) => f.fold_section(s, span),
        Expr::FieldAccess(fa) => f.fold_field_access(fa, span),
        Expr::Block(stmts) => f.fold_block(stmts, span),
        Expr::Tuple(elems) => f.fold_tuple(elems, span),
        Expr::List(elems) => f.fold_list(elems, span),
        Expr::Record(fields) => f.fold_record(fields, span),
        Expr::Map(entries) => f.fold_map(entries, span),
        Expr::Func(func) => f.fold_func(func, span),
        Expr::Match(m) => f.fold_match(m, span),
        Expr::Ternary(t) => f.fold_ternary(t, span),
        Expr::Propagate(inner) => f.fold_propagate(inner, span),
        Expr::Coalesce(c) => f.fold_coalesce(c, span),
        Expr::Slice(s) => f.fold_slice(s, span),
        Expr::NamedArg(na) => f.fold_named_arg(na, span),
        Expr::Loop(stmts) => f.fold_loop(stmts, span),
        Expr::Break(val) => f.fold_break(val, span),
        Expr::Assert(a) => f.fold_assert(a, span),
        Expr::Par(stmts) => f.fold_par(stmts, span),
        Expr::Sel(arms) => f.fold_sel(arms, span),
        Expr::Timeout(t) => f.fold_timeout(t, span),
        Expr::Emit(e) => f.fold_emit(e, span),
        Expr::Yield(y) => f.fold_yield(y, span),
        Expr::With(w) => f.fold_with(w, span),
    }
}

pub fn fold_literal<F: AstFolder + ?Sized>(f: &mut F, lit: Literal, _span: SourceSpan) -> Literal {
    match lit {
        Literal::Str(parts) => {
            let folded = parts
                .into_iter()
                .map(|part| match part {
                    StrPart::Text(s) => StrPart::Text(s),
                    StrPart::Interp(e) => {
                        StrPart::Interp(f.fold_expr(e.node, e.span))
                    },
                })
                .collect();
            Literal::Str(folded)
        },
        other => other,
    }
}

pub fn fold_binary<F: AstFolder + ?Sized>(f: &mut F, b: ExprBinary, span: SourceSpan) -> SExpr {
    let left = f.fold_expr(b.left.node, b.left.span);
    let right = f.fold_expr(b.right.node, b.right.span);
    SExpr::new(
        Expr::Binary(ExprBinary { op: b.op, left: Box::new(left), right: Box::new(right) }),
        span,
    )
}

pub fn fold_unary<F: AstFolder + ?Sized>(f: &mut F, u: ExprUnary, span: SourceSpan) -> SExpr {
    let operand = f.fold_expr(u.operand.node, u.operand.span);
    SExpr::new(Expr::Unary(ExprUnary { op: u.op, operand: Box::new(operand) }), span)
}

pub fn fold_pipe<F: AstFolder + ?Sized>(f: &mut F, p: ExprPipe, span: SourceSpan) -> SExpr {
    let left = f.fold_expr(p.left.node, p.left.span);
    let right = f.fold_expr(p.right.node, p.right.span);
    SExpr::new(
        Expr::Pipe(ExprPipe { left: Box::new(left), right: Box::new(right) }),
        span,
    )
}

pub fn fold_apply<F: AstFolder + ?Sized>(f: &mut F, a: ExprApply, span: SourceSpan) -> SExpr {
    let func = f.fold_expr(a.func.node, a.func.span);
    let arg = f.fold_expr(a.arg.node, a.arg.span);
    SExpr::new(
        Expr::Apply(ExprApply { func: Box::new(func), arg: Box::new(arg) }),
        span,
    )
}

pub fn fold_section<F: AstFolder + ?Sized>(f: &mut F, s: Section, span: SourceSpan) -> SExpr {
    let folded = match s {
        Section::Right { op, operand } => {
            let folded_operand = f.fold_expr(operand.node, operand.span);
            Section::Right { op, operand: Box::new(folded_operand) }
        },
        Section::Left { operand, op } => {
            let folded_operand = f.fold_expr(operand.node, operand.span);
            Section::Left { operand: Box::new(folded_operand), op }
        },
        other => other,
    };
    SExpr::new(Expr::Section(folded), span)
}

pub fn fold_field_access<F: AstFolder + ?Sized>(
    f: &mut F,
    fa: ExprFieldAccess,
    span: SourceSpan,
) -> SExpr {
    let expr = f.fold_expr(fa.expr.node, fa.expr.span);
    let field = match fa.field {
        FieldKind::Computed(c) => {
            FieldKind::Computed(Box::new(f.fold_expr(c.node, c.span)))
        },
        other => other,
    };
    SExpr::new(Expr::FieldAccess(ExprFieldAccess { expr: Box::new(expr), field }), span)
}

pub fn fold_block<F: AstFolder + ?Sized>(f: &mut F, stmts: Vec<SStmt>, span: SourceSpan) -> SExpr {
    let folded = fold_stmts(f, stmts);
    SExpr::new(Expr::Block(folded), span)
}

pub fn fold_tuple<F: AstFolder + ?Sized>(f: &mut F, elems: Vec<SExpr>, span: SourceSpan) -> SExpr {
    let folded = fold_exprs(f, elems);
    SExpr::new(Expr::Tuple(folded), span)
}

pub fn fold_list<F: AstFolder + ?Sized>(
    f: &mut F,
    elems: Vec<ListElem>,
    span: SourceSpan,
) -> SExpr {
    let folded = elems
        .into_iter()
        .map(|elem| match elem {
            ListElem::Single(e) => ListElem::Single(f.fold_expr(e.node, e.span)),
            ListElem::Spread(e) => ListElem::Spread(f.fold_expr(e.node, e.span)),
        })
        .collect();
    SExpr::new(Expr::List(folded), span)
}

pub fn fold_record<F: AstFolder + ?Sized>(
    f: &mut F,
    fields: Vec<RecordField>,
    span: SourceSpan,
) -> SExpr {
    let folded = fields
        .into_iter()
        .map(|field| RecordField {
            name: field.name,
            value: f.fold_expr(field.value.node, field.value.span),
            is_spread: field.is_spread,
        })
        .collect();
    SExpr::new(Expr::Record(folded), span)
}

pub fn fold_map<F: AstFolder + ?Sized>(
    f: &mut F,
    entries: Vec<MapEntry>,
    span: SourceSpan,
) -> SExpr {
    let folded = entries
        .into_iter()
        .map(|entry| MapEntry {
            key: entry.key.map(|k| f.fold_expr(k.node, k.span)),
            value: f.fold_expr(entry.value.node, entry.value.span),
            is_spread: entry.is_spread,
        })
        .collect();
    SExpr::new(Expr::Map(folded), span)
}

pub fn fold_stmts<F: AstFolder + ?Sized>(f: &mut F, stmts: Vec<SStmt>) -> Vec<SStmt> {
    stmts
        .into_iter()
        .map(|s| f.fold_stmt(s.node, s.span))
        .collect()
}

pub fn fold_exprs<F: AstFolder + ?Sized>(f: &mut F, exprs: Vec<SExpr>) -> Vec<SExpr> {
    exprs
        .into_iter()
        .map(|e| f.fold_expr(e.node, e.span))
        .collect()
}
