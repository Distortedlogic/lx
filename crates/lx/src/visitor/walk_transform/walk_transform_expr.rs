use crate::ast::{
  AstArena, Expr, ExprApply, ExprBinary, ExprFieldAccess, ExprFunc, ExprId, ExprMatch, ExprPipe, ExprUnary, FieldKind, ListElem, Literal, MapEntry, MatchArm,
  Param, RecordField, Section, StrPart,
};

use super::walk_transform_pattern::walk_transform_pattern;
use super::walk_transform_type::walk_transform_type_expr;
use super::{walk_transform_expr2, walk_transform_stmt};
use crate::visitor::transformer::{AstTransformer, TransformOp};

pub fn walk_transform_expr<T: AstTransformer + ?Sized>(t: &mut T, id: ExprId, arena: &mut AstArena) -> ExprId {
  let span = arena.expr_span(id);
  let original = arena.expr(id).clone();

  match t.transform_expr(id, original.clone(), span, arena) {
    TransformOp::Stop => id,
    TransformOp::Skip(node) => {
      let final_node = t.leave_expr(id, node, span, arena);
      if final_node == original {
        return id;
      }
      arena.alloc_expr(final_node, span)
    },
    TransformOp::Continue(node) => {
      let recursed = recurse_expr_children(t, node, arena);
      let final_node = t.leave_expr(id, recursed, span, arena);
      if final_node == original {
        return id;
      }
      arena.alloc_expr(final_node, span)
    },
  }
}

pub(super) fn recurse_expr_children<T: AstTransformer + ?Sized>(t: &mut T, expr: Expr, arena: &mut AstArena) -> Expr {
  match expr {
    Expr::Literal(lit) => Expr::Literal(recurse_literal(t, lit, arena)),
    Expr::Binary(b) => {
      let left = walk_transform_expr(t, b.left, arena);
      let right = walk_transform_expr(t, b.right, arena);
      Expr::Binary(ExprBinary { op: b.op, left, right })
    },
    Expr::Unary(u) => {
      let operand = walk_transform_expr(t, u.operand, arena);
      Expr::Unary(ExprUnary { op: u.op, operand })
    },
    Expr::Pipe(p) => {
      let left = walk_transform_expr(t, p.left, arena);
      let right = walk_transform_expr(t, p.right, arena);
      Expr::Pipe(ExprPipe { left, right })
    },
    Expr::Apply(a) => {
      let func = walk_transform_expr(t, a.func, arena);
      let arg = walk_transform_expr(t, a.arg, arena);
      Expr::Apply(ExprApply { func, arg })
    },
    Expr::Section(s) => Expr::Section(recurse_section(t, s, arena)),
    Expr::FieldAccess(fa) => {
      let expr = walk_transform_expr(t, fa.expr, arena);
      let field = match fa.field {
        FieldKind::Computed(c) => FieldKind::Computed(walk_transform_expr(t, c, arena)),
        other => other,
      };
      Expr::FieldAccess(ExprFieldAccess { expr, field })
    },
    Expr::Block(stmts) => Expr::Block(stmts.into_iter().map(|s| walk_transform_stmt(t, s, arena)).collect()),
    Expr::Tuple(elems) => Expr::Tuple(elems.into_iter().map(|e| walk_transform_expr(t, e, arena)).collect()),
    Expr::List(elems) => Expr::List(recurse_list(t, elems, arena)),
    Expr::Record(fields) => Expr::Record(recurse_record(t, fields, arena)),
    Expr::Map(entries) => Expr::Map(recurse_map(t, entries, arena)),
    Expr::Func(func) => Expr::Func(recurse_func(t, func, arena)),
    Expr::Match(m) => Expr::Match(recurse_match(t, m, arena)),
    expr => walk_transform_expr2::recurse_expr_children2(t, expr, arena),
  }
}

fn recurse_literal<T: AstTransformer + ?Sized>(t: &mut T, lit: Literal, arena: &mut AstArena) -> Literal {
  match lit {
    Literal::Str(parts) => {
      let folded = parts
        .into_iter()
        .map(|part| match part {
          part @ StrPart::Text(_) => part,
          StrPart::Interp(eid) => StrPart::Interp(walk_transform_expr(t, eid, arena)),
        })
        .collect();
      Literal::Str(folded)
    },
    other => other,
  }
}

fn recurse_section<T: AstTransformer + ?Sized>(t: &mut T, s: Section, arena: &mut AstArena) -> Section {
  match s {
    Section::Right { op, operand } => Section::Right { op, operand: walk_transform_expr(t, operand, arena) },
    Section::Left { operand, op } => Section::Left { operand: walk_transform_expr(t, operand, arena), op },
    other => other,
  }
}

fn recurse_list<T: AstTransformer + ?Sized>(t: &mut T, elems: Vec<ListElem>, arena: &mut AstArena) -> Vec<ListElem> {
  elems
    .into_iter()
    .map(|elem| match elem {
      ListElem::Single(e) => ListElem::Single(walk_transform_expr(t, e, arena)),
      ListElem::Spread(e) => ListElem::Spread(walk_transform_expr(t, e, arena)),
    })
    .collect()
}

fn recurse_record<T: AstTransformer + ?Sized>(t: &mut T, fields: Vec<RecordField>, arena: &mut AstArena) -> Vec<RecordField> {
  fields
    .into_iter()
    .map(|f| match f {
      RecordField::Named { name, value } => RecordField::Named { name, value: walk_transform_expr(t, value, arena) },
      RecordField::Spread(v) => RecordField::Spread(walk_transform_expr(t, v, arena)),
    })
    .collect()
}

fn recurse_map<T: AstTransformer + ?Sized>(t: &mut T, entries: Vec<MapEntry>, arena: &mut AstArena) -> Vec<MapEntry> {
  entries
    .into_iter()
    .map(|entry| match entry {
      MapEntry::Keyed { key, value } => MapEntry::Keyed { key: walk_transform_expr(t, key, arena), value: walk_transform_expr(t, value, arena) },
      MapEntry::Spread(v) => MapEntry::Spread(walk_transform_expr(t, v, arena)),
    })
    .collect()
}

fn recurse_func<T: AstTransformer + ?Sized>(t: &mut T, func: ExprFunc, arena: &mut AstArena) -> ExprFunc {
  let params: Vec<Param> = func
    .params
    .into_iter()
    .map(|p| Param {
      name: p.name,
      type_ann: p.type_ann.map(|te| walk_transform_type_expr(t, te, arena)),
      default: p.default.map(|d| walk_transform_expr(t, d, arena)),
    })
    .collect();
  let ret_type = func.ret_type.map(|te| walk_transform_type_expr(t, te, arena));
  let guard = func.guard.map(|g| walk_transform_expr(t, g, arena));
  let body = walk_transform_expr(t, func.body, arena);
  ExprFunc { params, ret_type, guard, body }
}

fn recurse_match<T: AstTransformer + ?Sized>(t: &mut T, m: ExprMatch, arena: &mut AstArena) -> ExprMatch {
  let scrutinee = walk_transform_expr(t, m.scrutinee, arena);
  let arms: Vec<MatchArm> = m
    .arms
    .into_iter()
    .map(|arm| MatchArm {
      pattern: walk_transform_pattern(t, arm.pattern, arena),
      guard: arm.guard.map(|g| walk_transform_expr(t, g, arena)),
      body: walk_transform_expr(t, arm.body, arena),
    })
    .collect();
  ExprMatch { scrutinee, arms }
}
