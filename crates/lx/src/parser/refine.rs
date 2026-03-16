use crate::ast::{Expr, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(crate) fn parse_refine(&mut self, start: u32) -> Result<SExpr, LxError> {
        let initial = self.parse_expr(0)?;
        self.expect_kind(&TokenKind::LBrace)?;
        let mut grade = None;
        let mut revise = None;
        let mut threshold = None;
        let mut max_rounds = None;
        let mut on_round = None;
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let field_tok = self.advance().clone();
            let field_name = match &field_tok.kind {
                TokenKind::Ident(n) => n.clone(),
                _ => {
                    return Err(LxError::parse(
                        "expected field name in refine",
                        field_tok.span,
                        None,
                    ));
                }
            };
            self.expect_kind(&TokenKind::Colon)?;
            let value = self.parse_expr(0)?;
            match field_name.as_str() {
                "grade" => grade = Some(Box::new(value)),
                "revise" => revise = Some(Box::new(value)),
                "threshold" => threshold = Some(Box::new(value)),
                "max_rounds" => max_rounds = Some(Box::new(value)),
                "on_round" => on_round = Some(Box::new(value)),
                _ => {
                    return Err(LxError::parse(
                        format!("unknown refine field '{field_name}'"),
                        field_tok.span,
                        None,
                    ));
                }
            }
            self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        let span = Span::from_range(start, end);
        let grade =
            grade.ok_or_else(|| LxError::parse("refine requires 'grade' field", span, None))?;
        let revise =
            revise.ok_or_else(|| LxError::parse("refine requires 'revise' field", span, None))?;
        let threshold = threshold
            .ok_or_else(|| LxError::parse("refine requires 'threshold' field", span, None))?;
        let max_rounds = max_rounds
            .ok_or_else(|| LxError::parse("refine requires 'max_rounds' field", span, None))?;
        Ok(SExpr::new(
            Expr::Refine {
                initial: Box::new(initial),
                grade,
                revise,
                threshold,
                max_rounds,
                on_round,
            },
            span,
        ))
    }
}
