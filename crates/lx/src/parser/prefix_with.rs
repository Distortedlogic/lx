use crate::ast::{Expr, SExpr};
use crate::error::LxError;
use crate::lexer::token::TokenKind;
use crate::span::Span;

impl super::Parser {
  pub(super) fn is_with_binding(&self) -> bool {
    if *self.peek() == TokenKind::Ident("mut".into()) {
      return true;
    }
    if let TokenKind::Ident(_) = self.peek()
      && self.pos + 1 < self.tokens.len()
    {
      let next = &self.tokens[self.pos + 1].kind;
      return *next == TokenKind::Assign || *next == TokenKind::DeclMut;
    }
    false
  }

  pub(super) fn parse_with(&mut self, start: u32) -> Result<SExpr, LxError> {
    if *self.peek() == TokenKind::Ident("context".into()) {
      return self.parse_with_context(start);
    }
    if self.is_with_binding() {
      return self.parse_with_binding(start);
    }
    self.parse_with_resource(start)
  }

  fn parse_with_binding(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mutable = *self.peek() == TokenKind::Ident("mut".into());
    if mutable {
      self.advance();
    }
    let name = match self.peek().clone() {
      TokenKind::Ident(n) => {
        self.advance();
        n
      },
      _ => {
        return Err(LxError::parse("expected name after 'with'", self.tokens[self.pos].span, None));
      },
    };
    let op = self.peek().clone();
    if op != TokenKind::Assign && op != TokenKind::DeclMut {
      return Err(LxError::parse("expected '=' or ':=' in with", self.tokens[self.pos].span, None));
    }
    let mutable = mutable || op == TokenKind::DeclMut;
    self.advance();
    let value = self.parse_expr(0)?;
    self.expect_kind(&TokenKind::LBrace)?;
    let body = self.parse_stmts_until_rbrace()?;
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::With { name, value: Box::new(value), body, mutable }, Span::from_range(start, end)))
  }

  fn parse_with_context(&mut self, start: u32) -> Result<SExpr, LxError> {
    self.advance();
    let mut fields = Vec::new();
    while *self.peek() != TokenKind::LBrace {
      let name = match self.peek().clone() {
        TokenKind::Ident(n) => {
          self.advance();
          n
        },
        _ => {
          return Err(LxError::parse("expected field name in 'with context'", self.tokens[self.pos].span, None));
        },
      };
      self.expect_kind(&TokenKind::Colon)?;
      let value = self.parse_expr(0)?;
      fields.push((name, value));
    }
    self.expect_kind(&TokenKind::LBrace)?;
    let body = self.parse_stmts_until_rbrace()?;
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::WithContext { fields, body }, Span::from_range(start, end)))
  }

  fn parse_with_resource(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut resources = Vec::new();
    loop {
      let saved_stop = self.stop_ident.take();
      self.stop_ident = Some("as".into());
      let expr = self.parse_expr(0)?;
      self.stop_ident = saved_stop;
      if *self.peek() != TokenKind::Ident("as".into()) {
        return Err(LxError::parse("expected 'as' after resource expression", self.tokens[self.pos].span, None));
      }
      self.advance();
      let name = match self.peek().clone() {
        TokenKind::Ident(n) => {
          self.advance();
          n
        },
        _ => {
          return Err(LxError::parse("expected name after 'as'", self.tokens[self.pos].span, None));
        },
      };
      resources.push((expr, name));
      if *self.peek() != TokenKind::Semi {
        break;
      }
      self.advance();
    }
    self.expect_kind(&TokenKind::LBrace)?;
    let body = self.parse_stmts_until_rbrace()?;
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::WithResource { resources, body }, Span::from_range(start, end)))
  }
}
