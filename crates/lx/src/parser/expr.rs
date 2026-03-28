use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr_helpers::{block_or_record_parser, list_parser, map_parser};
use super::{ArenaRef, ExprId, Span, StmtId, ss};
use crate::ast::{Expr, ExprAssert, ExprBreak, ExprEmit, ExprLoop, ExprPar, ExprTimeout, ExprYield, Literal, SelArm};
use crate::lexer::token::TokenKind;
use crate::sym::{Sym, intern};

pub(super) fn ident<'a, I>() -> impl Parser<'a, I, Sym, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! { TokenKind::Ident(n) => n }
}

pub(super) fn ident_or_keyword<'a, I>() -> impl Parser<'a, I, Sym, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! {
      TokenKind::Ident(n) => n,
      TokenKind::Use => intern("use"),
      TokenKind::Loop => intern("loop"),
      TokenKind::Break => intern("break"),
      TokenKind::Par => intern("par"),
      TokenKind::Sel => intern("sel"),
      TokenKind::Assert => intern("assert"),
      TokenKind::Emit => intern("emit"),
      TokenKind::Yield => intern("yield"),
      TokenKind::With => intern("with"),
      TokenKind::Timeout => intern("timeout"),
      TokenKind::Spawn => intern("spawn"),
      TokenKind::Stop => intern("stop"),
      TokenKind::As => intern("as"),
  }
}

fn keyword_as_type_name<'a, I>() -> impl Parser<'a, I, Sym, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! {
      TokenKind::AgentKw => intern("Agent"),
      TokenKind::ToolKw => intern("Tool"),
      TokenKind::PromptKw => intern("Prompt"),
      TokenKind::StoreKw => intern("Store"),
      TokenKind::SessionKw => intern("Session"),
      TokenKind::GuardKw => intern("Guard"),
      TokenKind::WorkflowKw => intern("Workflow"),
      TokenKind::SchemaKw => intern("Schema"),
      TokenKind::McpKw => intern("Mcp"),
      TokenKind::CliKw => intern("Cli"),
      TokenKind::HttpKw => intern("Http"),
  }
}

pub(super) fn type_name<'a, I>() -> impl Parser<'a, I, Sym, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! { TokenKind::TypeName(n) => n }.or(keyword_as_type_name())
}

pub(super) fn name_or_type<'a, I>() -> impl Parser<'a, I, Sym, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! { TokenKind::Ident(n) => n, TokenKind::TypeName(n) => n }.or(keyword_as_type_name())
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

pub(super) fn item_sep<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  just(TokenKind::Semi).or(just(TokenKind::Comma)).repeated().at_least(1).ignored()
}

pub(super) fn skip_item_sep<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  just(TokenKind::Semi).or(just(TokenKind::Comma)).repeated().ignored()
}

pub(super) fn expr_parser<'a, I>(arena: ArenaRef) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  recursive(move |expr| {
    let a = arena.clone();
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
    let a12 = arena.clone();
    let a13 = arena.clone();

    let literal = select! {
        TokenKind::Int(n) => Literal::Int(n),
        TokenKind::Float(f) => Literal::Float(f),
        TokenKind::True => Literal::Bool(true),
        TokenKind::False => Literal::Bool(false),
        TokenKind::Unit => Literal::Unit,
        TokenKind::RawStr(s) => Literal::RawStr(s),
    }
    .map_with(move |lit, e| a.borrow_mut().alloc_expr(Expr::Literal(lit), ss(e.span())));

    let string_lit = super::expr_pratt::string_parser(expr.clone(), a2.clone());
    let ident_expr = ident().map_with(move |n, e| a3.borrow_mut().alloc_expr(Expr::Ident(n), ss(e.span())));
    let type_ctor = type_name().map_with(move |n, e| a4.borrow_mut().alloc_expr(Expr::TypeConstructor(n), ss(e.span())));
    let list = list_parser(expr.clone(), a5.clone());
    let block_or_record = block_or_record_parser(expr.clone(), a6.clone());
    let map = map_parser(expr.clone(), a7.clone());
    let paren = super::expr_pratt::paren_parser(expr.clone(), a8.clone());

    let loop_expr = {
      let al = a9.clone();
      just(TokenKind::Loop)
        .ignore_then(just(TokenKind::LBrace))
        .ignore_then(stmts_block(expr.clone(), a9.clone()))
        .then_ignore(just(TokenKind::RBrace))
        .map_with(move |stmts, e| al.borrow_mut().alloc_expr(Expr::Loop(ExprLoop { stmts }), ss(e.span())))
    };

    let par_expr = {
      let al = a10.clone();
      just(TokenKind::Par)
        .ignore_then(just(TokenKind::LBrace))
        .ignore_then(stmts_block(expr.clone(), a10.clone()))
        .then_ignore(just(TokenKind::RBrace))
        .map_with(move |stmts, e| al.borrow_mut().alloc_expr(Expr::Par(ExprPar { stmts }), ss(e.span())))
    };

    let sel_arm = expr.clone().then_ignore(just(TokenKind::Arrow)).then(expr.clone()).map(|(ex, handler)| SelArm { expr: ex, handler });

    let sel_expr = {
      let al = a11.clone();
      just(TokenKind::Sel)
        .ignore_then(just(TokenKind::LBrace))
        .ignore_then(skip_semis())
        .ignore_then(sel_arm.separated_by(semi_sep()).allow_trailing().collect::<Vec<_>>())
        .then_ignore(skip_semis())
        .then_ignore(just(TokenKind::RBrace))
        .map_with(move |arms, e| al.borrow_mut().alloc_expr(Expr::Sel(arms), ss(e.span())))
    };

    let break_expr = {
      let al = arena.clone();
      just(TokenKind::Break)
        .ignore_then(expr.clone().or_not())
        .map_with(move |val, e| al.borrow_mut().alloc_expr(Expr::Break(ExprBreak { value: val }), ss(e.span())))
    };

    let assert_expr = {
      let al = arena.clone();
      just(TokenKind::Assert).ignore_then(expr.clone()).map_with(move |ex, e| {
        let (cond, msg) = {
          let ar = al.borrow();
          if let Expr::Apply(app) = ar.expr(ex)
            && let Expr::Literal(Literal::Str(_)) = ar.expr(app.arg)
          {
            (app.func, Some(app.arg))
          } else {
            (ex, None)
          }
        };
        al.borrow_mut().alloc_expr(Expr::Assert(ExprAssert { expr: cond, msg }), ss(e.span()))
      })
    };

    let emit_expr = {
      let al = arena.clone();
      just(TokenKind::Emit).ignore_then(expr.clone()).map_with(move |v, e| al.borrow_mut().alloc_expr(Expr::Emit(ExprEmit { value: v }), ss(e.span())))
    };

    let yield_expr = {
      let al = arena.clone();
      just(TokenKind::Yield).ignore_then(expr.clone()).map_with(move |v, e| al.borrow_mut().alloc_expr(Expr::Yield(ExprYield { value: v }), ss(e.span())))
    };

    let timeout_ms = choice((literal.clone(), ident_expr.clone(), paren.clone()));

    let timeout_expr = {
      let al = arena.clone();
      just(TokenKind::Timeout)
        .ignore_then(timeout_ms)
        .then(expr.clone())
        .map_with(move |(ms, body), e| al.borrow_mut().alloc_expr(Expr::Timeout(ExprTimeout { ms, body }), ss(e.span())))
    };

    let with_expr = super::expr_pratt::with_parser(expr.clone(), arena.clone());

    let spawn_expr = {
      let al = a12.clone();
      just(TokenKind::Spawn)
        .ignore_then(type_name().map_with(move |n, e| al.borrow_mut().alloc_expr(Expr::TypeConstructor(n), ss(e.span()))))
        .map_with(move |class_eid, e| a13.borrow_mut().alloc_expr(Expr::Spawn(class_eid), ss(e.span())))
    };

    let stop_expr = {
      let al = a12;
      just(TokenKind::Stop).map_with(move |_, e| al.borrow_mut().alloc_expr(Expr::Stop, ss(e.span())))
    };

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
      spawn_expr,
      stop_expr,
      type_ctor,
      ident_expr,
    ))
    .or(timeout_expr)
    .boxed();

    super::expr_pratt::pratt_expr(atom, expr, arena.clone())
  })
}

pub(super) fn stmts_block<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, Vec<StmtId>, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  skip_semis().ignore_then(super::stmt::stmt_parser(expr, arena).separated_by(semi_sep()).allow_trailing().collect::<Vec<_>>().then_ignore(skip_semis()))
}
