use crate::ast::{FieldPattern, Pattern, SPattern};
use miette::SourceSpan;

use crate::visitor::AstVisitor;

pub fn walk_pattern<V: AstVisitor + ?Sized>(v: &mut V, pattern: &Pattern, span: SourceSpan) {
  match pattern {
    Pattern::Literal(lit) => v.visit_pattern_literal(lit, span),
    Pattern::Bind(name) => v.visit_pattern_bind(name, span),
    Pattern::Wildcard => v.visit_pattern_wildcard(span),
    Pattern::Tuple(elems) => v.visit_pattern_tuple(elems, span),
    Pattern::List { elems, rest } => {
      v.visit_pattern_list(elems, rest.as_deref(), span);
    },
    Pattern::Record { fields, rest } => {
      v.visit_pattern_record(fields, rest.as_deref(), span);
    },
    Pattern::Constructor { name, args } => {
      v.visit_pattern_constructor(name, args, span);
    },
  }
}

pub fn walk_pattern_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SPattern], _span: SourceSpan) {
  for e in elems {
    v.visit_pattern(&e.node, e.span);
  }
}

pub fn walk_pattern_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SPattern], _span: SourceSpan) {
  for e in elems {
    v.visit_pattern(&e.node, e.span);
  }
}

pub fn walk_pattern_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[FieldPattern], _span: SourceSpan) {
  for f in fields {
    if let Some(ref p) = f.pattern {
      v.visit_pattern(&p.node, p.span);
    }
  }
}

pub fn walk_pattern_constructor<V: AstVisitor + ?Sized>(v: &mut V, args: &[SPattern], _span: SourceSpan) {
  for a in args {
    v.visit_pattern(&a.node, a.span);
  }
}
