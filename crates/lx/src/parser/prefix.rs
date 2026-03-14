use crate::ast::{SExpr, Expr, Literal, UnaryOp, StrPart, ListElem, RecordField, MapEntry, Section, Param, SStmt, Stmt, Binding, BindTarget, SPattern, ShellMode, SelArm};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
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
      TokenKind::Minus => self.parse_unary(UnaryOp::Neg, tok.span.offset),
      TokenKind::Bang => self.parse_unary(UnaryOp::Not, tok.span.offset),
      TokenKind::Loop => {
        self.expect_kind(&TokenKind::LBrace)?;
        let stmts = self.parse_stmts_until_rbrace()?;
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SExpr::new(Expr::Loop(stmts), Span::from_range(tok.span.offset, end)))
      },
      TokenKind::Par => {
        self.expect_kind(&TokenKind::LBrace)?;
        let stmts = self.parse_stmts_until_rbrace()?;
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SExpr::new(Expr::Par(stmts), Span::from_range(tok.span.offset, end)))
      },
      TokenKind::Sel => {
        self.expect_kind(&TokenKind::LBrace)?;
        let arms = self.parse_sel_arms()?;
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SExpr::new(Expr::Sel(arms), Span::from_range(tok.span.offset, end)))
      },
      TokenKind::Break => {
        let saved_nj = self.no_juxtapose;
        self.no_juxtapose = true;
        let val = if self.peek_is_expr_start() { Some(Box::new(self.parse_expr(0)?)) } else { None };
        self.no_juxtapose = saved_nj;
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
      TokenKind::Dollar => self.parse_shell(ShellMode::Normal, tok.span.offset),
      TokenKind::DollarCaret => self.parse_shell(ShellMode::Propagate, tok.span.offset),
      TokenKind::DollarBrace => self.parse_shell(ShellMode::Block, tok.span.offset),
      _ => Err(LxError::parse(format!("unexpected token: {:?}", tok.kind), tok.span, None)),
    }
  }

  fn parse_unary(&mut self, op: UnaryOp, start: u32) -> Result<SExpr, LxError> {
    let operand = self.parse_expr(29)?;
    let span = Span::from_range(start, operand.span.end());
    Ok(SExpr::new(Expr::Unary { op, operand: Box::new(operand) }, span))
  }

  fn parse_shell(&mut self, mode: ShellMode, start: u32) -> Result<SExpr, LxError> {
    let mut parts = Vec::new();
    loop {
      match self.peek().clone() {
        TokenKind::ShellText(s) => {
          self.advance();
          parts.push(StrPart::Text(s));
        },
        TokenKind::ShellEnd => {
          let end = self.advance().span.end();
          return Ok(SExpr::new(Expr::Shell { mode, parts }, Span::from_range(start, end)));
        },
        TokenKind::Eof => {
          return Ok(SExpr::new(Expr::Shell { mode, parts }, Span::from_range(start, self.tokens[self.pos].span.end())));
        },
        _ => {
          let expr = self.parse_expr(0)?;
          parts.push(StrPart::Interp(expr));
        },
      }
    }
  }

  fn parse_sel_arms(&mut self) -> Result<Vec<SelArm>, LxError> {
    let mut arms = Vec::new();
    self.skip_semis();
    while *self.peek() != TokenKind::RBrace {
      let expr = self.parse_expr(0)?;
      self.expect_kind(&TokenKind::Arrow)?;
      let handler = self.parse_expr(0)?;
      arms.push(SelArm { expr, handler });
      self.skip_semis();
    }
    Ok(arms)
  }

  pub(crate) fn parse_string(&mut self, start: u32) -> Result<SExpr, LxError> {
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
          self.expect_kind(&TokenKind::RBrace)?;
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
    let saved_depth = self.collection_depth;
    self.collection_depth = 0;
    let result = self.parse_paren_inner(start);
    self.collection_depth = saved_depth;
    result
  }

  fn parse_paren_inner(&mut self, start: u32) -> Result<SExpr, LxError> {
    if *self.peek() == TokenKind::RParen {
      if self.is_func_def() {
        return self.parse_func(start);
      }
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
    if is_op_or_minus(self.peek()) || *self.peek() == TokenKind::Minus {
      let op_tok = self.peek().clone();
      if let Some(next_after_op) = self.tokens.get(self.pos + 1)
        && next_after_op.kind == TokenKind::RParen
        && let Some(op) = super::token_to_binop(&op_tok) {
          self.advance();
          let end = self.advance().span.end();
          return Ok(SExpr::new(Expr::Section(Section::Left { operand: Box::new(first), op }), Span::from_range(start, end)));
        }
    }
    let mut elems = vec![first];
    loop {
      if *self.peek() == TokenKind::Semi {
        self.advance();
        self.skip_semis();
      }
      if *self.peek() == TokenKind::RParen {
        break;
      }
      if !self.peek_is_expr_start() {
        break;
      }
      elems.push(self.parse_expr(0)?);
    }
    let end = self.expect_kind(&TokenKind::RParen)?.span.end();
    Ok(SExpr::new(Expr::Tuple(elems), Span::from_range(start, end)))
  }

  fn try_section(&mut self, start: u32) -> Result<Option<SExpr>, LxError> {
    if *self.peek() == TokenKind::Dot
      && let Some(TokenKind::Ident(_)) = self.tokens.get(self.pos + 1).map(|t| &t.kind)
      && self.tokens.get(self.pos + 2).map(|t| &t.kind) == Some(&TokenKind::RParen) {
        self.advance();
        let TokenKind::Ident(name) = self.advance().clone().kind else { unreachable!() };
        let end = self.expect_kind(&TokenKind::RParen)?.span.end();
        return Ok(Some(SExpr::new(Expr::Section(Section::Field(name)), Span::from_range(start, end))));
      }
    if *self.peek() == TokenKind::Dot
      && matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Int(_)))
      && self.tokens.get(self.pos + 2).map(|t| &t.kind) == Some(&TokenKind::RParen) {
        self.advance();
        let tok = self.advance().clone();
        let TokenKind::Int(n) = tok.kind else { unreachable!() };
        let idx: i64 = n.try_into().map_err(|_| LxError::parse("index too large", tok.span, None))?;
        let end = self.expect_kind(&TokenKind::RParen)?.span.end();
        return Ok(Some(SExpr::new(Expr::Section(Section::Index(idx)), Span::from_range(start, end))));
      }
    if *self.peek() == TokenKind::QQ {
      let saved = self.pos;
      self.advance();
      if *self.peek() != TokenKind::RParen {
        let operand = self.parse_expr(0)?;
        if *self.peek() == TokenKind::RParen {
          let end = self.advance().span.end();
          let span = Span::from_range(start, end);
          let body = Expr::Coalesce {
            expr: Box::new(SExpr::new(Expr::Ident("_x".into()), span)),
            default: Box::new(operand),
          };
          let func = Expr::Func {
            params: vec![Param { name: "_x".into(), default: None }],
            body: Box::new(SExpr::new(body, span)),
            returns_result: false,
          };
          return Ok(Some(SExpr::new(func, span)));
        }
      }
      self.pos = saved;
    }
    if is_op(self.peek()) {
      let saved = self.pos;
      let op_tok = self.advance().clone();
      if let Some(op) = super::token_to_binop(&op_tok.kind) {
        if *self.peek() == TokenKind::RParen {
          let end = self.advance().span.end();
          return Ok(Some(SExpr::new(Expr::Section(Section::BinOp(op)), Span::from_range(start, end))));
        }
        let operand = self.parse_expr(0)?;
        let end = self.expect_kind(&TokenKind::RParen)?.span.end();
        return Ok(Some(SExpr::new(Expr::Section(Section::Right { op, operand: Box::new(operand) }), Span::from_range(start, end))));
      }
      self.pos = saved;
    }
    Ok(None)
  }

  fn is_func_def(&self) -> bool {
    let mut i = self.pos;
    let mut strong = false;
    let mut param_count = 0u32;
    loop {
      match self.tokens.get(i).map(|t| &t.kind) {
        Some(TokenKind::LParen) => {
          strong = true;
          param_count += 1;
          i += 1;
          let mut depth = 1u32;
          while depth > 0 {
            match self.tokens.get(i).map(|t| &t.kind) {
              Some(TokenKind::LParen) => { depth += 1; i += 1; },
              Some(TokenKind::RParen) => { depth -= 1; i += 1; },
              None | Some(TokenKind::Eof) => return false,
              _ => { i += 1; },
            }
          }
        },
        Some(TokenKind::Ident(_)) | Some(TokenKind::Underscore) => {
          if matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::Underscore)) {
            strong = true;
          }
          param_count += 1;
          i += 1;
          if self.tokens.get(i).map(|t| &t.kind) == Some(&TokenKind::Colon) {
            strong = true;
            i += 1;
            i = self.skip_type_at(i);
          }
          if self.tokens.get(i).map(|t| &t.kind) == Some(&TokenKind::Assign) {
            strong = true;
            i += 1;
            let mut depth = 0i32;
            loop {
              match self.tokens.get(i).map(|t| &t.kind) {
                None | Some(TokenKind::Eof) => return false,
                Some(TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace) => {
                  depth += 1;
                  i += 1;
                },
                Some(TokenKind::RParen) if depth > 0 => {
                  depth -= 1;
                  i += 1;
                },
                Some(TokenKind::RBracket | TokenKind::RBrace) if depth > 0 => {
                  depth -= 1;
                  i += 1;
                },
                Some(TokenKind::RParen) if depth == 0 => break,
                Some(TokenKind::Ident(_)) if depth == 0 => break,
                _ => i += 1,
              }
            }
          }
        },
        Some(TokenKind::RParen) => {
          i += 1;
          if matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::Arrow)) {
            strong = true;
            i += 1;
            i = self.skip_type_at(i);
            if matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::Caret)) {
              i += 1;
              i = self.skip_type_at(i);
            }
          }
          let is_body = self.tokens.get(i).map(|t| is_expr_start_kind(&t.kind)).unwrap_or(false);
          if !is_body {
            return false;
          }
          if !strong && param_count >= 2 && matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::LParen)) {
            return false;
          }
          return true;
        },
        _ => return false,
      }
    }
  }

  fn parse_func(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut params = Vec::new();
    let mut pat_desugars: Vec<(String, SPattern)> = Vec::new();
    let mut pat_count = 0u32;
    while *self.peek() != TokenKind::RParen {
      let tok = self.advance().clone();
      match tok.kind {
        TokenKind::Ident(name) => {
          if *self.peek() == TokenKind::Colon {
            self.advance();
            self.skip_type_expr();
          }
          let default = if *self.peek() == TokenKind::Assign {
            self.advance();
            Some(self.parse_expr(0)?)
          } else {
            None
          };
          params.push(Param { name, default });
        },
        TokenKind::Underscore => {
          params.push(Param { name: "_".into(), default: None });
        },
        TokenKind::LParen => {
          let pat = self.parse_tuple_pattern(tok.span.offset)?;
          let name = format!("_pat{pat_count}");
          pat_count += 1;
          pat_desugars.push((name.clone(), pat));
          params.push(Param { name, default: None });
        },
        _ => return Err(LxError::parse("expected parameter name", tok.span, None)),
      }
    }
    self.expect_kind(&TokenKind::RParen)?;
    let mut returns_result = false;
    if *self.peek() == TokenKind::Arrow {
      self.advance();
      self.skip_type_expr();
      if *self.peek() == TokenKind::Caret {
        self.advance();
        self.skip_type_expr();
        returns_result = true;
      }
    }
    let mut body = if *self.peek() == TokenKind::LBrace && !looks_like_record(self) {
      self.parse_prefix()?
    } else {
      self.parse_expr(0)?
    };
    if !pat_desugars.is_empty() {
      let body_span = body.span;
      let mut stmts: Vec<SStmt> = pat_desugars.into_iter().map(|(name, pat)| {
        SStmt::new(Stmt::Binding(Binding {
          exported: false,
          mutable: false,
          target: BindTarget::Pattern(pat),
          value: SExpr::new(Expr::Ident(name), body_span),
        }), body_span)
      }).collect();
      stmts.push(SStmt::new(Stmt::Expr(body), body_span));
      body = SExpr::new(Expr::Block(stmts), Span::from_range(start, body_span.end()));
    }
    let end = body.span.end();
    Ok(SExpr::new(Expr::Func { params, body: Box::new(body), returns_result }, Span::from_range(start, end)))
  }

  pub(super) fn skip_type_expr(&mut self) {
    match self.peek().clone() {
      TokenKind::TypeName(_) => {
        self.advance();
        while matches!(self.peek(), TokenKind::TypeName(_)) {
          self.advance();
        }
        if matches!(self.peek(), TokenKind::Ident(_))
          && matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind),
            Some(TokenKind::TypeName(_) | TokenKind::Arrow | TokenKind::RParen | TokenKind::Caret | TokenKind::Semi | TokenKind::Assign | TokenKind::DeclMut))
        {
          self.advance();
        }
        if matches!(self.peek(), TokenKind::Arrow) {
          self.advance();
          self.skip_type_expr();
        }
      },
      TokenKind::Ident(_) if matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Arrow)) => {
        self.advance();
        self.advance();
        self.skip_type_expr();
      },
      TokenKind::Ident(_) if matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind),
        Some(TokenKind::RParen | TokenKind::Ident(_) | TokenKind::Caret | TokenKind::Semi | TokenKind::Assign | TokenKind::DeclMut)) => {
        self.advance();
      },
      TokenKind::LBrace => {
        self.advance();
        let mut depth = 1u32;
        while depth > 0 {
          match self.peek() {
            TokenKind::LBrace => { depth += 1; self.advance(); },
            TokenKind::RBrace => { depth -= 1; self.advance(); },
            TokenKind::Eof => break,
            _ => { self.advance(); },
          }
        }
        if matches!(self.peek(), TokenKind::Arrow) {
          self.advance();
          self.skip_type_expr();
        }
      },
      TokenKind::LParen => {
        self.advance();
        let mut depth = 1u32;
        while depth > 0 {
          match self.peek() {
            TokenKind::LParen => { depth += 1; self.advance(); },
            TokenKind::RParen => { depth -= 1; self.advance(); },
            TokenKind::Eof => break,
            _ => { self.advance(); },
          }
        }
        if matches!(self.peek(), TokenKind::Arrow) {
          self.advance();
          self.skip_type_expr();
        }
      },
      TokenKind::LBracket => {
        self.advance();
        let mut depth = 1u32;
        while depth > 0 {
          match self.peek() {
            TokenKind::LBracket => { depth += 1; self.advance(); },
            TokenKind::RBracket => { depth -= 1; self.advance(); },
            TokenKind::Eof => break,
            _ => { self.advance(); },
          }
        }
      },
      TokenKind::PercentLBrace => {
        self.advance();
        let mut depth = 1u32;
        while depth > 0 {
          match self.peek() {
            TokenKind::LBrace | TokenKind::PercentLBrace => { depth += 1; self.advance(); },
            TokenKind::RBrace => { depth -= 1; self.advance(); },
            TokenKind::Eof => break,
            _ => { self.advance(); },
          }
        }
      },
      _ => {},
    }
  }

  fn parse_list(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut elems = Vec::new();
    self.collection_depth += 1;
    while *self.peek() != TokenKind::RBracket {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        elems.push(ListElem::Spread(self.parse_expr(32)?));
      } else {
        elems.push(ListElem::Single(self.parse_expr(0)?));
      }
      if *self.peek() == TokenKind::Semi {
        self.advance();
      }
    }
    self.collection_depth -= 1;
    let end = self.expect_kind(&TokenKind::RBracket)?.span.end();
    Ok(SExpr::new(Expr::List(elems), Span::from_range(start, end)))
  }

  fn parse_block_or_record(&mut self, start: u32) -> Result<SExpr, LxError> {
    self.skip_semis();
    if looks_like_record(self) {
      return self.parse_record(start);
    }
    let stmts = self.parse_stmts_until_rbrace()?;
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::Block(stmts), Span::from_range(start, end)))
  }

  fn parse_record(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut fields = Vec::new();
    self.skip_semis();
    self.collection_depth += 1;
    while *self.peek() != TokenKind::RBrace {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        let value = self.parse_expr(32)?;
        fields.push(RecordField { name: None, value, is_spread: true });
      } else {
        let tok = self.advance().clone();
        let TokenKind::Ident(name) = tok.kind else { return Err(LxError::parse("expected field name", tok.span, None)) };
        if *self.peek() == TokenKind::Colon {
          self.advance();
          let value = self.parse_expr(0)?;
          fields.push(RecordField { name: Some(name), value, is_spread: false });
        } else {
          let value = SExpr::new(Expr::Ident(name.clone()), tok.span);
          fields.push(RecordField { name: Some(name), value, is_spread: false });
        }
      }
      self.skip_semis();
    }
    self.collection_depth -= 1;
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::Record(fields), Span::from_range(start, end)))
  }

  fn parse_map(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut entries = Vec::new();
    self.collection_depth += 1;
    while *self.peek() != TokenKind::RBrace {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        let value = self.parse_expr(32)?;
        entries.push(MapEntry { key: None, value, is_spread: true });
      } else {
        let key = self.parse_expr(0)?;
        self.expect_kind(&TokenKind::Colon)?;
        let value = self.parse_expr(0)?;
        entries.push(MapEntry { key: Some(key), value, is_spread: false });
      }
      self.skip_semis();
    }
    self.collection_depth -= 1;
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SExpr::new(Expr::Map(entries), Span::from_range(start, end)))
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
        | TokenKind::Par
        | TokenKind::Sel
        | TokenKind::PercentLBrace
        | TokenKind::Dollar
        | TokenKind::DollarCaret
        | TokenKind::DollarBrace
    )
  }
}

fn looks_like_record(p: &super::Parser) -> bool {
  let (cur, next) = (p.tokens.get(p.pos).map(|t| &t.kind), p.tokens.get(p.pos + 1).map(|t| &t.kind));
  if matches!((cur, next), (Some(TokenKind::Ident(_)), Some(TokenKind::Colon)) | (Some(TokenKind::DotDot), _)) {
    return true;
  }
  if matches!(cur, Some(TokenKind::Ident(_))) {
    let mut j = p.pos;
    let mut ident_count = 0u32;
    loop {
      match p.tokens.get(j).map(|t| &t.kind) {
        Some(TokenKind::Ident(_)) => { ident_count += 1; j += 1; },
        Some(TokenKind::Semi) => { j += 1; },
        Some(TokenKind::RBrace) => return ident_count >= 2,
        _ => return false,
      }
    }
  }
  false
}

impl super::Parser {
  pub(super) fn skip_type_at(&self, start: usize) -> usize {
    let mut i = start;
    match self.tokens.get(i).map(|t| &t.kind) {
      Some(TokenKind::TypeName(_)) => {
        i += 1;
        while matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::TypeName(_))) {
          i += 1;
        }
        if matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::Ident(_)))
          && matches!(self.tokens.get(i + 1).map(|t| &t.kind),
            Some(TokenKind::TypeName(_) | TokenKind::Ident(_) | TokenKind::Arrow | TokenKind::RParen | TokenKind::Caret | TokenKind::Semi | TokenKind::Assign | TokenKind::DeclMut))
        {
          i += 1;
        }
        if matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::Arrow)) {
          i += 1;
          i = self.skip_type_at(i);
        }
      },
      Some(TokenKind::Ident(_)) if matches!(self.tokens.get(i + 1).map(|t| &t.kind), Some(TokenKind::Arrow)) => {
        i += 2;
        i = self.skip_type_at(i);
      },
      Some(TokenKind::Ident(_)) if matches!(self.tokens.get(i + 1).map(|t| &t.kind),
        Some(TokenKind::RParen | TokenKind::Ident(_) | TokenKind::Caret | TokenKind::Semi | TokenKind::Assign | TokenKind::DeclMut)) => {
        i += 1;
      },
      Some(TokenKind::LBrace) | Some(TokenKind::LParen) => {
        let open = self.tokens[i].kind.clone();
        let close = if matches!(open, TokenKind::LBrace) { TokenKind::RBrace } else { TokenKind::RParen };
        i += 1;
        let mut depth = 1u32;
        while depth > 0 {
          match self.tokens.get(i).map(|t| &t.kind) {
            Some(k) if std::mem::discriminant(k) == std::mem::discriminant(&open) => { depth += 1; i += 1; },
            Some(k) if std::mem::discriminant(k) == std::mem::discriminant(&close) => { depth -= 1; i += 1; },
            None | Some(TokenKind::Eof) => break,
            _ => { i += 1; },
          }
        }
        if matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::Arrow)) {
          i += 1;
          i = self.skip_type_at(i);
        }
      },
      Some(TokenKind::LBracket) => {
        i += 1;
        let mut depth = 1u32;
        while depth > 0 {
          match self.tokens.get(i).map(|t| &t.kind) {
            Some(TokenKind::LBracket) => { depth += 1; i += 1; },
            Some(TokenKind::RBracket) => { depth -= 1; i += 1; },
            None | Some(TokenKind::Eof) => break,
            _ => { i += 1; },
          }
        }
      },
      Some(TokenKind::PercentLBrace) => {
        i += 1;
        let mut depth = 1u32;
        while depth > 0 {
          match self.tokens.get(i).map(|t| &t.kind) {
            Some(TokenKind::LBrace | TokenKind::PercentLBrace) => { depth += 1; i += 1; },
            Some(TokenKind::RBrace) => { depth -= 1; i += 1; },
            None | Some(TokenKind::Eof) => break,
            _ => { i += 1; },
          }
        }
      },
      _ => {},
    }
    i
  }
}

fn is_expr_start_kind(k: &TokenKind) -> bool {
  matches!(
    k,
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
      | TokenKind::Par
      | TokenKind::Sel
      | TokenKind::PercentLBrace
      | TokenKind::Dollar
      | TokenKind::DollarCaret
      | TokenKind::DollarBrace
  )
}

fn is_op_or_minus(k: &TokenKind) -> bool {
  is_op(k) || matches!(k, TokenKind::Minus)
}

fn is_op(k: &TokenKind) -> bool {
  matches!(
    k,
    TokenKind::Plus
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

