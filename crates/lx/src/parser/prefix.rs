use crate::ast::*;
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl<'a> super::Parser<'a> {
  pub(crate) fn parse_prefix(&mut self) -> Result<SExpr, LxError> {
    let tok = self.advance().clone();
    match tok.kind {
      TokenKind::Int(n) => Ok(SExpr::new(Expr::Literal(Literal::Int(n)), tok.span)),
      TokenKind::Float(f) => Ok(SExpr::new(Expr::Literal(Literal::Float(f)), tok.span)),
      TokenKind::True => Ok(SExpr::new(Expr::Literal(Literal::Bool(true)), tok.span)),
      TokenKind::False => Ok(SExpr::new(Expr::Literal(Literal::Bool(false)), tok.span)),
      TokenKind::Unit => Ok(SExpr::new(Expr::Literal(Literal::Unit), tok.span)),
      TokenKind::RawStr(s) => Ok(SExpr::new(Expr::Literal(Literal::RawStr(s)), tok.span)),
      TokenKind::StrStart => self.parse_string(tok.span.offset),
      TokenKind::Ident(name) => Ok(SExpr::new(Expr::Ident(name), tok.span)),
      TokenKind::TypeName(name) => Ok(SExpr::new(Expr::TypeConstructor(name), tok.span)),
      TokenKind::LParen => self.parse_paren(tok.span.offset),
      TokenKind::LBracket => self.parse_list(tok.span.offset),
      TokenKind::LBrace => self.parse_block_or_record(tok.span.offset),
      TokenKind::PercentLBrace => self.parse_map(tok.span.offset),
      TokenKind::HashLBrace => self.parse_set(tok.span.offset),
      TokenKind::Minus => self.parse_unary(UnaryOp::Neg, tok.span.offset),
      TokenKind::Bang => self.parse_unary(UnaryOp::Not, tok.span.offset),
      TokenKind::Loop => {
        self.expect_kind(TokenKind::LBrace)?;
        let stmts = self.parse_stmts_until_rbrace()?;
        let end = self.expect_kind(TokenKind::RBrace)?.span.end();
        Ok(SExpr::new(Expr::Loop(stmts), Span::from_range(tok.span.offset, end)))
      },
      TokenKind::Break => {
        let val = if self.peek_is_expr_start() { Some(Box::new(self.parse_expr(0)?)) } else { None };
        let end = val.as_ref().map(|v| v.span.end()).unwrap_or(tok.span.end());
        Ok(SExpr::new(Expr::Break(val), Span::from_range(tok.span.offset, end)))
      },
      TokenKind::Assert => {
        let expr = self.parse_expr(0)?;
        let msg = if !matches!(self.peek(), TokenKind::Semi | TokenKind::Eof | TokenKind::RBrace) && self.peek_is_expr_start() {
          Some(Box::new(self.parse_expr(0)?))
        } else {
          None
        };
        let end = msg.as_ref().map(|m| m.span.end()).unwrap_or(expr.span.end());
        Ok(SExpr::new(Expr::Assert { expr: Box::new(expr), msg }, Span::from_range(tok.span.offset, end)))
      },
      _ => Err(LxError::parse(format!("unexpected token: {:?}", tok.kind), tok.span, None)),
    }
  }

  fn parse_unary(&mut self, op: UnaryOp, start: u32) -> Result<SExpr, LxError> {
    let operand = self.parse_expr(29)?;
    let span = Span::from_range(start, operand.span.end());
    Ok(SExpr::new(Expr::Unary { op, operand: Box::new(operand) }, span))
  }

  fn parse_string(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut parts = Vec::new();
    loop {
      match self.peek().clone() {
        TokenKind::StrChunk(s) => {
          self.advance();
          parts.push(StrPart::Text(s));
        },
        TokenKind::StrEnd => {
          let end = self.advance().span.end();
          return Ok(SExpr::new(Expr::Literal(Literal::Str(parts)), Span::from_range(start, end)));
        },
        TokenKind::LBrace => {
          self.advance();
          let expr = self.parse_expr(0)?;
          self.expect_kind(TokenKind::RBrace)?;
          parts.push(StrPart::Interp(expr));
        },
        TokenKind::Eof => {
          return Err(LxError::parse("unterminated string", self.tokens[self.pos].span, None));
        },
        _ => {
          let expr = self.parse_expr(0)?;
          parts.push(StrPart::Interp(expr));
        },
      }
    }
  }

  fn parse_paren(&mut self, start: u32) -> Result<SExpr, LxError> {
    if *self.peek() == TokenKind::RParen {
      let end = self.advance().span.end();
      return Ok(SExpr::new(Expr::Literal(Literal::Unit), Span::from_range(start, end)));
    }
    if let Some(section) = self.try_section(start)? {
      return Ok(section);
    }
    if self.is_func_def() {
      return self.parse_func(start);
    }
    let first = self.parse_expr(0)?;
    if *self.peek() == TokenKind::RParen {
      let end = self.advance().span.end();
      return Ok(SExpr::new(first.node, Span::from_range(start, end)));
    }
    let mut elems = vec![first];
    while *self.peek() == TokenKind::Semi {
      self.advance();
      self.skip_semis();
      if *self.peek() == TokenKind::RParen {
        break;
      }
      elems.push(self.parse_expr(0)?);
    }
    let end = self.expect_kind(TokenKind::RParen)?.span.end();
    Ok(SExpr::new(Expr::Tuple(elems), Span::from_range(start, end)))
  }

  fn try_section(&mut self, start: u32) -> Result<Option<SExpr>, LxError> {
    if *self.peek() == TokenKind::Dot {
      if let Some(TokenKind::Ident(_)) = self.tokens.get(self.pos + 1).map(|t| &t.kind) {
        if self.tokens.get(self.pos + 2).map(|t| &t.kind) == Some(&TokenKind::RParen) {
          self.advance();
          let name = match self.advance().clone().kind {
            TokenKind::Ident(n) => n,
            _ => unreachable!(),
          };
          let end = self.expect_kind(TokenKind::RParen)?.span.end();
          return Ok(Some(SExpr::new(Expr::Section(Section::Field(name)), Span::from_range(start, end))));
        }
      }
    }
    if is_op(self.peek()) {
      let saved = self.pos;
      let op_tok = self.advance().clone();
      if let Some(op) = super::token_to_binop(&op_tok.kind) {
        if *self.peek() == TokenKind::RParen {
          self.pos = saved;
          return Ok(None);
        }
        let operand = self.parse_expr(0)?;
        let end = self.expect_kind(TokenKind::RParen)?.span.end();
        return Ok(Some(SExpr::new(Expr::Section(Section::Right { op, operand: Box::new(operand) }), Span::from_range(start, end))));
      }
      self.pos = saved;
    }
    Ok(None)
  }

  fn is_func_def(&self) -> bool {
    let mut i = self.pos;
    loop {
      match self.tokens.get(i).map(|t| &t.kind) {
        Some(TokenKind::Ident(_)) => i += 1,
        Some(TokenKind::RParen) => {
          return self
            .tokens
            .get(i + 1)
            .map(|t| !matches!(t.kind, TokenKind::Semi | TokenKind::Eof | TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket) && !is_infix_op(&t.kind))
            .unwrap_or(false);
        },
        _ => return false,
      }
    }
  }

  fn parse_func(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut params = Vec::new();
    while *self.peek() != TokenKind::RParen {
      let tok = self.advance().clone();
      match tok.kind {
        TokenKind::Ident(name) => params.push(Param { name, default: None }),
        _ => return Err(LxError::parse("expected parameter name", tok.span, None)),
      }
    }
    self.expect_kind(TokenKind::RParen)?;
    let body = self.parse_expr(0)?;
    let end = body.span.end();
    Ok(SExpr::new(Expr::Func { params, body: Box::new(body) }, Span::from_range(start, end)))
  }

  fn parse_list(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut elems = Vec::new();
    while *self.peek() != TokenKind::RBracket {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        elems.push(ListElem::Spread(self.parse_expr(0)?));
      } else {
        elems.push(ListElem::Single(self.parse_expr(0)?));
      }
      if *self.peek() == TokenKind::Semi {
        self.advance();
      }
    }
    let end = self.expect_kind(TokenKind::RBracket)?.span.end();
    Ok(SExpr::new(Expr::List(elems), Span::from_range(start, end)))
  }

  fn parse_block_or_record(&mut self, start: u32) -> Result<SExpr, LxError> {
    if looks_like_record(self) {
      return self.parse_record(start);
    }
    let stmts = self.parse_stmts_until_rbrace()?;
    let end = self.expect_kind(TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::Block(stmts), Span::from_range(start, end)))
  }

  fn parse_record(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut fields = Vec::new();
    while *self.peek() != TokenKind::RBrace {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        let value = self.parse_expr(0)?;
        fields.push(RecordField { name: None, value, is_spread: true });
      } else {
        let tok = self.advance().clone();
        let name = match tok.kind {
          TokenKind::Ident(n) => n,
          _ => return Err(LxError::parse("expected field name", tok.span, None)),
        };
        self.expect_kind(TokenKind::Colon)?;
        let value = self.parse_expr(0)?;
        fields.push(RecordField { name: Some(name), value, is_spread: false });
      }
      if *self.peek() == TokenKind::Semi {
        self.advance();
      }
    }
    let end = self.expect_kind(TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::Record(fields), Span::from_range(start, end)))
  }

  fn parse_map(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut entries = Vec::new();
    while *self.peek() != TokenKind::RBrace {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        let value = self.parse_expr(0)?;
        entries.push(MapEntry { key: None, value, is_spread: true });
      } else {
        let key = self.parse_expr(0)?;
        self.expect_kind(TokenKind::Colon)?;
        let value = self.parse_expr(0)?;
        entries.push(MapEntry { key: Some(key), value, is_spread: false });
      }
      if *self.peek() == TokenKind::Semi {
        self.advance();
      }
    }
    let end = self.expect_kind(TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::Map(entries), Span::from_range(start, end)))
  }

  fn parse_set(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut elems = Vec::new();
    while *self.peek() != TokenKind::RBrace {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        elems.push(SetElem::Spread(self.parse_expr(0)?));
      } else {
        elems.push(SetElem::Single(self.parse_expr(0)?));
      }
      if *self.peek() == TokenKind::Semi {
        self.advance();
      }
    }
    let end = self.expect_kind(TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::Set(elems), Span::from_range(start, end)))
  }

  pub(crate) fn parse_stmts_until_rbrace(&mut self) -> Result<Vec<SStmt>, LxError> {
    self.skip_semis();
    let mut stmts = Vec::new();
    while *self.peek() != TokenKind::RBrace {
      stmts.push(self.parse_stmt()?);
      self.skip_semis();
    }
    Ok(stmts)
  }

  pub(crate) fn peek_is_expr_start(&self) -> bool {
    matches!(
      self.peek(),
      TokenKind::Int(_)
        | TokenKind::Float(_)
        | TokenKind::StrStart
        | TokenKind::RawStr(_)
        | TokenKind::Ident(_)
        | TokenKind::TypeName(_)
        | TokenKind::LParen
        | TokenKind::LBracket
        | TokenKind::LBrace
        | TokenKind::True
        | TokenKind::False
        | TokenKind::Unit
        | TokenKind::Minus
        | TokenKind::Bang
        | TokenKind::Loop
        | TokenKind::Break
        | TokenKind::Assert
        | TokenKind::PercentLBrace
        | TokenKind::HashLBrace
    )
  }
}

fn looks_like_record(p: &super::Parser<'_>) -> bool {
  matches!((p.tokens.get(p.pos).map(|t| &t.kind), p.tokens.get(p.pos + 1).map(|t| &t.kind)), (Some(TokenKind::Ident(_)), Some(TokenKind::Colon)))
}

fn is_op(k: &TokenKind) -> bool {
  matches!(
    k,
    TokenKind::Plus
      | TokenKind::Minus
      | TokenKind::Star
      | TokenKind::Slash
      | TokenKind::Percent
      | TokenKind::IntDiv
      | TokenKind::PlusPlus
      | TokenKind::Eq
      | TokenKind::NotEq
      | TokenKind::Lt
      | TokenKind::Gt
      | TokenKind::LtEq
      | TokenKind::GtEq
      | TokenKind::And
      | TokenKind::Or
  )
}

fn is_infix_op(k: &TokenKind) -> bool {
  is_op(k)
    || matches!(
      k,
      TokenKind::Diamond
        | TokenKind::Pipe
        | TokenKind::QQ
        | TokenKind::Caret
        | TokenKind::Amp
        | TokenKind::Arrow
        | TokenKind::Question
        | TokenKind::Dot
        | TokenKind::DotDot
        | TokenKind::DotDotEq
    )
}
