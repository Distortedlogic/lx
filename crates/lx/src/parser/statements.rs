use crate::ast::{BindTarget, Binding, Expr, FieldKind, SExpr, SStmt, Stmt};
use crate::error::LxError;
use crate::lexer::token::TokenKind;
use miette::SourceSpan;

impl super::Parser {
  pub(crate) fn parse_stmt(&mut self) -> Result<SStmt, LxError> {
    let start = self.tokens[self.pos].span.offset();
    if *self.peek() == TokenKind::Use {
      return self.parse_use_stmt(start);
    }
    let exported = if *self.peek() == TokenKind::Export {
      self.advance();
      true
    } else {
      false
    };
    if *self.peek() == TokenKind::Trait {
      return self.parse_trait_decl(exported, start);
    }
    if *self.peek() == TokenKind::ClassKw {
      return self.parse_class_decl(exported, start);
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
    let Expr::FieldAccess { expr: inner, field: FieldKind::Named(f) } = &expr.node else {
      return Err(LxError::parse("'<-' requires name.field target", expr.span, None));
    };
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

  pub(super) fn try_parse_binding(&mut self, exported: bool) -> Result<Option<Binding>, LxError> {
    if !matches!(self.peek(), TokenKind::Ident(_)) {
      return Ok(None);
    }
    let next = self.tokens.get(self.pos + 1).map(|t| &t.kind);
    let (mutable, reassign, has_type) = match next {
      Some(TokenKind::Assign) => (false, false, false),
      Some(TokenKind::DeclMut) => (true, false, false),
      Some(TokenKind::Reassign) => (false, true, false),
      Some(TokenKind::Colon) => {
        if self.is_typed_binding() {
          (false, false, true)
        } else {
          return Ok(None);
        }
      },
      _ => return Ok(None),
    };
    let name = self.expect_ident("binding name")?;
    let type_ann = if has_type {
      self.advance();
      let ty = self.parse_type()?;
      self.expect_kind(&TokenKind::Assign)?;
      Some(ty)
    } else {
      self.advance();
      None
    };
    let value = self.parse_expr(0)?;
    let target = if reassign { BindTarget::Reassign(name) } else { BindTarget::Name(name) };
    Ok(Some(Binding { exported, mutable, target, type_ann, value }))
  }

  fn is_typed_binding(&self) -> bool {
    let mut j = self.pos + 2;
    j = self.skip_type_tokens(j);
    matches!(self.tokens.get(j).map(|t| &t.kind), Some(TokenKind::Assign))
  }

  pub(super) fn try_parse_type_def(&mut self, exported: bool, start: usize) -> Result<Option<SStmt>, LxError> {
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
    let name = self.expect_type_name("type definition")?;
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
        while matches!(
          self.peek(),
          TokenKind::TypeName(_) | TokenKind::Ident(_) | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace | TokenKind::PercentLBrace
        ) {
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
                if k == TokenKind::Eof {
                  break;
                }
                if std::mem::discriminant(&k) == std::mem::discriminant(&close) {
                  depth -= 1;
                } else if matches!(k, TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace | TokenKind::PercentLBrace) {
                  depth += 1;
                }
                self.advance();
              }
            },
            _ => {
              self.advance();
            },
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
}
