use chumsky::input::ValueInput;
use chumsky::pratt::{Operator, infix, left, postfix, prefix};
use chumsky::prelude::*;

use super::expr::{semi_sep, skip_semis, type_name};
use super::{ArenaRef, ExprId, Span, ss};
use crate::ast::{
  BinOp, Expr, ExprApply, ExprAsk, ExprBinary, ExprCoalesce, ExprFieldAccess, ExprMatch, ExprPipe, ExprPropagate, ExprTell, ExprTernary, ExprUnary, FieldKind,
  Literal, MatchArm, StrPart, UnaryOp,
};
use crate::lexer::token::TokenKind;

pub(super) fn tok_to_op(tok: &TokenKind) -> BinOp {
  super::token_to_binop(tok).expect("section_op guarantees valid operator")
}

pub(super) fn string_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let chunk = select! { TokenKind::StrChunk(s) => StrPart::Text(s) };
  let interp_braced = just(TokenKind::LBrace).ignore_then(expr.clone()).then_ignore(just(TokenKind::RBrace)).map(StrPart::Interp);
  let interp_bare = expr.map(StrPart::Interp);
  let part = choice((chunk, interp_braced, interp_bare));

  just(TokenKind::StrStart)
    .ignore_then(part.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::StrEnd))
    .map_with(move |parts, e| arena.borrow_mut().alloc_expr(Expr::Literal(Literal::Str(parts)), ss(e.span())))
}

pub(super) fn section_op<'a, I>() -> impl Parser<'a, I, TokenKind, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  any().filter(|k: &TokenKind| {
    matches!(
      k,
      TokenKind::Plus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::Percent
        | TokenKind::IntDiv
        | TokenKind::PlusPlus
        | TokenKind::Eq
        | TokenKind::NotEq
        | TokenKind::Lt
        | TokenKind::Gt
        | TokenKind::LtEq
        | TokenKind::GtEq
        | TokenKind::And
        | TokenKind::Or
        | TokenKind::Minus
    )
  })
}

fn dot_rhs<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, FieldKind, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let named = super::expr::ident_or_keyword().map(FieldKind::Named);
  let type_field = type_name().map(FieldKind::Named);
  let indexed = select! { TokenKind::Int(n) => n }.map(|n| {
    let idx: i64 = n.try_into().unwrap_or(0);
    FieldKind::Index(idx)
  });
  let neg_indexed = just(TokenKind::Minus).ignore_then(select! { TokenKind::Int(n) => n }).map(|n| {
    let idx: i64 = n.try_into().unwrap_or(0);
    FieldKind::Index(-idx)
  });
  let computed = expr.clone().delimited_by(just(TokenKind::LBracket), just(TokenKind::RBracket)).map(FieldKind::Computed);
  let str_key = string_parser(expr, arena).map(FieldKind::Computed);

  choice((named, type_field, neg_indexed, indexed, computed, str_key))
}

pub(super) fn pratt_expr<'a, I>(
  atom: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone + 'a,
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone + 'a,
  arena: ArenaRef,
) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
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

  let match_arms = skip_semis()
    .ignore_then(
      super::pattern::pattern_parser(arena.clone())
        .then(just(TokenKind::Amp).ignore_then(expr.clone()).or_not())
        .then_ignore(just(TokenKind::Arrow))
        .then(expr.clone())
        .map(|((pattern, guard), body)| MatchArm { pattern, guard, body })
        .separated_by(semi_sep())
        .allow_trailing()
        .collect::<Vec<_>>(),
    )
    .then_ignore(skip_semis())
    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace));

  let ternary_tail = expr.clone().then(just(TokenKind::Colon).ignore_then(expr.clone()).or_not());
  let question_rhs = match_arms.map(QRhs::Match).or(ternary_tail.map(|(t, e)| QRhs::Ternary(t, e)));
  let dot_field = dot_rhs(expr, arena.clone());

  macro_rules! binop {
    ($assoc:ident($bp:expr), $tok:expr, $op:expr) => {
      infix($assoc($bp), just($tok), {
        let al = arena.clone();
        move |l: ExprId, _, r: ExprId, e| al.borrow_mut().alloc_expr(Expr::Binary(ExprBinary { op: $op, left: l, right: r }), ss(e.span()))
      })
      .boxed()
    };
  }

  atom.pratt(vec![
    prefix(29, just(TokenKind::Minus), move |_, o: ExprId, e| {
      a1.borrow_mut().alloc_expr(Expr::Unary(ExprUnary { op: UnaryOp::Neg, operand: o }), ss(e.span()))
    })
    .boxed(),
    prefix(29, just(TokenKind::Bang), move |_, o: ExprId, e| a2.borrow_mut().alloc_expr(Expr::Unary(ExprUnary { op: UnaryOp::Not, operand: o }), ss(e.span())))
      .boxed(),
    postfix(33, just(TokenKind::Dot).then(dot_field), move |l: ExprId, (_, f), e| {
      a3.borrow_mut().alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: l, field: f }), ss(e.span()))
    })
    .boxed(),
    postfix(10, just(TokenKind::Caret), move |o: ExprId, _, e| a4.borrow_mut().alloc_expr(Expr::Propagate(ExprPropagate { inner: o }), ss(e.span()))).boxed(),
    postfix(3, just(TokenKind::Question).then(question_rhs), move |s: ExprId, (_, rhs), e| match rhs {
      QRhs::Match(arms) => a5.borrow_mut().alloc_expr(Expr::Match(ExprMatch { scrutinee: s, arms }), ss(e.span())),
      QRhs::Ternary(t, el) => a5.borrow_mut().alloc_expr(Expr::Ternary(ExprTernary { cond: s, then_: t, else_: el }), ss(e.span())),
    })
    .boxed(),
    binop!(left(27), TokenKind::Star, BinOp::Mul),
    binop!(left(27), TokenKind::Slash, BinOp::Div),
    binop!(left(27), TokenKind::Percent, BinOp::Mod),
    binop!(left(27), TokenKind::IntDiv, BinOp::IntDiv),
    binop!(left(25), TokenKind::Plus, BinOp::Add),
    binop!(left(25), TokenKind::Minus, BinOp::Sub),
    binop!(left(23), TokenKind::DotDot, BinOp::Range),
    binop!(left(23), TokenKind::DotDotEq, BinOp::RangeInclusive),
    binop!(left(21), TokenKind::PlusPlus, BinOp::Concat),
    infix(left(19), just(TokenKind::Pipe), move |l: ExprId, _, r: ExprId, e| {
      a6.borrow_mut().alloc_expr(Expr::Pipe(ExprPipe { left: l, right: r }), ss(e.span()))
    })
    .boxed(),
    infix(left(18), just(TokenKind::TildeArrow), move |l: ExprId, _, r: ExprId, e| {
      a9.borrow_mut().alloc_expr(Expr::Tell(ExprTell { target: l, msg: r }), ss(e.span()))
    })
    .boxed(),
    infix(left(18), just(TokenKind::TildeArrowQ), move |l: ExprId, _, r: ExprId, e| {
      a10.borrow_mut().alloc_expr(Expr::Ask(ExprAsk { target: l, msg: r }), ss(e.span()))
    })
    .boxed(),
    binop!(left(17), TokenKind::Eq, BinOp::Eq),
    binop!(left(17), TokenKind::NotEq, BinOp::NotEq),
    binop!(left(17), TokenKind::Lt, BinOp::Lt),
    binop!(left(17), TokenKind::Gt, BinOp::Gt),
    binop!(left(17), TokenKind::LtEq, BinOp::LtEq),
    binop!(left(17), TokenKind::GtEq, BinOp::GtEq),
    binop!(left(15), TokenKind::And, BinOp::And),
    binop!(left(13), TokenKind::Or, BinOp::Or),
    infix(left(11), just(TokenKind::QQ), move |l: ExprId, _, r: ExprId, e| {
      a7.borrow_mut().alloc_expr(Expr::Coalesce(ExprCoalesce { expr: l, default: r }), ss(e.span()))
    })
    .boxed(),
    infix(left(7), just(TokenKind::Amp), move |l: ExprId, _, r: ExprId, e| {
      a8.borrow_mut().alloc_expr(Expr::Binary(ExprBinary { op: BinOp::And, left: l, right: r }), ss(e.span()))
    })
    .boxed(),
    infix(left(31), empty(), {
      let al = arena;
      move |f: ExprId, _, a: ExprId, e| al.borrow_mut().alloc_expr(Expr::Apply(ExprApply { func: f, arg: a }), ss(e.span()))
    })
    .boxed(),
  ])
}

enum QRhs {
  Match(Vec<MatchArm>),
  Ternary(ExprId, Option<ExprId>),
}

pub(super) use super::expr_compound::{paren_parser, with_parser};
