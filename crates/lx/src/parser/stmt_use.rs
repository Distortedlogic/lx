use crate::ast::{SStmt, Stmt, UseKind, UseStmt};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn parse_use_stmt(&mut self, start: u32) -> Result<SStmt, LxError> {
        self.advance();
        let mut path = Vec::new();
        if *self.peek() == TokenKind::DotDot {
            self.advance();
            path.push("..".to_string());
            self.expect_kind(&TokenKind::Slash)?;
        } else if *self.peek() == TokenKind::Dot
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|t| t.kind == TokenKind::Slash)
        {
            self.advance();
            path.push(".".to_string());
            self.expect_kind(&TokenKind::Slash)?;
        }
        while let TokenKind::Ident(name) = self.peek().clone() {
            self.advance();
            path.push(name);
            if *self.peek() == TokenKind::Slash {
                self.advance();
            } else {
                break;
            }
        }
        if path.is_empty() {
            return Err(LxError::parse(
                "expected module path after 'use'",
                self.tokens[self.pos].span,
                None,
            ));
        }
        let kind = if *self.peek() == TokenKind::Colon {
            self.advance();
            UseKind::Alias(self.expect_ident("use alias")?)
        } else if *self.peek() == TokenKind::LBrace {
            self.advance();
            let mut names = Vec::new();
            while *self.peek() != TokenKind::RBrace {
                match self.peek().clone() {
                    TokenKind::Ident(name) | TokenKind::TypeName(name) => {
                        self.advance();
                        names.push(name);
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected name in selective import",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                }
                self.skip_semis();
            }
            self.expect_kind(&TokenKind::RBrace)?;
            UseKind::Selective(names)
        } else {
            UseKind::Whole
        };
        let end = self.tokens[self.pos.saturating_sub(1)].span.end();
        Ok(SStmt::new(
            Stmt::Use(UseStmt { path, kind }),
            Span::from_range(start, end),
        ))
    }
}
