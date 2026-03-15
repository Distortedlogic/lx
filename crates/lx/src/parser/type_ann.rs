use crate::ast::{SType, TypeExpr, TypeField};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
  pub(crate) fn parse_type(&mut self) -> Result<SType, LxError> {
    let left = self.parse_type_app()?;
    if *self.peek() == TokenKind::Caret {
      self.advance();
      let err = self.parse_type_app()?;
      let span = Span::from_range(left.span.offset, err.span.end());
      let fallible = TypeExpr::Fallible {
        ok: Box::new(left),
        err: Box::new(err),
      };
      if *self.peek() == TokenKind::Arrow {
        self.advance();
        let ret = self.parse_type()?;
        let fspan = Span::from_range(span.offset, ret.span.end());
        return Ok(SType::new(
          TypeExpr::Func { param: Box::new(SType::new(fallible, span)), ret: Box::new(ret) },
          fspan,
        ));
      }
      return Ok(SType::new(fallible, span));
    }
    if *self.peek() == TokenKind::Arrow {
      self.advance();
      let ret = self.parse_type()?;
      let span = Span::from_range(left.span.offset, ret.span.end());
      return Ok(SType::new(TypeExpr::Func { param: Box::new(left), ret: Box::new(ret) }, span));
    }
    Ok(left)
  }

  fn parse_type_app(&mut self) -> Result<SType, LxError> {
    if let TokenKind::TypeName(name) = self.peek().clone() {
      let tok = self.advance().clone();
      let mut args = Vec::new();
      while self.is_type_app_arg_start() {
        args.push(self.parse_type_atom()?);
      }
      if let Some(last) = args.last() {
        let end = last.span.end();
        Ok(SType::new(TypeExpr::Applied(name, args), Span::from_range(tok.span.offset, end)))
      } else {
        Ok(SType::new(TypeExpr::Named(name), tok.span))
      }
    } else {
      self.parse_type_atom()
    }
  }

  fn parse_type_atom(&mut self) -> Result<SType, LxError> {
    match self.peek().clone() {
      TokenKind::TypeName(name) => {
        let tok = self.advance().clone();
        Ok(SType::new(TypeExpr::Named(name), tok.span))
      },
      TokenKind::Ident(name) => {
        let tok = self.advance().clone();
        Ok(SType::new(TypeExpr::Var(name), tok.span))
      },
      TokenKind::LBracket => {
        let start = self.advance().span.offset;
        let inner = self.parse_type()?;
        let end = self.expect_kind(&TokenKind::RBracket)?.span.end();
        Ok(SType::new(TypeExpr::List(Box::new(inner)), Span::from_range(start, end)))
      },
      TokenKind::LBrace => {
        let start = self.advance().span.offset;
        let mut fields = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
          let field_name = match self.peek().clone() {
            TokenKind::Ident(n) => { self.advance(); n },
            _ => return Err(LxError::parse(
              "expected field name in record type",
              self.tokens[self.pos].span, None,
            )),
          };
          self.expect_kind(&TokenKind::Colon)?;
          let ty = self.parse_type()?;
          fields.push(TypeField { name: field_name, ty });
          self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SType::new(TypeExpr::Record(fields), Span::from_range(start, end)))
      },
      TokenKind::PercentLBrace => {
        let start = self.advance().span.offset;
        let key = self.parse_type()?;
        self.expect_kind(&TokenKind::Colon)?;
        let value = self.parse_type()?;
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SType::new(
          TypeExpr::Map { key: Box::new(key), value: Box::new(value) },
          Span::from_range(start, end),
        ))
      },
      TokenKind::LParen => {
        let start = self.advance().span.offset;
        if *self.peek() == TokenKind::RParen {
          let end = self.advance().span.end();
          return Ok(SType::new(TypeExpr::Named("Unit".into()), Span::from_range(start, end)));
        }
        let first = self.parse_type()?;
        if *self.peek() == TokenKind::RParen {
          let end = self.advance().span.end();
          let mut grouped = first;
          grouped.span = Span::from_range(start, end);
          return Ok(grouped);
        }
        let mut types = vec![first];
        while *self.peek() != TokenKind::RParen {
          types.push(self.parse_type()?);
        }
        let end = self.expect_kind(&TokenKind::RParen)?.span.end();
        Ok(SType::new(TypeExpr::Tuple(types), Span::from_range(start, end)))
      },
      _ => Err(LxError::parse("expected type", self.tokens[self.pos].span, None)),
    }
  }

  pub(crate) fn is_type_start(&self) -> bool {
    matches!(
      self.peek(),
      TokenKind::TypeName(_)
        | TokenKind::Ident(_)
        | TokenKind::LBracket
        | TokenKind::LBrace
        | TokenKind::PercentLBrace
        | TokenKind::LParen
    )
  }

  fn is_type_app_arg_start(&self) -> bool {
    matches!(
      self.peek(),
      TokenKind::TypeName(_)
        | TokenKind::LBracket
        | TokenKind::LBrace
        | TokenKind::PercentLBrace
        | TokenKind::LParen
    )
  }
}
