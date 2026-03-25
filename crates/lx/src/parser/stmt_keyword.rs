use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::Span;
use super::expr::type_name;
use super::stmt::trait_body;
use super::stmt_class::class_body;
use crate::ast::{AgentMethod, ClassField, ExprId, KeywordDeclData, KeywordKind, TraitEntry};
use crate::lexer::token::TokenKind;

#[derive(Clone)]
enum KeywordBody {
  Class(Vec<ClassField>, Vec<AgentMethod>),
  Trait(Vec<TraitEntry>, Vec<AgentMethod>),
}

pub fn keyword_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, KeywordDeclData, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let schema_kw = just(TokenKind::SchemaKw).to(KeywordKind::Schema);

  let other_kw = choice((
    just(TokenKind::AgentKw).to(KeywordKind::Agent),
    just(TokenKind::ToolKw).to(KeywordKind::Tool),
    just(TokenKind::PromptKw).to(KeywordKind::Prompt),
    just(TokenKind::ConnectorKw).to(KeywordKind::Connector),
    just(TokenKind::StoreKw).to(KeywordKind::Store),
    just(TokenKind::SessionKw).to(KeywordKind::Session),
    just(TokenKind::GuardKw).to(KeywordKind::Guard),
    just(TokenKind::WorkflowKw).to(KeywordKind::Workflow),
    just(TokenKind::McpKw).to(KeywordKind::Mcp),
    just(TokenKind::CliKw).to(KeywordKind::Cli),
    just(TokenKind::HttpKw).to(KeywordKind::Http),
  ));

  let schema_branch = schema_kw
    .then(type_name())
    .then_ignore(just(TokenKind::Assign))
    .then_ignore(just(TokenKind::LBrace))
    .then(trait_body(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map(|((kw, name), (entries, defaults))| (kw, name, KeywordBody::Trait(entries, defaults)));

  let other_branch = other_kw
    .then(type_name())
    .then_ignore(just(TokenKind::Assign))
    .then(class_body(expr))
    .map(|((kw, name), (fields, methods))| (kw, name, KeywordBody::Class(fields, methods)));

  choice((schema_branch, other_branch)).map(|(kw, name, body)| match body {
    KeywordBody::Class(fields, methods) => KeywordDeclData { keyword: kw, name, type_params: vec![], fields, methods, trait_entries: None, exported: false },
    KeywordBody::Trait(entries, defaults) => {
      KeywordDeclData { keyword: kw, name, type_params: vec![], fields: vec![], methods: defaults, trait_entries: Some(entries), exported: false }
    },
  })
}
