use crate::ast::{SStmt, Stmt, Binding, BindTarget, ProtocolField, McpToolDecl, McpOutputType, UseStmt, UseKind};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
  pub(super) fn try_parse_binding(&mut self, exported: bool) -> Result<Option<Binding>, LxError> {
    if !matches!(self.peek(), TokenKind::Ident(_)) {
      return Ok(None);
    }
    let next = self.tokens.get(self.pos + 1).map(|t| &t.kind);
    let (mutable, reassign) = match next {
      Some(TokenKind::Assign) => (false, false),
      Some(TokenKind::DeclMut) => (true, false),
      Some(TokenKind::Reassign) => (false, true),
      _ => return Ok(None),
    };
    let TokenKind::Ident(name) = self.advance().clone().kind else { unreachable!() };
    self.advance();
    let value = self.parse_expr(0)?;
    let target = if reassign { BindTarget::Reassign(name) } else { BindTarget::Name(name) };
    Ok(Some(Binding { exported, mutable, target, value }))
  }

  pub(super) fn try_parse_type_def(&mut self, exported: bool, start: u32) -> Result<Option<SStmt>, LxError> {
    if !matches!(self.peek(), TokenKind::TypeName(_)) {
      return Ok(None);
    }
    let mut j = self.pos + 1;
    while matches!(self.tokens.get(j).map(|t| &t.kind), Some(TokenKind::Ident(_))) {
      j += 1;
    }
    if self.tokens.get(j).map(|t| &t.kind) != Some(&TokenKind::Assign) {
      return Ok(None);
    }
    let TokenKind::TypeName(name) = self.advance().clone().kind else { unreachable!() };
    while matches!(self.peek(), TokenKind::Ident(_)) {
      self.advance();
    }
    self.expect_kind(&TokenKind::Assign)?;
    self.skip_semis();
    let mut variants = Vec::new();
    if *self.peek() == TokenKind::Pipe {
      while *self.peek() == TokenKind::Pipe {
        self.advance();
        let ctor_name = if let TokenKind::TypeName(n) = self.peek().clone() {
          self.advance();
          n
        } else {
          continue;
        };
        let mut arity = 0usize;
        while matches!(self.peek(), TokenKind::TypeName(_) | TokenKind::Ident(_) | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace | TokenKind::PercentLBrace) {
          match self.peek() {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace | TokenKind::PercentLBrace => {
              let close = match self.peek() {
                TokenKind::LParen => TokenKind::RParen,
                TokenKind::LBracket => TokenKind::RBracket,
                _ => TokenKind::RBrace,
              };
              self.advance();
              let mut depth = 1u32;
              while depth > 0 {
                let k = self.peek().clone();
                if k == TokenKind::Eof { break; }
                if std::mem::discriminant(&k) == std::mem::discriminant(&close) { depth -= 1; }
                else if matches!(k, TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace | TokenKind::PercentLBrace) { depth += 1; }
                self.advance();
              }
            },
            _ => { self.advance(); },
          }
          arity += 1;
        }
        variants.push((ctor_name, arity));
        self.skip_semis();
      }
    } else {
      while !matches!(self.peek(), TokenKind::Semi | TokenKind::Eof) {
        self.advance();
      }
    }
    let end = self.tokens[self.pos.saturating_sub(1)].span.end();
    Ok(Some(SStmt::new(Stmt::TypeDef { name, variants, exported }, Span::from_range(start, end))))
  }

  pub(super) fn parse_protocol(&mut self, exported: bool, start: u32) -> Result<SStmt, LxError> {
    self.advance();
    let name = match self.peek().clone() {
      TokenKind::TypeName(n) => { self.advance(); n },
      _ => return Err(LxError::parse("expected type name after 'Protocol'", self.tokens[self.pos].span, None)),
    };
    self.expect_kind(&TokenKind::Assign)?;
    self.expect_kind(&TokenKind::LBrace)?;
    let mut fields = Vec::new();
    self.skip_semis();
    while *self.peek() != TokenKind::RBrace {
      let field_name = match self.peek().clone() {
        TokenKind::Ident(n) => { self.advance(); n },
        _ => return Err(LxError::parse("expected field name in Protocol", self.tokens[self.pos].span, None)),
      };
      self.expect_kind(&TokenKind::Colon)?;
      let type_name = match self.peek().clone() {
        TokenKind::TypeName(n) => { self.advance(); n },
        _ => return Err(LxError::parse("expected type name after ':'", self.tokens[self.pos].span, None)),
      };
      let default = if *self.peek() == TokenKind::Assign {
        self.advance();
        Some(self.parse_expr(0)?)
      } else {
        None
      };
      fields.push(ProtocolField { name: field_name, type_name, default });
      self.skip_semis();
    }
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SStmt::new(Stmt::Protocol { name, fields, exported }, Span::from_range(start, end)))
  }

  pub(super) fn parse_mcp_decl(&mut self, exported: bool, start: u32) -> Result<SStmt, LxError> {
    self.advance();
    let name = match self.peek().clone() {
      TokenKind::TypeName(n) => { self.advance(); n },
      _ => return Err(LxError::parse("expected type name after 'MCP'", self.tokens[self.pos].span, None)),
    };
    self.expect_kind(&TokenKind::Assign)?;
    self.expect_kind(&TokenKind::LBrace)?;
    let mut tools = Vec::new();
    self.skip_semis();
    while *self.peek() != TokenKind::RBrace {
      let tool_name = match self.peek().clone() {
        TokenKind::Ident(n) => { self.advance(); n },
        _ => return Err(LxError::parse("expected tool name in MCP declaration", self.tokens[self.pos].span, None)),
      };
      self.expect_kind(&TokenKind::Colon)?;
      self.expect_kind(&TokenKind::LBrace)?;
      let mut input = Vec::new();
      self.skip_semis();
      while *self.peek() != TokenKind::RBrace {
        let field_name = match self.peek().clone() {
          TokenKind::Ident(n) => { self.advance(); n },
          _ => return Err(LxError::parse("expected field name in MCP tool input", self.tokens[self.pos].span, None)),
        };
        self.expect_kind(&TokenKind::Colon)?;
        let type_name = match self.peek().clone() {
          TokenKind::TypeName(n) => { self.advance(); n },
          _ => return Err(LxError::parse("expected type name after ':'", self.tokens[self.pos].span, None)),
        };
        let default = if *self.peek() == TokenKind::Assign {
          self.advance();
          Some(self.parse_expr(0)?)
        } else {
          None
        };
        input.push(ProtocolField { name: field_name, type_name, default });
        self.skip_semis();
      }
      self.expect_kind(&TokenKind::RBrace)?;
      self.expect_kind(&TokenKind::Arrow)?;
      let output = self.parse_mcp_output_type()?;
      tools.push(McpToolDecl { name: tool_name, input, output });
      self.skip_semis();
    }
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SStmt::new(Stmt::McpDecl { name, tools, exported }, Span::from_range(start, end)))
  }

  fn parse_mcp_output_type(&mut self) -> Result<McpOutputType, LxError> {
    if *self.peek() == TokenKind::LBracket {
      self.advance();
      let inner = self.parse_mcp_output_type()?;
      self.expect_kind(&TokenKind::RBracket)?;
      return Ok(McpOutputType::List(Box::new(inner)));
    }
    if *self.peek() == TokenKind::LBrace {
      self.advance();
      let mut fields = Vec::new();
      self.skip_semis();
      while *self.peek() != TokenKind::RBrace {
        let field_name = match self.peek().clone() {
          TokenKind::Ident(n) => { self.advance(); n },
          _ => return Err(LxError::parse("expected field name in MCP output type", self.tokens[self.pos].span, None)),
        };
        self.expect_kind(&TokenKind::Colon)?;
        let type_name = match self.peek().clone() {
          TokenKind::TypeName(n) => { self.advance(); n },
          _ => return Err(LxError::parse("expected type name after ':'", self.tokens[self.pos].span, None)),
        };
        fields.push(ProtocolField { name: field_name, type_name, default: None });
        self.skip_semis();
      }
      self.expect_kind(&TokenKind::RBrace)?;
      return Ok(McpOutputType::Record(fields));
    }
    match self.peek().clone() {
      TokenKind::TypeName(n) => { self.advance(); Ok(McpOutputType::Named(n)) },
      _ => Err(LxError::parse("expected output type (TypeName, [...], or {...})", self.tokens[self.pos].span, None)),
    }
  }

  pub(super) fn parse_use_stmt(&mut self, start: u32) -> Result<SStmt, LxError> {
    self.advance();
    let mut path = Vec::new();
    if *self.peek() == TokenKind::DotDot {
      self.advance();
      path.push("..".to_string());
      self.expect_kind(&TokenKind::Slash)?;
    } else if *self.peek() == TokenKind::Dot
      && self.tokens.get(self.pos + 1).is_some_and(|t| t.kind == TokenKind::Slash) {
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
      return Err(LxError::parse("expected module path after 'use'", self.tokens[self.pos].span, None));
    }
    let kind = if *self.peek() == TokenKind::Colon {
      self.advance();
      match self.peek().clone() {
        TokenKind::Ident(name) => {
          self.advance();
          UseKind::Alias(name)
        },
        _ => return Err(LxError::parse("expected alias name after ':'", self.tokens[self.pos].span, None)),
      }
    } else if *self.peek() == TokenKind::LBrace {
      self.advance();
      let mut names = Vec::new();
      while *self.peek() != TokenKind::RBrace {
        match self.peek().clone() {
          TokenKind::Ident(name) | TokenKind::TypeName(name) => {
            self.advance();
            names.push(name);
          },
          _ => return Err(LxError::parse("expected name in selective import", self.tokens[self.pos].span, None)),
        }
        self.skip_semis();
      }
      self.expect_kind(&TokenKind::RBrace)?;
      UseKind::Selective(names)
    } else {
      UseKind::Whole
    };
    let end = self.tokens[self.pos.saturating_sub(1)].span.end();
    Ok(SStmt::new(Stmt::Use(UseStmt { path, kind }), Span::from_range(start, end)))
  }

}
