use crate::ast::{SType, TypeExpr, TypeField};
use miette::SourceSpan;

use super::AstFolder;

pub fn fold_type_expr<F: AstFolder + ?Sized>(
    f: &mut F,
    type_expr: TypeExpr,
    span: SourceSpan,
) -> SType {
    match type_expr {
        TypeExpr::Named(_) | TypeExpr::Var(_) => SType::new(type_expr, span),
        TypeExpr::Applied(name, args) => {
            let folded = args
                .into_iter()
                .map(|a| f.fold_type_expr(a.node, a.span))
                .collect();
            SType::new(TypeExpr::Applied(name, folded), span)
        },
        TypeExpr::List(inner) => {
            let folded = f.fold_type_expr(inner.node, inner.span);
            SType::new(TypeExpr::List(Box::new(folded)), span)
        },
        TypeExpr::Map { key, value } => {
            let k = f.fold_type_expr(key.node, key.span);
            let v = f.fold_type_expr(value.node, value.span);
            SType::new(TypeExpr::Map { key: Box::new(k), value: Box::new(v) }, span)
        },
        TypeExpr::Record(fields) => {
            let folded = fields
                .into_iter()
                .map(|field| TypeField {
                    name: field.name,
                    ty: f.fold_type_expr(field.ty.node, field.ty.span),
                })
                .collect();
            SType::new(TypeExpr::Record(folded), span)
        },
        TypeExpr::Tuple(elems) => {
            let folded = elems
                .into_iter()
                .map(|e| f.fold_type_expr(e.node, e.span))
                .collect();
            SType::new(TypeExpr::Tuple(folded), span)
        },
        TypeExpr::Func { param, ret } => {
            let p = f.fold_type_expr(param.node, param.span);
            let r = f.fold_type_expr(ret.node, ret.span);
            SType::new(TypeExpr::Func { param: Box::new(p), ret: Box::new(r) }, span)
        },
        TypeExpr::Fallible { ok, err } => {
            let o = f.fold_type_expr(ok.node, ok.span);
            let e = f.fold_type_expr(err.node, err.span);
            SType::new(
                TypeExpr::Fallible { ok: Box::new(o), err: Box::new(e) },
                span,
            )
        },
    }
}
