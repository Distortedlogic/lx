use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::type_name;
use super::stmt_class::class_body;
use super::{ArenaRef, Span, StmtId, ss};
use crate::ast::{KeywordDeclData, KeywordKind, Stmt};
use crate::lexer::token::TokenKind;

pub fn keyword_parser<'a, I>(
  expr: impl Parser<'a, I, crate::ast::ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let keyword = choice((
    just(TokenKind::AgentKw).to(KeywordKind::Agent),
    just(TokenKind::ToolKw).to(KeywordKind::Tool),
    just(TokenKind::PromptKw).to(KeywordKind::Prompt),
    just(TokenKind::ConnectorKw).to(KeywordKind::Connector),
    just(TokenKind::StoreKw).to(KeywordKind::Store),
    just(TokenKind::SessionKw).to(KeywordKind::Session),
    just(TokenKind::GuardKw).to(KeywordKind::Guard),
    just(TokenKind::WorkflowKw).to(KeywordKind::Workflow),
    just(TokenKind::SchemaKw).to(KeywordKind::Schema),
    just(TokenKind::McpKw).to(KeywordKind::Mcp),
    just(TokenKind::CliKw).to(KeywordKind::Cli),
    just(TokenKind::HttpKw).to(KeywordKind::Http),
  ));

  keyword
    .then(type_name())
    .then_ignore(just(TokenKind::Assign))
    .then(class_body(expr))
    .map_with(move |((kw, name), (fields, methods)), e| {
      let data = KeywordDeclData {
        keyword: kw,
        name,
        type_params: vec![],
        fields,
        methods,
        trait_entries: None,
        exported: false,
      };
      arena.borrow_mut().alloc_stmt(Stmt::KeywordDecl(data), ss(e.span()))
    })
}
