use super::Lexer;
use super::raw_token::RawToken;
use super::token::{Token, TokenKind};
use crate::error::LxError;
use logos::Logos;
use miette::{SourceOffset, SourceSpan};

impl<'src> Lexer<'src> {
  fn flush_buf(&mut self, buf: &mut String, chunk_start: usize, make_kind: fn(String) -> TokenKind) {
    if !buf.is_empty() {
      self.push(make_kind(std::mem::take(buf)), chunk_start, self.pos);
    }
  }

  pub(super) fn read_string(&mut self, start: usize) -> Result<(), LxError> {
    self.push(TokenKind::StrStart, start, start + 1);
    let mut buf = String::new();
    let mut chunk_start = self.pos;
    loop {
      match self.peek() {
        None => {
          return Err(LxError::parse("unterminated string", self.sp(start, self.pos), None));
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
          let esc = self.advance().ok_or_else(|| LxError::parse("unterminated escape sequence", self.sp(start, self.pos), None))?;
          match esc {
            'n' => buf.push('\n'),
            't' => buf.push('\t'),
            '\\' => buf.push('\\'),
            '"' => buf.push('"'),
            '{' => buf.push('{'),
            '0' => buf.push('\0'),
            other => {
              return Err(LxError::parse(format!("unknown escape: \\{other}"), SourceSpan::new(SourceOffset::from(self.pos - 2), 2), None));
            },
          }
        },
        Some('{') => {
          self.flush_buf(&mut buf, chunk_start, TokenKind::StrChunk);
          let brace_start = self.pos;
          self.advance();
          self.push(TokenKind::LBrace, brace_start, brace_start + 1);
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

  pub(crate) fn lex_interpolation(&mut self, str_start: usize) -> Result<(), LxError> {
    let mut brace_depth = 1i32;
    'restart: loop {
      let base = self.pos;
      let mut logos_lex = RawToken::lexer(&self.source[base..]);
      while let Some(result) = logos_lex.next() {
        let rel = logos_lex.span();
        let (start, end) = (base + rel.start, base + rel.end);
        let slice = logos_lex.slice();
        self.pos = end;
        let Ok(raw) = &result else {
          let c = self.source[start..].chars().next().unwrap_or('?');
          return Err(LxError::parse(format!("unexpected character: {c}"), self.sp(start, start + c.len_utf8()), None));
        };
        if matches!(raw, RawToken::RBrace) {
          brace_depth -= 1;
          if brace_depth == 0 {
            return Ok(());
          }
        }
        if matches!(raw, RawToken::LBrace) {
          brace_depth += 1;
        }
        if self.dispatch(start, end, slice, result)? {
          continue 'restart;
        }
      }
      return Err(LxError::parse("unterminated string interpolation", self.sp(str_start, self.pos), None));
    }
  }

  pub(super) fn read_raw_string(&mut self, start: usize) -> Result<Token, LxError> {
    let mut buf = String::new();
    loop {
      match self.peek() {
        None => {
          return Err(LxError::parse("unterminated raw string", self.sp(start, self.pos), None));
        },
        Some('`') => {
          self.advance();
          return Ok(Token::new(TokenKind::RawStr(buf), self.sp(start, self.pos)));
        },
        Some(c) => {
          self.advance();
          buf.push(c);
        },
      }
    }
  }
}
