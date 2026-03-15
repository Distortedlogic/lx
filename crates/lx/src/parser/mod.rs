mod func;
mod infix;
mod paren;
mod pattern;
mod prefix;
mod statements;
mod type_ann;

pub(crate) use infix::token_to_binop;

use crate::ast::{Program, SStmt, Stmt, Binding, BindTarget, SExpr, Expr, FieldKind};
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
    if *self.peek() == TokenKind::Mcp {
      return self.parse_mcp_decl(exported, start);
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
        Stmt::Binding(Binding { exported: false, mutable, target: BindTarget::Pattern(pat), type_ann: None, value }),
        Span::from_range(start, end),
      ));
    }
    if *self.peek() == TokenKind::Reassign {
      self.advance();
      let value = self.parse_expr(0)?;
      let end = value.span.end();
      let (name, fields) = Self::expr_to_field_chain(&expr)?;
      return Ok(SStmt::new(Stmt::FieldUpdate { name, fields, value }, Span::from_range(start, end)));
    }
    let end = expr.span.end();
    Ok(SStmt::new(Stmt::Expr(expr), Span::from_range(start, end)))
  }

  fn expr_to_field_chain(expr: &SExpr) -> Result<(String, Vec<String>), LxError> {
    let Expr::FieldAccess { expr: inner, field: FieldKind::Named(f) } = &expr.node
      else { return Err(LxError::parse("'<-' requires name.field target", expr.span, None)); };
    match &inner.node {
      Expr::Ident(name) => Ok((name.clone(), vec![f.clone()])),
      Expr::FieldAccess { .. } => {
        let (name, mut fields) = Self::expr_to_field_chain(inner)?;
        fields.push(f.clone());
        Ok((name, fields))
      },
      _ => Err(LxError::parse("invalid field update target", expr.span, None)),
    }
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
}

fn looks_like_record(p: &Parser) -> bool {
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
