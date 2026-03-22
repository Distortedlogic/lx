use crate::sym::intern;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::{Span, ss};
use crate::ast::{Expr, ListElem, Literal, MapEntry, Param, RecordField, SExpr, SStmt, SelArm};
use crate::lexer::token::TokenKind;
use crate::sym::Sym;

pub(super) fn ident<'a, I>() -> impl Parser<'a, I, Sym, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! { TokenKind::Ident(n) => n }
}

pub(super) fn type_name<'a, I>() -> impl Parser<'a, I, Sym, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! { TokenKind::TypeName(n) => n }
}

pub(super) fn name_or_type<'a, I>() -> impl Parser<'a, I, Sym, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! { TokenKind::Ident(n) => n, TokenKind::TypeName(n) => n }
}

pub(super) fn skip_semis<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  just(TokenKind::Semi).repeated().ignored()
}

pub(super) fn semi_sep<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  just(TokenKind::Semi).repeated().at_least(1).ignored()
}

pub(super) fn expr_parser<'a, I>() -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  recursive(|expr| {
    let literal = select! {
        TokenKind::Int(n) => Expr::Literal(Literal::Int(n)),
        TokenKind::Float(f) => Expr::Literal(Literal::Float(f)),
        TokenKind::True => Expr::Literal(Literal::Bool(true)),
        TokenKind::False => Expr::Literal(Literal::Bool(false)),
        TokenKind::Unit => Expr::Literal(Literal::Unit),
        TokenKind::RawStr(s) => Expr::Literal(Literal::RawStr(s)),
    }
    .map_with(|node, e| SExpr::new(node, ss(e.span())));

    let string_lit = super::expr_pratt::string_parser(expr.clone());
    let ident_expr = ident().map_with(|n, e| SExpr::new(Expr::Ident(n), ss(e.span())));
    let type_ctor = type_name().map_with(|n, e| SExpr::new(Expr::TypeConstructor(n), ss(e.span())));
    let list = list_parser(expr.clone());
    let block_or_record = block_or_record_parser(expr.clone());
    let map = map_parser(expr.clone());
    let paren = super::expr_pratt::paren_parser(expr.clone());

    let loop_expr = just(TokenKind::Loop)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(stmts_block(expr.clone()))
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|stmts, e| SExpr::new(Expr::Loop(stmts), ss(e.span())));

    let par_expr = just(TokenKind::Par)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(stmts_block(expr.clone()))
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|stmts, e| SExpr::new(Expr::Par(stmts), ss(e.span())));

    let sel_arm = expr.clone().then_ignore(just(TokenKind::Arrow)).then(expr.clone()).map(|(ex, handler)| SelArm { expr: ex, handler });

    let sel_expr = just(TokenKind::Sel)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(skip_semis())
      .ignore_then(sel_arm.separated_by(semi_sep()).allow_trailing().collect::<Vec<_>>())
      .then_ignore(skip_semis())
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|arms, e| SExpr::new(Expr::Sel(arms), ss(e.span())));

    let break_expr = just(TokenKind::Break).ignore_then(expr.clone().or_not()).map_with(|val, e| SExpr::new(Expr::Break(val.map(Box::new)), ss(e.span())));

    let assert_expr = just(TokenKind::Assert)
      .ignore_then(expr.clone())
      .then(expr.clone().or_not())
      .map_with(|(ex, msg), e| SExpr::new(Expr::Assert { expr: Box::new(ex), msg: msg.map(Box::new) }, ss(e.span())));

    let emit_expr = just(TokenKind::Emit).ignore_then(expr.clone()).map_with(|v, e| SExpr::new(Expr::Emit { value: Box::new(v) }, ss(e.span())));

    let yield_expr = just(TokenKind::Yield).ignore_then(expr.clone()).map_with(|v, e| SExpr::new(Expr::Yield { value: Box::new(v) }, ss(e.span())));

    let with_expr = super::expr_pratt::with_parser(expr.clone());

    let atom = choice((
      literal,
      string_lit,
      paren,
      list,
      block_or_record,
      map,
      loop_expr,
      par_expr,
      sel_expr,
      emit_expr,
      yield_expr,
      with_expr,
      break_expr,
      assert_expr,
      type_ctor,
      ident_expr,
    ))
    .boxed();

    super::expr_pratt::pratt_expr(atom, expr)
  })
}

fn list_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread = just(TokenKind::DotDot).ignore_then(expr.clone()).map(ListElem::Spread);
  let single = expr.map(ListElem::Single);
  let elem = spread.or(single);

  elem
    .separated_by(just(TokenKind::Semi).or_not())
    .allow_trailing()
    .collect::<Vec<_>>()
    .delimited_by(just(TokenKind::LBracket), just(TokenKind::RBracket))
    .map_with(|elems, e| SExpr::new(Expr::List(elems), ss(e.span())))
}

fn block_or_record_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let empty_record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(just(TokenKind::Colon))
    .then(just(TokenKind::RBrace))
    .map_with(|_, e| SExpr::new(Expr::Record(vec![]), ss(e.span())));

  let record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(looks_like_record().rewind())
    .ignore_then(record_fields(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|fields, e| SExpr::new(Expr::Record(fields), ss(e.span())));

  let block = just(TokenKind::LBrace)
    .ignore_then(stmts_block(expr))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|stmts, e| SExpr::new(Expr::Block(stmts), ss(e.span())));

  choice((empty_record, record, block))
}

fn looks_like_record<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  choice((ident().then_ignore(just(TokenKind::Colon)).ignored(), just(TokenKind::DotDot).ignored()))
}

fn record_fields<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, Vec<RecordField>, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread_field = just(TokenKind::DotDot).ignore_then(expr.clone()).map(|value| RecordField { name: None, value, is_spread: true });

  let named_field = ident().then(just(TokenKind::Colon).ignore_then(expr).or_not()).map_with(|(name, val), e| {
    let value = val.unwrap_or_else(|| SExpr::new(Expr::Ident(name), ss(e.span())));
    RecordField { name: Some(name), value, is_spread: false }
  });

  let field = spread_field.or(named_field);

  skip_semis().ignore_then(field.separated_by(skip_semis()).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_semis())
}

fn map_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread = just(TokenKind::DotDot).ignore_then(expr.clone()).map(|v| MapEntry { key: None, value: v, is_spread: true });

  let kv = expr.clone().then_ignore(just(TokenKind::Colon)).then(expr).map(|(k, v)| MapEntry { key: Some(k), value: v, is_spread: false });

  let entry = spread.or(kv);

  entry
    .separated_by(just(TokenKind::Semi).or_not())
    .allow_trailing()
    .collect::<Vec<_>>()
    .delimited_by(just(TokenKind::PercentLBrace), just(TokenKind::RBrace))
    .map_with(|entries, e| SExpr::new(Expr::Map(entries), ss(e.span())))
}

pub(super) fn stmts_block<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, Vec<SStmt>, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  skip_semis().ignore_then(super::stmt::stmt_parser(expr).separated_by(semi_sep()).allow_trailing().collect::<Vec<_>>().then_ignore(skip_semis()))
}

pub(super) fn param_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, Param, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let typed = ident()
    .then(just(TokenKind::Colon).ignore_then(super::type_ann::type_parser()).or_not())
    .then(just(TokenKind::Assign).ignore_then(expr).or_not())
    .map(|((name, type_ann), default)| Param { name, type_ann, default });

  let underscore = just(TokenKind::Underscore).to(Param { name: intern("_"), type_ann: None, default: None });

  typed.or(underscore)
}
