use crate::ast::{Expr, ListElem, MapEntry, RecordField, SExpr, SStmt};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn parse_list(&mut self, start: u32) -> Result<SExpr, LxError> {
        let mut elems = Vec::new();
        self.collection_depth += 1;
        while *self.peek() != TokenKind::RBracket {
            if *self.peek() == TokenKind::DotDot {
                self.advance();
                elems.push(ListElem::Spread(self.parse_expr(0)?));
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

    pub(super) fn parse_block_or_record(&mut self, start: u32) -> Result<SExpr, LxError> {
        self.skip_semis();
        if *self.peek() == TokenKind::Colon
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|t| t.kind == TokenKind::RBrace)
        {
            self.advance();
            let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
            return Ok(SExpr::new(
                Expr::Record(vec![]),
                Span::from_range(start, end),
            ));
        }
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
                let saved_depth = self.collection_depth;
                let saved_app = self.application_depth;
                self.collection_depth = 0;
                self.application_depth = 0;
                self.record_field_depth += 1;
                let value = self.parse_expr(31)?;
                self.record_field_depth -= 1;
                self.collection_depth = saved_depth;
                self.application_depth = saved_app;
                fields.push(RecordField {
                    name: None,
                    value,
                    is_spread: true,
                });
            } else {
                let name_span = self.tokens[self.pos].span;
                let name = self.expect_ident("record field")?;
                if *self.peek() == TokenKind::Colon {
                    self.advance();
                    let saved_depth = self.collection_depth;
                    let saved_app = self.application_depth;
                    self.collection_depth = 0;
                    self.application_depth = 0;
                    self.record_field_depth += 1;
                    let value = self.parse_expr(0)?;
                    self.record_field_depth -= 1;
                    self.collection_depth = saved_depth;
                    self.application_depth = saved_app;
                    fields.push(RecordField {
                        name: Some(name),
                        value,
                        is_spread: false,
                    });
                } else {
                    let value = SExpr::new(Expr::Ident(name.clone()), name_span);
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

    pub(super) fn parse_map(&mut self, start: u32) -> Result<SExpr, LxError> {
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
}
