use crate::error::LxError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

use super::Lexer;

impl<'src> Lexer<'src> {
    pub(super) fn tok(&self, kind: TokenKind, start: usize) -> Token {
        Token::new(kind, Span::new(start as u32, 1))
    }

    pub(super) fn tok2(&self, kind: TokenKind, start: usize) -> Token {
        Token::new(kind, Span::from_range(start as u32, self.pos as u32))
    }

    pub(super) fn eat(
        &mut self,
        expected: char,
        yes: TokenKind,
        no: TokenKind,
        start: usize,
    ) -> Result<Option<Token>, LxError> {
        if self.peek() == Some(expected) {
            self.advance();
            Ok(Some(self.tok2(yes, start)))
        } else {
            Ok(Some(self.tok(no, start)))
        }
    }

    pub(super) fn read_ident_or_keyword(&mut self, start: usize) -> Result<Token, LxError> {
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '\'')
        {
            self.advance();
        }
        if self.peek() == Some('?') {
            self.advance();
        }
        let text = &self.source[start..self.pos];
        let span = Span::from_range(start as u32, self.pos as u32);
        let kind = super::keywords::ident_or_keyword(text);
        Ok(Token::new(kind, span))
    }

    pub(super) fn read_type_name(&mut self, start: usize) -> Result<Token, LxError> {
        while self.peek().is_some_and(|c| c.is_ascii_alphanumeric()) {
            self.advance();
        }
        let text = &self.source[start..self.pos];
        let span = Span::from_range(start as u32, self.pos as u32);
        let kind = super::keywords::type_name_or_keyword(text);
        Ok(Token::new(kind, span))
    }
}
