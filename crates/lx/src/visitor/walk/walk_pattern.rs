use std::ops::ControlFlow;

use crate::ast::{FieldPattern, Pattern, PatternConstructor, PatternList, PatternRecord, SPattern};
use crate::sym::Sym;
use miette::SourceSpan;

use crate::visitor::AstVisitor;

pub fn walk_pattern<V: AstVisitor + ?Sized>(v: &mut V, pattern: &Pattern, span: SourceSpan) -> ControlFlow<()> {
  match pattern {
    Pattern::Literal(lit) => v.visit_pattern_literal(lit, span)?,
    Pattern::Bind(name) => v.visit_pattern_bind(*name, span)?,
    Pattern::Wildcard => v.visit_pattern_wildcard(span)?,
    Pattern::Tuple(elems) => v.visit_pattern_tuple(elems, span)?,
    Pattern::List(PatternList { elems, rest }) => {
      v.visit_pattern_list(elems, *rest, span)?;
    },
    Pattern::Record(PatternRecord { fields, rest }) => {
      v.visit_pattern_record(fields, *rest, span)?;
    },
    Pattern::Constructor(PatternConstructor { name, args }) => {
      v.visit_pattern_constructor(*name, args, span)?;
    },
  }
  v.leave_pattern(pattern, span)
}

pub fn walk_pattern_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SPattern], span: SourceSpan) -> ControlFlow<()> {
  for e in elems {
    v.visit_pattern(&e.node, e.span)?;
  }
  v.leave_pattern_tuple(elems, span)
}

pub fn walk_pattern_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SPattern], rest: Option<Sym>, span: SourceSpan) -> ControlFlow<()> {
  for e in elems {
    v.visit_pattern(&e.node, e.span)?;
  }
  v.leave_pattern_list(elems, rest, span)
}

pub fn walk_pattern_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[FieldPattern], rest: Option<Sym>, span: SourceSpan) -> ControlFlow<()> {
  for f in fields {
    if let Some(ref p) = f.pattern {
      v.visit_pattern(&p.node, p.span)?;
    }
  }
  v.leave_pattern_record(fields, rest, span)
}

pub fn walk_pattern_constructor<V: AstVisitor + ?Sized>(v: &mut V, name: Sym, args: &[SPattern], span: SourceSpan) -> ControlFlow<()> {
  for a in args {
    v.visit_pattern(&a.node, a.span)?;
  }
  v.leave_pattern_constructor(name, args, span)
}
