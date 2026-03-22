use std::ops::ControlFlow;

use crate::ast::{AstArena, FieldPattern, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord};
use crate::sym::Sym;
use miette::SourceSpan;

use crate::visitor::{AstVisitor, VisitAction};

pub(crate) fn walk_pattern_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.pattern_span(id);
  let pattern = arena.pattern(id);
  let action = v.visit_pattern(pattern, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern(pattern, span, arena),
    VisitAction::Descend => walk_pattern(v, pattern, span, arena),
  }
}

pub fn walk_pattern<V: AstVisitor + ?Sized>(v: &mut V, pattern: &Pattern, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match pattern {
    Pattern::Literal(lit) => {
      let action = v.visit_pattern_literal(lit, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Pattern::Bind(name) => {
      let action = v.visit_pattern_bind(*name, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Pattern::Wildcard => {
      let action = v.visit_pattern_wildcard(span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Pattern::Tuple(elems) => walk_pattern_tuple_dispatch(v, elems, span, arena)?,
    Pattern::List(PatternList { elems, rest }) => {
      walk_pattern_list_dispatch(v, elems, *rest, span, arena)?;
    },
    Pattern::Record(PatternRecord { fields, rest }) => {
      walk_pattern_record_dispatch(v, fields, *rest, span, arena)?;
    },
    Pattern::Constructor(PatternConstructor { name, args }) => {
      walk_pattern_constructor_dispatch(v, *name, args, span, arena)?;
    },
  }
  v.leave_pattern(pattern, span, arena)
}

fn walk_pattern_tuple_dispatch<V: AstVisitor + ?Sized>(v: &mut V, elems: &[PatternId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_pattern_tuple(elems, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern_tuple(elems, span, arena),
    VisitAction::Descend => walk_pattern_tuple(v, elems, span, arena),
  }
}

pub fn walk_pattern_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[PatternId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &e in elems {
    walk_pattern_dispatch(v, e, arena)?;
  }
  v.leave_pattern_tuple(elems, span, arena)
}

fn walk_pattern_list_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  elems: &[PatternId],
  rest: Option<Sym>,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_pattern_list(elems, rest, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern_list(elems, rest, span, arena),
    VisitAction::Descend => walk_pattern_list(v, elems, rest, span, arena),
  }
}

pub fn walk_pattern_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[PatternId], rest: Option<Sym>, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &e in elems {
    walk_pattern_dispatch(v, e, arena)?;
  }
  v.leave_pattern_list(elems, rest, span, arena)
}

fn walk_pattern_record_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  fields: &[FieldPattern],
  rest: Option<Sym>,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_pattern_record(fields, rest, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern_record(fields, rest, span, arena),
    VisitAction::Descend => walk_pattern_record(v, fields, rest, span, arena),
  }
}

pub fn walk_pattern_record<V: AstVisitor + ?Sized>(
  v: &mut V,
  fields: &[FieldPattern],
  rest: Option<Sym>,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  for f in fields {
    if let Some(pid) = f.pattern {
      walk_pattern_dispatch(v, pid, arena)?;
    }
  }
  v.leave_pattern_record(fields, rest, span, arena)
}

fn walk_pattern_constructor_dispatch<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, args: &[PatternId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_pattern_constructor(name, args, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern_constructor(name, args, span, arena),
    VisitAction::Descend => walk_pattern_constructor(v, name, args, span, arena),
  }
}

pub fn walk_pattern_constructor<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, args: &[PatternId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &a in args {
    walk_pattern_dispatch(v, a, arena)?;
  }
  v.leave_pattern_constructor(name, args, span, arena)
}
