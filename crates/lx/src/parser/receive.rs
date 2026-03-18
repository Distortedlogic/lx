use crate::ast::{Expr, ReceiveArm, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn parse_receive(&mut self, start: u32) -> Result<SExpr, LxError> {
        self.expect_kind(&TokenKind::LBrace)?;
        self.skip_semis();
        let mut arms = Vec::new();
        while *self.peek() != TokenKind::RBrace {
            let action = match self.peek().clone() {
                TokenKind::Ident(s) => {
                    self.advance();
                    s
                }
                TokenKind::Underscore => {
                    self.advance();
                    "_".to_string()
                }
                _ => {
                    let tok = &self.tokens[self.pos];
                    return Err(LxError::parse(
                        format!("receive arm: expected action name or _, got {:?}", tok.kind),
                        tok.span,
                        None,
                    ));
                }
            };
            self.expect_kind(&TokenKind::Arrow)?;
            let handler = self.parse_expr(0)?;
            arms.push(ReceiveArm { action, handler });
            self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SExpr::new(
            Expr::Receive(arms),
            Span::from_range(start, end),
        ))
    }
}
