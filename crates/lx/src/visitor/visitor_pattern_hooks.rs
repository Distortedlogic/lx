use crate::ast::{FieldPattern, Literal, Pattern, PatternId};
use crate::sym::Sym;
use miette::SourceSpan;

use super::VisitAction;

pub trait PatternVisitor {
  fn visit_pattern(&mut self, _id: PatternId, _pattern: &Pattern, _span: SourceSpan) -> VisitAction {
    VisitAction::Descend
  }
  fn leave_pattern(&mut self, _id: PatternId, _pattern: &Pattern, _span: SourceSpan) {}
  fn visit_pattern_literal(&mut self, _id: PatternId, _lit: &Literal, _span: SourceSpan) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_bind(&mut self, _id: PatternId, _name: Sym, _span: SourceSpan) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_wildcard(&mut self, _id: PatternId, _span: SourceSpan) -> VisitAction {
    VisitAction::Descend
  }
  fn visit_pattern_tuple(&mut self, _id: PatternId, _elems: &[PatternId], _span: SourceSpan) -> VisitAction {
    VisitAction::Descend
  }
  fn leave_pattern_tuple(&mut self, _id: PatternId, _elems: &[PatternId], _span: SourceSpan) {}
  fn visit_pattern_list(&mut self, _id: PatternId, _elems: &[PatternId], _rest: Option<Sym>, _span: SourceSpan) -> VisitAction {
    VisitAction::Descend
  }
  fn leave_pattern_list(&mut self, _id: PatternId, _elems: &[PatternId], _rest: Option<Sym>, _span: SourceSpan) {}
  fn visit_pattern_record(&mut self, _id: PatternId, _fields: &[FieldPattern], _rest: Option<Sym>, _span: SourceSpan) -> VisitAction {
    VisitAction::Descend
  }
  fn leave_pattern_record(&mut self, _id: PatternId, _fields: &[FieldPattern], _rest: Option<Sym>, _span: SourceSpan) {}
  fn visit_pattern_constructor(&mut self, _id: PatternId, _name: Sym, _args: &[PatternId], _span: SourceSpan) -> VisitAction {
    VisitAction::Descend
  }
  fn leave_pattern_constructor(&mut self, _id: PatternId, _name: Sym, _args: &[PatternId], _span: SourceSpan) {}
}
