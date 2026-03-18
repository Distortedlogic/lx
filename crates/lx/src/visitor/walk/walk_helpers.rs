use crate::ast::{
    FieldKind, ListElem, Literal, MapEntry, RecordField, SExpr, SStmt, Section, StrPart,
};
use crate::span::Span;

use super::super::AstVisitor;

pub fn walk_literal<V: AstVisitor + ?Sized>(v: &mut V, lit: &Literal, _span: Span) {
    if let Literal::Str(parts) = lit {
        for part in parts {
            if let StrPart::Interp(e) = part {
                v.visit_expr(&e.node, e.span);
            }
        }
    }
}

pub fn walk_binary<V: AstVisitor + ?Sized>(v: &mut V, left: &SExpr, right: &SExpr, _span: Span) {
    v.visit_expr(&left.node, left.span);
    v.visit_expr(&right.node, right.span);
}

pub fn walk_unary<V: AstVisitor + ?Sized>(v: &mut V, operand: &SExpr, _span: Span) {
    v.visit_expr(&operand.node, operand.span);
}

pub fn walk_pipe<V: AstVisitor + ?Sized>(v: &mut V, left: &SExpr, right: &SExpr, _span: Span) {
    v.visit_expr(&left.node, left.span);
    v.visit_expr(&right.node, right.span);
}

pub fn walk_apply<V: AstVisitor + ?Sized>(v: &mut V, func: &SExpr, arg: &SExpr, _span: Span) {
    v.visit_expr(&func.node, func.span);
    v.visit_expr(&arg.node, arg.span);
}

pub fn walk_section<V: AstVisitor + ?Sized>(v: &mut V, section: &Section, _span: Span) {
    match section {
        Section::Right { operand, .. } | Section::Left { operand, .. } => {
            v.visit_expr(&operand.node, operand.span);
        }
        _ => {}
    }
}

pub fn walk_field_access<V: AstVisitor + ?Sized>(
    v: &mut V,
    expr: &SExpr,
    field: &FieldKind,
    _span: Span,
) {
    v.visit_expr(&expr.node, expr.span);
    if let FieldKind::Computed(c) = field {
        v.visit_expr(&c.node, c.span);
    }
}

pub fn walk_block<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], _span: Span) {
    for s in stmts {
        v.visit_stmt(&s.node, s.span);
    }
}

pub fn walk_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SExpr], _span: Span) {
    for e in elems {
        v.visit_expr(&e.node, e.span);
    }
}

pub fn walk_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[ListElem], _span: Span) {
    for e in elems {
        match e {
            ListElem::Single(se) | ListElem::Spread(se) => v.visit_expr(&se.node, se.span),
        }
    }
}

pub fn walk_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[RecordField], _span: Span) {
    for f in fields {
        v.visit_expr(&f.value.node, f.value.span);
    }
}

pub fn walk_map<V: AstVisitor + ?Sized>(v: &mut V, entries: &[MapEntry], _span: Span) {
    for e in entries {
        if let Some(ref k) = e.key {
            v.visit_expr(&k.node, k.span);
        }
        v.visit_expr(&e.value.node, e.value.span);
    }
}
