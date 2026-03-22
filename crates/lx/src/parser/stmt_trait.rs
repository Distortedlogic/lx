use crate::ast::{AgentMethod, FieldDecl, SStmt, Stmt, TraitDeclData, TraitEntry, TraitMethodDecl, TraitUnionDef};
use crate::error::LxError;
use crate::lexer::token::TokenKind;
use miette::SourceSpan;

impl super::Parser {
  pub(super) fn parse_trait_decl(&mut self, mut exported: bool, start: usize) -> Result<SStmt, LxError> {
    self.advance();
    if *self.peek() == TokenKind::Plus {
      self.advance();
      exported = true;
    }
    let name = self.expect_type_name("Trait declaration")?;
    self.expect_kind(&TokenKind::Assign)?;
    if matches!(self.peek(), TokenKind::TypeName(_)) {
      return self.parse_trait_union(name, exported, start);
    }
    self.expect_kind(&TokenKind::LBrace)?;
    let mut entries = Vec::new();
    let mut methods = Vec::new();
    let mut defaults = Vec::new();
    let mut requires = Vec::new();
    let mut description = None;
    let mut tags = Vec::new();
    self.skip_semis();
    while *self.peek() != TokenKind::RBrace {
      if *self.peek() == TokenKind::DotDot {
        self.advance();
        let base = self.expect_type_name("Trait spread")?;
        entries.push(TraitEntry::Spread(base));
        self.skip_semis();
        continue;
      }
      let field = self.expect_ident("Trait field")?;
      if *self.peek() == TokenKind::Assign {
        self.advance();
        let handler = self.parse_expr(0)?;
        defaults.push(AgentMethod { name: field, handler });
      } else {
        self.expect_kind(&TokenKind::Colon)?;
        let is_meta_keyword = matches!(field.as_str(), "requires" | "description" | "tags") && !matches!(self.peek(), TokenKind::TypeName(_));
        if is_meta_keyword {
          match field.as_str() {
            "requires" => requires = self.parse_trait_symbol_list()?,
            "description" => description = Some(self.parse_trait_string()?),
            "tags" => tags = self.parse_trait_string_list()?,
            _ => unreachable!(),
          }
        } else if *self.peek() == TokenKind::LBrace {
          let method = self.parse_trait_method_braced(field)?;
          methods.push(method);
        } else {
          let type_name = self.expect_type_name("Trait field/method type")?;
          if *self.peek() == TokenKind::Arrow {
            self.advance();
            let output = self.expect_type_name("Trait method output type")?;
            methods.push(TraitMethodDecl { name: field, input: vec![FieldDecl { name: "_input".into(), type_name, default: None, constraint: None }], output });
          } else {
            let entry = self.parse_field_rest(field, type_name)?;
            entries.push(TraitEntry::Field(Box::new(entry)));
          }
        }
      }
      self.skip_semis();
    }
    let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
    Ok(SStmt::new(Stmt::TraitDecl(TraitDeclData { name, entries, methods, defaults, requires, description, tags, exported }), Span::from_range(start, end)))
  }

  fn parse_field_rest(&mut self, field_name: String, type_name: String) -> Result<FieldDecl, LxError> {
    let default = if *self.peek() == TokenKind::Assign {
      self.advance();
      Some(self.parse_expr(0)?)
    } else {
      None
    };
    let constraint = if let TokenKind::Ident(kw) = self.peek().clone()
      && kw == "where"
    {
      self.advance();
      Some(self.parse_expr(0)?)
    } else {
      None
    };
    Ok(FieldDecl { name: field_name, type_name, default, constraint })
  }

  fn parse_trait_union(&mut self, name: String, exported: bool, start: usize) -> Result<SStmt, LxError> {
    let mut variants = Vec::new();
    variants.push(self.expect_type_name("Trait union variant")?);
    while *self.peek() == TokenKind::Pipe {
      self.advance();
      variants.push(self.expect_type_name("Trait union variant")?);
    }
    let end = self.tokens[self.pos.saturating_sub(1)].span.end();
    Ok(SStmt::new(Stmt::TraitUnion(TraitUnionDef { name, variants, exported }), Span::from_range(start, end)))
  }

  fn parse_trait_method_braced(&mut self, name: String) -> Result<TraitMethodDecl, LxError> {
    let input = self.parse_trait_method_input()?;
    self.expect_kind(&TokenKind::Arrow)?;
    let output = self.expect_type_name("Trait method output type")?;
    Ok(TraitMethodDecl { name, input, output })
  }

  fn parse_trait_method_input(&mut self) -> Result<Vec<FieldDecl>, LxError> {
    self.expect_kind(&TokenKind::LBrace)?;
    let mut fields = Vec::new();
    self.skip_semis();
    while *self.peek() != TokenKind::RBrace {
      let field_name = self.expect_ident("Trait method input field")?;
      self.expect_kind(&TokenKind::Colon)?;
      let type_name = self.expect_type_name("Trait method input type")?;
      let default = if *self.peek() == TokenKind::Assign {
        self.advance();
        Some(self.parse_expr(0)?)
      } else {
        None
      };
      fields.push(FieldDecl { name: field_name, type_name, default, constraint: None });
      self.skip_semis();
    }
    self.expect_kind(&TokenKind::RBrace)?;
    Ok(fields)
  }

  fn parse_trait_symbol_list(&mut self) -> Result<Vec<String>, LxError> {
    self.expect_kind(&TokenKind::LBracket)?;
    let mut names = Vec::new();
    while *self.peek() != TokenKind::RBracket {
      match self.peek().clone() {
        TokenKind::Colon => {
          self.advance();
          if let TokenKind::Ident(sym) = self.peek().clone() {
            self.advance();
            names.push(format!(":{sym}"));
          }
        },
        TokenKind::Ident(n) | TokenKind::TypeName(n) => {
          self.advance();
          names.push(n);
        },
        _ => {
          return Err(LxError::parse("expected symbol in Trait requires list", self.tokens[self.pos].span, None));
        },
      }
      self.skip_semis();
    }
    self.expect_kind(&TokenKind::RBracket)?;
    Ok(names)
  }

  fn parse_trait_string(&mut self) -> Result<String, LxError> {
    match self.peek().clone() {
      TokenKind::StrStart => {
        self.advance();
        let mut result = String::new();
        loop {
          match self.peek().clone() {
            TokenKind::StrChunk(s) => {
              self.advance();
              result.push_str(&s);
            },
            TokenKind::StrEnd => {
              self.advance();
              break;
            },
            _ => break,
          }
        }
        Ok(result)
      },
      TokenKind::RawStr(s) => {
        self.advance();
        Ok(s)
      },
      _ => Err(LxError::parse("expected string in Trait description", self.tokens[self.pos].span, None)),
    }
  }

  fn parse_trait_string_list(&mut self) -> Result<Vec<String>, LxError> {
    self.expect_kind(&TokenKind::LBracket)?;
    let mut items = Vec::new();
    while *self.peek() != TokenKind::RBracket {
      items.push(self.parse_trait_string()?);
      self.skip_semis();
    }
    self.expect_kind(&TokenKind::RBracket)?;
    Ok(items)
  }
}
