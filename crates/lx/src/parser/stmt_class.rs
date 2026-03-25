use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::ExprId;
use super::Span;
use super::expr::{name_or_type, type_name};
use crate::ast::{AgentMethod, ClassDeclData, ClassField};
use crate::lexer::token::TokenKind;

pub fn class_body<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, (Vec<ClassField>, Vec<AgentMethod>), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let class_field =
    name_or_type().then_ignore(just(TokenKind::Colon)).then(expr.clone()).map(|(name, default)| ClassMember::Field(ClassField { name, default }));

  let class_method = name_or_type().then_ignore(just(TokenKind::Assign)).then(expr).map(|(name, handler)| ClassMember::Method(AgentMethod { name, handler }));

  let member = class_field.or(class_method);

  just(TokenKind::LBrace)
    .ignore_then(super::expr::skip_semis())
    .ignore_then(member.separated_by(super::expr::skip_semis()).collect::<Vec<_>>())
    .then_ignore(super::expr::skip_semis())
    .then_ignore(just(TokenKind::RBrace))
    .map(|members| {
      let mut fields = Vec::new();
      let mut methods = Vec::new();
      for m in members {
        match m {
          ClassMember::Field(f) => fields.push(f),
          ClassMember::Method(m) => methods.push(m),
        }
      }
      (fields, methods)
    })
}

pub(super) fn class_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
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

  just(TokenKind::ClassKw)
    .ignore_then(just(TokenKind::Export).or_not())
    .ignore_then(type_name())
    .then(super::type_ann::generic_params())
    .then(trait_list.or_not().map(|t| t.unwrap_or_default()))
    .then_ignore(just(TokenKind::Assign))
    .then(class_body(expr))
    .map(|(((name, type_params), traits), (fields, methods))| ClassDeclData { name, type_params, traits, fields, methods, exported: false })
}

#[derive(Clone)]
enum ClassMember {
  Field(ClassField),
  Method(AgentMethod),
}
