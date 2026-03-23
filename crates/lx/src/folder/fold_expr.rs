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
      if folded == lit {
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
    Expr::Binary(b) => f.fold_binary(id, b, span, arena),
    Expr::Unary(u) => f.fold_unary(id, u, span, arena),
    Expr::Pipe(p) => f.fold_pipe(id, p, span, arena),
    Expr::Apply(a) => f.fold_apply(id, a, span, arena),
    Expr::Section(s) => f.fold_section(id, s, span, arena),
    Expr::FieldAccess(fa) => f.fold_field_access(id, fa, span, arena),
    Expr::Block(ref stmts) => f.fold_block(id, stmts, span, arena),
    Expr::Tuple(ref elems) => f.fold_tuple(id, elems, span, arena),
    Expr::List(elems) => f.fold_list(id, elems, span, arena),
    Expr::Record(fields) => f.fold_record(id, fields, span, arena),
    Expr::Map(entries) => f.fold_map(id, entries, span, arena),
    Expr::Func(func) => f.fold_func(id, func, span, arena),
    Expr::Match(m) => f.fold_match(id, m, span, arena),
    Expr::Ternary(t) => f.fold_ternary(id, t, span, arena),
    Expr::Propagate(inner) => f.fold_propagate(id, inner, span, arena),
    Expr::Coalesce(c) => f.fold_coalesce(id, c, span, arena),
    Expr::Slice(s) => f.fold_slice(id, s, span, arena),
    Expr::NamedArg(na) => f.fold_named_arg(id, na, span, arena),
    Expr::Loop(stmts) => f.fold_loop(id, stmts, span, arena),
    Expr::Break(val) => f.fold_break(id, val, span, arena),
    Expr::Assert(a) => f.fold_assert(id, a, span, arena),
    Expr::Par(stmts) => f.fold_par(id, stmts, span, arena),
    Expr::Sel(arms) => f.fold_sel(id, arms, span, arena),
    Expr::Timeout(t) => f.fold_timeout(id, t, span, arena),
    Expr::Emit(e) => f.fold_emit(id, e, span, arena),
    Expr::Yield(y) => f.fold_yield(id, y, span, arena),
    Expr::With(w) => f.fold_with(id, w, span, arena),
  }
}

pub fn fold_literal<F: AstFolder + ?Sized>(f: &mut F, lit: Literal, _span: SourceSpan, arena: &mut AstArena) -> Literal {
  match lit {
    Literal::Str(parts) => {
      let folded = parts
        .into_iter()
        .map(|part| match part {
          part @ StrPart::Text(_) => part,
          StrPart::Interp(eid) => StrPart::Interp(f.fold_expr(eid, arena)),
        })
        .collect();
      Literal::Str(folded)
    },
    other => other,
  }
}

pub fn fold_binary<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, b: ExprBinary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let left = f.fold_expr(b.left, arena);
  let right = f.fold_expr(b.right, arena);
  if left == b.left && right == b.right {
    return id;
  }
  arena.alloc_expr(Expr::Binary(ExprBinary { op: b.op, left, right }), span)
}

pub fn fold_unary<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, u: ExprUnary, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let operand = f.fold_expr(u.operand, arena);
  if operand == u.operand {
    return id;
  }
  arena.alloc_expr(Expr::Unary(ExprUnary { op: u.op, operand }), span)
}

pub fn fold_pipe<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, p: ExprPipe, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let left = f.fold_expr(p.left, arena);
  let right = f.fold_expr(p.right, arena);
  if left == p.left && right == p.right {
    return id;
  }
  arena.alloc_expr(Expr::Pipe(ExprPipe { left, right }), span)
}

pub fn fold_apply<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, a: ExprApply, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let func = f.fold_expr(a.func, arena);
  let arg = f.fold_expr(a.arg, arena);
  if func == a.func && arg == a.arg {
    return id;
  }
  arena.alloc_expr(Expr::Apply(ExprApply { func, arg }), span)
}

pub fn fold_section<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, s: Section, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  match s {
    Section::Right { op, operand } => {
      let folded_operand = f.fold_expr(operand, arena);
      if folded_operand == operand {
        return id;
      }
      arena.alloc_expr(Expr::Section(Section::Right { op, operand: folded_operand }), span)
    },
    Section::Left { operand, op } => {
      let folded_operand = f.fold_expr(operand, arena);
      if folded_operand == operand {
        return id;
      }
      arena.alloc_expr(Expr::Section(Section::Left { operand: folded_operand, op }), span)
    },
    Section::Field(_) | Section::Index(_) | Section::BinOp(_) => id,
  }
}

pub fn fold_field_access<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, fa: ExprFieldAccess, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let expr = f.fold_expr(fa.expr, arena);
  let (field, field_changed) = match fa.field {
    FieldKind::Computed(c) => {
      let folded = f.fold_expr(c, arena);
      (FieldKind::Computed(folded), folded != c)
    },
    other => (other, false),
  };
  if expr == fa.expr && !field_changed {
    return id;
  }
  arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr, field }), span)
}

pub fn fold_block<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, stmts: &[StmtId], span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded: Vec<_> = stmts.iter().map(|s| f.fold_stmt(*s, arena)).collect();
  if folded.as_slice() == stmts {
    return id;
  }
  arena.alloc_expr(Expr::Block(folded), span)
}

pub fn fold_tuple<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, elems: &[ExprId], span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded: Vec<_> = elems.iter().map(|e| f.fold_expr(*e, arena)).collect();
  if folded.as_slice() == elems {
    return id;
  }
  arena.alloc_expr(Expr::Tuple(folded), span)
}

pub fn fold_list<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, elems: Vec<ListElem>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded: Vec<ListElem> = elems
    .iter()
    .map(|elem| match elem {
      ListElem::Single(e) => ListElem::Single(f.fold_expr(*e, arena)),
      ListElem::Spread(e) => ListElem::Spread(f.fold_expr(*e, arena)),
    })
    .collect();
  let changed = folded.iter().zip(elems.iter()).any(|(a, b)| match (a, b) {
    (ListElem::Single(a), ListElem::Single(b)) | (ListElem::Spread(a), ListElem::Spread(b)) => a != b,
    _ => true,
  });
  if !changed {
    return id;
  }
  arena.alloc_expr(Expr::List(folded), span)
}

pub fn fold_record<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, fields: Vec<RecordField>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded: Vec<RecordField> = fields
    .iter()
    .map(|field| match field {
      RecordField::Named { name, value } => RecordField::Named { name: *name, value: f.fold_expr(*value, arena) },
      RecordField::Spread(value) => RecordField::Spread(f.fold_expr(*value, arena)),
    })
    .collect();
  let changed = folded.iter().zip(fields.iter()).any(|(a, b)| match (a, b) {
    (RecordField::Named { value: av, .. }, RecordField::Named { value: bv, .. }) | (RecordField::Spread(av), RecordField::Spread(bv)) => av != bv,
    _ => true,
  });
  if !changed {
    return id;
  }
  arena.alloc_expr(Expr::Record(folded), span)
}

pub fn fold_map<F: AstFolder + ?Sized>(f: &mut F, id: ExprId, entries: Vec<MapEntry>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let folded: Vec<MapEntry> = entries
    .iter()
    .map(|entry| match entry {
      MapEntry::Keyed { key, value } => MapEntry::Keyed { key: f.fold_expr(*key, arena), value: f.fold_expr(*value, arena) },
      MapEntry::Spread(value) => MapEntry::Spread(f.fold_expr(*value, arena)),
    })
    .collect();
  let changed = folded.iter().zip(entries.iter()).any(|(a, b)| match (a, b) {
    (MapEntry::Keyed { key: ak, value: av }, MapEntry::Keyed { key: bk, value: bv }) => ak != bk || av != bv,
    (MapEntry::Spread(a), MapEntry::Spread(b)) => a != b,
    _ => true,
  });
  if !changed {
    return id;
  }
  arena.alloc_expr(Expr::Map(folded), span)
}
