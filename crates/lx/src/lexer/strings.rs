use super::Lexer;
use super::token::{Token, TokenKind};
use crate::error::LxError;
use crate::span::Span;

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
          self.flush_buf(&mut buf, chunk_start, TokenKind::StrChunk);
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
      if let Some(tok) = self.next_token()? {
        self.emit(tok)
      }
    }
  }

  pub(super) fn read_shell_line(&mut self, interpolate: bool) -> Result<(), LxError> {
    let stop_at_rparen = self.depth > 0;
    let mut buf = String::new();
    let mut chunk_start = self.pos;
    loop {
      match self.peek() {
        None | Some('\n') => break,
        Some(')') if stop_at_rparen => break,
        Some('{') if interpolate => {
          self.flush_buf(&mut buf, chunk_start, TokenKind::ShellText);
          self.advance();
          self.lex_interpolation(self.pos)?;
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
    if !buf.is_empty() {
      self.push(TokenKind::ShellText(buf), chunk_start, self.pos);
    }
    self.push(TokenKind::ShellEnd, self.pos, self.pos);
    Ok(())
  }

  pub(super) fn read_shell_cmd(&mut self) -> Result<(), LxError> {
    let stop_at_rparen = self.depth > 0;
    let mut buf = String::new();
    let mut chunk_start = self.pos;
    loop {
      match self.peek() {
        None | Some('\n') | Some('|') | Some(';') => break,
        Some(')') if stop_at_rparen => break,
        Some('{') => {
          self.flush_buf(&mut buf, chunk_start, TokenKind::ShellText);
          self.advance();
          self.lex_interpolation(self.pos)?;
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
    if !buf.is_empty() {
      let trimmed = buf.trim_end().to_string();
      if !trimmed.is_empty() {
        self.push(TokenKind::ShellText(trimmed), chunk_start, self.pos);
      }
    }
    self.push(TokenKind::ShellEnd, self.pos, self.pos);
    Ok(())
  }

  pub(super) fn read_shell_block(&mut self) -> Result<(), LxError> {
    let start = self.pos;
    let mut buf = String::new();
    let mut chunk_start = self.pos;
    loop {
      match self.peek() {
        None => {
          let span = Span::from_range(start as u32, self.pos as u32);
          return Err(LxError::parse("unterminated ${...} block", span, None));
        },
        Some('}') => {
          if !buf.is_empty() {
            self.push(TokenKind::ShellText(buf), chunk_start, self.pos);
          }
          self.advance();
          self.push(TokenKind::ShellEnd, self.pos - 1, self.pos);
          return Ok(());
        },
        Some('{') => {
          self.flush_buf(&mut buf, chunk_start, TokenKind::ShellText);
          self.advance();
          self.lex_interpolation(self.pos)?;
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

  pub(super) fn read_regex(&mut self, start: usize) -> Result<Token, LxError> {
    self.advance();
    let mut pattern = String::new();
    loop {
      match self.peek() {
        None | Some('\n') => {
          return Err(LxError::parse("unterminated regex literal", Span::new(start as u32, 1), None));
        },
        Some('/') => {
          self.advance();
          break;
        },
        Some('\\') => {
          self.advance();
          match self.peek() {
            Some('/') => {
              self.advance();
              pattern.push('/');
            },
            Some(c) => {
              self.advance();
              pattern.push('\\');
              pattern.push(c);
            },
            None => {
              return Err(LxError::parse("unterminated regex literal", Span::new(start as u32, 1), None));
            },
          }
        },
        Some(c) => {
          self.advance();
          pattern.push(c);
        },
      }
    }
    let mut flags = String::new();
    while let Some(c @ ('i' | 'm' | 's' | 'x')) = self.peek() {
      self.advance();
      flags.push(c);
    }
    let full = if flags.is_empty() { pattern } else { format!("(?{flags}){pattern}") };
    let span = Span::from_range(start as u32, self.pos as u32);
    Ok(Token::new(TokenKind::Regex(full), span))
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
