mod prefix;

use crate::ast::*;
use crate::error::LxError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub fn parse(tokens: Vec<Token>, source: &str) -> Result<Program, LxError> {
  Parser { tokens, pos: 0, source }.parse_program()
}

pub(crate) struct Parser<'a> {
  pub(crate) tokens: Vec<Token>,
  pub(crate) pos: usize,
  pub(crate) source: &'a str,
}

impl<'a> Parser<'a> {
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
    let exported = if *self.peek() == TokenKind::Export {
      self.advance();
      true
    } else {
      false
    };
    if let Some(binding) = self.try_parse_binding(exported)? {
      let end = binding.value.span.end();
      return Ok(SStmt::new(Stmt::Binding(binding), Span::from_range(start, end)));
    }
    if exported {
      let sp = self.tokens[self.pos].span;
      return Err(LxError::parse("export must precede a binding", sp, None));
    }
    let expr = self.parse_expr(0)?;
    let end = expr.span.end();
    Ok(SStmt::new(Stmt::Expr(expr), Span::from_range(start, end)))
  }

  fn try_parse_binding(&mut self, exported: bool) -> Result<Option<Binding>, LxError> {
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
    let name = match self.advance().clone().kind {
      TokenKind::Ident(n) => n,
      _ => unreachable!(),
    };
    self.advance();
    let value = self.parse_expr(0)?;
    let target = if reassign { BindTarget::Reassign(name) } else { BindTarget::Name(name) };
    Ok(Some(Binding { exported, mutable, target, value }))
  }

  pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<SExpr, LxError> {
    let mut left = self.parse_prefix()?;
    loop {
      let kind = self.peek().clone();
      if matches!(
        kind,
        TokenKind::Eof
          | TokenKind::Semi
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
      if let Some(lbp) = postfix_bp(&kind) {
        if lbp < min_bp {
          break;
        }
        left = self.parse_postfix(left)?;
        continue;
      }
      if self.is_application_candidate(&left, min_bp) {
        let arg = self.parse_expr(32)?;
        let span = Span::from_range(left.span.offset, arg.span.end());
        left = SExpr::new(Expr::Apply { func: Box::new(left), arg: Box::new(arg) }, span);
        continue;
      }
      if let Some((lbp, rbp)) = infix_bp(&kind) {
        if lbp < min_bp {
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
    match kind {
      TokenKind::Pipe => {
        let right = self.parse_expr(rbp)?;
        let span = Span::from_range(start, right.span.end());
        Ok(SExpr::new(Expr::Pipe { left: Box::new(left), right: Box::new(right) }, span))
      },
      TokenKind::Diamond => {
        let right = self.parse_expr(rbp)?;
        let span = Span::from_range(start, right.span.end());
        Ok(SExpr::new(Expr::Compose { left: Box::new(left), right: Box::new(right) }, span))
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
    let span = Span::from_range(start, tok.span.end());
    match tok.kind {
      TokenKind::Ident(name) => Ok(SExpr::new(Expr::FieldAccess { expr: Box::new(left), field: FieldKind::Named(name) }, span)),
      TokenKind::Int(n) => {
        let idx: i64 = n.try_into().map_err(|_| LxError::parse("field index too large", tok.span, None))?;
        Ok(SExpr::new(Expr::FieldAccess { expr: Box::new(left), field: FieldKind::Index(idx) }, span))
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
        self.expect_kind(TokenKind::Arrow)?;
        let body = self.parse_expr(0)?;
        arms.push(MatchArm { pattern, guard, body });
        self.skip_semis();
      }
      let end = self.expect_kind(TokenKind::RBrace)?.span.end();
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
    if min_bp > 31 {
      return false;
    }
    let callable =
      matches!(left.node, Expr::Ident(_) | Expr::TypeConstructor(_) | Expr::Apply { .. } | Expr::FieldAccess { .. } | Expr::Section(_) | Expr::Func { .. });
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

  pub(crate) fn expect_kind(&mut self, expected: TokenKind) -> Result<&Token, LxError> {
    let tok = &self.tokens[self.pos];
    if std::mem::discriminant(&tok.kind) == std::mem::discriminant(&expected) {
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

  pub(crate) fn parse_pattern(&mut self) -> Result<SPattern, LxError> {
    let tok = self.advance().clone();
    match tok.kind {
      TokenKind::Underscore => Ok(SPattern::new(Pattern::Wildcard, tok.span)),
      TokenKind::Ident(name) => Ok(SPattern::new(Pattern::Bind(name), tok.span)),
      TokenKind::Int(n) => Ok(SPattern::new(Pattern::Literal(Literal::Int(n)), tok.span)),
      TokenKind::Float(f) => Ok(SPattern::new(Pattern::Literal(Literal::Float(f)), tok.span)),
      TokenKind::True => Ok(SPattern::new(Pattern::Literal(Literal::Bool(true)), tok.span)),
      TokenKind::False => Ok(SPattern::new(Pattern::Literal(Literal::Bool(false)), tok.span)),
      TokenKind::RawStr(s) => Ok(SPattern::new(Pattern::Literal(Literal::RawStr(s)), tok.span)),
      TokenKind::LParen => {
        let mut pats = Vec::new();
        while *self.peek() != TokenKind::RParen {
          pats.push(self.parse_pattern()?);
          if *self.peek() == TokenKind::Semi {
            self.advance();
          }
        }
        let end = self.expect_kind(TokenKind::RParen)?.span.end();
        Ok(SPattern::new(Pattern::Tuple(pats), Span::from_range(tok.span.offset, end)))
      },
      _ => Err(LxError::parse(format!("unexpected token in pattern: {:?}", tok.kind), tok.span, None)),
    }
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
    TokenKind::PlusPlus | TokenKind::Diamond => Some((21, 22)),
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
