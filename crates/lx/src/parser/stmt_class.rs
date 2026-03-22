use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::Span;
use super::expr::{name_or_type, type_name};
use crate::ast::{AgentMethod, ClassDeclData, ClassField, SExpr};
use crate::lexer::token::TokenKind;

pub(super) fn class_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, ClassDeclData, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let trait_list = just(TokenKind::Colon).ignore_then(
    name_or_type()
      .separated_by(super::expr::skip_semis())
      .collect::<Vec<_>>()
      .delimited_by(just(TokenKind::LBracket), just(TokenKind::RBracket))
      .or(name_or_type().map(|n| vec![n])),
  );

  let class_field =
    name_or_type().then_ignore(just(TokenKind::Colon)).then(expr.clone()).map(|(name, default)| ClassMember::Field(ClassField { name, default }));

  let class_method = name_or_type().then_ignore(just(TokenKind::Assign)).then(expr).map(|(name, handler)| ClassMember::Method(AgentMethod { name, handler }));

  let member = class_field.or(class_method);

  just(TokenKind::ClassKw)
    .ignore_then(just(TokenKind::Export).or_not())
    .ignore_then(type_name())
    .then(trait_list.or_not().map(|t| t.unwrap_or_default()))
    .then_ignore(just(TokenKind::Assign))
    .then_ignore(just(TokenKind::LBrace))
    .then_ignore(super::expr::skip_semis())
    .then(member.separated_by(super::expr::skip_semis()).collect::<Vec<_>>())
    .then_ignore(super::expr::skip_semis())
    .then_ignore(just(TokenKind::RBrace))
    .map(|((name, traits), members)| {
      let mut fields = Vec::new();
      let mut methods = Vec::new();
      for m in members {
        match m {
          ClassMember::Field(f) => fields.push(f),
          ClassMember::Method(m) => methods.push(m),
        }
      }
      ClassDeclData { name, traits, fields, methods, exported: false }
    })
}

#[derive(Clone)]
enum ClassMember {
  Field(ClassField),
  Method(AgentMethod),
}
