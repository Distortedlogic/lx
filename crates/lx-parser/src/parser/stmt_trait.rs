use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::{ident, type_name};
use super::{ArenaRef, ExprId, Span};
use crate::lexer::token::TokenKind;
use lx_ast::ast::{AgentMethod, FieldDecl, Stmt, TraitDeclData, TraitEntry, TraitUnionDef};
use lx_span::sym::intern;

pub(super) fn trait_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  _arena: ArenaRef,
) -> impl Parser<'a, I, Stmt, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let trait_union = just(TokenKind::Trait)
    .ignore_then(just(TokenKind::Export).or_not())
    .ignore_then(type_name())
    .then(super::type_ann::generic_params())
    .then_ignore(just(TokenKind::Assign))
    .then(type_name().separated_by(just(TokenKind::Pipe)).at_least(1).collect::<Vec<_>>())
    .map(|((name, type_params), variants)| Stmt::TraitUnion(TraitUnionDef { name, type_params, variants, exported: false }));

  let trait_decl = just(TokenKind::Trait)
    .ignore_then(just(TokenKind::Export).or_not())
    .ignore_then(type_name())
    .then(super::type_ann::generic_params())
    .then_ignore(just(TokenKind::Assign))
    .then_ignore(just(TokenKind::LBrace))
    .then(trait_body(expr))
    .then_ignore(just(TokenKind::RBrace))
    .map(|((name, type_params), (entries, defaults))| {
      Stmt::TraitDecl(TraitDeclData {
        name,
        type_params,
        entries,
        methods: vec![],
        defaults,
        requires: vec![],
        description: None,
        tags: vec![],
        exported: false,
      })
    });

  trait_union.or(trait_decl)
}

pub(super) fn trait_body<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, (Vec<TraitEntry>, Vec<AgentMethod>), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread_entry = just(TokenKind::DotDot).ignore_then(type_name()).map(TraitEntry::Spread);

  let default_method =
    ident().then_ignore(just(TokenKind::Assign)).then(expr.clone()).map(|(name, handler)| TraitBodyItem::Default(AgentMethod { name, handler }));

  let field_entry = ident()
    .then_ignore(just(TokenKind::Colon))
    .then(type_name())
    .then(just(TokenKind::Assign).ignore_then(expr.clone()).or_not())
    .then(just(TokenKind::Ident(intern("where"))).ignore_then(expr).or_not())
    .map(|(((name, typ), default), constraint)| TraitBodyItem::Field(FieldDecl { name, type_name: typ, default, constraint }));

  let item = spread_entry.map(TraitBodyItem::Entry).or(default_method).or(field_entry);

  super::expr::skip_semis().ignore_then(item.separated_by(super::expr::skip_semis()).collect::<Vec<_>>()).then_ignore(super::expr::skip_semis()).map(|items| {
    let mut entries = Vec::new();
    let mut defaults = Vec::new();
    for item in items {
      match item {
        TraitBodyItem::Entry(e) => entries.push(e),
        TraitBodyItem::Default(m) => defaults.push(m),
        TraitBodyItem::Field(f) => entries.push(TraitEntry::Field(Box::new(f))),
      }
    }
    (entries, defaults)
  })
}

#[derive(Clone)]
enum TraitBodyItem {
  Entry(TraitEntry),
  Default(AgentMethod),
  Field(FieldDecl),
}
