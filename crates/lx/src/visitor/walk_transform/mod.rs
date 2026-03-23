use crate::ast::{AstArena, Program, StmtId};

use super::transformer::{AstTransformer, TransformOp};

macro_rules! walk_transform_fn {
  ($fn_name:ident, $id_ty:ty, $transform:ident, $leave:ident, $get_span:ident, $get_node:ident, $alloc:ident) => {
    pub fn $fn_name<T: AstTransformer + ?Sized>(t: &mut T, id: $id_ty, arena: &mut AstArena) -> $id_ty {
      let span = arena.$get_span(id);
      let original = arena.$get_node(id).clone();
      match t.$transform(id, original.clone(), span, arena) {
        TransformOp::Stop => id,
        TransformOp::Skip(node) => {
          let final_node = t.$leave(id, node, span, arena);
          if final_node == original {
            return id;
          }
          arena.$alloc(final_node, span)
        },
        TransformOp::Continue(node) => {
          let recursed = node.recurse_children(t, arena);
          let final_node = t.$leave(id, recursed, span, arena);
          if final_node == original {
            return id;
          }
          arena.$alloc(final_node, span)
        },
      }
    }
  };
}

walk_transform_fn!(walk_transform_stmt, StmtId, transform_stmt, leave_stmt, stmt_span, stmt, alloc_stmt);
walk_transform_fn!(walk_transform_expr, crate::ast::ExprId, transform_expr, leave_expr, expr_span, expr, alloc_expr);
walk_transform_fn!(walk_transform_pattern, crate::ast::PatternId, transform_pattern, leave_pattern, pattern_span, pattern, alloc_pattern);
walk_transform_fn!(walk_transform_type_expr, crate::ast::TypeExprId, transform_type_expr, leave_type_expr, type_expr_span, type_expr, alloc_type_expr);

pub fn walk_transform_program<T: AstTransformer + ?Sized, P>(t: &mut T, mut program: Program<P>) -> Program<P> {
  let stmts: Vec<StmtId> = program.stmts.clone();
  let folded: Vec<StmtId> = stmts.into_iter().map(|s| walk_transform_stmt(t, s, &mut program.arena)).collect();
  program.stmts = folded;
  program
}
