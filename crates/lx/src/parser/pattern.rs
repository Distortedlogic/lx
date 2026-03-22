use chumsky::prelude::*;

use super::TInput;
use super::ss;
use crate::ast::*;
use crate::lexer::token::TokenKind;

pub(super) fn pattern_parser<'a>() -> impl Parser<'a, TInput<'a>, SPattern, extra::Err<Rich<'a, TokenKind>>> + Clone {
  recursive(|pat| {
    let wildcard = just(TokenKind::Underscore).map_with(|_, e| SPattern::new(Pattern::Wildcard, ss(e.span())));

    let bind = select! { TokenKind::Ident(n) => n }.map_with(|n, e| SPattern::new(Pattern::Bind(n), ss(e.span())));

    let int_lit = select! { TokenKind::Int(n) => n }.map_with(|n, e| SPattern::new(Pattern::Literal(Literal::Int(n)), ss(e.span())));

    let float_lit = select! { TokenKind::Float(f) => f }.map_with(|f, e| SPattern::new(Pattern::Literal(Literal::Float(f)), ss(e.span())));

    let true_lit = just(TokenKind::True).map_with(|_, e| SPattern::new(Pattern::Literal(Literal::Bool(true)), ss(e.span())));

    let false_lit = just(TokenKind::False).map_with(|_, e| SPattern::new(Pattern::Literal(Literal::Bool(false)), ss(e.span())));

    let raw_str_lit = select! { TokenKind::RawStr(s) => s }.map_with(|s, e| SPattern::new(Pattern::Literal(Literal::RawStr(s)), ss(e.span())));

    let neg_num = just(TokenKind::Minus)
      .ignore_then(select! { TokenKind::Int(n) => Literal::Int(-n) }.or(select! { TokenKind::Float(f) => Literal::Float(-f) }))
      .map_with(|lit, e| SPattern::new(Pattern::Literal(lit), ss(e.span())));

    let str_pat = str_pattern_parser();

    let tuple_pat = just(TokenKind::LParen)
      .ignore_then(pat.clone().separated_by(just(TokenKind::Semi).or_not()).collect::<Vec<_>>())
      .then_ignore(just(TokenKind::RParen))
      .map_with(|pats, e| SPattern::new(Pattern::Tuple(pats), ss(e.span())));

    let record_pat = record_pattern_parser(pat.clone());
    let list_pat = list_pattern_parser(pat.clone());
    let ctor_pat = constructor_pattern_parser(pat.clone());

    choice((wildcard, neg_num, int_lit, float_lit, true_lit, false_lit, raw_str_lit, str_pat, tuple_pat, record_pat, list_pat, ctor_pat, bind))
  })
}

fn str_pattern_parser<'a>() -> impl Parser<'a, TInput<'a>, SPattern, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let chunk = select! { TokenKind::StrChunk(s) => StrPart::Text(s) };

  just(TokenKind::StrStart)
    .ignore_then(chunk.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::StrEnd))
    .map_with(|parts, e| SPattern::new(Pattern::Literal(Literal::Str(parts)), ss(e.span())))
}

fn record_pattern_parser<'a>(
  pat: impl Parser<'a, TInput<'a>, SPattern, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SPattern, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let rest = just(TokenKind::DotDot).ignore_then(select! { TokenKind::Ident(n) => n }.or_not());

  let field =
    select! { TokenKind::Ident(n) => n }.then(just(TokenKind::Colon).ignore_then(pat.clone()).or_not()).map(|(name, pattern)| FieldPattern { name, pattern });

  just(TokenKind::LBrace)
    .ignore_then(super::expr::skip_semis_pub())
    .ignore_then(field.separated_by(super::expr::skip_semis_pub()).collect::<Vec<_>>())
    .then(super::expr::skip_semis_pub().ignore_then(rest).or_not())
    .then_ignore(super::expr::skip_semis_pub())
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(fields, rest), e| {
      let rest = rest.flatten();
      SPattern::new(Pattern::Record { fields, rest }, ss(e.span()))
    })
}

fn list_pattern_parser<'a>(
  pat: impl Parser<'a, TInput<'a>, SPattern, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SPattern, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let rest = just(TokenKind::DotDot).ignore_then(select! { TokenKind::Ident(n) => n }.or(just(TokenKind::Underscore).to("_".to_string())).or_not());

  let elem = pat.clone();

  just(TokenKind::LBracket)
    .ignore_then(elem.separated_by(just(TokenKind::Semi).or_not()).collect::<Vec<_>>())
    .then(rest.or_not())
    .then_ignore(just(TokenKind::RBracket))
    .map_with(|(elems, rest), e| {
      let rest = rest.flatten();
      SPattern::new(Pattern::List { elems, rest }, ss(e.span()))
    })
}

fn constructor_pattern_parser<'a>(
  pat: impl Parser<'a, TInput<'a>, SPattern, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SPattern, extra::Err<Rich<'a, TokenKind>>> + Clone {
  select! { TokenKind::TypeName(n) => n }
    .then(pat.repeated().collect::<Vec<_>>())
    .map_with(|(name, args), e| SPattern::new(Pattern::Constructor { name, args }, ss(e.span())))
}
