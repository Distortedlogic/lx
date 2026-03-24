mod expr;
mod expr_compound;
mod expr_helpers;
mod expr_pratt;
mod pattern;
mod stmt;
mod stmt_class;
mod type_ann;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use chumsky::input::{Input as _, Stream};
use chumsky::prelude::*;
use miette::SourceSpan;

use crate::ast::{AstArena, BinOp, ExprId, PatternId, Program, StmtId, Surface, TypeExprId};
use crate::error::LxError;
use crate::lexer::token::{Token, TokenKind};
use crate::source::{CommentStore, FileId};

type Span = SimpleSpan;
type ArenaRef = Rc<RefCell<AstArena>>;

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

pub struct ParseResult {
  pub program: Option<Program<Surface>>,
  pub errors: Vec<LxError>,
}

pub fn parse(tokens: Vec<Token>, file: FileId, comments: CommentStore, source: &str) -> ParseResult {
  parse_with_recovery(tokens, file, comments, source)
}

fn parse_with_recovery(tokens: Vec<Token>, file: FileId, comments: CommentStore, source: &str) -> ParseResult {
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
  let arena = Rc::new(RefCell::new(AstArena::new()));

  let (output, errs) = stmt::program_parser(arena.clone()).parse(input).into_output_errors();

  let errors: Vec<LxError> = errs.into_iter().map(|e| LxError::parse(format!("{e:?}"), ss(*e.span()), None)).collect();

  let program = output.map(|stmts| {
    let arena = Rc::try_unwrap(arena).expect("arena still borrowed").into_inner();
    let comment_map = crate::ast::attach_comments(&stmts, &arena, &comments, source);
    Program { stmts, arena, comments, comment_map, file, _phase: PhantomData }
  });

  ParseResult { program, errors }
}
