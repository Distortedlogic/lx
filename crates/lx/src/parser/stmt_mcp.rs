use crate::ast::{McpOutputType, McpToolDecl, ProtocolField, SStmt, Stmt};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn parse_mcp_decl(&mut self, exported: bool, start: u32) -> Result<SStmt, LxError> {
        self.advance();
        let name = self.expect_type_name("MCP declaration")?;
        self.expect_kind(&TokenKind::Assign)?;
        self.expect_kind(&TokenKind::LBrace)?;
        let mut tools = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let tool_name = self.expect_ident("MCP tool name")?;
            self.expect_kind(&TokenKind::Colon)?;
            self.expect_kind(&TokenKind::LBrace)?;
            let mut input = Vec::new();
            self.skip_semis();
            while *self.peek() != TokenKind::RBrace {
                let field_name = self.expect_ident("MCP tool input field")?;
                self.expect_kind(&TokenKind::Colon)?;
                let type_name = self.expect_type_name("MCP tool input type")?;
                let default = if *self.peek() == TokenKind::Assign {
                    self.advance();
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                input.push(ProtocolField {
                    name: field_name,
                    type_name,
                    default,
                    constraint: None,
                });
                self.skip_semis();
            }
            self.expect_kind(&TokenKind::RBrace)?;
            self.expect_kind(&TokenKind::Arrow)?;
            let output = self.parse_mcp_output_type()?;
            tools.push(McpToolDecl {
                name: tool_name,
                input,
                output,
            });
            self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SStmt::new(
            Stmt::McpDecl {
                name,
                tools,
                exported,
            },
            Span::from_range(start, end),
        ))
    }

    fn parse_mcp_output_type(&mut self) -> Result<McpOutputType, LxError> {
        if *self.peek() == TokenKind::LBracket {
            self.advance();
            let inner = self.parse_mcp_output_type()?;
            self.expect_kind(&TokenKind::RBracket)?;
            return Ok(McpOutputType::List(Box::new(inner)));
        }
        if *self.peek() == TokenKind::LBrace {
            self.advance();
            let mut fields = Vec::new();
            self.skip_semis();
            while *self.peek() != TokenKind::RBrace {
                let field_name = self.expect_ident("MCP output field")?;
                self.expect_kind(&TokenKind::Colon)?;
                let type_name = self.expect_type_name("MCP output field type")?;
                fields.push(ProtocolField {
                    name: field_name,
                    type_name,
                    default: None,
                    constraint: None,
                });
                self.skip_semis();
            }
            self.expect_kind(&TokenKind::RBrace)?;
            return Ok(McpOutputType::Record(fields));
        }
        let n = self.expect_type_name("MCP output type")?;
        Ok(McpOutputType::Named(n))
    }
}
