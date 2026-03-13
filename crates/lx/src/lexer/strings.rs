use super::Lexer;
use crate::error::LxError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

impl<'src> Lexer<'src> {
  pub(super) fn read_string(&mut self, start: usize) -> Result<(), LxError> {
    self.push(TokenKind::StrStart, start, start + 1);
    let mut buf = String::new();
    let mut chunk_start = self.pos;
    loop {
      match self.peek() {
        None => {
          let span = Span::from_range(start as u32, self.pos as u32);
          return Err(LxError::parse("unterminated string", span, None));
        },
        Some('"') => {
          if !buf.is_empty() {
            self.push(TokenKind::StrChunk(buf), chunk_start, self.pos);
          }
          self.advance();
          self.push(TokenKind::StrEnd, self.pos - 1, self.pos);
          return Ok(());
        },
        Some('\\') => {
          self.advance();
          let esc = self.advance().ok_or_else(|| {
            let span = Span::from_range(start as u32, self.pos as u32);
            LxError::parse("unterminated escape sequence", span, None)
          })?;
          match esc {
            'n' => buf.push('\n'),
            't' => buf.push('\t'),
            '\\' => buf.push('\\'),
            '"' => buf.push('"'),
            '{' => buf.push('{'),
            '0' => buf.push('\0'),
            other => {
              let span = Span::from_range(self.pos as u32 - 2, self.pos as u32);
              return Err(LxError::parse(format!("unknown escape: \\{other}"), span, None));
            },
          }
        },
        Some('{') => {
          if !buf.is_empty() {
            self.push(TokenKind::StrChunk(std::mem::take(&mut buf)), chunk_start, self.pos);
          }
          self.advance();
          self.lex_interpolation(start)?;
          chunk_start = self.pos;
        },
        Some(c) => {
          if buf.is_empty() {
            chunk_start = self.pos;
          }
          self.advance();
          buf.push(c);
        },
      }
    }
  }

  fn lex_interpolation(&mut self, str_start: usize) -> Result<(), LxError> {
    let mut brace_depth = 1i32;
    loop {
      self.skip_whitespace_and_comments();
      if self.pos >= self.source.len() {
        let span = Span::from_range(str_start as u32, self.pos as u32);
        return Err(LxError::parse("unterminated string interpolation", span, None));
      }
      let c = self.source[self.pos..].chars().next().unwrap_or('\0');
      if c == '}' {
        brace_depth -= 1;
        if brace_depth == 0 {
          self.advance();
          return Ok(());
        }
      }
      if c == '{' {
        brace_depth += 1;
      }
      match self.next_token()? {
        Some(tok) => self.emit(tok),
        None => {
          let span = Span::from_range(str_start as u32, self.pos as u32);
          return Err(LxError::parse("unterminated string interpolation", span, None));
        },
      }
    }
  }

  pub(super) fn read_raw_string(&mut self, start: usize) -> Result<Token, LxError> {
    let mut buf = String::new();
    loop {
      match self.peek() {
        None => {
          let span = Span::from_range(start as u32, self.pos as u32);
          return Err(LxError::parse("unterminated raw string", span, None));
        },
        Some('`') => {
          self.advance();
          let span = Span::from_range(start as u32, self.pos as u32);
          return Ok(Token::new(TokenKind::RawStr(buf), span));
        },
        Some(c) => {
          self.advance();
          buf.push(c);
        },
      }
    }
  }
}
