use crate::ast::{AgentMethod, ProtocolField, SStmt, Stmt, TraitMethodDecl};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn parse_trait_decl(
        &mut self,
        mut exported: bool,
        start: u32,
    ) -> Result<SStmt, LxError> {
        self.advance();
        if *self.peek() == TokenKind::Plus {
            self.advance();
            exported = true;
        }
        let name = self.expect_type_name("Trait declaration")?;
        self.expect_kind(&TokenKind::Assign)?;
        self.expect_kind(&TokenKind::LBrace)?;
        let mut methods = Vec::new();
        let mut requires = Vec::new();
        let mut description = None;
        let mut tags = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let field = self.expect_ident("Trait field")?;
            self.expect_kind(&TokenKind::Colon)?;
            match field.as_str() {
                "requires" => {
                    requires = self.parse_trait_symbol_list()?;
                }
                "description" => {
                    description = Some(self.parse_trait_string()?);
                }
                "tags" => {
                    tags = self.parse_trait_string_list()?;
                }
                _ => {
                    let method = self.parse_trait_method(field)?;
                    methods.push(method);
                }
            }
            self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SStmt::new(
            Stmt::TraitDecl {
                name,
                methods,
                requires,
                description,
                tags,
                exported,
            },
            Span::from_range(start, end),
        ))
    }

    fn parse_trait_method(&mut self, name: String) -> Result<TraitMethodDecl, LxError> {
        if *self.peek() == TokenKind::LBrace {
            let input = self.parse_trait_method_input()?;
            self.expect_kind(&TokenKind::Arrow)?;
            let output = self.parse_mcp_output_type()?;
            Ok(TraitMethodDecl {
                name,
                input,
                output,
            })
        } else {
            let input_type = self.expect_type_name("Trait method input type")?;
            self.expect_kind(&TokenKind::Arrow)?;
            let output = self.parse_mcp_output_type()?;
            Ok(TraitMethodDecl {
                name,
                input: vec![ProtocolField {
                    name: "_input".into(),
                    type_name: input_type,
                    default: None,
                    constraint: None,
                }],
                output,
            })
        }
    }

    fn parse_trait_method_input(&mut self) -> Result<Vec<ProtocolField>, LxError> {
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
            fields.push(ProtocolField {
                name: field_name,
                type_name,
                default,
                constraint: None,
            });
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
                }
                TokenKind::Ident(n) | TokenKind::TypeName(n) => {
                    self.advance();
                    names.push(n);
                }
                _ => {
                    return Err(LxError::parse(
                        "expected symbol in Trait requires list",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
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
                        }
                        TokenKind::StrEnd => {
                            self.advance();
                            break;
                        }
                        _ => break,
                    }
                }
                Ok(result)
            }
            TokenKind::RawStr(s) => {
                self.advance();
                Ok(s)
            }
            _ => Err(LxError::parse(
                "expected string in Trait description",
                self.tokens[self.pos].span,
                None,
            )),
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
            let field_name = self.expect_ident("Agent body field")?;
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

    fn parse_agent_trait_list(&mut self) -> Result<Vec<String>, LxError> {
        if *self.peek() == TokenKind::LBracket {
            self.advance();
            let mut traits = Vec::new();
            while *self.peek() != TokenKind::RBracket {
                match self.peek().clone() {
                    TokenKind::TypeName(n) | TokenKind::Ident(n) => {
                        self.advance();
                        traits.push(n);
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
