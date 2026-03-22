use crate::sym::intern;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::{ident, type_name};
use super::{Span, ss};
use crate::ast::{SType, TypeExpr, TypeField};
use crate::lexer::token::TokenKind;

pub(super) fn type_parser<'a, I>() -> impl Parser<'a, I, SType, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  recursive(|ty| {
    let atom = type_atom(ty.clone());
    let app = type_app(ty.clone(), atom);

    app.clone().then(just(TokenKind::Caret).ignore_then(app.clone()).or_not()).then(just(TokenKind::Arrow).ignore_then(ty).or_not()).map_with(
      |((left, err_opt), ret_opt), e| match (err_opt, ret_opt) {
        (Some(err), Some(ret)) => {
          let fspan = ss(e.span());
          let fallible = SType::new(TypeExpr::Fallible { ok: Box::new(left), err: Box::new(err) }, fspan);
          SType::new(TypeExpr::Func { param: Box::new(fallible), ret: Box::new(ret) }, fspan)
        },
        (Some(err), None) => SType::new(TypeExpr::Fallible { ok: Box::new(left), err: Box::new(err) }, ss(e.span())),
        (None, Some(ret)) => SType::new(TypeExpr::Func { param: Box::new(left), ret: Box::new(ret) }, ss(e.span())),
        (None, None) => left,
      },
    )
  })
}

fn type_app<'a, I>(
  _ty: impl Parser<'a, I, SType, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  atom: impl Parser<'a, I, SType, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SType, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  type_name()
    .map_with(|n, e| (n, ss(e.span())))
    .then(atom.clone().repeated().collect::<Vec<_>>())
    .map(|((name, span), args)| {
      if args.is_empty() {
        SType::new(TypeExpr::Named(name), span)
      } else {
        let end = args.last().map(|a| a.span.offset() + a.span.len()).unwrap_or(span.offset() + span.len());
        SType::new(TypeExpr::Applied(name, args), (span.offset(), end - span.offset()).into())
      }
    })
    .or(atom)
}

fn type_atom<'a, I>(
  ty: impl Parser<'a, I, SType, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SType, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let named = type_name().map_with(|n, e| SType::new(TypeExpr::Named(n), ss(e.span())));

  let var = ident().map_with(|n, e| SType::new(TypeExpr::Var(n), ss(e.span())));

  let list_ty = ty
    .clone()
    .delimited_by(just(TokenKind::LBracket), just(TokenKind::RBracket))
    .map_with(|inner, e| SType::new(TypeExpr::List(Box::new(inner)), ss(e.span())));

  let record_field = ident().then_ignore(just(TokenKind::Colon)).then(ty.clone()).map(|(name, ty)| TypeField { name, ty });

  let record_ty = just(TokenKind::Semi)
    .repeated()
    .ignore_then(record_field.separated_by(just(TokenKind::Semi).repeated().at_least(1)).allow_trailing().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::Semi).repeated())
    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
    .map_with(|fields, e| SType::new(TypeExpr::Record(fields), ss(e.span())));

  let map_ty = just(TokenKind::PercentLBrace)
    .ignore_then(ty.clone())
    .then_ignore(just(TokenKind::Colon))
    .then(ty.clone())
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(key, value), e| SType::new(TypeExpr::Map { key: Box::new(key), value: Box::new(value) }, ss(e.span())));

  let unit_ty = just(TokenKind::LParen).then(just(TokenKind::RParen)).map_with(|_, e| SType::new(TypeExpr::Named(intern("Unit")), ss(e.span())));

  let grouped_or_tuple =
    just(TokenKind::LParen).ignore_then(ty.separated_by(empty()).at_least(1).collect::<Vec<_>>()).then_ignore(just(TokenKind::RParen)).map_with(|types, e| {
      if types.len() == 1 {
        let mut t = types.into_iter().next().expect("at_least(1)");
        t.span = ss(e.span());
        t
      } else {
        SType::new(TypeExpr::Tuple(types), ss(e.span()))
      }
    });

  choice((list_ty, record_ty, map_ty, unit_ty, grouped_or_tuple, named, var))
}
