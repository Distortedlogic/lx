use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::{ident, type_name};
use super::{ArenaRef, PatternId, Span, ss};
use crate::ast::{FieldPattern, Literal, Pattern, PatternConstructor, PatternList, PatternRecord, StrPart};
use crate::lexer::token::TokenKind;
use crate::sym::intern;

pub(super) fn pattern_parser<'a, I>(arena: ArenaRef) -> impl Parser<'a, I, PatternId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  recursive(move |pat| {
    let a1 = arena.clone();
    let a2 = arena.clone();
    let a3 = arena.clone();
    let a4 = arena.clone();
    let a5 = arena.clone();
    let a6 = arena.clone();
    let a7 = arena.clone();
    let a8 = arena.clone();
    let a9 = arena.clone();
    let a10 = arena.clone();
    let a11 = arena.clone();

    let wildcard = just(TokenKind::Underscore).map_with(move |_, e| a1.borrow_mut().alloc_pattern(Pattern::Wildcard, ss(e.span())));
    let bind = ident().map_with(move |n, e| a2.borrow_mut().alloc_pattern(Pattern::Bind(n), ss(e.span())));
    let int_lit = select! { TokenKind::Int(n) => n }.map_with(move |n, e| a3.borrow_mut().alloc_pattern(Pattern::Literal(Literal::Int(n)), ss(e.span())));
    let float_lit = select! { TokenKind::Float(f) => f }.map_with(move |f, e| a4.borrow_mut().alloc_pattern(Pattern::Literal(Literal::Float(f)), ss(e.span())));
    let true_lit = just(TokenKind::True).map_with(move |_, e| a5.borrow_mut().alloc_pattern(Pattern::Literal(Literal::Bool(true)), ss(e.span())));
    let false_lit = just(TokenKind::False).map_with(move |_, e| a6.borrow_mut().alloc_pattern(Pattern::Literal(Literal::Bool(false)), ss(e.span())));
    let raw_str = select! { TokenKind::RawStr(s) => s }.map_with(move |s, e| a7.borrow_mut().alloc_pattern(Pattern::Literal(Literal::RawStr(s)), ss(e.span())));

    let neg_num = just(TokenKind::Minus)
      .ignore_then(select! { TokenKind::Int(n) => Literal::Int(-n) }.or(select! { TokenKind::Float(f) => Literal::Float(-f) }))
      .map_with(move |lit, e| a8.borrow_mut().alloc_pattern(Pattern::Literal(lit), ss(e.span())));

    let str_pat = str_pattern(arena.clone());

    let tuple_pat = pat
      .clone()
      .separated_by(just(TokenKind::Semi).or_not())
      .collect::<Vec<_>>()
      .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))
      .map_with(move |pats, e| a9.borrow_mut().alloc_pattern(Pattern::Tuple(pats), ss(e.span())));

    let record_pat = record_pattern(pat.clone(), a10.clone());
    let list_pat = list_pattern(pat.clone(), a11.clone());
    let ctor_pat = ctor_pattern(pat, arena.clone());

    choice((wildcard, neg_num, int_lit, float_lit, true_lit, false_lit, raw_str, str_pat, tuple_pat, record_pat, list_pat, ctor_pat, bind))
  })
}

fn str_pattern<'a, I>(arena: ArenaRef) -> impl Parser<'a, I, PatternId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let chunk = select! { TokenKind::StrChunk(s) => StrPart::Text(s) };

  just(TokenKind::StrStart)
    .ignore_then(chunk.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::StrEnd))
    .map_with(move |parts, e| arena.borrow_mut().alloc_pattern(Pattern::Literal(Literal::Str(parts)), ss(e.span())))
}

fn record_pattern<'a, I>(
  pat: impl Parser<'a, I, PatternId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, PatternId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
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
    .map_with(move |(fields, rest), e| arena.borrow_mut().alloc_pattern(Pattern::Record(PatternRecord { fields, rest: rest.flatten() }), ss(e.span())))
}

fn list_pattern<'a, I>(
  pat: impl Parser<'a, I, PatternId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, PatternId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let rest = just(TokenKind::DotDot).ignore_then(ident().or(just(TokenKind::Underscore).to(intern("_"))).or_not());

  just(TokenKind::LBracket)
    .ignore_then(pat.separated_by(just(TokenKind::Semi).or_not()).collect::<Vec<_>>())
    .then(rest.or_not())
    .then_ignore(just(TokenKind::RBracket))
    .map_with(move |(elems, rest), e| arena.borrow_mut().alloc_pattern(Pattern::List(PatternList { elems, rest: rest.flatten() }), ss(e.span())))
}

fn ctor_pattern<'a, I>(
  pat: impl Parser<'a, I, PatternId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, PatternId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  type_name()
    .then(pat.repeated().collect::<Vec<_>>())
    .map_with(move |(name, args), e| arena.borrow_mut().alloc_pattern(Pattern::Constructor(PatternConstructor { name, args }), ss(e.span())))
}
