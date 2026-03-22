use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::ident;
use super::expr_pratt::{section_op, tok_to_op};
use super::{ArenaRef, ExprId, Span, ss};
use crate::ast::{Expr, ExprFunc, ExprWith, Literal, Section, WithKind};
use crate::lexer::token::TokenKind;
use crate::sym::intern;

pub(super) fn with_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let a1 = arena.clone();
  let a2 = arena.clone();
  let a3 = arena.clone();

  let with_context = just(TokenKind::With)
    .ignore_then(just(TokenKind::Ident(intern("context"))))
    .ignore_then(ident().then_ignore(just(TokenKind::Colon)).then(expr.clone()).repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::LBrace))
    .then(super::expr::stmts_block(expr.clone(), arena.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(move |(fields, body), e| a1.borrow_mut().alloc_expr(Expr::With(ExprWith { kind: WithKind::Context { fields }, body }), ss(e.span())));

  let with_binding = just(TokenKind::With)
    .ignore_then(just(TokenKind::Ident(intern("mut"))).to(true).or_not().map(|x| x.unwrap_or(false)))
    .then(ident())
    .then(just(TokenKind::DeclMut).to(true).or(just(TokenKind::Assign).to(false)))
    .then(expr.clone())
    .then_ignore(just(TokenKind::LBrace))
    .then(super::expr::stmts_block(expr.clone(), arena.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(move |((((explicit_mut, name), is_decl_mut), value), body), e| {
      a2.borrow_mut().alloc_expr(Expr::With(ExprWith { kind: WithKind::Binding { name, value, mutable: explicit_mut || is_decl_mut }, body }), ss(e.span()))
    });

  let resource = expr.clone().then_ignore(just(TokenKind::As)).then(ident());

  let with_resource = just(TokenKind::With)
    .ignore_then(resource.separated_by(just(TokenKind::Semi)).at_least(1).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::LBrace))
    .then(super::expr::stmts_block(expr, arena))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(move |(resources, body), e| a3.borrow_mut().alloc_expr(Expr::With(ExprWith { kind: WithKind::Resources { resources }, body }), ss(e.span())));

  choice((with_context, with_binding, with_resource))
}

pub(super) fn paren_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
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

  let unit = just(TokenKind::LParen).then(just(TokenKind::RParen)).map_with(move |_, e| a1.borrow_mut().alloc_expr(Expr::Literal(Literal::Unit), ss(e.span())));

  let field_section = just(TokenKind::LParen)
    .ignore_then(just(TokenKind::Dot))
    .ignore_then(ident())
    .then_ignore(just(TokenKind::RParen))
    .map_with(move |name, e| a2.borrow_mut().alloc_expr(Expr::Section(Section::Field(name)), ss(e.span())));

  let index_section =
    just(TokenKind::LParen).ignore_then(just(TokenKind::Dot)).ignore_then(select! { TokenKind::Int(n) => n }).then_ignore(just(TokenKind::RParen)).map_with(
      move |n, e| {
        let idx: i64 = n.try_into().unwrap_or(0);
        a3.borrow_mut().alloc_expr(Expr::Section(Section::Index(idx)), ss(e.span()))
      },
    );

  let binop_section = just(TokenKind::LParen).ignore_then(section_op()).then_ignore(just(TokenKind::RParen)).map_with(move |op_tok, e| {
    let op = tok_to_op(&op_tok);
    a4.borrow_mut().alloc_expr(Expr::Section(Section::BinOp(op)), ss(e.span()))
  });

  let right_section =
    just(TokenKind::LParen).ignore_then(section_op()).then(expr.clone()).then_ignore(just(TokenKind::RParen)).map_with(move |(op_tok, operand), e| {
      let op = tok_to_op(&op_tok);
      a5.borrow_mut().alloc_expr(Expr::Section(Section::Right { op, operand }), ss(e.span()))
    });

  let left_section =
    just(TokenKind::LParen).ignore_then(expr.clone()).then(section_op()).then_ignore(just(TokenKind::RParen)).map_with(move |(operand, op_tok), e| {
      let op = tok_to_op(&op_tok);
      a6.borrow_mut().alloc_expr(Expr::Section(Section::Left { operand, op }), ss(e.span()))
    });

  let param = super::expr_helpers::param_parser(expr.clone(), arena.clone());
  let func_def = just(TokenKind::LParen)
    .ignore_then(param.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .then(just(TokenKind::Arrow).ignore_then(super::type_ann::type_parser(arena.clone())).or_not())
    .then(just(TokenKind::Amp).ignore_then(expr.clone().delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))).or_not())
    .then(expr.clone())
    .map_with(move |(((params, ret_type), guard), body), e| a7.borrow_mut().alloc_expr(Expr::Func(ExprFunc { params, ret_type, guard, body }), ss(e.span())));

  let tuple = just(TokenKind::LParen)
    .ignore_then(expr.clone().separated_by(just(TokenKind::Semi).or_not()).at_least(2).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .map_with(move |elems, e| a8.borrow_mut().alloc_expr(Expr::Tuple(elems), ss(e.span())));

  let grouped = just(TokenKind::LParen).ignore_then(expr).then_ignore(just(TokenKind::RParen)).map_with(move |inner, e| {
    let node = arena.borrow().expr(inner).clone();
    arena.borrow_mut().alloc_expr(node, ss(e.span()))
  });

  choice((field_section, index_section, binop_section, right_section, func_def, unit, left_section, tuple, grouped))
}
