use super::Lexer;
use crate::error::LxError;
use crate::span::Span;
use crate::token::{Token, TokenKind};
use num_bigint::BigInt;

impl<'src> Lexer<'src> {
    pub(super) fn read_number(&mut self, start: usize) -> Result<Token, LxError> {
        if self.current_char(start) == '0' {
            match self.peek() {
                Some('x' | 'X') => {
                    return self.read_radix_int(start, 16, |c| c.is_ascii_hexdigit());
                }
                Some('b' | 'B') => return self.read_radix_int(start, 2, |c| c == '0' || c == '1'),
                Some('o' | 'O') => {
                    return self.read_radix_int(start, 8, |c| ('0'..='7').contains(&c));
                }
                _ => {}
            }
        }
        self.read_decimal(start)
    }

    fn current_char(&self, pos: usize) -> char {
        self.source[pos..].chars().next().unwrap_or('\0')
    }

    fn read_radix_int(
        &mut self,
        start: usize,
        radix: u32,
        valid: fn(char) -> bool,
    ) -> Result<Token, LxError> {
        self.advance();
        let digit_start = self.pos;
        while self.peek().is_some_and(|c| valid(c) || c == '_') {
            self.advance();
        }
        let raw: String = self.source[digit_start..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();
        if raw.is_empty() {
            let span = Span::from_range(start as u32, self.pos as u32);
            return Err(LxError::parse(
                "expected digits after base prefix",
                span,
                None,
            ));
        }
        let value = BigInt::parse_bytes(raw.as_bytes(), radix).ok_or_else(|| {
            let span = Span::from_range(start as u32, self.pos as u32);
            LxError::parse("invalid integer literal", span, None)
        })?;
        Ok(Token::new(
            TokenKind::Int(value),
            Span::from_range(start as u32, self.pos as u32),
        ))
    }

    fn read_decimal(&mut self, start: usize) -> Result<Token, LxError> {
        while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
            self.advance();
        }
        let mut is_float = false;
        if self.peek() == Some('.') && self.peek_ahead(1).is_some_and(|c| c.is_ascii_digit()) {
            is_float = true;
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
                self.advance();
            }
        }
        if self.peek().is_some_and(|c| c == 'e' || c == 'E') {
            is_float = true;
            self.advance();
            if self.peek().is_some_and(|c| c == '+' || c == '-') {
                self.advance();
            }
            if !self.peek().is_some_and(|c| c.is_ascii_digit()) {
                let span = Span::from_range(start as u32, self.pos as u32);
                return Err(LxError::parse("expected digits in exponent", span, None));
            }
            while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
                self.advance();
            }
        }
        let span = Span::from_range(start as u32, self.pos as u32);
        let raw: String = self.source[start..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();
        if is_float {
            let val: f64 = raw
                .parse()
                .map_err(|_| LxError::parse("invalid float literal", span, None))?;
            Ok(Token::new(TokenKind::Float(val), span))
        } else {
            let val: BigInt = raw
                .parse()
                .map_err(|_| LxError::parse("invalid integer literal", span, None))?;
            Ok(Token::new(TokenKind::Int(val), span))
        }
    }
}
