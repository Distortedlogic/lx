use crate::ast::{BinOp, Expr, FieldKind, Literal, MatchArm, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

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

impl super::Parser {
    pub(super) fn parse_infix(
        &mut self,
        left: SExpr,
        kind: &TokenKind,
        rbp: u8,
    ) -> Result<SExpr, LxError> {
        let start = left.span.offset;
        self.advance();
        self.skip_semis();
        match kind {
            TokenKind::Pipe => {
                let right = self.parse_expr(rbp)?;
                let span = Span::from_range(start, right.span.end());
                Ok(SExpr::new(
                    Expr::Pipe {
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                ))
            }
            TokenKind::TildeArrow => {
                let right = self.parse_expr(rbp)?;
                let span = Span::from_range(start, right.span.end());
                Ok(SExpr::new(
                    Expr::AgentSend {
                        target: Box::new(left),
                        msg: Box::new(right),
                    },
                    span,
                ))
            }
            TokenKind::TildeArrowQ => {
                let right = self.parse_expr(rbp)?;
                let span = Span::from_range(start, right.span.end());
                Ok(SExpr::new(
                    Expr::AgentAsk {
                        target: Box::new(left),
                        msg: Box::new(right),
                    },
                    span,
                ))
            }
            TokenKind::QQ => {
                let right = self.parse_expr(rbp)?;
                let span = Span::from_range(start, right.span.end());
                Ok(SExpr::new(
                    Expr::Coalesce {
                        expr: Box::new(left),
                        default: Box::new(right),
                    },
                    span,
                ))
            }
            TokenKind::Dot => self.parse_dot(left, start),
            TokenKind::Question => self.parse_question(left, start),
            _ => {
                if let Some(op) = super::token_to_binop(kind) {
                    let right = self.parse_expr(rbp)?;
                    let span = Span::from_range(start, right.span.end());
                    Ok(SExpr::new(
                        Expr::Binary {
                            op,
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                        span,
                    ))
                } else {
                    let sp = self.tokens[self.pos.saturating_sub(1)].span;
                    Err(LxError::parse(
                        format!("unexpected infix token: {kind:?}"),
                        sp,
                        None,
                    ))
                }
            }
        }
    }

    fn parse_dot(&mut self, left: SExpr, start: u32) -> Result<SExpr, LxError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Ident(name) => {
                let span = Span::from_range(start, tok.span.end());
                Ok(SExpr::new(
                    Expr::FieldAccess {
                        expr: Box::new(left),
                        field: FieldKind::Named(name),
                    },
                    span,
                ))
            }
            TokenKind::Int(ref n) => {
                if *self.peek() == TokenKind::DotDot {
                    let start_expr = SExpr::new(Expr::Literal(Literal::Int(n.clone())), tok.span);
                    self.advance();
                    let end_expr = if matches!(self.peek(), TokenKind::Int(_)) {
                        let end_tok = self.advance().clone();
                        let TokenKind::Int(end_n) = end_tok.kind else {
                            unreachable!()
                        };
                        Some(Box::new(SExpr::new(
                            Expr::Literal(Literal::Int(end_n)),
                            end_tok.span,
                        )))
                    } else {
                        None
                    };
                    let end_pos = end_expr
                        .as_ref()
                        .map(|e| e.span.end())
                        .unwrap_or(tok.span.end() + 2);
                    let span = Span::from_range(start, end_pos);
                    return Ok(SExpr::new(
                        Expr::Slice {
                            expr: Box::new(left),
                            start: Some(Box::new(start_expr)),
                            end: end_expr,
                        },
                        span,
                    ));
                }
                let idx: i64 = n
                    .try_into()
                    .map_err(|_| LxError::parse("field index too large", tok.span, None))?;
                let span = Span::from_range(start, tok.span.end());
                Ok(SExpr::new(
                    Expr::FieldAccess {
                        expr: Box::new(left),
                        field: FieldKind::Index(idx),
                    },
                    span,
                ))
            }
            TokenKind::DotDot => {
                let end_expr = if matches!(self.peek(), TokenKind::Int(_)) {
                    let end_tok = self.advance().clone();
                    let TokenKind::Int(end_n) = end_tok.kind else {
                        unreachable!()
                    };
                    Some(Box::new(SExpr::new(
                        Expr::Literal(Literal::Int(end_n)),
                        end_tok.span,
                    )))
                } else {
                    None
                };
                let end_pos = end_expr
                    .as_ref()
                    .map(|e| e.span.end())
                    .unwrap_or(tok.span.end());
                let span = Span::from_range(start, end_pos);
                Ok(SExpr::new(
                    Expr::Slice {
                        expr: Box::new(left),
                        start: None,
                        end: end_expr,
                    },
                    span,
                ))
            }
            TokenKind::Minus => {
                let num_tok = self.advance().clone();
                match num_tok.kind {
                    TokenKind::Int(n) => {
                        let idx: i64 = n.try_into().map_err(|_| {
                            LxError::parse("field index too large", num_tok.span, None)
                        })?;
                        let span = Span::from_range(start, num_tok.span.end());
                        Ok(SExpr::new(
                            Expr::FieldAccess {
                                expr: Box::new(left),
                                field: FieldKind::Index(-idx),
                            },
                            span,
                        ))
                    }
                    _ => Err(LxError::parse(
                        "expected integer after '-' in field access",
                        num_tok.span,
                        None,
                    )),
                }
            }
            TokenKind::LBracket => {
                let key_expr = self.parse_expr(0)?;
                let end = self.expect_kind(&TokenKind::RBracket)?.span.end();
                let span = Span::from_range(start, end);
                Ok(SExpr::new(
                    Expr::FieldAccess {
                        expr: Box::new(left),
                        field: FieldKind::Computed(Box::new(key_expr)),
                    },
                    span,
                ))
            }
            TokenKind::StrStart => {
                let key_expr = self.parse_string(tok.span.offset)?;
                let end = key_expr.span.end();
                let span = Span::from_range(start, end);
                Ok(SExpr::new(
                    Expr::FieldAccess {
                        expr: Box::new(left),
                        field: FieldKind::Computed(Box::new(key_expr)),
                    },
                    span,
                ))
            }
            _ => Err(LxError::parse(
                "expected field name or index after '.'",
                tok.span,
                None,
            )),
        }
    }

    pub(super) fn parse_postfix(&mut self, left: SExpr) -> Result<SExpr, LxError> {
        let tok = self.advance();
        let span = Span::from_range(left.span.offset, tok.span.end());
        Ok(SExpr::new(Expr::Propagate(Box::new(left)), span))
    }

    pub(super) fn parse_question(
        &mut self,
        scrutinee: SExpr,
        start: u32,
    ) -> Result<SExpr, LxError> {
        if *self.peek() == TokenKind::LBrace {
            self.advance();
            let mut arms = Vec::new();
            self.skip_semis();
            while *self.peek() != TokenKind::RBrace {
                let pattern = self.parse_pattern()?;
                let guard = if *self.peek() == TokenKind::Amp {
                    self.advance();
                    Some(self.parse_expr(8)?)
                } else {
                    None
                };
                self.expect_kind(&TokenKind::Arrow)?;
                let body = self.parse_expr(0)?;
                arms.push(MatchArm {
                    pattern,
                    guard,
                    body,
                });
                self.skip_semis();
            }
            let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
            Ok(SExpr::new(
                Expr::Match {
                    scrutinee: Box::new(scrutinee),
                    arms,
                },
                Span::from_range(start, end),
            ))
        } else {
            let then_ = self.parse_expr(4)?;
            let (else_, end) = if *self.peek() == TokenKind::Colon {
                self.advance();
                let e = self.parse_expr(4)?;
                let end = e.span.end();
                (Some(Box::new(e)), end)
            } else {
                (None, then_.span.end())
            };
            let span = Span::from_range(start, end);
            Ok(SExpr::new(
                Expr::Ternary {
                    cond: Box::new(scrutinee),
                    then_: Box::new(then_),
                    else_,
                },
                span,
            ))
        }
    }
}
