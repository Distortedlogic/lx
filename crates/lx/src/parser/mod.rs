mod expr;
mod expr_pratt;
mod pattern;
mod stmt;
mod stmt_class;
mod type_ann;

use chumsky::input::{Input as _, Stream};
use chumsky::prelude::*;
use miette::SourceSpan;

use crate::ast::{BinOp, Program};
use crate::error::LxError;
use crate::lexer::token::{Token, TokenKind};

type Span = SimpleSpan;

fn ss(s: Span) -> SourceSpan {
  (s.start, s.end - s.start).into()
}

pub(crate) fn token_to_binop(kind: &TokenKind) -> Option<BinOp> {
  match kind {
    TokenKind::Plus => Some(BinOp::Add),
    TokenKind::Minus => Some(BinOp::Sub),
    TokenKind::Star => Some(BinOp::Mul),
    TokenKind::Slash => Some(BinOp::Div),
    TokenKind::Percent => Some(BinOp::Mod),
    TokenKind::IntDiv => Some(BinOp::IntDiv),
    TokenKind::PlusPlus => Some(BinOp::Concat),
    TokenKind::DotDot => Some(BinOp::Range),
    TokenKind::DotDotEq => Some(BinOp::RangeInclusive),
    TokenKind::Eq => Some(BinOp::Eq),
    TokenKind::NotEq => Some(BinOp::NotEq),
    TokenKind::Lt => Some(BinOp::Lt),
    TokenKind::Gt => Some(BinOp::Gt),
    TokenKind::LtEq => Some(BinOp::LtEq),
    TokenKind::GtEq => Some(BinOp::GtEq),
    TokenKind::And => Some(BinOp::And),
    TokenKind::Or => Some(BinOp::Or),
    _ => None,
  }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, LxError> {
  let len = tokens.last().map(|t| t.span.offset() + t.span.len()).unwrap_or(0);
  let eoi: Span = (len..len).into();

  let spanned: Vec<(TokenKind, Span)> = tokens
    .into_iter()
    .map(|t| {
      let start = t.span.offset();
      let end = start + t.span.len();
      (t.kind, (start..end).into())
    })
    .collect();

  let input = Stream::from_iter(spanned).map(eoi, |(t, s)| (t, s));

  match stmt::program_parser().parse(input).into_result() {
    Ok(prog) => Ok(prog),
    Err(errs) => {
      let e = &errs[0];
      let sp = ss(*e.span());
      Err(LxError::parse(format!("{e:?}"), sp, None))
    },
  }
}
