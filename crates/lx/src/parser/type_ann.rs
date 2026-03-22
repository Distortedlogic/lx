use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::{ident, type_name};
use super::{ArenaRef, Span, TypeExprId, ss};
use crate::ast::{TypeExpr, TypeField};
use crate::lexer::token::TokenKind;
use crate::sym::intern;

pub(super) fn type_parser<'a, I>(arena: ArenaRef) -> impl Parser<'a, I, TypeExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  recursive(move |ty| {
    let a1 = arena.clone();
    let a2 = arena.clone();
    let a3 = arena.clone();

    let atom = type_atom(ty.clone(), arena.clone());
    let app = type_app(atom, arena.clone());

    app.clone().then(just(TokenKind::Caret).ignore_then(app.clone()).or_not()).then(just(TokenKind::Arrow).ignore_then(ty).or_not()).map_with(
      move |((left, err_opt), ret_opt), e| match (err_opt, ret_opt) {
        (Some(err), Some(ret)) => {
          let fallible = a1.borrow_mut().alloc_type_expr(TypeExpr::Fallible { ok: left, err }, ss(e.span()));
          a1.borrow_mut().alloc_type_expr(TypeExpr::Func { param: fallible, ret }, ss(e.span()))
        },
        (Some(err), None) => a2.borrow_mut().alloc_type_expr(TypeExpr::Fallible { ok: left, err }, ss(e.span())),
        (None, Some(ret)) => a3.borrow_mut().alloc_type_expr(TypeExpr::Func { param: left, ret }, ss(e.span())),
        (None, None) => left,
      },
    )
  })
}

fn type_app<'a, I>(
  atom: impl Parser<'a, I, TypeExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, TypeExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let a1 = arena;

  type_name()
    .map_with(|n, e| (n, e.span()))
    .then(atom.clone().repeated().collect::<Vec<_>>())
    .map_with(move |((name, name_span), args), e| {
      if args.is_empty() {
        a1.borrow_mut().alloc_type_expr(TypeExpr::Named(name), ss(name_span))
      } else {
        a1.borrow_mut().alloc_type_expr(TypeExpr::Applied(name, args), ss(e.span()))
      }
    })
    .or(atom)
}

fn type_atom<'a, I>(
  ty: impl Parser<'a, I, TypeExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, TypeExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let a1 = arena.clone();
  let a2 = arena.clone();
  let a3 = arena.clone();
  let a4 = arena.clone();
  let a5 = arena.clone();
  let a6 = arena.clone();
  let a7 = arena;

  let named = type_name().map_with(move |n, e| a1.borrow_mut().alloc_type_expr(TypeExpr::Named(n), ss(e.span())));
  let var = ident().map_with(move |n, e| a2.borrow_mut().alloc_type_expr(TypeExpr::Var(n), ss(e.span())));

  let list_ty = ty
    .clone()
    .delimited_by(just(TokenKind::LBracket), just(TokenKind::RBracket))
    .map_with(move |inner, e| a3.borrow_mut().alloc_type_expr(TypeExpr::List(inner), ss(e.span())));

  let record_field = ident().then_ignore(just(TokenKind::Colon)).then(ty.clone()).map(|(name, ty)| TypeField { name, ty });

  let record_ty = just(TokenKind::Semi)
    .repeated()
    .ignore_then(record_field.separated_by(just(TokenKind::Semi).repeated().at_least(1)).allow_trailing().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::Semi).repeated())
    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
    .map_with(move |fields, e| a4.borrow_mut().alloc_type_expr(TypeExpr::Record(fields), ss(e.span())));

  let map_ty = just(TokenKind::PercentLBrace)
    .ignore_then(ty.clone())
    .then_ignore(just(TokenKind::Colon))
    .then(ty.clone())
    .then_ignore(just(TokenKind::RBrace))
    .map_with(move |(key, value), e| a5.borrow_mut().alloc_type_expr(TypeExpr::Map { key, value }, ss(e.span())));

  let unit_ty =
    just(TokenKind::LParen).then(just(TokenKind::RParen)).map_with(move |_, e| a6.borrow_mut().alloc_type_expr(TypeExpr::Named(intern("Unit")), ss(e.span())));

  let grouped_or_tuple = just(TokenKind::LParen)
    .ignore_then(ty.separated_by(empty()).at_least(1).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .map_with(move |types, e| {
      if types.len() == 1 {
        let t = types.into_iter().next().expect("at_least(1)");
        let node = a7.borrow().type_expr(t).clone();
        a7.borrow_mut().alloc_type_expr(node, ss(e.span()))
      } else {
        a7.borrow_mut().alloc_type_expr(TypeExpr::Tuple(types), ss(e.span()))
      }
    });

  choice((list_ty, record_ty, map_ty, unit_ty, grouped_or_tuple, named, var))
}
