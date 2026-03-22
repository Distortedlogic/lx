use crate::ast::{Expr, FieldPattern, Literal, Pattern, SExpr, SPattern, StrPart};
use crate::error::LxError;
use crate::lexer::token::TokenKind;
use miette::SourceSpan;

impl super::Parser {
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
      TokenKind::Minus => {
        let next = self.advance().clone();
        match next.kind {
          TokenKind::Int(n) => Ok(SPattern::new(Pattern::Literal(Literal::Int(-n)), Span::from_range(tok.span.offset(), next.span.end()))),
          TokenKind::Float(f) => Ok(SPattern::new(Pattern::Literal(Literal::Float(-f)), Span::from_range(tok.span.offset(), next.span.end()))),
          _ => Err(LxError::parse("expected number after '-' in pattern", next.span, None)),
        }
      },
      TokenKind::StrStart => self.parse_str_pattern(tok.span.offset()),
      TokenKind::LParen => self.parse_tuple_pattern(tok.span.offset()),
      TokenKind::LBrace => self.parse_record_pattern(tok.span.offset()),
      TokenKind::LBracket => self.parse_list_pattern(tok.span.offset()),
      TokenKind::TypeName(name) => self.parse_constructor_pattern(name, tok.span),
      _ => Err(LxError::parse(format!("unexpected token in pattern: {:?}", tok.kind), tok.span, None)),
    }
  }

  fn parse_str_pattern(&mut self, start: usize) -> Result<SPattern, LxError> {
    let mut parts = Vec::new();
    loop {
      match self.peek().clone() {
        TokenKind::StrChunk(s) => {
          self.advance();
          parts.push(StrPart::Text(s));
        },
        TokenKind::StrEnd => {
          let end = self.advance().span.end();
          return Ok(SPattern::new(Pattern::Literal(Literal::Str(parts)), Span::from_range(start, end)));
        },
        _ => {
          return Err(LxError::parse("string patterns cannot contain interpolation", self.tokens[self.pos].span, None));
        },
      }
    }
  }

  pub(super) fn parse_tuple_pattern(&mut self, start: usize) -> Result<SPattern, LxError> {
    let mut pats = Vec::new();
    while *self.peek() != TokenKind::RParen {
      pats.push(self.parse_pattern()?);
      if *self.peek() == TokenKind::Semi {
        self.advance();
      }
    }
    let end = self.expect_kind(&TokenKind::RParen)?.span.end();
    Ok(SPattern::new(Pattern::Tuple(pats), Span::from_range(start, end)))
  }

  fn parse_record_pattern(&mut self, start: usize) -> Result<SPattern, LxError> {
    let mut fields = Vec::new();
    let mut rest = None;
    while *self.peek() != TokenKind::RBrace {
      self.skip_semis();
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        if let TokenKind::Ident(name) = self.peek().clone() {
          self.advance();
          rest = Some(name);
        }
        break;
      }
      let name = self.expect_ident("record pattern field")?;
      let pattern = if *self.peek() == TokenKind::Colon {
        self.advance();
        Some(self.parse_pattern()?)
      } else {
        None
      };
      fields.push(FieldPattern { name, pattern });
      self.skip_semis();
    }
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SPattern::new(Pattern::Record { fields, rest }, Span::from_range(start, end)))
  }

  fn parse_list_pattern(&mut self, start: usize) -> Result<SPattern, LxError> {
    let mut elems = Vec::new();
    let mut rest = None;
    while *self.peek() != TokenKind::RBracket {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        match self.peek().clone() {
          TokenKind::Ident(name) => {
            self.advance();
            rest = Some(name);
          },
          TokenKind::Underscore => {
            self.advance();
            rest = Some("_".into());
          },
          _ => {},
        }
        break;
      }
      elems.push(self.parse_pattern()?);
      if *self.peek() == TokenKind::Semi {
        self.advance();
      }
    }
    let end = self.expect_kind(&TokenKind::RBracket)?.span.end();
    Ok(SPattern::new(Pattern::List { elems, rest }, Span::from_range(start, end)))
  }

  fn parse_constructor_pattern(&mut self, name: String, name_span: Span) -> Result<SPattern, LxError> {
    let mut args = Vec::new();
    while matches!(
      self.peek(),
      TokenKind::Ident(_)
        | TokenKind::Int(_)
        | TokenKind::Float(_)
        | TokenKind::True
        | TokenKind::False
        | TokenKind::LParen
        | TokenKind::LBracket
        | TokenKind::LBrace
        | TokenKind::StrStart
        | TokenKind::RawStr(_)
        | TokenKind::Underscore
        | TokenKind::TypeName(_)
        | TokenKind::Minus
    ) {
      args.push(self.parse_pattern()?);
    }
    let end = args.last().map(|a| a.span.end()).unwrap_or(name_span.end());
    Ok(SPattern::new(Pattern::Constructor { name, args }, Span::from_range(name_span.offset, end)))
  }

  pub(super) fn expr_to_pattern(&self, expr: &SExpr) -> Result<SPattern, LxError> {
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
              && id == name
            {
              None
            } else {
              Some(self.expr_to_pattern(&f.value)?)
            };
            fps.push(FieldPattern { name: name.clone(), pattern });
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
