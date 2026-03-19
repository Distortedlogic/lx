mod func;
mod helpers;
mod infix;
mod paren;
mod pattern;
mod prefix;
mod prefix_coll;
mod prefix_with;
mod receive;
mod refine;
mod statements;
mod stmt_agent;
mod stmt_class;
mod stmt_mcp;
mod stmt_protocol;
mod stmt_trait;
mod stmt_use;
mod type_ann;

use helpers::{infix_bp, looks_like_record, postfix_bp};
pub(crate) use infix::token_to_binop;

use crate::ast::{Expr, Program, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub fn parse(tokens: Vec<Token>) -> Result<Program, LxError> {
    Parser {
        tokens,
        pos: 0,
        no_juxtapose: false,
        collection_depth: 0,
        record_field_depth: 0,
        application_depth: 0,
        stop_ident: None,
    }
    .parse_program()
}

pub(crate) struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
    pub(crate) no_juxtapose: bool,
    pub(crate) collection_depth: u32,
    pub(crate) record_field_depth: u32,
    pub(crate) application_depth: u32,
    pub(crate) stop_ident: Option<String>,
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program, LxError> {
        self.skip_semis();
        let mut stmts = Vec::new();
        while *self.peek() != TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
            self.skip_semis();
        }
        Ok(Program { stmts })
    }

    pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<SExpr, LxError> {
        let mut left = self.parse_prefix()?;
        loop {
            if let Some(ref stop) = self.stop_ident
                && *self.peek() == TokenKind::Ident(stop.clone())
            {
                break;
            }
            let kind = self.peek().clone();
            if matches!(
                kind,
                TokenKind::Eof
                    | TokenKind::RParen
                    | TokenKind::RBracket
                    | TokenKind::RBrace
                    | TokenKind::Colon
                    | TokenKind::Arrow
                    | TokenKind::Assign
                    | TokenKind::DeclMut
                    | TokenKind::Reassign
            ) {
                break;
            }
            if kind == TokenKind::Semi {
                if let Some(next) = self.tokens.get(self.pos + 1)
                    && matches!(
                        next.kind,
                        TokenKind::Pipe
                            | TokenKind::QQ
                            | TokenKind::Caret
                            | TokenKind::Dot
                            | TokenKind::Question
                            | TokenKind::Plus
                            | TokenKind::Minus
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
                            | TokenKind::DotDot
                            | TokenKind::DotDotEq
                            | TokenKind::Amp
                            | TokenKind::TildeArrow
                            | TokenKind::TildeArrowQ
                    )
                {
                    self.advance();
                    continue;
                }
                break;
            }
            if let Some(lbp) = postfix_bp(&kind) {
                if lbp < min_bp {
                    break;
                }
                left = self.parse_postfix(left)?;
                continue;
            }
            if self.is_application_candidate(&left, min_bp) {
                if self.record_field_depth > 0
                    && matches!(self.peek(), TokenKind::Ident(_))
                    && self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Colon)
                {
                    break;
                }
                if matches!(self.peek(), TokenKind::Ident(_))
                    && self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Colon)
                {
                    let tok = self.advance().clone();
                    let TokenKind::Ident(name) = tok.kind else {
                        unreachable!()
                    };
                    self.advance();
                    let value = self.parse_expr(32)?;
                    let arg_span = Span::from_range(tok.span.offset, value.span.end());
                    let arg = SExpr::new(
                        Expr::NamedArg {
                            name,
                            value: Box::new(value),
                        },
                        arg_span,
                    );
                    let span = Span::from_range(left.span.offset, arg.span.end());
                    left = SExpr::new(
                        Expr::Apply {
                            func: Box::new(left),
                            arg: Box::new(arg),
                        },
                        span,
                    );
                } else {
                    self.application_depth += 1;
                    let arg = self.parse_expr(32)?;
                    self.application_depth -= 1;
                    let span = Span::from_range(left.span.offset, arg.span.end());
                    left = SExpr::new(
                        Expr::Apply {
                            func: Box::new(left),
                            arg: Box::new(arg),
                        },
                        span,
                    );
                }
                continue;
            }
            if let Some((lbp, rbp)) = infix_bp(&kind) {
                if lbp < min_bp {
                    break;
                }
                if self.collection_depth > 0
                    && matches!(kind, TokenKind::DotDot | TokenKind::DotDotEq)
                {
                    break;
                }
                if self
                    .tokens
                    .get(self.pos + 1)
                    .is_some_and(|t| t.kind == TokenKind::RParen)
                {
                    break;
                }
                left = self.parse_infix(left, &kind, rbp)?;
                continue;
            }
            break;
        }
        Ok(left)
    }

    pub(crate) fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    pub(crate) fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if tok.kind != TokenKind::Eof {
            self.pos += 1;
        }
        tok
    }

    pub(crate) fn expect_kind(&mut self, expected: &TokenKind) -> Result<&Token, LxError> {
        let tok = &self.tokens[self.pos];
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            Err(LxError::parse(
                format!("expected {expected:?}, found {:?}", tok.kind),
                tok.span,
                None,
            ))
        }
    }

    pub(crate) fn skip_semis(&mut self) {
        while *self.peek() == TokenKind::Semi {
            self.advance();
        }
    }

    pub(crate) fn expect_ident(&mut self, context: &str) -> Result<String, LxError> {
        let tok = &self.tokens[self.pos];
        if let TokenKind::Ident(name) = &tok.kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(LxError::parse(
                format!("expected identifier in {context}, found {:?}", tok.kind),
                tok.span,
                None,
            ))
        }
    }

    pub(crate) fn expect_type_name(&mut self, context: &str) -> Result<String, LxError> {
        let tok = &self.tokens[self.pos];
        if let TokenKind::TypeName(name) = &tok.kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(LxError::parse(
                format!("expected type name in {context}, found {:?}", tok.kind),
                tok.span,
                None,
            ))
        }
    }
}
