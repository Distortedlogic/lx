use crate::ast::{AgentMethod, ClassField, SStmt, Stmt};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn parse_class_decl(
        &mut self,
        mut exported: bool,
        start: u32,
    ) -> Result<SStmt, LxError> {
        self.advance();
        if *self.peek() == TokenKind::Plus {
            self.advance();
            exported = true;
        }
        let name = self.expect_type_name("Class declaration")?;
        let traits = if *self.peek() == TokenKind::Colon {
            self.advance();
            self.parse_agent_trait_list()?
        } else {
            Vec::new()
        };
        self.expect_kind(&TokenKind::Assign)?;
        self.expect_kind(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let field_name = match self.peek().clone() {
                TokenKind::Ident(n) | TokenKind::TypeName(n) => {
                    self.advance();
                    n
                }
                _ => {
                    return Err(LxError::parse(
                        format!("expected identifier in Class body, found {:?}", self.peek()),
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            };
            match self.peek() {
                TokenKind::Colon => {
                    self.advance();
                    let default = self.parse_expr(0)?;
                    fields.push(ClassField {
                        name: field_name,
                        default,
                    });
                }
                TokenKind::Assign => {
                    self.advance();
                    let handler = self.parse_expr(0)?;
                    methods.push(AgentMethod {
                        name: field_name,
                        handler,
                    });
                }
                _ => {
                    return Err(LxError::parse(
                        format!(
                            "expected ':' (field) or '=' (method) after '{}' in Class body, found {:?}",
                            field_name,
                            self.peek()
                        ),
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            }
            self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SStmt::new(
            Stmt::ClassDecl {
                name,
                traits,
                fields,
                methods,
                exported,
            },
            Span::from_range(start, end),
        ))
    }
}
