mod helpers;
mod raw_token;
mod strings;
pub mod token;

use helpers::{ident_or_keyword, type_name_or_keyword};
use logos::Logos;
use lx_span::error::ParseError;
use miette::{SourceOffset, SourceSpan};
use num_bigint::BigInt;
use raw_token::RawToken;
use token::{Token, TokenKind};

pub(crate) struct Lexer<'src> {
  source: &'src str,
  pos: usize,
  tokens: Vec<Token>,
  comments: Vec<lx_span::source::Comment>,
  paren_bracket_depth: i32,
  last_was_semi: bool,
  brace_stack: Vec<i32>,
}

pub fn lex(source: &str) -> Result<(Vec<Token>, lx_span::source::CommentStore), ParseError> {
  let mut lexer = Lexer { source, pos: 0, tokens: Vec::new(), comments: Vec::new(), paren_bracket_depth: 0, last_was_semi: true, brace_stack: Vec::new() };
  lexer.run()?;
  lexer.tokens.push(Token::new(TokenKind::Eof, SourceSpan::new(SourceOffset::from(source.len()), 0)));
  Ok((lexer.tokens, lx_span::source::CommentStore::from_vec(lexer.comments)))
}

impl<'src> Lexer<'src> {
  fn sp(&self, start: usize, end: usize) -> SourceSpan {
    SourceSpan::new(SourceOffset::from(start), end - start)
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
    self.emit(Token::new(kind, self.sp(start, end)));
  }

  fn advance(&mut self) -> Option<char> {
    let c = self.source[self.pos..].chars().next()?;
    self.pos += c.len_utf8();
    Some(c)
  }

  fn peek(&self) -> Option<char> {
    self.source[self.pos..].chars().next()
  }

  fn at_line_start(&self, pos: usize) -> bool {
    if pos == 0 {
      return true;
    }
    for c in self.source[..pos].chars().rev() {
      match c {
        '\n' => return true,
        ' ' | '\t' | '\r' => continue,
        _ => return false,
      }
    }
    true
  }

  fn strip_underscores(s: &str) -> String {
    s.chars().filter(|c| *c != '_').collect()
  }

  fn run(&mut self) -> Result<(), ParseError> {
    let mut base: usize = 0;
    'outer: loop {
      let mut logos_lex = RawToken::lexer(&self.source[base..]);
      while let Some(result) = logos_lex.next() {
        let rel = logos_lex.span();
        let (start, end) = (base + rel.start, base + rel.end);
        let slice = logos_lex.slice();
        self.pos = end;
        if self.dispatch(start, end, slice, result)? {
          base = self.pos;
          continue 'outer;
        }
      }
      break;
    }
    Ok(())
  }

  fn dispatch(&mut self, start: usize, end: usize, slice: &str, result: Result<RawToken, ()>) -> Result<bool, ParseError> {
    let raw = result.map_err(|()| {
      let c = self.source[start..].chars().next().unwrap_or('?');
      ParseError::new(format!("unexpected character: {c}"), self.sp(start, start + c.len_utf8()), None)
    })?;
    let span = self.sp(start, end);
    match raw {
      RawToken::Newline => {
        if self.paren_bracket_depth <= 0 {
          self.emit(Token::new(TokenKind::Semi, span));
        }
      },
      RawToken::Comment => {
        let text = self.source[start..end].to_string();
        self.comments.push(lx_span::source::Comment { span: self.sp(start, end), text });
      },
      RawToken::LParen => {
        self.paren_bracket_depth += 1;
        self.emit(Token::new(TokenKind::LParen, span));
      },
      RawToken::RParen => {
        self.paren_bracket_depth -= 1;
        self.emit(Token::new(TokenKind::RParen, span));
      },
      RawToken::LBracket => {
        self.brace_stack.push(self.paren_bracket_depth);
        self.paren_bracket_depth = 0;
        self.emit(Token::new(TokenKind::LBracket, span));
      },
      RawToken::RBracket => {
        self.paren_bracket_depth = self.brace_stack.pop().unwrap_or(0);
        self.emit(Token::new(TokenKind::RBracket, span));
      },
      RawToken::LBrace => {
        self.brace_stack.push(self.paren_bracket_depth);
        self.paren_bracket_depth = 0;
        self.emit(Token::new(TokenKind::LBrace, span));
      },
      RawToken::RBrace => {
        self.paren_bracket_depth = self.brace_stack.pop().unwrap_or(0);
        self.emit(Token::new(TokenKind::RBrace, span));
      },
      RawToken::PercentLBrace => {
        self.brace_stack.push(self.paren_bracket_depth);
        self.paren_bracket_depth = 1;
        self.emit(Token::new(TokenKind::PercentLBrace, span));
      },
      RawToken::Semi => self.emit(Token::new(TokenKind::Semi, span)),
      RawToken::Comma => self.emit(Token::new(TokenKind::Comma, span)),
      RawToken::Hash => return Err(ParseError::new("unexpected character: #", span, None)),
      RawToken::Quote => {
        self.read_string(start)?;
        return Ok(true);
      },
      RawToken::Backtick => {
        let tok = self.read_raw_string(start)?;
        self.emit(tok);
        return Ok(true);
      },
      RawToken::Ident if slice == "_" => self.emit(Token::new(TokenKind::Underscore, span)),
      RawToken::Ident => self.emit(Token::new(ident_or_keyword(slice), span)),
      RawToken::ScreamingCase => self.emit(Token::new(TokenKind::Ident(lx_span::sym::intern(slice)), span)),
      RawToken::TypeName => self.emit(Token::new(type_name_or_keyword(slice), span)),
      RawToken::Plus => {
        let kind = if self.at_line_start(start) && self.source[end..].starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
          TokenKind::Export
        } else {
          TokenKind::Plus
        };
        self.emit(Token::new(kind, span));
      },
      RawToken::DotDot if self.source[end..].starts_with('.') => {
        self.emit(Token::new(TokenKind::Dot, self.sp(start, start + 1)));
        self.pos = start + 1;
        return Ok(true);
      },
      RawToken::HexInt => {
        self.emit_int(slice, 2, 16, span)?;
      },
      RawToken::BinInt => {
        self.emit_int(slice, 2, 2, span)?;
      },
      RawToken::OctInt => {
        self.emit_int(slice, 2, 8, span)?;
      },
      RawToken::FloatLit | RawToken::FloatExp => {
        let v: f64 = Self::strip_underscores(slice).parse().map_err(|_| ParseError::new("invalid float literal", span, None))?;
        self.emit(Token::new(TokenKind::Float(v), span));
      },
      RawToken::DecInt => {
        let v: BigInt = Self::strip_underscores(slice).parse().map_err(|_| ParseError::new("invalid integer literal", span, None))?;
        self.emit(Token::new(TokenKind::Int(v), span));
      },
      RawToken::TildeArrow => self.emit(Token::new(TokenKind::TildeArrow, span)),
      RawToken::TildeArrowQ => self.emit(Token::new(TokenKind::TildeArrowQ, span)),
      RawToken::Tilde | RawToken::BangExcl => self.emit(Token::new(TokenKind::Bang, span)),
      RawToken::Caret => self.emit(Token::new(TokenKind::Caret, span)),
      RawToken::QQ => self.emit(Token::new(TokenKind::QQ, span)),
      RawToken::Question => self.emit(Token::new(TokenKind::Question, span)),
      RawToken::And => self.emit(Token::new(TokenKind::And, span)),
      RawToken::Amp => self.emit(Token::new(TokenKind::Amp, span)),
      RawToken::Or => self.emit(Token::new(TokenKind::Or, span)),
      RawToken::Pipe => self.emit(Token::new(TokenKind::Pipe, span)),
      RawToken::NotEq => self.emit(Token::new(TokenKind::NotEq, span)),
      RawToken::Eq => self.emit(Token::new(TokenKind::Eq, span)),
      RawToken::Assign => self.emit(Token::new(TokenKind::Assign, span)),
      RawToken::DeclMut => self.emit(Token::new(TokenKind::DeclMut, span)),
      RawToken::Colon => self.emit(Token::new(TokenKind::Colon, span)),
      RawToken::Star => self.emit(Token::new(TokenKind::Star, span)),
      RawToken::Percent => self.emit(Token::new(TokenKind::Percent, span)),
      RawToken::PlusPlus => self.emit(Token::new(TokenKind::PlusPlus, span)),
      RawToken::Arrow => self.emit(Token::new(TokenKind::Arrow, span)),
      RawToken::Minus => self.emit(Token::new(TokenKind::Minus, span)),
      RawToken::IntDiv => self.emit(Token::new(TokenKind::IntDiv, span)),
      RawToken::Slash => self.emit(Token::new(TokenKind::Slash, span)),
      RawToken::Reassign => self.emit(Token::new(TokenKind::Reassign, span)),
      RawToken::LtEq => self.emit(Token::new(TokenKind::LtEq, span)),
      RawToken::Lt => self.emit(Token::new(TokenKind::Lt, span)),
      RawToken::GtEq => self.emit(Token::new(TokenKind::GtEq, span)),
      RawToken::Gt => self.emit(Token::new(TokenKind::Gt, span)),
      RawToken::DotDotEq => self.emit(Token::new(TokenKind::DotDotEq, span)),
      RawToken::DotDot => self.emit(Token::new(TokenKind::DotDot, span)),
      RawToken::Dot => self.emit(Token::new(TokenKind::Dot, span)),
    }
    Ok(false)
  }
}
