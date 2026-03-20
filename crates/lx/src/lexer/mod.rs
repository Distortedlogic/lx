mod helpers;
mod keywords;
mod numbers;
mod strings;

use crate::error::LxError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

struct Lexer<'src> {
    source: &'src str,
    pos: usize,
    tokens: Vec<Token>,
    depth: i32,
    last_was_semi: bool,
    brace_stack: Vec<bool>,
}

pub fn lex(source: &str) -> Result<Vec<Token>, LxError> {
    let mut lexer = Lexer {
        source,
        pos: 0,
        tokens: Vec::new(),
        depth: 0,
        last_was_semi: true,
        brace_stack: Vec::new(),
    };
    loop {
        lexer.skip_whitespace_and_comments();
        if lexer.pos >= source.len() {
            break;
        }
        if let Some(tok) = lexer.next_token()? {
            lexer.emit(tok);
        }
    }
    lexer.tokens.push(Token::new(
        TokenKind::Eof,
        Span::new(source.len() as u32, 0),
    ));
    Ok(lexer.tokens)
}

impl<'src> Lexer<'src> {
    fn advance(&mut self) -> Option<char> {
        let c = self.source[self.pos..].chars().next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.source[self.pos..].chars().nth(n)
    }

    fn emit(&mut self, tok: Token) {
        let is_semi = tok.kind == TokenKind::Semi;
        if is_semi && self.last_was_semi {
            return;
        }
        self.last_was_semi = is_semi;
        self.tokens.push(tok);
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        let tok = Token::new(kind, Span::from_range(start as u32, end as u32));
        self.emit(tok);
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap_or('\0');
                if c == ' ' || c == '\t' || c == '\r' {
                    self.pos += 1;
                } else if c == '\n' {
                    self.pos += 1;
                    if self.depth <= 0 {
                        let span = Span::new(self.pos as u32 - 1, 1);
                        self.emit(Token::new(TokenKind::Semi, span));
                    }
                } else {
                    break;
                }
            }
            if self.source[self.pos..].starts_with("--") {
                while self.pos < self.source.len() && !self.source[self.pos..].starts_with('\n') {
                    self.pos += self.source[self.pos..]
                        .chars()
                        .next()
                        .map_or(1, |c| c.len_utf8());
                }
            } else {
                break;
            }
        }
    }

    fn at_line_start(&self, pos: usize) -> bool {
        if pos == 0 {
            return true;
        }
        for c in self.source[..pos].chars().rev() {
            if c == '\n' {
                return true;
            }
            if c != ' ' && c != '\t' && c != '\r' {
                return false;
            }
        }
        true
    }

    fn next_token(&mut self) -> Result<Option<Token>, LxError> {
        if self.pos >= self.source.len() {
            return Ok(None);
        }
        let start = self.pos;
        let c = self.advance().expect("next_token called at end of source");
        match c {
            '"' => {
                self.read_string(start)?;
                Ok(None)
            }
            '`' => self.read_raw_string(start).map(Some),
            '(' => {
                self.depth += 1;
                Ok(Some(self.tok(TokenKind::LParen, start)))
            }
            ')' => {
                self.depth -= 1;
                Ok(Some(self.tok(TokenKind::RParen, start)))
            }
            '[' => {
                self.depth += 1;
                Ok(Some(self.tok(TokenKind::LBracket, start)))
            }
            ']' => {
                self.depth -= 1;
                Ok(Some(self.tok(TokenKind::RBracket, start)))
            }
            '{' => {
                self.brace_stack.push(false);
                Ok(Some(self.tok(TokenKind::LBrace, start)))
            }
            '}' => {
                if let Some(suppresses) = self.brace_stack.pop()
                    && suppresses
                {
                    self.depth -= 1;
                }
                Ok(Some(self.tok(TokenKind::RBrace, start)))
            }
            ';' | ',' => Ok(Some(self.tok(TokenKind::Semi, start))),
            '^' => Ok(Some(self.tok(TokenKind::Caret, start))),
            '~' => {
                if self.peek() == Some('>') {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        if self.peek() == Some('?') {
                            self.advance();
                            Ok(Some(self.tok2(TokenKind::TildeArrowArrowQ, start)))
                        } else {
                            Ok(Some(self.tok2(TokenKind::TildeArrow, start)))
                        }
                    } else if self.peek() == Some('?') {
                        self.advance();
                        Ok(Some(self.tok2(TokenKind::TildeArrowQ, start)))
                    } else {
                        Ok(Some(self.tok2(TokenKind::TildeArrow, start)))
                    }
                } else {
                    Ok(Some(self.tok(TokenKind::Bang, start)))
                }
            }
            '?' => self.eat('?', TokenKind::QQ, TokenKind::Question, start),
            '&' => self.eat('&', TokenKind::And, TokenKind::Amp, start),
            '|' => self.eat('|', TokenKind::Or, TokenKind::Pipe, start),
            '!' => self.eat('=', TokenKind::NotEq, TokenKind::Bang, start),
            '=' => self.eat('=', TokenKind::Eq, TokenKind::Assign, start),
            ':' => self.eat('=', TokenKind::DeclMut, TokenKind::Colon, start),
            '*' => Ok(Some(self.tok(TokenKind::Star, start))),
            '%' => {
                if self.peek() == Some('{') {
                    self.advance();
                    self.depth += 1;
                    self.brace_stack.push(true);
                    Ok(Some(self.tok2(TokenKind::PercentLBrace, start)))
                } else {
                    Ok(Some(self.tok(TokenKind::Percent, start)))
                }
            }
            '#' => Err(LxError::parse(
                "unexpected character: #",
                Span::new(start as u32, 1),
                None,
            )),
            '+' => {
                if self.peek() == Some('+') {
                    self.advance();
                    Ok(Some(self.tok2(TokenKind::PlusPlus, start)))
                } else if self.at_line_start(start)
                    && self
                        .peek()
                        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
                {
                    Ok(Some(self.tok(TokenKind::Export, start)))
                } else {
                    Ok(Some(self.tok(TokenKind::Plus, start)))
                }
            }
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Ok(Some(self.tok2(TokenKind::Arrow, start)))
                } else {
                    Ok(Some(self.tok(TokenKind::Minus, start)))
                }
            }
            '/' => {
                if self.peek() == Some('/') {
                    self.advance();
                    Ok(Some(self.tok2(TokenKind::IntDiv, start)))
                } else {
                    Ok(Some(self.tok(TokenKind::Slash, start)))
                }
            }
            '<' => {
                if self.peek() == Some('-') {
                    self.advance();
                    Ok(Some(self.tok2(TokenKind::Reassign, start)))
                } else if self.peek() == Some('=') {
                    self.advance();
                    Ok(Some(self.tok2(TokenKind::LtEq, start)))
                } else {
                    Ok(Some(self.tok(TokenKind::Lt, start)))
                }
            }
            '>' => self.eat('=', TokenKind::GtEq, TokenKind::Gt, start),
            '.' => {
                if self.peek() == Some('.') {
                    if self.peek_ahead(1) == Some('.') {
                        Ok(Some(self.tok(TokenKind::Dot, start)))
                    } else {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            Ok(Some(Token::new(
                                TokenKind::DotDotEq,
                                Span::from_range(start as u32, self.pos as u32),
                            )))
                        } else {
                            Ok(Some(self.tok2(TokenKind::DotDot, start)))
                        }
                    }
                } else {
                    Ok(Some(self.tok(TokenKind::Dot, start)))
                }
            }
            '$' => {
                if self.peek() == Some('^') {
                    self.advance();
                    self.push(TokenKind::DollarCaret, start, self.pos);
                    self.read_shell_cmd()?;
                    Ok(None)
                } else if self.peek() == Some('{') {
                    self.advance();
                    self.push(TokenKind::DollarBrace, start, self.pos);
                    self.read_shell_block()?;
                    Ok(None)
                } else {
                    self.push(TokenKind::Dollar, start, start + 1);
                    self.read_shell_line(true)?;
                    Ok(None)
                }
            }
            '_' if !self
                .peek()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'') =>
            {
                Ok(Some(self.tok(TokenKind::Underscore, start)))
            }
            c if c.is_ascii_digit() => self.read_number(start).map(Some),
            'r' if self.peek() == Some('/') => self.read_regex(start).map(Some),
            c if c.is_ascii_lowercase() || c == '_' => self.read_ident_or_keyword(start).map(Some),
            c if c.is_ascii_uppercase() => self.read_type_name(start).map(Some),
            other => Err(LxError::parse(
                format!("unexpected character: {other}"),
                Span::new(start as u32, other.len_utf8() as u16),
                None,
            )),
        }
    }
}
