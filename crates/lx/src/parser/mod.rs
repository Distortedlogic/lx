mod expr;
mod pattern;
mod stmt;
mod type_ann;

use chumsky::prelude::*;
use chumsky::input::Stream;
use miette::SourceSpan;

use crate::ast::*;
use crate::error::LxError;
use crate::lexer::token::{Token, TokenKind};

type Spn = SimpleSpan<usize>;
type TInput<'a> = Stream<std::vec::IntoIter<(TokenKind, Spn)>>;

fn ss(s: Spn) -> SourceSpan {
    (s.start, s.end - s.start).into()
}

fn merge(a: SourceSpan, b: SourceSpan) -> SourceSpan {
    let start = a.offset();
    let end = b.offset() + b.len();
    (start, end - start).into()
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
    let eoi = SimpleSpan::new(len, len);

    let spanned: Vec<(TokenKind, Spn)> = tokens
        .into_iter()
        .map(|t| {
            let start = t.span.offset();
            let len = t.span.len();
            (t.kind, SimpleSpan::new(start, start + len))
        })
        .collect();

    let stream = Stream::from_iter(spanned).with_span(eoi);

    let result = stmt::program_parser().parse(stream);
    match result.into_result() {
        Ok(prog) => Ok(prog),
        Err(errs) => {
            let e = &errs[0];
            let sp: SourceSpan =
                (e.span().start, e.span().end.saturating_sub(e.span().start)).into();
            Err(LxError::parse(format!("{e}"), sp, None))
        },
    }
}
