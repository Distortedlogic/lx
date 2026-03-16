use crate::ast::{
    AgentMethod, BindTarget, Binding, McpOutputType, McpToolDecl, ProtocolEntry, ProtocolField,
    ProtocolUnionDef, SStmt, Stmt, UseKind, UseStmt,
};
use crate::error::LxError;
use crate::span::Span;
use crate::token::TokenKind;

impl super::Parser {
    pub(super) fn try_parse_binding(&mut self, exported: bool) -> Result<Option<Binding>, LxError> {
        if !matches!(self.peek(), TokenKind::Ident(_)) {
            return Ok(None);
        }
        let next = self.tokens.get(self.pos + 1).map(|t| &t.kind);
        let (mutable, reassign, has_type) = match next {
            Some(TokenKind::Assign) => (false, false, false),
            Some(TokenKind::DeclMut) => (true, false, false),
            Some(TokenKind::Reassign) => (false, true, false),
            Some(TokenKind::Colon) => {
                if self.is_typed_binding() {
                    (false, false, true)
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };
        let TokenKind::Ident(name) = self.advance().clone().kind else {
            unreachable!()
        };
        let type_ann = if has_type {
            self.advance();
            let ty = self.parse_type()?;
            self.expect_kind(&TokenKind::Assign)?;
            Some(ty)
        } else {
            self.advance();
            None
        };
        let value = self.parse_expr(0)?;
        let target = if reassign {
            BindTarget::Reassign(name)
        } else {
            BindTarget::Name(name)
        };
        Ok(Some(Binding {
            exported,
            mutable,
            target,
            type_ann,
            value,
        }))
    }

    fn is_typed_binding(&self) -> bool {
        let mut j = self.pos + 2;
        j = self.skip_type_tokens(j);
        matches!(self.tokens.get(j).map(|t| &t.kind), Some(TokenKind::Assign))
    }

    pub(super) fn try_parse_type_def(
        &mut self,
        exported: bool,
        start: u32,
    ) -> Result<Option<SStmt>, LxError> {
        if !matches!(self.peek(), TokenKind::TypeName(_)) {
            return Ok(None);
        }
        let mut j = self.pos + 1;
        while matches!(
            self.tokens.get(j).map(|t| &t.kind),
            Some(TokenKind::Ident(_))
        ) {
            j += 1;
        }
        if self.tokens.get(j).map(|t| &t.kind) != Some(&TokenKind::Assign) {
            return Ok(None);
        }
        let TokenKind::TypeName(name) = self.advance().clone().kind else {
            unreachable!()
        };
        while matches!(self.peek(), TokenKind::Ident(_)) {
            self.advance();
        }
        self.expect_kind(&TokenKind::Assign)?;
        self.skip_semis();
        let mut variants = Vec::new();
        if *self.peek() == TokenKind::Pipe {
            while *self.peek() == TokenKind::Pipe {
                self.advance();
                let ctor_name = if let TokenKind::TypeName(n) = self.peek().clone() {
                    self.advance();
                    n
                } else {
                    continue;
                };
                let mut arity = 0usize;
                while matches!(
                    self.peek(),
                    TokenKind::TypeName(_)
                        | TokenKind::Ident(_)
                        | TokenKind::LParen
                        | TokenKind::LBracket
                        | TokenKind::LBrace
                        | TokenKind::PercentLBrace
                ) {
                    match self.peek() {
                        TokenKind::LParen
                        | TokenKind::LBracket
                        | TokenKind::LBrace
                        | TokenKind::PercentLBrace => {
                            let close = match self.peek() {
                                TokenKind::LParen => TokenKind::RParen,
                                TokenKind::LBracket => TokenKind::RBracket,
                                _ => TokenKind::RBrace,
                            };
                            self.advance();
                            let mut depth = 1u32;
                            while depth > 0 {
                                let k = self.peek().clone();
                                if k == TokenKind::Eof {
                                    break;
                                }
                                if std::mem::discriminant(&k) == std::mem::discriminant(&close) {
                                    depth -= 1;
                                } else if matches!(
                                    k,
                                    TokenKind::LParen
                                        | TokenKind::LBracket
                                        | TokenKind::LBrace
                                        | TokenKind::PercentLBrace
                                ) {
                                    depth += 1;
                                }
                                self.advance();
                            }
                        }
                        _ => {
                            self.advance();
                        }
                    }
                    arity += 1;
                }
                variants.push((ctor_name, arity));
                self.skip_semis();
            }
        } else {
            while !matches!(self.peek(), TokenKind::Semi | TokenKind::Eof) {
                self.advance();
            }
        }
        let end = self.tokens[self.pos.saturating_sub(1)].span.end();
        Ok(Some(SStmt::new(
            Stmt::TypeDef {
                name,
                variants,
                exported,
            },
            Span::from_range(start, end),
        )))
    }

    pub(super) fn parse_protocol(&mut self, exported: bool, start: u32) -> Result<SStmt, LxError> {
        self.advance();
        let name = match self.peek().clone() {
            TokenKind::TypeName(n) => {
                self.advance();
                n
            }
            _ => {
                return Err(LxError::parse(
                    "expected type name after 'Protocol'",
                    self.tokens[self.pos].span,
                    None,
                ));
            }
        };
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
        let first = match self.peek().clone() {
            TokenKind::TypeName(n) => {
                self.advance();
                n
            }
            _ => unreachable!(),
        };
        variants.push(first);
        while *self.peek() == TokenKind::Pipe {
            self.advance();
            let variant = match self.peek().clone() {
                TokenKind::TypeName(n) => {
                    self.advance();
                    n
                }
                _ => {
                    return Err(LxError::parse(
                        "expected type name after '|' in Protocol union",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            };
            variants.push(variant);
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
                let base = match self.peek().clone() {
                    TokenKind::TypeName(n) => {
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected Protocol name after '..'",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                };
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
        let field_name = match self.peek().clone() {
            TokenKind::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                return Err(LxError::parse(
                    "expected field name in Protocol",
                    self.tokens[self.pos].span,
                    None,
                ));
            }
        };
        self.expect_kind(&TokenKind::Colon)?;
        let type_name = match self.peek().clone() {
            TokenKind::TypeName(n) => {
                self.advance();
                n
            }
            _ => {
                return Err(LxError::parse(
                    "expected type name after ':'",
                    self.tokens[self.pos].span,
                    None,
                ));
            }
        };
        let default = if *self.peek() == TokenKind::Assign {
            self.advance();
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        let constraint =
            if let TokenKind::Ident(kw) = self.peek().clone()
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

    pub(super) fn parse_mcp_decl(&mut self, exported: bool, start: u32) -> Result<SStmt, LxError> {
        self.advance();
        let name = match self.peek().clone() {
            TokenKind::TypeName(n) => {
                self.advance();
                n
            }
            _ => {
                return Err(LxError::parse(
                    "expected type name after 'MCP'",
                    self.tokens[self.pos].span,
                    None,
                ));
            }
        };
        self.expect_kind(&TokenKind::Assign)?;
        self.expect_kind(&TokenKind::LBrace)?;
        let mut tools = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let tool_name = match self.peek().clone() {
                TokenKind::Ident(n) => {
                    self.advance();
                    n
                }
                _ => {
                    return Err(LxError::parse(
                        "expected tool name in MCP declaration",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            };
            self.expect_kind(&TokenKind::Colon)?;
            self.expect_kind(&TokenKind::LBrace)?;
            let mut input = Vec::new();
            self.skip_semis();
            while *self.peek() != TokenKind::RBrace {
                let field_name = match self.peek().clone() {
                    TokenKind::Ident(n) => {
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected field name in MCP tool input",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                };
                self.expect_kind(&TokenKind::Colon)?;
                let type_name = match self.peek().clone() {
                    TokenKind::TypeName(n) => {
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected type name after ':'",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                };
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
                let field_name = match self.peek().clone() {
                    TokenKind::Ident(n) => {
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected field name in MCP output type",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                };
                self.expect_kind(&TokenKind::Colon)?;
                let type_name = match self.peek().clone() {
                    TokenKind::TypeName(n) => {
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected type name after ':'",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                };
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
        match self.peek().clone() {
            TokenKind::TypeName(n) => {
                self.advance();
                Ok(McpOutputType::Named(n))
            }
            _ => Err(LxError::parse(
                "expected output type (TypeName, [...], or {...})",
                self.tokens[self.pos].span,
                None,
            )),
        }
    }

    pub(super) fn parse_trait_decl(
        &mut self,
        exported: bool,
        start: u32,
    ) -> Result<SStmt, LxError> {
        self.advance();
        let name = match self.peek().clone() {
            TokenKind::TypeName(n) => {
                self.advance();
                n
            }
            _ => {
                return Err(LxError::parse(
                    "expected type name after 'Trait'",
                    self.tokens[self.pos].span,
                    None,
                ));
            }
        };
        self.expect_kind(&TokenKind::Assign)?;
        self.expect_kind(&TokenKind::LBrace)?;
        let mut handles = Vec::new();
        let mut provides = Vec::new();
        let mut requires = Vec::new();
        self.skip_semis();
        while *self.peek() != TokenKind::RBrace {
            let field = match self.peek().clone() {
                TokenKind::Ident(n) => {
                    self.advance();
                    n
                }
                _ => {
                    return Err(LxError::parse(
                        "expected field name in Trait",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            };
            self.expect_kind(&TokenKind::Colon)?;
            self.expect_kind(&TokenKind::LBracket)?;
            let mut names = Vec::new();
            while *self.peek() != TokenKind::RBracket {
                match self.peek().clone() {
                    TokenKind::TypeName(n) | TokenKind::Ident(n) => {
                        self.advance();
                        names.push(n);
                    }
                    TokenKind::Colon => {
                        self.advance();
                        if let TokenKind::Ident(sym) = self.peek().clone() {
                            self.advance();
                            names.push(format!(":{sym}"));
                        }
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected name in Trait field list",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                }
                self.skip_semis();
            }
            self.expect_kind(&TokenKind::RBracket)?;
            match field.as_str() {
                "handles" => handles = names,
                "provides" => provides = names,
                "requires" => requires = names,
                _ => {
                    return Err(LxError::parse(
                        format!("unknown Trait field '{field}'"),
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            }
            self.skip_semis();
        }
        let end = self.expect_kind(&TokenKind::RBrace)?.span.end();
        Ok(SStmt::new(
            Stmt::TraitDecl {
                name,
                handles,
                provides,
                requires,
                exported,
            },
            Span::from_range(start, end),
        ))
    }

    pub(super) fn parse_agent_decl(
        &mut self,
        exported: bool,
        start: u32,
    ) -> Result<SStmt, LxError> {
        self.advance();
        let name = match self.peek().clone() {
            TokenKind::TypeName(n) => {
                self.advance();
                n
            }
            _ => {
                return Err(LxError::parse(
                    "expected type name after 'Agent'",
                    self.tokens[self.pos].span,
                    None,
                ));
            }
        };
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
                TokenKind::Ident(n) => n,
                _ => {
                    return Err(LxError::parse(
                        "expected field name in Agent body",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            };
            match field_name.as_str() {
                "uses" => {
                    self.advance();
                    self.expect_kind(&TokenKind::Colon)?;
                    uses = self.parse_agent_uses()?;
                }
                "init" => {
                    self.advance();
                    self.expect_kind(&TokenKind::Colon)?;
                    init = Some(self.parse_expr(0)?);
                }
                "on" => {
                    self.advance();
                    self.expect_kind(&TokenKind::Colon)?;
                    on = Some(self.parse_expr(0)?);
                }
                _ => {
                    self.advance();
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
            let binding = match self.peek().clone() {
                TokenKind::Ident(n) => {
                    self.advance();
                    n
                }
                _ => {
                    return Err(LxError::parse(
                        "expected binding name in uses",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            };
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

    pub(super) fn parse_use_stmt(&mut self, start: u32) -> Result<SStmt, LxError> {
        self.advance();
        let mut path = Vec::new();
        if *self.peek() == TokenKind::DotDot {
            self.advance();
            path.push("..".to_string());
            self.expect_kind(&TokenKind::Slash)?;
        } else if *self.peek() == TokenKind::Dot
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|t| t.kind == TokenKind::Slash)
        {
            self.advance();
            path.push(".".to_string());
            self.expect_kind(&TokenKind::Slash)?;
        }
        while let TokenKind::Ident(name) = self.peek().clone() {
            self.advance();
            path.push(name);
            if *self.peek() == TokenKind::Slash {
                self.advance();
            } else {
                break;
            }
        }
        if path.is_empty() {
            return Err(LxError::parse(
                "expected module path after 'use'",
                self.tokens[self.pos].span,
                None,
            ));
        }
        let kind = if *self.peek() == TokenKind::Colon {
            self.advance();
            match self.peek().clone() {
                TokenKind::Ident(name) => {
                    self.advance();
                    UseKind::Alias(name)
                }
                _ => {
                    return Err(LxError::parse(
                        "expected alias name after ':'",
                        self.tokens[self.pos].span,
                        None,
                    ));
                }
            }
        } else if *self.peek() == TokenKind::LBrace {
            self.advance();
            let mut names = Vec::new();
            while *self.peek() != TokenKind::RBrace {
                match self.peek().clone() {
                    TokenKind::Ident(name) | TokenKind::TypeName(name) => {
                        self.advance();
                        names.push(name);
                    }
                    _ => {
                        return Err(LxError::parse(
                            "expected name in selective import",
                            self.tokens[self.pos].span,
                            None,
                        ));
                    }
                }
                self.skip_semis();
            }
            self.expect_kind(&TokenKind::RBrace)?;
            UseKind::Selective(names)
        } else {
            UseKind::Whole
        };
        let end = self.tokens[self.pos.saturating_sub(1)].span.end();
        Ok(SStmt::new(
            Stmt::Use(UseStmt { path, kind }),
            Span::from_range(start, end),
        ))
    }
}
