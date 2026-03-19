use crate::ast::{AgentMethod, SStmt, Stmt};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn parse_agent_decl(
        &mut self,
        mut exported: bool,
        start: u32,
    ) -> Result<SStmt, LxError> {
        self.advance();
        if *self.peek() == TokenKind::Plus {
            self.advance();
            exported = true;
        }
        let name = self.expect_type_name("Agent declaration")?;
        let traits = if *self.peek() == TokenKind::Colon {
            self.advance();
            self.parse_agent_trait_list()?
        } else {
            Vec::new()
        };
        self.expect_kind(&TokenKind::Assign)?;
        self.expect_kind(&TokenKind::LBrace)?;
        let mut uses = Vec::new();
        let mut init = None;
        let mut on = None;
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
                        format!("expected identifier in Agent body, found {:?}", self.peek()),
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            };
            match field_name.as_str() {
                "uses" => {
                    self.expect_kind(&TokenKind::Colon)?;
                    uses = self.parse_agent_uses()?;
                }
                "init" => {
                    self.expect_kind(&TokenKind::Colon)?;
                    init = Some(self.parse_expr(0)?);
                }
                "on" => {
                    self.expect_kind(&TokenKind::Colon)?;
                    on = Some(self.parse_expr(0)?);
                }
                _ => {
                    self.expect_kind(&TokenKind::Assign)?;
                    let handler = self.parse_expr(0)?;
                    methods.push(AgentMethod {
                        name: field_name,
                        handler,
                    });
                }
            }
            self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SStmt::new(
            Stmt::AgentDecl {
                name,
                traits,
                uses,
                init,
                on,
                methods,
                exported,
            },
            Span::from_range(start, end),
        ))
    }

    pub(super) fn parse_agent_trait_list(&mut self) -> Result<Vec<String>, LxError> {
        if *self.peek() == TokenKind::LBracket {
            self.advance();
            let mut traits = Vec::new();
            while *self.peek() != TokenKind::RBracket {
                match self.peek().clone() {
                    TokenKind::TypeName(n) | TokenKind::Ident(n) => {
                        self.advance();
                        traits.push(n);
                    }
                    TokenKind::AgentKw => {
                        self.advance();
                        traits.push("Agent".into());
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected trait name",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                }
                self.skip_semis();
            }
            self.expect_kind(&TokenKind::RBracket)?;
            Ok(traits)
        } else {
            match self.peek().clone() {
                TokenKind::TypeName(n) | TokenKind::Ident(n) => {
                    self.advance();
                    Ok(vec![n])
                }
                TokenKind::AgentKw => {
                    self.advance();
                    Ok(vec!["Agent".into()])
                }
                _ => Err(LxError::parse(
                    "expected trait name after ':'",
                    self.tokens[self.pos].span,
                    None,
                )),
            }
        }
    }

    fn parse_agent_uses(&mut self) -> Result<Vec<(String, String)>, LxError> {
        self.expect_kind(&TokenKind::LBrace)?;
        let mut uses = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let binding = self.expect_ident("Agent uses binding")?;
            self.expect_kind(&TokenKind::Colon)?;
            let module = match self.peek().clone() {
                TokenKind::Ident(n) | TokenKind::TypeName(n) => {
                    self.advance();
                    n
                }
                _ => {
                    return Err(LxError::parse(
                        "expected module name in uses",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            };
            uses.push((binding, module));
            self.skip_semis();
        }
        self.expect_kind(&TokenKind::RBrace)?;
        Ok(uses)
    }
}
