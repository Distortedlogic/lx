use std::ops::ControlFlow;

use crate::ast::{AstArena, FieldPattern, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord};
use crate::sym::Sym;
use miette::SourceSpan;

use crate::visitor::{AstVisitor, VisitAction};

pub(crate) fn walk_pattern_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.pattern_span(id);
  let pattern = arena.pattern(id);
  let action = v.visit_pattern(id, pattern, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern(id, pattern, span, arena),
    VisitAction::Descend => walk_pattern(v, id, pattern, span, arena),
  }
}

pub fn walk_pattern<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, pattern: &Pattern, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match pattern {
    Pattern::Literal(lit) => {
      let action = v.visit_pattern_literal(id, lit, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Pattern::Bind(name) => {
      let action = v.visit_pattern_bind(id, *name, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Pattern::Wildcard => {
      let action = v.visit_pattern_wildcard(id, span, arena);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Pattern::Tuple(elems) => walk_pattern_tuple_dispatch(v, id, elems, span, arena)?,
    Pattern::List(PatternList { elems, rest }) => {
      walk_pattern_list_dispatch(v, id, elems, *rest, span, arena)?;
    },
    Pattern::Record(PatternRecord { fields, rest }) => {
      walk_pattern_record_dispatch(v, id, fields, *rest, span, arena)?;
    },
    Pattern::Constructor(PatternConstructor { name, args }) => {
      walk_pattern_constructor_dispatch(v, id, *name, args, span, arena)?;
    },
  }
  v.leave_pattern(id, pattern, span, arena)
}

fn walk_pattern_tuple_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, elems: &[PatternId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_pattern_tuple(id, elems, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern_tuple(id, elems, span, arena),
    VisitAction::Descend => walk_pattern_tuple(v, id, elems, span, arena),
  }
}

pub fn walk_pattern_tuple<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, elems: &[PatternId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &e in elems {
    walk_pattern_dispatch(v, e, arena)?;
  }
  v.leave_pattern_tuple(id, elems, span, arena)
}

fn walk_pattern_list_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
  elems: &[PatternId],
  rest: Option<Sym>,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_pattern_list(id, elems, rest, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern_list(id, elems, rest, span, arena),
    VisitAction::Descend => walk_pattern_list(v, id, elems, rest, span, arena),
  }
}

pub fn walk_pattern_list<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
  elems: &[PatternId],
  rest: Option<Sym>,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  for &e in elems {
    walk_pattern_dispatch(v, e, arena)?;
  }
  v.leave_pattern_list(id, elems, rest, span, arena)
}

fn walk_pattern_record_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
  fields: &[FieldPattern],
  rest: Option<Sym>,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_pattern_record(id, fields, rest, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern_record(id, fields, rest, span, arena),
    VisitAction::Descend => walk_pattern_record(v, id, fields, rest, span, arena),
  }
}

pub fn walk_pattern_record<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
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
  v.leave_pattern_record(id, fields, rest, span, arena)
}

fn walk_pattern_constructor_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
  name: Sym,
  args: &[PatternId],
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_pattern_constructor(id, name, args, span, arena);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => v.leave_pattern_constructor(id, name, args, span, arena),
    VisitAction::Descend => walk_pattern_constructor(v, id, name, args, span, arena),
  }
}

pub fn walk_pattern_constructor<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
  name: Sym,
  args: &[PatternId],
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  for &a in args {
    walk_pattern_dispatch(v, a, arena)?;
  }
  v.leave_pattern_constructor(id, name, args, span, arena)
}
