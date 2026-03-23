use crate::ast::{AstArena, TypeExpr, TypeExprId, TypeField};

use super::AstFolder;

pub fn fold_type_expr<F: AstFolder + ?Sized>(f: &mut F, id: TypeExprId, arena: &mut AstArena) -> TypeExprId {
  let span = arena.type_expr_span(id);
  let type_expr = arena.type_expr(id).clone();
  match type_expr {
    TypeExpr::Named(_) | TypeExpr::Var(_) => id,
    TypeExpr::Applied(name, args) => {
      let folded: Vec<_> = args.iter().map(|a| f.fold_type_expr(*a, arena)).collect();
      if folded == args {
        return id;
      }
      arena.alloc_type_expr(TypeExpr::Applied(name, folded), span)
    },
    TypeExpr::List(inner) => {
      let folded = f.fold_type_expr(inner, arena);
      if folded == inner {
        return id;
      }
      arena.alloc_type_expr(TypeExpr::List(folded), span)
    },
    TypeExpr::Map { key, value } => {
      let k = f.fold_type_expr(key, arena);
      let v = f.fold_type_expr(value, arena);
      if k == key && v == value {
        return id;
      }
      arena.alloc_type_expr(TypeExpr::Map { key: k, value: v }, span)
    },
    TypeExpr::Record(fields) => {
      let folded: Vec<_> = fields.iter().map(|field| TypeField { name: field.name, ty: f.fold_type_expr(field.ty, arena) }).collect();
      let changed = folded.iter().zip(fields.iter()).any(|(a, b)| a.ty != b.ty);
      if !changed {
        return id;
      }
      arena.alloc_type_expr(TypeExpr::Record(folded), span)
    },
    TypeExpr::Tuple(elems) => {
      let folded: Vec<_> = elems.iter().map(|e| f.fold_type_expr(*e, arena)).collect();
      if folded == elems {
        return id;
      }
      arena.alloc_type_expr(TypeExpr::Tuple(folded), span)
    },
    TypeExpr::Func { param, ret } => {
      let p = f.fold_type_expr(param, arena);
      let r = f.fold_type_expr(ret, arena);
      if p == param && r == ret {
        return id;
      }
      arena.alloc_type_expr(TypeExpr::Func { param: p, ret: r }, span)
    },
    TypeExpr::Fallible { ok, err } => {
      let o = f.fold_type_expr(ok, arena);
      let e = f.fold_type_expr(err, arena);
      if o == ok && e == err {
        return id;
      }
      arena.alloc_type_expr(TypeExpr::Fallible { ok: o, err: e }, span)
    },
  }
}
