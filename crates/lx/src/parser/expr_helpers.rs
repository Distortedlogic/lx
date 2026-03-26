use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::{ident, ident_or_keyword, skip_semis};
use super::{ArenaRef, ExprId, Span, ss};
use crate::ast::{Expr, ExprBlock, ListElem, MapEntry, Param, RecordField};
use crate::lexer::token::TokenKind;
use crate::sym::intern;

pub(super) fn list_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread = just(TokenKind::DotDot).ignore_then(expr.clone()).map(ListElem::Spread);
  let single = expr.map(ListElem::Single);
  let elem = spread.or(single);

  let al = arena;
  just(TokenKind::LBracket)
    .ignore_then(super::expr::skip_semis())
    .ignore_then(elem.separated_by(super::expr::semi_sep()).allow_trailing().collect::<Vec<_>>())
    .then_ignore(super::expr::skip_semis())
    .then_ignore(just(TokenKind::RBracket))
    .map_with(move |elems, e| al.borrow_mut().alloc_expr(Expr::List(elems), ss(e.span())))
}

pub(super) fn block_or_record_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let a1 = arena.clone();
  let a2 = arena.clone();
  let a3 = arena.clone();
  let a4 = arena;

  let empty_record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(just(TokenKind::Colon))
    .then(just(TokenKind::RBrace))
    .map_with(move |_, e| a1.borrow_mut().alloc_expr(Expr::Record(vec![]), ss(e.span())));

  let record_inner = record_fields(expr.clone(), a2).then_ignore(just(TokenKind::RBrace)).map(Expr::Record);

  let block_inner = super::expr::stmts_block(expr, a3).then_ignore(just(TokenKind::RBrace)).map(|stmts| Expr::Block(ExprBlock { stmts }));

  let brace_expr = just(TokenKind::LBrace).ignore_then(record_inner.or(block_inner)).map_with(move |node, e| a4.borrow_mut().alloc_expr(node, ss(e.span())));

  choice((empty_record, brace_expr))
}

fn record_fields<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, Vec<RecordField>, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread_field = just(TokenKind::DotDot).ignore_then(expr.clone()).map(RecordField::Spread);

  let named_field = {
    let al = arena;
    ident_or_keyword().then(just(TokenKind::Colon).ignore_then(expr).or_not()).map_with(move |(name, val), e| {
      let value = val.unwrap_or_else(|| al.borrow_mut().alloc_expr(Expr::Ident(name), ss(e.span())));
      RecordField::Named { name, value }
    })
  };

  let field = spread_field.or(named_field);

  skip_semis().ignore_then(field.separated_by(skip_semis()).at_least(1).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_semis())
}

pub(super) fn map_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread = just(TokenKind::DotDot).ignore_then(expr.clone()).map(MapEntry::Spread);
  let kv = expr.clone().then_ignore(just(TokenKind::Colon)).then(expr).map(|(k, v)| MapEntry::Keyed { key: k, value: v });
  let entry = spread.or(kv);

  let al = arena;
  entry
    .separated_by(just(TokenKind::Semi).or_not())
    .allow_trailing()
    .collect::<Vec<_>>()
    .delimited_by(just(TokenKind::PercentLBrace), just(TokenKind::RBrace))
    .map_with(move |entries, e| al.borrow_mut().alloc_expr(Expr::Map(entries), ss(e.span())))
}

pub(super) fn param_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, Param, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let typed = ident()
    .then(just(TokenKind::Colon).ignore_then(super::type_ann::type_parser(arena)).or_not())
    .then(just(TokenKind::Assign).ignore_then(expr).or_not())
    .map(|((name, type_ann), default)| Param { name, type_ann, default });

  let underscore = just(TokenKind::Underscore).to(Param { name: intern("_"), type_ann: None, default: None });

  typed.or(underscore)
}
