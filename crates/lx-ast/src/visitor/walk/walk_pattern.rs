use std::ops::ControlFlow;

use crate::ast::{AstArena, FieldPattern, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord};
use lx_span::sym::Sym;
use miette::SourceSpan;

use crate::visitor::{AstVisitor, VisitAction};

pub fn walk_pattern_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, arena: &AstArena) -> ControlFlow<()> {
  let span = arena.pattern_span(id);
  let pattern = arena.pattern(id);
  let action = v.visit_pattern(id, pattern, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_pattern(id, pattern, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_pattern(v, id, pattern, span, arena)?;
      let pattern = arena.pattern(id);
      let span = arena.pattern_span(id);
      v.leave_pattern(id, pattern, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_pattern<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, pattern: &Pattern, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  match pattern {
    Pattern::Literal(lit) => {
      let action = v.visit_pattern_literal(id, lit, span);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Pattern::Bind(name) => {
      let action = v.visit_pattern_bind(id, *name, span);
      if action.is_stop() {
        return ControlFlow::Break(());
      }
    },
    Pattern::Wildcard => {
      let action = v.visit_pattern_wildcard(id, span);
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
  ControlFlow::Continue(())
}

fn walk_pattern_tuple_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, elems: &[PatternId], span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  let action = v.visit_pattern_tuple(id, elems, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_pattern_tuple(id, elems, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_pattern_tuple(v, id, elems, span, arena)?;
      v.leave_pattern_tuple(id, elems, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_pattern_tuple<V: AstVisitor + ?Sized>(v: &mut V, _id: PatternId, elems: &[PatternId], _span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
  for &e in elems {
    walk_pattern_dispatch(v, e, arena)?;
  }
  ControlFlow::Continue(())
}

fn walk_pattern_list_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
  elems: &[PatternId],
  rest: Option<Sym>,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_pattern_list(id, elems, rest, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_pattern_list(id, elems, rest, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_pattern_list(v, id, elems, rest, span, arena)?;
      v.leave_pattern_list(id, elems, rest, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_pattern_list<V: AstVisitor + ?Sized>(
  v: &mut V,
  _id: PatternId,
  elems: &[PatternId],
  _rest: Option<Sym>,
  _span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  for &e in elems {
    walk_pattern_dispatch(v, e, arena)?;
  }
  ControlFlow::Continue(())
}

fn walk_pattern_record_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
  fields: &[FieldPattern],
  rest: Option<Sym>,
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_pattern_record(id, fields, rest, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_pattern_record(id, fields, rest, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_pattern_record(v, id, fields, rest, span, arena)?;
      v.leave_pattern_record(id, fields, rest, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_pattern_record<V: AstVisitor + ?Sized>(
  v: &mut V,
  _id: PatternId,
  fields: &[FieldPattern],
  _rest: Option<Sym>,
  _span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  for field in fields {
    field.walk_children(v, arena)?;
  }
  ControlFlow::Continue(())
}

fn walk_pattern_constructor_dispatch<V: AstVisitor + ?Sized>(
  v: &mut V,
  id: PatternId,
  name: Sym,
  args: &[PatternId],
  span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  let action = v.visit_pattern_constructor(id, name, args, span);
  match action {
    VisitAction::Stop => ControlFlow::Break(()),
    VisitAction::Skip => {
      v.leave_pattern_constructor(id, name, args, span);
      ControlFlow::Continue(())
    },
    VisitAction::Descend => {
      walk_pattern_constructor(v, id, name, args, span, arena)?;
      v.leave_pattern_constructor(id, name, args, span);
      ControlFlow::Continue(())
    },
  }
}

pub fn walk_pattern_constructor<V: AstVisitor + ?Sized>(
  v: &mut V,
  _id: PatternId,
  _name: Sym,
  args: &[PatternId],
  _span: SourceSpan,
  arena: &AstArena,
) -> ControlFlow<()> {
  for &a in args {
    walk_pattern_dispatch(v, a, arena)?;
  }
  ControlFlow::Continue(())
}
