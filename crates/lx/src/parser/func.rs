use crate::token::TokenKind;

impl super::Parser {
  pub(super) fn is_func_def(&self) -> bool {
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
            i = self.skip_type_tokens(i);
          }
          if self.tokens.get(i).map(|t| &t.kind) == Some(&TokenKind::Assign) {
            strong = true;
            i += 1;
            i = self.skip_default_expr(i);
          }
        },
        Some(TokenKind::RParen) => {
          i += 1;
          if self.tokens.get(i).map(|t| &t.kind) == Some(&TokenKind::Arrow) {
            strong = true;
            i += 1;
            i = self.skip_type_tokens(i);
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

  pub(super) fn skip_type_tokens(&self, mut i: usize) -> usize {
    loop {
      match self.tokens.get(i).map(|t| &t.kind) {
        Some(TokenKind::TypeName(_)) => i += 1,
        Some(TokenKind::Arrow | TokenKind::Caret) => i += 1,
        Some(TokenKind::LBrace)
          if matches!(
            (self.tokens.get(i + 1).map(|t| &t.kind), self.tokens.get(i + 2).map(|t| &t.kind)),
            (Some(TokenKind::Ident(_)), Some(TokenKind::Colon))
          ) => {
          i += 1;
          let mut depth = 1u32;
          while depth > 0 {
            match self.tokens.get(i).map(|t| &t.kind) {
              None | Some(TokenKind::Eof) => return i,
              Some(TokenKind::RBrace) => { depth -= 1; i += 1; },
              Some(TokenKind::LBrace) => { depth += 1; i += 1; },
              _ => i += 1,
            }
          }
        },
        Some(TokenKind::LParen | TokenKind::LBracket | TokenKind::PercentLBrace) => {
          let close = match self.tokens[i].kind {
            TokenKind::LParen => TokenKind::RParen,
            TokenKind::LBracket => TokenKind::RBracket,
            _ => TokenKind::RBrace,
          };
          i += 1;
          let mut depth = 1u32;
          while depth > 0 {
            match self.tokens.get(i).map(|t| &t.kind) {
              None | Some(TokenKind::Eof) => return i,
              Some(k) if std::mem::discriminant(k) == std::mem::discriminant(&close) => { depth -= 1; i += 1; },
              Some(TokenKind::LParen | TokenKind::LBracket | TokenKind::PercentLBrace) => { depth += 1; i += 1; },
              _ => i += 1,
            }
          }
        },
        _ => break,
      }
    }
    i
  }

  fn skip_default_expr(&self, mut i: usize) -> usize {
    let mut depth = 0i32;
    loop {
      match self.tokens.get(i).map(|t| &t.kind) {
        None | Some(TokenKind::Eof) => return i,
        Some(TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace) => {
          depth += 1;
          i += 1;
        },
        Some(TokenKind::RParen) if depth > 0 => { depth -= 1; i += 1; },
        Some(TokenKind::RBracket | TokenKind::RBrace) if depth > 0 => { depth -= 1; i += 1; },
        Some(TokenKind::RParen) if depth == 0 => break,
        Some(TokenKind::Ident(_)) if depth == 0 => break,
        _ => i += 1,
      }
    }
    i
  }
}

pub(super) fn is_expr_start_kind(k: &TokenKind) -> bool {
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
      | TokenKind::Yield
      | TokenKind::With
      | TokenKind::Dollar
      | TokenKind::DollarCaret
      | TokenKind::DollarBrace
  )
}
