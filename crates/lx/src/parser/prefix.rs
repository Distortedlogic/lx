use crate::ast::{
    Expr, ListElem, Literal, MapEntry, RecordField, SExpr, SStmt, SelArm, ShellMode, StrPart,
    UnaryOp,
};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(crate) fn parse_prefix(&mut self) -> Result<SExpr, LxError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Int(n) => Ok(SExpr::new(Expr::Literal(Literal::Int(n)), tok.span)),
            TokenKind::Float(f) => Ok(SExpr::new(Expr::Literal(Literal::Float(f)), tok.span)),
            TokenKind::True => Ok(SExpr::new(Expr::Literal(Literal::Bool(true)), tok.span)),
            TokenKind::False => Ok(SExpr::new(Expr::Literal(Literal::Bool(false)), tok.span)),
            TokenKind::Unit => Ok(SExpr::new(Expr::Literal(Literal::Unit), tok.span)),
            TokenKind::RawStr(s) => Ok(SExpr::new(Expr::Literal(Literal::RawStr(s)), tok.span)),
            TokenKind::Regex(s) => Ok(SExpr::new(Expr::Literal(Literal::Regex(s)), tok.span)),
            TokenKind::StrStart => self.parse_string(tok.span.offset),
            TokenKind::Ident(name) => Ok(SExpr::new(Expr::Ident(name), tok.span)),
            TokenKind::TypeName(name) => Ok(SExpr::new(Expr::TypeConstructor(name), tok.span)),
            TokenKind::LParen => self.parse_paren(tok.span.offset),
            TokenKind::LBracket => self.parse_list(tok.span.offset),
            TokenKind::LBrace => self.parse_block_or_record(tok.span.offset),
            TokenKind::PercentLBrace => self.parse_map(tok.span.offset),
            TokenKind::Minus => self.parse_unary(UnaryOp::Neg, tok.span.offset),
            TokenKind::Bang => self.parse_unary(UnaryOp::Not, tok.span.offset),
            TokenKind::Loop => {
                self.expect_kind(&TokenKind::LBrace)?;
                let stmts = self.parse_stmts_until_rbrace()?;
                let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
                Ok(SExpr::new(
                    Expr::Loop(stmts),
                    Span::from_range(tok.span.offset, end),
                ))
            }
            TokenKind::Par => {
                self.expect_kind(&TokenKind::LBrace)?;
                let stmts = self.parse_stmts_until_rbrace()?;
                let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
                Ok(SExpr::new(
                    Expr::Par(stmts),
                    Span::from_range(tok.span.offset, end),
                ))
            }
            TokenKind::Sel => {
                self.expect_kind(&TokenKind::LBrace)?;
                let arms = self.parse_sel_arms()?;
                let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
                Ok(SExpr::new(
                    Expr::Sel(arms),
                    Span::from_range(tok.span.offset, end),
                ))
            }
            TokenKind::Break => {
                let saved_nj = self.no_juxtapose;
                self.no_juxtapose = true;
                let val = if self.peek_is_expr_start() {
                    Some(Box::new(self.parse_expr(0)?))
                } else {
                    None
                };
                self.no_juxtapose = saved_nj;
                let end = val.as_ref().map(|v| v.span.end()).unwrap_or(tok.span.end());
                Ok(SExpr::new(
                    Expr::Break(val),
                    Span::from_range(tok.span.offset, end),
                ))
            }
            TokenKind::Assert => {
                let expr = self.parse_expr(0)?;
                let msg = if !matches!(
                    self.peek(),
                    TokenKind::Semi | TokenKind::Eof | TokenKind::RBrace
                ) && self.peek_is_expr_start()
                {
                    Some(Box::new(self.parse_expr(0)?))
                } else {
                    None
                };
                let end = msg
                    .as_ref()
                    .map(|m| m.span.end())
                    .unwrap_or(expr.span.end());
                Ok(SExpr::new(
                    Expr::Assert {
                        expr: Box::new(expr),
                        msg,
                    },
                    Span::from_range(tok.span.offset, end),
                ))
            }
            TokenKind::Yield => {
                let value = self.parse_expr(0)?;
                let end = value.span.end();
                Ok(SExpr::new(
                    Expr::Yield {
                        value: Box::new(value),
                    },
                    Span::from_range(tok.span.offset, end),
                ))
            }
            TokenKind::With => {
                let mutable = *self.peek() == TokenKind::Ident("mut".into());
                if mutable {
                    self.advance();
                }
                let name = match self.peek().clone() {
                    TokenKind::Ident(n) => {
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected name after 'with'",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                };
                let op = self.peek().clone();
                if op != TokenKind::Assign && op != TokenKind::DeclMut {
                    return Err(LxError::parse(
                        "expected '=' or ':=' in with",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
                let mutable = mutable || op == TokenKind::DeclMut;
                self.advance();
                let value = self.parse_expr(0)?;
                self.expect_kind(&TokenKind::LBrace)?;
                let body = self.parse_stmts_until_rbrace()?;
                let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
                Ok(SExpr::new(
                    Expr::With {
                        name,
                        value: Box::new(value),
                        body,
                        mutable,
                    },
                    Span::from_range(tok.span.offset, end),
                ))
            }
            TokenKind::Refine => self.parse_refine(tok.span.offset),
            TokenKind::Dollar => self.parse_shell(ShellMode::Normal, tok.span.offset),
            TokenKind::DollarCaret => self.parse_shell(ShellMode::Propagate, tok.span.offset),
            TokenKind::DollarBrace => self.parse_shell(ShellMode::Block, tok.span.offset),
            _ => Err(LxError::parse(
                format!("unexpected token: {:?}", tok.kind),
                tok.span,
                None,
            )),
        }
    }

    fn parse_unary(&mut self, op: UnaryOp, start: u32) -> Result<SExpr, LxError> {
        let operand = self.parse_expr(29)?;
        let span = Span::from_range(start, operand.span.end());
        Ok(SExpr::new(
            Expr::Unary {
                op,
                operand: Box::new(operand),
            },
            span,
        ))
    }

    fn parse_shell(&mut self, mode: ShellMode, start: u32) -> Result<SExpr, LxError> {
        let mut parts = Vec::new();
        loop {
            match self.peek().clone() {
                TokenKind::ShellText(s) => {
                    self.advance();
                    parts.push(StrPart::Text(s));
                }
                TokenKind::ShellEnd => {
                    let end = self.advance().span.end();
                    return Ok(SExpr::new(
                        Expr::Shell { mode, parts },
                        Span::from_range(start, end),
                    ));
                }
                TokenKind::Eof => {
                    return Ok(SExpr::new(
                        Expr::Shell { mode, parts },
                        Span::from_range(start, self.tokens[self.pos].span.end()),
                    ));
                }
                _ => {
                    let expr = self.parse_expr(0)?;
                    parts.push(StrPart::Interp(expr));
                }
            }
        }
    }

    fn parse_sel_arms(&mut self) -> Result<Vec<SelArm>, LxError> {
        let mut arms = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let expr = self.parse_expr(0)?;
            self.expect_kind(&TokenKind::Arrow)?;
            let handler = self.parse_expr(0)?;
            arms.push(SelArm { expr, handler });
            self.skip_semis();
        }
        Ok(arms)
    }

    pub(crate) fn parse_string(&mut self, start: u32) -> Result<SExpr, LxError> {
        let mut parts = Vec::new();
        loop {
            match self.peek().clone() {
                TokenKind::StrChunk(s) => {
                    self.advance();
                    parts.push(StrPart::Text(s));
                }
                TokenKind::StrEnd => {
                    let end = self.advance().span.end();
                    return Ok(SExpr::new(
                        Expr::Literal(Literal::Str(parts)),
                        Span::from_range(start, end),
                    ));
                }
                TokenKind::LBrace => {
                    self.advance();
                    let expr = self.parse_expr(0)?;
                    self.expect_kind(&TokenKind::RBrace)?;
                    parts.push(StrPart::Interp(expr));
                }
                TokenKind::Eof => {
                    return Err(LxError::parse(
                        "unterminated string",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
                _ => {
                    let expr = self.parse_expr(0)?;
                    parts.push(StrPart::Interp(expr));
                }
            }
        }
    }

    fn parse_list(&mut self, start: u32) -> Result<SExpr, LxError> {
        let mut elems = Vec::new();
        self.collection_depth += 1;
        while *self.peek() != TokenKind::RBracket {
            if *self.peek() == TokenKind::DotDot {
                self.advance();
                elems.push(ListElem::Spread(self.parse_expr(32)?));
            } else {
                elems.push(ListElem::Single(self.parse_expr(0)?));
            }
            if *self.peek() == TokenKind::Semi {
                self.advance();
            }
        }
        self.collection_depth -= 1;
        let end = self.expect_kind(&TokenKind::RBracket)?.span.end();
        Ok(SExpr::new(Expr::List(elems), Span::from_range(start, end)))
    }

    fn parse_block_or_record(&mut self, start: u32) -> Result<SExpr, LxError> {
        self.skip_semis();
        if super::looks_like_record(self) {
            return self.parse_record(start);
        }
        let stmts = self.parse_stmts_until_rbrace()?;
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SExpr::new(Expr::Block(stmts), Span::from_range(start, end)))
    }

    fn parse_record(&mut self, start: u32) -> Result<SExpr, LxError> {
        let mut fields = Vec::new();
        self.skip_semis();
        self.collection_depth += 1;
        while *self.peek() != TokenKind::RBrace {
            if *self.peek() == TokenKind::DotDot {
                self.advance();
                let value = self.parse_expr(32)?;
                fields.push(RecordField {
                    name: None,
                    value,
                    is_spread: true,
                });
            } else {
                let tok = self.advance().clone();
                let TokenKind::Ident(name) = tok.kind else {
                    return Err(LxError::parse("expected field name", tok.span, None));
                };
                if *self.peek() == TokenKind::Colon {
                    self.advance();
                    let value = self.parse_expr(0)?;
                    fields.push(RecordField {
                        name: Some(name),
                        value,
                        is_spread: false,
                    });
                } else {
                    let value = SExpr::new(Expr::Ident(name.clone()), tok.span);
                    fields.push(RecordField {
                        name: Some(name),
                        value,
                        is_spread: false,
                    });
                }
            }
            self.skip_semis();
        }
        self.collection_depth -= 1;
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SExpr::new(
            Expr::Record(fields),
            Span::from_range(start, end),
        ))
    }

    fn parse_map(&mut self, start: u32) -> Result<SExpr, LxError> {
        let mut entries = Vec::new();
        self.collection_depth += 1;
        while *self.peek() != TokenKind::RBrace {
            if *self.peek() == TokenKind::DotDot {
                self.advance();
                let value = self.parse_expr(32)?;
                entries.push(MapEntry {
                    key: None,
                    value,
                    is_spread: true,
                });
            } else {
                let key = self.parse_expr(0)?;
                self.expect_kind(&TokenKind::Colon)?;
                let value = self.parse_expr(0)?;
                entries.push(MapEntry {
                    key: Some(key),
                    value,
                    is_spread: false,
                });
            }
            self.skip_semis();
        }
        self.collection_depth -= 1;
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SExpr::new(Expr::Map(entries), Span::from_range(start, end)))
    }

    pub(crate) fn parse_stmts_until_rbrace(&mut self) -> Result<Vec<SStmt>, LxError> {
        self.skip_semis();
        let mut stmts = Vec::new();
        while *self.peek() != TokenKind::RBrace {
            stmts.push(self.parse_stmt()?);
            self.skip_semis();
        }
        Ok(stmts)
    }

    pub(crate) fn peek_is_expr_start(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::StrStart
                | TokenKind::RawStr(_)
                | TokenKind::Regex(_)
                | TokenKind::Ident(_)
                | TokenKind::TypeName(_)
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::LBrace
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Unit
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Loop
                | TokenKind::Break
                | TokenKind::Assert
                | TokenKind::Par
                | TokenKind::Sel
                | TokenKind::PercentLBrace
                | TokenKind::Yield
                | TokenKind::With
                | TokenKind::Refine
                | TokenKind::Dollar
                | TokenKind::DollarCaret
                | TokenKind::DollarBrace
        )
    }
}
