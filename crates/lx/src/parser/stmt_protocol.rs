use crate::ast::{ProtocolEntry, ProtocolField, ProtocolUnionDef, SStmt, Stmt};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn parse_protocol(&mut self, exported: bool, start: u32) -> Result<SStmt, LxError> {
        self.advance();
        let name = self.expect_type_name("Protocol declaration")?;
        self.expect_kind(&TokenKind::Assign)?;
        if matches!(self.peek(), TokenKind::TypeName(_)) {
            return self.parse_protocol_union(name, exported, start);
        }
        self.expect_kind(&TokenKind::LBrace)?;
        let entries = self.parse_protocol_entries()?;
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SStmt::new(
            Stmt::Protocol {
                name,
                entries,
                exported,
            },
            Span::from_range(start, end),
        ))
    }

    fn parse_protocol_union(
        &mut self,
        name: String,
        exported: bool,
        start: u32,
    ) -> Result<SStmt, LxError> {
        let mut variants = Vec::new();
        variants.push(self.expect_type_name("Protocol union variant")?);
        while *self.peek() == TokenKind::Pipe {
            self.advance();
            variants.push(self.expect_type_name("Protocol union variant")?)
        }
        let end = self.tokens[self.pos.saturating_sub(1)].span.end();
        Ok(SStmt::new(
            Stmt::ProtocolUnion(ProtocolUnionDef {
                name,
                variants,
                exported,
            }),
            Span::from_range(start, end),
        ))
    }

    fn parse_protocol_entries(&mut self) -> Result<Vec<ProtocolEntry>, LxError> {
        let mut entries = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            if *self.peek() == TokenKind::DotDot {
                self.advance();
                let base = self.expect_type_name("Protocol spread")?;
                entries.push(ProtocolEntry::Spread(base));
                self.skip_semis();
                continue;
            }
            let field = self.parse_protocol_field()?;
            entries.push(ProtocolEntry::Field(field));
            self.skip_semis();
        }
        Ok(entries)
    }

    fn parse_protocol_field(&mut self) -> Result<ProtocolField, LxError> {
        let field_name = self.expect_ident("Protocol field")?;
        self.expect_kind(&TokenKind::Colon)?;
        let type_name = self.expect_type_name("Protocol field type")?;
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
        Ok(ProtocolField {
            name: field_name,
            type_name,
            default,
            constraint,
        })
    }
}
