use crate::ast::{
  AstArena, Expr, ExprApply, ExprBinary, ExprFieldAccess, ExprId, ExprPipe, ExprUnary, FieldKind, ListElem, Literal, MapEntry, RecordField, Section, StmtId,
  StrPart,
};
use miette::SourceSpan;

use super::AstFolder;

pub fn fold_expr<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, arena: &mut AstArena) -> ExprId {
  let span = arena.expr_span(id);
  let expr = arena.expr(id).clone();
  match expr {
    Expr::Literal(lit) => {
      let folded = f.fold_literal(lit.clone(), span, arena);
      if literal_eq(&folded, &lit) {
        return id;
      }
      arena.alloc_expr(Expr::Literal(folded), span)
    },
    Expr::Ident(name) => {
      let folded = f.fold_ident(name, span, arena);
      if folded == name { id } else { arena.alloc_expr(Expr::Ident(folded), span) }
    },
    Expr::TypeConstructor(name) => {
      let folded = f.fold_type_constructor(name, span, arena);
      if folded == name { id } else { arena.alloc_expr(Expr::TypeConstructor(folded), span) }
    },
    Expr::Binary(b) => f.fold_binary(b, span, arena),
    Expr::Unary(u) => f.fold_unary(u, span, arena),
    Expr::Pipe(p) => f.fold_pipe(p, span, arena),
    Expr::Apply(a) => f.fold_apply(a, span, arena),
    Expr::Section(s) => f.fold_section(s, span, arena),
    Expr::FieldAccess(fa) => f.fold_field_access(fa, span, arena),
    Expr::Block(stmts) => f.fold_block(stmts, span, arena),
    Expr::Tuple(elems) => f.fold_tuple(elems, span, arena),
    Expr::List(elems) => f.fold_list(elems, span, arena),
    Expr::Record(fields) => f.fold_record(fields, span, arena),
    Expr::Map(entries) => f.fold_map(entries, span, arena),
    Expr::Func(func) => f.fold_func(func, span, arena),
    Expr::Match(m) => f.fold_match(m, span, arena),
    Expr::Ternary(t) => f.fold_ternary(t, span, arena),
    Expr::Propagate(inner) => f.fold_propagate(inner, span, arena),
    Expr::Coalesce(c) => f.fold_coalesce(c, span, arena),
    Expr::Slice(s) => f.fold_slice(s, span, arena),
    Expr::NamedArg(na) => f.fold_named_arg(na, span, arena),
    Expr::Loop(stmts) => f.fold_loop(stmts, span, arena),
    Expr::Break(val) => f.fold_break(val, span, arena),
    Expr::Assert(a) => f.fold_assert(a, span, arena),
    Expr::Par(stmts) => f.fold_par(stmts, span, arena),
    Expr::Sel(arms) => f.fold_sel(arms, span, arena),
    Expr::Timeout(t) => f.fold_timeout(t, span, arena),
    Expr::Emit(e) => f.fold_emit(e, span, arena),
    Expr::Yield(y) => f.fold_yield(y, span, arena),
    Expr::With(w) => f.fold_with(w, span, arena),
  }
}

fn literal_eq(a: &Literal, b: &Literal) -> bool {
  match (a, b) {
    (Literal::Int(x), Literal::Int(y)) => x == y,
    (Literal::Float(x), Literal::Float(y)) => x.to_bits() == y.to_bits(),
    (Literal::Bool(x), Literal::Bool(y)) => x == y,
    (Literal::Unit, Literal::Unit) => true,
    (Literal::RawStr(x), Literal::RawStr(y)) => x == y,
    (Literal::Str(x), Literal::Str(y)) => str_parts_eq(x, y),
    _ => false,
  }
}

fn str_parts_eq(a: &[StrPart], b: &[StrPart]) -> bool {
  a.len() == b.len()
    && a.iter().zip(b.iter()).all(|(x, y)| match (x, y) {
      (StrPart::Text(a), StrPart::Text(b)) => a == b,
      (StrPart::Interp(a), StrPart::Interp(b)) => a == b,
      _ => false,
    })
}

pub fn fold_literal<F: AstFolder + ?Sized>(f: &mut F, lit: Literal, _span: SourceSpan, arena: &mut AstArena) -> Literal {
  match lit {
    Literal::Str(parts) => {
      let folded = parts
        .into_iter()
        .map(|part| match part {
          StrPart::Text(s) => StrPart::Text(s),
          StrPart::Interp(eid) => StrPart::Interp(f.fold_expr(eid, arena)),
        })
        .collect();
      Literal::Str(folded)
    },
    other => other,
  }
}

pub fn fold_binary<F: AstFolder + ?Sized>(f: &mut F, b: ExprBinary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let left = f.fold_expr(b.left, arena);
  let right = f.fold_expr(b.right, arena);
  if left == b.left && right == b.right {
    return arena.alloc_expr(Expr::Binary(ExprBinary { op: b.op, left: b.left, right: b.right }), span);
  }
  arena.alloc_expr(Expr::Binary(ExprBinary { op: b.op, left, right }), span)
}

pub fn fold_unary<F: AstFolder + ?Sized>(f: &mut F, u: ExprUnary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let operand = f.fold_expr(u.operand, arena);
  if operand == u.operand {
    return arena.alloc_expr(Expr::Unary(ExprUnary { op: u.op, operand: u.operand }), span);
  }
  arena.alloc_expr(Expr::Unary(ExprUnary { op: u.op, operand }), span)
}

pub fn fold_pipe<F: AstFolder + ?Sized>(f: &mut F, p: ExprPipe, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let left = f.fold_expr(p.left, arena);
  let right = f.fold_expr(p.right, arena);
  if left == p.left && right == p.right {
    return arena.alloc_expr(Expr::Pipe(ExprPipe { left: p.left, right: p.right }), span);
  }
  arena.alloc_expr(Expr::Pipe(ExprPipe { left, right }), span)
}

pub fn fold_apply<F: AstFolder + ?Sized>(f: &mut F, a: ExprApply, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let func = f.fold_expr(a.func, arena);
  let arg = f.fold_expr(a.arg, arena);
  if func == a.func && arg == a.arg {
    return arena.alloc_expr(Expr::Apply(ExprApply { func: a.func, arg: a.arg }), span);
  }
  arena.alloc_expr(Expr::Apply(ExprApply { func, arg }), span)
}

pub fn fold_section<F: AstFolder + ?Sized>(f: &mut F, s: Section, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = match s {
    Section::Right { op, operand } => {
      let folded_operand = f.fold_expr(operand, arena);
      if folded_operand == operand {
        return arena.alloc_expr(Expr::Section(Section::Right { op, operand }), span);
      }
      Section::Right { op, operand: folded_operand }
    },
    Section::Left { operand, op } => {
      let folded_operand = f.fold_expr(operand, arena);
      if folded_operand == operand {
        return arena.alloc_expr(Expr::Section(Section::Left { operand, op }), span);
      }
      Section::Left { operand: folded_operand, op }
    },
    other => other,
  };
  arena.alloc_expr(Expr::Section(folded), span)
}

pub fn fold_field_access<F: AstFolder + ?Sized>(f: &mut F, fa: ExprFieldAccess, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let expr = f.fold_expr(fa.expr, arena);
  let field = match fa.field {
    FieldKind::Computed(c) => {
      let fc = f.fold_expr(c, arena);
      if expr == fa.expr && fc == c {
        return arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: fa.expr, field: FieldKind::Computed(c) }), span);
      }
      FieldKind::Computed(fc)
    },
    ref other => {
      if expr == fa.expr {
        return arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: fa.expr, field: other.clone() }), span);
      }
      fa.field
    },
  };
  arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr, field }), span)
}

pub fn fold_block<F: AstFolder + ?Sized>(f: &mut F, stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = fold_stmts(f, stmts, arena);
  arena.alloc_expr(Expr::Block(folded), span)
}

pub fn fold_tuple<F: AstFolder + ?Sized>(f: &mut F, elems: Vec<ExprId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = fold_exprs(f, elems, arena);
  arena.alloc_expr(Expr::Tuple(folded), span)
}

pub fn fold_list<F: AstFolder + ?Sized>(f: &mut F, elems: Vec<ListElem>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = elems
    .into_iter()
    .map(|elem| match elem {
      ListElem::Single(e) => ListElem::Single(f.fold_expr(e, arena)),
      ListElem::Spread(e) => ListElem::Spread(f.fold_expr(e, arena)),
    })
    .collect();
  arena.alloc_expr(Expr::List(folded), span)
}

pub fn fold_record<F: AstFolder + ?Sized>(f: &mut F, fields: Vec<RecordField>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = fields
    .into_iter()
    .map(|field| match field {
      RecordField::Named { name, value } => RecordField::Named { name, value: f.fold_expr(value, arena) },
      RecordField::Spread(value) => RecordField::Spread(f.fold_expr(value, arena)),
    })
    .collect();
  arena.alloc_expr(Expr::Record(folded), span)
}

pub fn fold_map<F: AstFolder + ?Sized>(f: &mut F, entries: Vec<MapEntry>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded = entries
    .into_iter()
    .map(|entry| match entry {
      MapEntry::Keyed { key, value } => MapEntry::Keyed { key: f.fold_expr(key, arena), value: f.fold_expr(value, arena) },
      MapEntry::Spread(value) => MapEntry::Spread(f.fold_expr(value, arena)),
    })
    .collect();
  arena.alloc_expr(Expr::Map(folded), span)
}

pub fn fold_stmts<F: AstFolder + ?Sized>(f: &mut F, stmts: Vec<StmtId>, arena: &mut AstArena) -> Vec<StmtId> {
  stmts.into_iter().map(|s| f.fold_stmt(s, arena)).collect()
}

pub fn fold_exprs<F: AstFolder + ?Sized>(f: &mut F, exprs: Vec<ExprId>, arena: &mut AstArena) -> Vec<ExprId> {
  exprs.into_iter().map(|e| f.fold_expr(e, arena)).collect()
}
