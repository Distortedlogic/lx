use crate::ast::{Expr, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(crate) fn looks_like_meta_block(&self) -> bool {
        let j = self.skip_meta_task_expr(self.pos);
        if j == self.pos {
            return false;
        }
        matches!(self.tokens.get(j).map(|t| &t.kind), Some(TokenKind::LBrace))
    }

    fn skip_meta_task_expr(&self, i: usize) -> usize {
        match self.tokens.get(i).map(|t| &t.kind) {
            Some(TokenKind::Ident(_)) | Some(TokenKind::TypeName(_)) => i + 1,
            Some(TokenKind::Int(_))
            | Some(TokenKind::Float(_))
            | Some(TokenKind::True)
            | Some(TokenKind::False)
            | Some(TokenKind::Unit)
            | Some(TokenKind::RawStr(_))
            | Some(TokenKind::Regex(_)) => i + 1,
            Some(TokenKind::StrStart) => {
                let mut j = i + 1;
                loop {
                    match self.tokens.get(j).map(|t| &t.kind) {
                        Some(TokenKind::StrEnd) => return j + 1,
                        None | Some(TokenKind::Eof) => return i,
                        _ => j += 1,
                    }
                }
            }
            Some(TokenKind::LParen) => self.skip_balanced(i, &TokenKind::RParen),
            Some(TokenKind::LBracket) => self.skip_balanced(i, &TokenKind::RBracket),
            _ => i,
        }
    }

    fn skip_balanced(&self, start: usize, close: &TokenKind) -> usize {
        let mut j = start + 1;
        let mut depth = 1u32;
        while depth > 0 {
            match self.tokens.get(j).map(|t| &t.kind) {
                None | Some(TokenKind::Eof) => return start,
                Some(k) if std::mem::discriminant(k) == std::mem::discriminant(close) => {
                    depth -= 1;
                    j += 1;
                }
                Some(TokenKind::LParen | TokenKind::LBracket) => {
                    depth += 1;
                    j += 1;
                }
                _ => j += 1,
            }
        }
        j
    }

    pub(crate) fn parse_meta(&mut self, start: u32) -> Result<SExpr, LxError> {
        let task = self.parse_expr(32)?;
        self.expect_kind(&TokenKind::LBrace)?;
        let mut strategies = None;
        let mut attempt = None;
        let mut evaluate = None;
        let mut select = None;
        let mut on_switch = None;
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let field_tok = self.advance().clone();
            let field_name = match &field_tok.kind {
                TokenKind::Ident(n) => n.clone(),
                _ => {
                    return Err(LxError::parse(
                        "expected field name in meta",
                        field_tok.span,
                        None,
                    ));
                }
            };
            self.expect_kind(&TokenKind::Colon)?;
            let value = self.parse_expr(0)?;
            match field_name.as_str() {
                "strategies" => strategies = Some(Box::new(value)),
                "attempt" => attempt = Some(Box::new(value)),
                "evaluate" => evaluate = Some(Box::new(value)),
                "select" => select = Some(Box::new(value)),
                "on_switch" => on_switch = Some(Box::new(value)),
                _ => {
                    return Err(LxError::parse(
                        format!("unknown meta field '{field_name}'"),
                        field_tok.span,
                        None,
                    ));
                }
            }
            self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        let span = Span::from_range(start, end);
        let strategies = strategies
            .ok_or_else(|| LxError::parse("meta requires 'strategies' field", span, None))?;
        let attempt =
            attempt.ok_or_else(|| LxError::parse("meta requires 'attempt' field", span, None))?;
        let evaluate =
            evaluate.ok_or_else(|| LxError::parse("meta requires 'evaluate' field", span, None))?;
        Ok(SExpr::new(
            Expr::Meta {
                task: Box::new(task),
                strategies,
                attempt,
                evaluate,
                select,
                on_switch,
            },
            span,
        ))
    }
}
