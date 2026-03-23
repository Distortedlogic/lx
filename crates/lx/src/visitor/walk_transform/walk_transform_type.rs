use crate::ast::{AstArena, TypeExpr, TypeExprId, TypeField};

use crate::visitor::transformer::{AstTransformer, TransformOp};

pub fn walk_transform_type_expr<T: AstTransformer + ?Sized>(t: &mut T, id: TypeExprId, arena: &mut AstArena) -> TypeExprId {
  let span = arena.type_expr_span(id);
  let original = arena.type_expr(id).clone();

  match t.transform_type_expr(id, original.clone(), span, arena) {
    TransformOp::Stop => id,
    TransformOp::Skip(node) => {
      let final_node = t.leave_type_expr(id, node, span, arena);
      if final_node == original {
        return id;
      }
      arena.alloc_type_expr(final_node, span)
    },
    TransformOp::Continue(node) => {
      let recursed = recurse_type_children(t, node, arena);
      let final_node = t.leave_type_expr(id, recursed, span, arena);
      if final_node == original {
        return id;
      }
      arena.alloc_type_expr(final_node, span)
    },
  }
}

fn recurse_type_children<T: AstTransformer + ?Sized>(t: &mut T, te: TypeExpr, arena: &mut AstArena) -> TypeExpr {
  match te {
    TypeExpr::Applied(name, args) => {
      let folded: Vec<_> = args.into_iter().map(|a| walk_transform_type_expr(t, a, arena)).collect();
      TypeExpr::Applied(name, folded)
    },
    TypeExpr::List(inner) => TypeExpr::List(walk_transform_type_expr(t, inner, arena)),
    TypeExpr::Map { key, value } => {
      let k = walk_transform_type_expr(t, key, arena);
      let v = walk_transform_type_expr(t, value, arena);
      TypeExpr::Map { key: k, value: v }
    },
    TypeExpr::Record(fields) => {
      let folded: Vec<_> = fields.into_iter().map(|f| TypeField { name: f.name, ty: walk_transform_type_expr(t, f.ty, arena) }).collect();
      TypeExpr::Record(folded)
    },
    TypeExpr::Tuple(elems) => {
      let folded: Vec<_> = elems.into_iter().map(|e| walk_transform_type_expr(t, e, arena)).collect();
      TypeExpr::Tuple(folded)
    },
    TypeExpr::Func { param, ret } => {
      let p = walk_transform_type_expr(t, param, arena);
      let r = walk_transform_type_expr(t, ret, arena);
      TypeExpr::Func { param: p, ret: r }
    },
    TypeExpr::Fallible { ok, err } => {
      let o = walk_transform_type_expr(t, ok, arena);
      let e = walk_transform_type_expr(t, err, arena);
      TypeExpr::Fallible { ok: o, err: e }
    },
    other => other,
  }
}
