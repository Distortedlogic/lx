use crate::ast::{BindTarget, Binding, Expr, Param, SExpr, SStmt, Section, Stmt};
use crate::error::LxError;
use crate::lexer::token::TokenKind;
use crate::span::Span;

impl super::Parser {
  pub(super) fn parse_paren(&mut self, start: u32) -> Result<SExpr, LxError> {
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
      return Ok(SExpr::new(Expr::Literal(crate::ast::Literal::Unit), Span::from_range(start, end)));
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
        && let Some(op) = super::infix::token_to_binop(&op_tok)
      {
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
      && self.tokens.get(self.pos + 2).map(|t| &t.kind) == Some(&TokenKind::RParen)
    {
      self.advance();
      let name = self.expect_ident("field section")?;
      let end = self.expect_kind(&TokenKind::RParen)?.span.end();
      return Ok(Some(SExpr::new(Expr::Section(Section::Field(name)), Span::from_range(start, end))));
    }
    if *self.peek() == TokenKind::Dot
      && matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Int(_)))
      && self.tokens.get(self.pos + 2).map(|t| &t.kind) == Some(&TokenKind::RParen)
    {
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
          let body = Expr::Coalesce { expr: Box::new(SExpr::new(Expr::Ident("_x".into()), span)), default: Box::new(operand) };
          let func =
            Expr::Func { params: vec![Param { name: "_x".into(), type_ann: None, default: None }], ret_type: None, body: Box::new(SExpr::new(body, span)) };
          return Ok(Some(SExpr::new(func, span)));
        }
      }
      self.pos = saved;
    }
    if is_op(self.peek()) {
      let saved = self.pos;
      let op_tok = self.advance().clone();
      if let Some(op) = super::infix::token_to_binop(&op_tok.kind) {
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

  pub(super) fn parse_func(&mut self, start: u32) -> Result<SExpr, LxError> {
    let mut params = Vec::new();
    let mut pat_desugars: Vec<(String, crate::ast::SPattern)> = Vec::new();
    let mut pat_count = 0u32;
    while *self.peek() != TokenKind::RParen {
      let tok = self.advance().clone();
      match tok.kind {
        TokenKind::Ident(name) => {
          let type_ann = if *self.peek() == TokenKind::Colon {
            self.advance();
            Some(self.parse_type()?)
          } else {
            None
          };
          let default = if *self.peek() == TokenKind::Assign {
            self.advance();
            Some(self.parse_expr(0)?)
          } else {
            None
          };
          params.push(Param { name, type_ann, default });
        },
        TokenKind::Underscore => {
          params.push(Param { name: "_".into(), type_ann: None, default: None });
        },
        TokenKind::LParen => {
          let pat = self.parse_tuple_pattern(tok.span.offset)?;
          let name = format!("_pat{pat_count}");
          pat_count += 1;
          pat_desugars.push((name.clone(), pat));
          params.push(Param { name, type_ann: None, default: None });
        },
        _ => return Err(LxError::parse("expected parameter name", tok.span, None)),
      }
    }
    self.expect_kind(&TokenKind::RParen)?;
    let ret_type = if *self.peek() == TokenKind::Arrow {
      self.advance();
      Some(self.parse_type()?)
    } else {
      None
    };
    let mut body = if *self.peek() == TokenKind::LBrace && !super::looks_like_record(self) { self.parse_prefix()? } else { self.parse_expr(0)? };
    if !pat_desugars.is_empty() {
      let body_span = body.span;
      let mut stmts: Vec<SStmt> = pat_desugars
        .into_iter()
        .map(|(name, pat)| {
          SStmt::new(
            Stmt::Binding(Binding {
              exported: false,
              mutable: false,
              target: BindTarget::Pattern(pat),
              type_ann: None,
              value: SExpr::new(Expr::Ident(name), body_span),
            }),
            body_span,
          )
        })
        .collect();
      stmts.push(SStmt::new(Stmt::Expr(body), body_span));
      body = SExpr::new(Expr::Block(stmts), Span::from_range(start, body_span.end()));
    }
    let end = body.span.end();
    Ok(SExpr::new(Expr::Func { params, ret_type, body: Box::new(body) }, Span::from_range(start, end)))
  }
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
