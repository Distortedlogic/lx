use crate::ast::{AstArena, TypeExpr, TypeExprId, TypeField};

use super::AstFolder;

pub fn fold_type_expr<F: AstFolder + ?Sized>(f: &mut F, id: TypeExprId, arena: &mut AstArena) -> TypeExprId {
  let span = arena.type_expr_span(id);
  let type_expr = arena.type_expr(id).clone();
  match type_expr {
    TypeExpr::Named(_) | TypeExpr::Var(_) => arena.alloc_type_expr(type_expr, span),
    TypeExpr::Applied(name, args) => {
      let folded = args.into_iter().map(|a| f.fold_type_expr(a, arena)).collect();
      arena.alloc_type_expr(TypeExpr::Applied(name, folded), span)
    },
    TypeExpr::List(inner) => {
      let folded = f.fold_type_expr(inner, arena);
      arena.alloc_type_expr(TypeExpr::List(folded), span)
    },
    TypeExpr::Map { key, value } => {
      let k = f.fold_type_expr(key, arena);
      let v = f.fold_type_expr(value, arena);
      arena.alloc_type_expr(TypeExpr::Map { key: k, value: v }, span)
    },
    TypeExpr::Record(fields) => {
      let folded = fields.into_iter().map(|field| TypeField { name: field.name, ty: f.fold_type_expr(field.ty, arena) }).collect();
      arena.alloc_type_expr(TypeExpr::Record(folded), span)
    },
    TypeExpr::Tuple(elems) => {
      let folded = elems.into_iter().map(|e| f.fold_type_expr(e, arena)).collect();
      arena.alloc_type_expr(TypeExpr::Tuple(folded), span)
    },
    TypeExpr::Func { param, ret } => {
      let p = f.fold_type_expr(param, arena);
      let r = f.fold_type_expr(ret, arena);
      arena.alloc_type_expr(TypeExpr::Func { param: p, ret: r }, span)
    },
    TypeExpr::Fallible { ok, err } => {
      let o = f.fold_type_expr(ok, arena);
      let e = f.fold_type_expr(err, arena);
      arena.alloc_type_expr(TypeExpr::Fallible { ok: o, err: e }, span)
    },
  }
}
