use chumsky::input::ValueInput;
use chumsky::pratt::{Operator, infix, left, postfix, prefix};
use chumsky::prelude::*;

use super::expr::{ident, semi_sep, skip_semis, type_name};
use super::{Span, ss};
use crate::ast::{
  BinOp, Expr, ExprApply, ExprBinary, ExprCoalesce, ExprFieldAccess, ExprFunc, ExprMatch, ExprPipe, ExprTernary, ExprUnary, ExprWith, FieldKind, MatchArm,
  SExpr, Section, StrPart, UnaryOp, WithKind,
};
use crate::lexer::token::TokenKind;
use crate::sym::intern;

fn tok_to_op(tok: &TokenKind) -> BinOp {
  super::token_to_binop(tok).expect("section_op guarantees valid operator")
}

pub(super) fn string_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
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
    .map_with(|parts, e| SExpr::new(Expr::Literal(crate::ast::Literal::Str(parts)), ss(e.span())))
}

fn section_op<'a, I>() -> impl Parser<'a, I, TokenKind, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
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
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, FieldKind, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let named = ident().map(FieldKind::Named);
  let type_field = type_name().map(FieldKind::Named);
  let indexed = select! { TokenKind::Int(n) => n }.map(|n| {
    let idx: i64 = n.try_into().unwrap_or(0);
    FieldKind::Index(idx)
  });
  let neg_indexed = just(TokenKind::Minus).ignore_then(select! { TokenKind::Int(n) => n }).map(|n| {
    let idx: i64 = n.try_into().unwrap_or(0);
    FieldKind::Index(-idx)
  });
  let computed = expr.clone().delimited_by(just(TokenKind::LBracket), just(TokenKind::RBracket)).map(|e| FieldKind::Computed(Box::new(e)));
  let str_key = string_parser(expr).map(|e| FieldKind::Computed(Box::new(e)));

  choice((named, type_field, neg_indexed, indexed, computed, str_key))
}

pub(super) fn pratt_expr<'a, I>(
  atom: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone + 'a,
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone + 'a,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let match_arms = skip_semis()
    .ignore_then(
      super::pattern::pattern_parser()
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

  let dot_field = dot_rhs(expr.clone());

  macro_rules! binop {
    ($assoc:ident($bp:expr), $tok:expr, $op:expr) => {
      infix($assoc($bp), just($tok), |l: SExpr, _, r: SExpr, e| SExpr::new(Expr::Binary(ExprBinary { op: $op, left: Box::new(l), right: Box::new(r) }), ss(e.span())))
        .boxed()
    };
  }

  atom.pratt(vec![
    prefix(29, just(TokenKind::Minus), |_, operand: SExpr, e| SExpr::new(Expr::Unary(ExprUnary { op: UnaryOp::Neg, operand: Box::new(operand) }), ss(e.span()))).boxed(),
    prefix(29, just(TokenKind::Bang), |_, operand: SExpr, e| SExpr::new(Expr::Unary(ExprUnary { op: UnaryOp::Not, operand: Box::new(operand) }), ss(e.span()))).boxed(),
    postfix(33, just(TokenKind::Dot).then(dot_field), |left: SExpr, (_, field), e| SExpr::new(Expr::FieldAccess(ExprFieldAccess { expr: Box::new(left), field }), ss(e.span())))
      .boxed(),
    postfix(10, just(TokenKind::Caret), |operand: SExpr, _, e| SExpr::new(Expr::Propagate(Box::new(operand)), ss(e.span()))).boxed(),
    postfix(3, just(TokenKind::Question).then(question_rhs), |scrutinee: SExpr, (_, rhs), e| match rhs {
      QRhs::Match(arms) => SExpr::new(Expr::Match(ExprMatch { scrutinee: Box::new(scrutinee), arms }), ss(e.span())),
      QRhs::Ternary(then_, else_) => SExpr::new(Expr::Ternary(ExprTernary { cond: Box::new(scrutinee), then_: Box::new(then_), else_: else_.map(Box::new) }), ss(e.span())),
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
    infix(left(19), just(TokenKind::Pipe), |l: SExpr, _, r: SExpr, e| SExpr::new(Expr::Pipe(ExprPipe { left: Box::new(l), right: Box::new(r) }), ss(e.span()))).boxed(),
    binop!(left(17), TokenKind::Eq, BinOp::Eq),
    binop!(left(17), TokenKind::NotEq, BinOp::NotEq),
    binop!(left(17), TokenKind::Lt, BinOp::Lt),
    binop!(left(17), TokenKind::Gt, BinOp::Gt),
    binop!(left(17), TokenKind::LtEq, BinOp::LtEq),
    binop!(left(17), TokenKind::GtEq, BinOp::GtEq),
    binop!(left(15), TokenKind::And, BinOp::And),
    binop!(left(13), TokenKind::Or, BinOp::Or),
    infix(left(11), just(TokenKind::QQ), |l: SExpr, _, r: SExpr, e| SExpr::new(Expr::Coalesce(ExprCoalesce { expr: Box::new(l), default: Box::new(r) }), ss(e.span())))
      .boxed(),
    infix(left(7), just(TokenKind::Amp), |l: SExpr, _, r: SExpr, e| {
      SExpr::new(Expr::Binary(ExprBinary { op: BinOp::And, left: Box::new(l), right: Box::new(r) }), ss(e.span()))
    })
    .boxed(),
    infix(left(31), empty(), |func: SExpr, _, arg: SExpr, e| SExpr::new(Expr::Apply(ExprApply { func: Box::new(func), arg: Box::new(arg) }), ss(e.span()))).boxed(),
  ])
}

enum QRhs {
  Match(Vec<MatchArm>),
  Ternary(SExpr, Option<SExpr>),
}

pub(super) fn with_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let with_context = just(TokenKind::With)
    .ignore_then(just(TokenKind::Ident(intern("context"))))
    .ignore_then(ident().then_ignore(just(TokenKind::Colon)).then(expr.clone()).repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::LBrace))
    .then(super::expr::stmts_block(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(fields, body), e| SExpr::new(Expr::With(ExprWith { kind: WithKind::Context { fields }, body }), ss(e.span())));

  let with_binding = just(TokenKind::With)
    .ignore_then(just(TokenKind::Ident(intern("mut"))).to(true).or_not().map(|x| x.unwrap_or(false)))
    .then(ident())
    .then(just(TokenKind::DeclMut).to(true).or(just(TokenKind::Assign).to(false)))
    .then(expr.clone())
    .then_ignore(just(TokenKind::LBrace))
    .then(super::expr::stmts_block(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|((((explicit_mut, name), is_decl_mut), value), body), e| {
      SExpr::new(Expr::With(ExprWith { kind: WithKind::Binding { name, value: Box::new(value), mutable: explicit_mut || is_decl_mut }, body }), ss(e.span()))
    });

  let resource = expr.clone().then_ignore(just(TokenKind::As)).then(ident());

  let with_resource = just(TokenKind::With)
    .ignore_then(resource.separated_by(just(TokenKind::Semi)).at_least(1).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::LBrace))
    .then(super::expr::stmts_block(expr))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(resources, body), e| SExpr::new(Expr::With(ExprWith { kind: WithKind::Resources { resources }, body }), ss(e.span())));

  choice((with_context, with_binding, with_resource))
}

pub(super) fn paren_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let unit = just(TokenKind::LParen).then(just(TokenKind::RParen)).map_with(|_, e| SExpr::new(Expr::Literal(crate::ast::Literal::Unit), ss(e.span())));

  let field_section = just(TokenKind::LParen)
    .ignore_then(just(TokenKind::Dot))
    .ignore_then(ident())
    .then_ignore(just(TokenKind::RParen))
    .map_with(|name, e| SExpr::new(Expr::Section(Section::Field(name)), ss(e.span())));

  let index_section =
    just(TokenKind::LParen).ignore_then(just(TokenKind::Dot)).ignore_then(select! { TokenKind::Int(n) => n }).then_ignore(just(TokenKind::RParen)).map_with(
      |n, e| {
        let idx: i64 = n.try_into().unwrap_or(0);
        SExpr::new(Expr::Section(Section::Index(idx)), ss(e.span()))
      },
    );

  let binop_section = just(TokenKind::LParen).ignore_then(section_op()).then_ignore(just(TokenKind::RParen)).map_with(|op_tok, e| {
    let op = tok_to_op(&op_tok);
    SExpr::new(Expr::Section(Section::BinOp(op)), ss(e.span()))
  });

  let right_section =
    just(TokenKind::LParen).ignore_then(section_op()).then(expr.clone()).then_ignore(just(TokenKind::RParen)).map_with(|(op_tok, operand), e| {
      let op = tok_to_op(&op_tok);
      SExpr::new(Expr::Section(Section::Right { op, operand: Box::new(operand) }), ss(e.span()))
    });

  let left_section =
    just(TokenKind::LParen).ignore_then(expr.clone()).then(section_op()).then_ignore(just(TokenKind::RParen)).map_with(|(operand, op_tok), e| {
      let op = tok_to_op(&op_tok);
      SExpr::new(Expr::Section(Section::Left { operand: Box::new(operand), op }), ss(e.span()))
    });

  let param = super::expr::param_parser(expr.clone());
  let func_def = just(TokenKind::LParen)
    .ignore_then(param.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .then(just(TokenKind::Arrow).ignore_then(super::type_ann::type_parser()).or_not())
    .then(just(TokenKind::Amp).ignore_then(expr.clone().delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))).or_not())
    .then(expr.clone())
    .map_with(|(((params, ret_type), guard), body), e| {
      SExpr::new(Expr::Func(ExprFunc { params, ret_type, guard: guard.map(Box::new), body: Box::new(body) }), ss(e.span()))
    });

  let tuple = just(TokenKind::LParen)
    .ignore_then(expr.clone().separated_by(just(TokenKind::Semi).or_not()).at_least(2).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .map_with(|elems, e| SExpr::new(Expr::Tuple(elems), ss(e.span())));

  let grouped = just(TokenKind::LParen).ignore_then(expr).then_ignore(just(TokenKind::RParen)).map_with(|inner, e| SExpr::new(inner.node, ss(e.span())));

  choice((field_section, index_section, binop_section, right_section, func_def, unit, left_section, tuple, grouped))
}
