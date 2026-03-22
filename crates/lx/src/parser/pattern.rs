use crate::sym::intern;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::{ident, type_name};
use super::{Span, ss};
use crate::ast::{FieldPattern, Literal, Pattern, SPattern, StrPart};
use crate::lexer::token::TokenKind;

pub(super) fn pattern_parser<'a, I>() -> impl Parser<'a, I, SPattern, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  recursive(|pat| {
    let wildcard = just(TokenKind::Underscore).map_with(|_, e| SPattern::new(Pattern::Wildcard, ss(e.span())));

    let bind = ident().map_with(|n, e| SPattern::new(Pattern::Bind(n), ss(e.span())));

    let int_lit = select! { TokenKind::Int(n) => n }.map_with(|n, e| SPattern::new(Pattern::Literal(Literal::Int(n)), ss(e.span())));

    let float_lit = select! { TokenKind::Float(f) => f }.map_with(|f, e| SPattern::new(Pattern::Literal(Literal::Float(f)), ss(e.span())));

    let true_lit = just(TokenKind::True).map_with(|_, e| SPattern::new(Pattern::Literal(Literal::Bool(true)), ss(e.span())));

    let false_lit = just(TokenKind::False).map_with(|_, e| SPattern::new(Pattern::Literal(Literal::Bool(false)), ss(e.span())));

    let raw_str = select! { TokenKind::RawStr(s) => s }.map_with(|s, e| SPattern::new(Pattern::Literal(Literal::RawStr(s)), ss(e.span())));

    let neg_num = just(TokenKind::Minus)
      .ignore_then(select! { TokenKind::Int(n) => Literal::Int(-n) }.or(select! { TokenKind::Float(f) => Literal::Float(-f) }))
      .map_with(|lit, e| SPattern::new(Pattern::Literal(lit), ss(e.span())));

    let str_pat = str_pattern();

    let tuple_pat = pat
      .clone()
      .separated_by(just(TokenKind::Semi).or_not())
      .collect::<Vec<_>>()
      .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))
      .map_with(|pats, e| SPattern::new(Pattern::Tuple(pats), ss(e.span())));

    let record_pat = record_pattern(pat.clone());
    let list_pat = list_pattern(pat.clone());
    let ctor_pat = ctor_pattern(pat);

    choice((wildcard, neg_num, int_lit, float_lit, true_lit, false_lit, raw_str, str_pat, tuple_pat, record_pat, list_pat, ctor_pat, bind))
  })
}

fn str_pattern<'a, I>() -> impl Parser<'a, I, SPattern, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let chunk = select! { TokenKind::StrChunk(s) => StrPart::Text(s) };

  just(TokenKind::StrStart)
    .ignore_then(chunk.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::StrEnd))
    .map_with(|parts, e| SPattern::new(Pattern::Literal(Literal::Str(parts)), ss(e.span())))
}

fn record_pattern<'a, I>(
  pat: impl Parser<'a, I, SPattern, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SPattern, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let rest = just(TokenKind::DotDot).ignore_then(ident().or_not());

  let field = ident().then(just(TokenKind::Colon).ignore_then(pat).or_not()).map(|(name, pattern)| FieldPattern { name, pattern });

  just(TokenKind::LBrace)
    .ignore_then(super::expr::skip_semis())
    .ignore_then(field.separated_by(super::expr::skip_semis()).collect::<Vec<_>>())
    .then(super::expr::skip_semis().ignore_then(rest).or_not())
    .then_ignore(super::expr::skip_semis())
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(fields, rest), e| SPattern::new(Pattern::Record { fields, rest: rest.flatten() }, ss(e.span())))
}

fn list_pattern<'a, I>(
  pat: impl Parser<'a, I, SPattern, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SPattern, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let rest = just(TokenKind::DotDot).ignore_then(ident().or(just(TokenKind::Underscore).to(intern("_"))).or_not());

  just(TokenKind::LBracket)
    .ignore_then(pat.separated_by(just(TokenKind::Semi).or_not()).collect::<Vec<_>>())
    .then(rest.or_not())
    .then_ignore(just(TokenKind::RBracket))
    .map_with(|(elems, rest), e| SPattern::new(Pattern::List { elems, rest: rest.flatten() }, ss(e.span())))
}

fn ctor_pattern<'a, I>(
  pat: impl Parser<'a, I, SPattern, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SPattern, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  type_name().then(pat.repeated().collect::<Vec<_>>()).map_with(|(name, args), e| SPattern::new(Pattern::Constructor { name, args }, ss(e.span())))
}
