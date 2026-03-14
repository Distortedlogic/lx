mod pattern;
mod prefix;

use crate::ast::{Program, SStmt, Stmt, Binding, BindTarget, SExpr, Expr, FieldKind, MatchArm, SPattern, Pattern, Literal, BinOp, UseStmt, UseKind, ProtocolField};
use crate::error::LxError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub fn parse(tokens: Vec<Token>) -> Result<Program, LxError> {
  Parser { tokens, pos: 0, no_juxtapose: false, collection_depth: 0 }.parse_program()
}

pub(crate) struct Parser {
  pub(crate) tokens: Vec<Token>,
  pub(crate) pos: usize,
  pub(crate) no_juxtapose: bool,
  pub(crate) collection_depth: u32,
}

impl Parser {
  fn parse_program(&mut self) -> Result<Program, LxError> {
    self.skip_semis();
    let mut stmts = Vec::new();
    while *self.peek() != TokenKind::Eof {
      stmts.push(self.parse_stmt()?);
      self.skip_semis();
    }
    Ok(Program { stmts })
  }

  pub(crate) fn parse_stmt(&mut self) -> Result<SStmt, LxError> {
    let start = self.tokens[self.pos].span.offset;
    if *self.peek() == TokenKind::Use {
      return self.parse_use_stmt(start);
    }
    let exported = if *self.peek() == TokenKind::Export {
      self.advance();
      true
    } else {
      false
    };
    if *self.peek() == TokenKind::Protocol {
      return self.parse_protocol(exported, start);
    }
    if let Some(type_def) = self.try_parse_type_def(exported, start)? {
      return Ok(type_def);
    }
    if let Some(binding) = self.try_parse_binding(exported)? {
      let end = binding.value.span.end();
      return Ok(SStmt::new(Stmt::Binding(binding), Span::from_range(start, end)));
    }
    if exported {
      let sp = self.tokens[self.pos].span;
      return Err(LxError::parse("export must precede a binding", sp, None));
    }
    let expr = self.parse_expr(0)?;
    if *self.peek() == TokenKind::Assign || *self.peek() == TokenKind::DeclMut {
      let mutable = *self.peek() == TokenKind::DeclMut;
      self.advance();
      let pat = self.expr_to_pattern(&expr)?;
      let value = self.parse_expr(0)?;
      let end = value.span.end();
      return Ok(SStmt::new(
        Stmt::Binding(Binding { exported: false, mutable, target: BindTarget::Pattern(pat), value }),
        Span::from_range(start, end),
      ));
    }
    let end = expr.span.end();
    Ok(SStmt::new(Stmt::Expr(expr), Span::from_range(start, end)))
  }

  fn try_parse_binding(&mut self, exported: bool) -> Result<Option<Binding>, LxError> {
    if !matches!(self.peek(), TokenKind::Ident(_)) {
      return Ok(None);
    }
    let next = self.tokens.get(self.pos + 1).map(|t| &t.kind);
    let (mutable, reassign, skip_type) = match next {
      Some(TokenKind::Assign) => (false, false, false),
      Some(TokenKind::DeclMut) => (true, false, false),
      Some(TokenKind::Reassign) => (false, true, false),
      Some(TokenKind::Colon) => {
        let j = self.skip_type_at(self.pos + 2);
        match self.tokens.get(j).map(|t| &t.kind) {
          Some(TokenKind::Assign) => (false, false, true),
          Some(TokenKind::DeclMut) => (true, false, true),
          _ => return Ok(None),
        }
      },
      _ => return Ok(None),
    };
    let TokenKind::Ident(name) = self.advance().clone().kind else { unreachable!() };
    if skip_type {
      self.advance();
      self.skip_type_expr();
    }
    self.advance();
    let value = self.parse_expr(0)?;
    let target = if reassign { BindTarget::Reassign(name) } else { BindTarget::Name(name) };
    Ok(Some(Binding { exported, mutable, target, value }))
  }

  fn try_parse_type_def(&mut self, exported: bool, start: u32) -> Result<Option<SStmt>, LxError> {
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

  pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<SExpr, LxError> {
    let mut left = self.parse_prefix()?;
    loop {
      let kind = self.peek().clone();
      if matches!(
        kind,
        TokenKind::Eof
          | TokenKind::RParen
          | TokenKind::RBracket
          | TokenKind::RBrace
          | TokenKind::Colon
          | TokenKind::Arrow
          | TokenKind::Assign
          | TokenKind::DeclMut
          | TokenKind::Reassign
      ) {
        break;
      }
      if kind == TokenKind::Semi {
        if let Some(next) = self.tokens.get(self.pos + 1)
          && matches!(
            next.kind,
            TokenKind::Pipe
              | TokenKind::QQ
              | TokenKind::Caret
              | TokenKind::Dot
              | TokenKind::Question
              | TokenKind::Plus
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
              | TokenKind::DotDot
              | TokenKind::DotDotEq
              | TokenKind::Amp
              | TokenKind::TildeArrow
              | TokenKind::TildeArrowQ
          ) {
            self.advance();
            continue;
          }
        break;
      }
      if let Some(lbp) = postfix_bp(&kind) {
        if lbp < min_bp {
          break;
        }
        left = self.parse_postfix(left)?;
        continue;
      }
      if self.is_application_candidate(&left, min_bp) {
        if matches!(self.peek(), TokenKind::Ident(_))
          && self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Colon)
        {
          let tok = self.advance().clone();
          let TokenKind::Ident(name) = tok.kind else { unreachable!() };
          self.advance();
          let value = self.parse_expr(32)?;
          let arg_span = Span::from_range(tok.span.offset, value.span.end());
          let arg = SExpr::new(Expr::NamedArg { name, value: Box::new(value) }, arg_span);
          let span = Span::from_range(left.span.offset, arg.span.end());
          left = SExpr::new(Expr::Apply { func: Box::new(left), arg: Box::new(arg) }, span);
        } else {
          let arg = self.parse_expr(32)?;
          let span = Span::from_range(left.span.offset, arg.span.end());
          left = SExpr::new(Expr::Apply { func: Box::new(left), arg: Box::new(arg) }, span);
        }
        continue;
      }
      if let Some((lbp, rbp)) = infix_bp(&kind) {
        if lbp < min_bp {
          break;
        }
        if self.collection_depth > 0 && matches!(kind, TokenKind::DotDot | TokenKind::DotDotEq) {
          break;
        }
        if self.tokens.get(self.pos + 1).is_some_and(|t| t.kind == TokenKind::RParen) {
          break;
        }
        left = self.parse_infix(left, &kind, rbp)?;
        continue;
      }
      break;
    }
    Ok(left)
  }

  fn parse_infix(&mut self, left: SExpr, kind: &TokenKind, rbp: u8) -> Result<SExpr, LxError> {
    let start = left.span.offset;
    self.advance();
    self.skip_semis();
    match kind {
      TokenKind::Pipe => {
        let right = self.parse_expr(rbp)?;
        let span = Span::from_range(start, right.span.end());
        Ok(SExpr::new(Expr::Pipe { left: Box::new(left), right: Box::new(right) }, span))
      },
      TokenKind::TildeArrow => {
        let right = self.parse_expr(rbp)?;
        let span = Span::from_range(start, right.span.end());
        Ok(SExpr::new(Expr::AgentSend { target: Box::new(left), msg: Box::new(right) }, span))
      },
      TokenKind::TildeArrowQ => {
        let right = self.parse_expr(rbp)?;
        let span = Span::from_range(start, right.span.end());
        Ok(SExpr::new(Expr::AgentAsk { target: Box::new(left), msg: Box::new(right) }, span))
      },
      TokenKind::QQ => {
        let right = self.parse_expr(rbp)?;
        let span = Span::from_range(start, right.span.end());
        Ok(SExpr::new(Expr::Coalesce { expr: Box::new(left), default: Box::new(right) }, span))
      },
      TokenKind::Dot => self.parse_dot(left, start),
      TokenKind::Question => self.parse_question(left, start),
      _ => {
        if let Some(op) = token_to_binop(kind) {
          let right = self.parse_expr(rbp)?;
          let span = Span::from_range(start, right.span.end());
          Ok(SExpr::new(Expr::Binary { op, left: Box::new(left), right: Box::new(right) }, span))
        } else {
          let sp = self.tokens[self.pos.saturating_sub(1)].span;
          Err(LxError::parse(format!("unexpected infix token: {kind:?}"), sp, None))
        }
      },
    }
  }

  fn parse_dot(&mut self, left: SExpr, start: u32) -> Result<SExpr, LxError> {
    let tok = self.advance().clone();
    match tok.kind {
      TokenKind::Ident(name) => {
        let span = Span::from_range(start, tok.span.end());
        Ok(SExpr::new(Expr::FieldAccess { expr: Box::new(left), field: FieldKind::Named(name) }, span))
      },
      TokenKind::Int(ref n) => {
        if *self.peek() == TokenKind::DotDot {
          let start_expr = SExpr::new(Expr::Literal(Literal::Int(n.clone())), tok.span);
          self.advance();
          let end_expr = if matches!(self.peek(), TokenKind::Int(_)) {
            let end_tok = self.advance().clone();
            let TokenKind::Int(end_n) = end_tok.kind else { unreachable!() };
            Some(Box::new(SExpr::new(Expr::Literal(Literal::Int(end_n)), end_tok.span)))
          } else {
            None
          };
          let end_pos = end_expr.as_ref().map(|e| e.span.end()).unwrap_or(tok.span.end() + 2);
          let span = Span::from_range(start, end_pos);
          return Ok(SExpr::new(Expr::Slice { expr: Box::new(left), start: Some(Box::new(start_expr)), end: end_expr }, span));
        }
        let idx: i64 = n.try_into().map_err(|_| LxError::parse("field index too large", tok.span, None))?;
        let span = Span::from_range(start, tok.span.end());
        Ok(SExpr::new(Expr::FieldAccess { expr: Box::new(left), field: FieldKind::Index(idx) }, span))
      },
      TokenKind::DotDot => {
        let end_expr = if matches!(self.peek(), TokenKind::Int(_)) {
          let end_tok = self.advance().clone();
          let TokenKind::Int(end_n) = end_tok.kind else { unreachable!() };
          Some(Box::new(SExpr::new(Expr::Literal(Literal::Int(end_n)), end_tok.span)))
        } else {
          None
        };
        let end_pos = end_expr.as_ref().map(|e| e.span.end()).unwrap_or(tok.span.end());
        let span = Span::from_range(start, end_pos);
        Ok(SExpr::new(Expr::Slice { expr: Box::new(left), start: None, end: end_expr }, span))
      },
      TokenKind::Minus => {
        let num_tok = self.advance().clone();
        match num_tok.kind {
          TokenKind::Int(n) => {
            let idx: i64 = n.try_into().map_err(|_| LxError::parse("field index too large", num_tok.span, None))?;
            let span = Span::from_range(start, num_tok.span.end());
            Ok(SExpr::new(Expr::FieldAccess { expr: Box::new(left), field: FieldKind::Index(-idx) }, span))
          },
          _ => Err(LxError::parse("expected integer after '-' in field access", num_tok.span, None)),
        }
      },
      TokenKind::LBracket => {
        let key_expr = self.parse_expr(0)?;
        let end = self.expect_kind(&TokenKind::RBracket)?.span.end();
        let span = Span::from_range(start, end);
        Ok(SExpr::new(Expr::FieldAccess { expr: Box::new(left), field: FieldKind::Computed(Box::new(key_expr)) }, span))
      },
      TokenKind::StrStart => {
        let key_expr = self.parse_string(tok.span.offset)?;
        let end = key_expr.span.end();
        let span = Span::from_range(start, end);
        Ok(SExpr::new(Expr::FieldAccess { expr: Box::new(left), field: FieldKind::Computed(Box::new(key_expr)) }, span))
      },
      _ => Err(LxError::parse("expected field name or index after '.'", tok.span, None)),
    }
  }

  fn parse_postfix(&mut self, left: SExpr) -> Result<SExpr, LxError> {
    let tok = self.advance();
    let span = Span::from_range(left.span.offset, tok.span.end());
    Ok(SExpr::new(Expr::Propagate(Box::new(left)), span))
  }

  fn parse_question(&mut self, scrutinee: SExpr, start: u32) -> Result<SExpr, LxError> {
    if *self.peek() == TokenKind::LBrace {
      self.advance();
      let mut arms = Vec::new();
      self.skip_semis();
      while *self.peek() != TokenKind::RBrace {
        let pattern = self.parse_pattern()?;
        let guard = if *self.peek() == TokenKind::Amp {
          self.advance();
          Some(self.parse_expr(8)?)
        } else {
          None
        };
        self.expect_kind(&TokenKind::Arrow)?;
        let body = self.parse_expr(0)?;
        arms.push(MatchArm { pattern, guard, body });
        self.skip_semis();
      }
      let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
      Ok(SExpr::new(Expr::Match { scrutinee: Box::new(scrutinee), arms }, Span::from_range(start, end)))
    } else {
      let then_ = self.parse_expr(4)?;
      let (else_, end) = if *self.peek() == TokenKind::Colon {
        self.advance();
        let e = self.parse_expr(4)?;
        let end = e.span.end();
        (Some(Box::new(e)), end)
      } else {
        (None, then_.span.end())
      };
      let span = Span::from_range(start, end);
      Ok(SExpr::new(Expr::Ternary { cond: Box::new(scrutinee), then_: Box::new(then_), else_ }, span))
    }
  }

  fn is_application_candidate(&self, left: &SExpr, min_bp: u8) -> bool {
    if min_bp > 31 || self.no_juxtapose {
      return false;
    }
    let callable = if self.collection_depth > 0 {
      matches!(left.node, Expr::TypeConstructor(_)) && !matches!(self.peek(), TokenKind::TypeName(_))
    } else {
      matches!(left.node, Expr::Ident(_) | Expr::TypeConstructor(_) | Expr::Apply { .. } | Expr::FieldAccess { .. } | Expr::Section(_) | Expr::Func { .. })
    };
    if !callable {
      return false;
    }
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
               | TokenKind::PercentLBrace
    )
  }

  pub(crate) fn peek(&self) -> &TokenKind {
    self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof)
  }

  pub(crate) fn advance(&mut self) -> &Token {
    let tok = &self.tokens[self.pos];
    if tok.kind != TokenKind::Eof {
      self.pos += 1;
    }
    tok
  }

  pub(crate) fn expect_kind(&mut self, expected: &TokenKind) -> Result<&Token, LxError> {
    let tok = &self.tokens[self.pos];
    if std::mem::discriminant(&tok.kind) == std::mem::discriminant(expected) {
      Ok(self.advance())
    } else {
      Err(LxError::parse(format!("expected {expected:?}, found {:?}", tok.kind), tok.span, None))
    }
  }

  pub(crate) fn skip_semis(&mut self) {
    while *self.peek() == TokenKind::Semi {
      self.advance();
    }
  }

  fn parse_protocol(&mut self, exported: bool, start: u32) -> Result<SStmt, LxError> {
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

  fn parse_use_stmt(&mut self, start: u32) -> Result<SStmt, LxError> {
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

  fn expr_to_pattern(&self, expr: &SExpr) -> Result<SPattern, LxError> {
    let span = expr.span;
    match &expr.node {
      Expr::Ident(name) => Ok(SPattern::new(Pattern::Bind(name.clone()), span)),
      Expr::Literal(Literal::Unit) => Ok(SPattern::new(Pattern::Wildcard, span)),
      Expr::Tuple(elems) => {
        let mut pats = Vec::new();
        for e in elems {
          pats.push(self.expr_to_pattern(e)?);
        }
        Ok(SPattern::new(Pattern::Tuple(pats), span))
      },
      Expr::Apply { .. } => {
        let mut parts = vec![];
        self.flatten_apply(expr, &mut parts)?;
        Ok(SPattern::new(Pattern::Tuple(parts), span))
      },
      Expr::Record(fields) => {
        let mut fps = Vec::new();
        for f in fields {
          if f.is_spread {
            continue;
          }
          if let Some(ref name) = f.name {
            let pattern = if let Expr::Ident(ref id) = f.value.node
              && id == name {
                None
              } else {
                Some(self.expr_to_pattern(&f.value)?)
              };
            fps.push(crate::ast::FieldPattern { name: name.clone(), pattern });
          }
        }
        Ok(SPattern::new(Pattern::Record { fields: fps, rest: None }, span))
      },
      Expr::List(elems) => {
        let mut pats = Vec::new();
        let mut rest = None;
        for e in elems {
          match e {
            crate::ast::ListElem::Single(e) => pats.push(self.expr_to_pattern(e)?),
            crate::ast::ListElem::Spread(e) => {
              if let Expr::Ident(name) = &e.node {
                rest = Some(name.clone());
              }
            },
          }
        }
        Ok(SPattern::new(Pattern::List { elems: pats, rest }, span))
      },
      _ => Err(LxError::parse("expected pattern on left side of '='", span, None)),
    }
  }

  fn flatten_apply(&self, expr: &SExpr, out: &mut Vec<SPattern>) -> Result<(), LxError> {
    match &expr.node {
      Expr::Apply { func, arg } => {
        self.flatten_apply(func, out)?;
        out.push(self.expr_to_pattern(arg)?);
      },
      _ => {
        out.push(self.expr_to_pattern(expr)?);
      },
    }
    Ok(())
  }
}

fn infix_bp(kind: &TokenKind) -> Option<(u8, u8)> {
  match kind {
    TokenKind::Question => Some((3, 4)),
    TokenKind::Amp => Some((7, 8)),
    TokenKind::QQ => Some((11, 12)),
    TokenKind::Or => Some((13, 14)),
    TokenKind::And => Some((15, 16)),
    TokenKind::Eq | TokenKind::NotEq | TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => Some((17, 18)),
    TokenKind::Pipe => Some((19, 20)),
    TokenKind::PlusPlus | TokenKind::TildeArrow | TokenKind::TildeArrowQ => Some((21, 22)),
    TokenKind::DotDot | TokenKind::DotDotEq => Some((23, 24)),
    TokenKind::Plus | TokenKind::Minus => Some((25, 26)),
    TokenKind::Star | TokenKind::Slash | TokenKind::Percent | TokenKind::IntDiv => Some((27, 28)),
    TokenKind::Dot => Some((33, 34)),
    _ => None,
  }
}

fn postfix_bp(kind: &TokenKind) -> Option<u8> {
  match kind {
    TokenKind::Caret => Some(10),
    _ => None,
  }
}

pub(crate) fn token_to_binop(kind: &TokenKind) -> Option<BinOp> {
  match kind {
    TokenKind::Plus => Some(BinOp::Add),
    TokenKind::Minus => Some(BinOp::Sub),
    TokenKind::Star => Some(BinOp::Mul),
    TokenKind::Slash => Some(BinOp::Div),
    TokenKind::Percent => Some(BinOp::Mod),
    TokenKind::IntDiv => Some(BinOp::IntDiv),
    TokenKind::PlusPlus => Some(BinOp::Concat),
    TokenKind::DotDot => Some(BinOp::Range),
    TokenKind::DotDotEq => Some(BinOp::RangeInclusive),
    TokenKind::Eq => Some(BinOp::Eq),
    TokenKind::NotEq => Some(BinOp::NotEq),
    TokenKind::Lt => Some(BinOp::Lt),
    TokenKind::Gt => Some(BinOp::Gt),
    TokenKind::LtEq => Some(BinOp::LtEq),
    TokenKind::GtEq => Some(BinOp::GtEq),
    TokenKind::And => Some(BinOp::And),
    TokenKind::Or => Some(BinOp::Or),
    _ => None,
  }
}
