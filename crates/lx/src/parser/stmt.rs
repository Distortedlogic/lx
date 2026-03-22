use crate::sym::intern;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::{ident, name_or_type, type_name};
use super::{Span, ss};
use crate::ast::{
  AgentMethod, BindTarget, Binding, Expr, FieldDecl, FieldKind, Program, SExpr, SStmt, Stmt, TraitDeclData, TraitEntry, TraitMethodDecl, TraitUnionDef,
  UseKind, UseStmt,
};
use crate::lexer::token::TokenKind;
use crate::sym::Sym;

pub(super) fn program_parser<'a, I>() -> impl Parser<'a, I, Program, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let expr = super::expr::expr_parser();
  super::expr::skip_semis()
    .ignore_then(stmt_parser(expr).separated_by(just(TokenKind::Semi).repeated().at_least(1)).allow_trailing().collect::<Vec<_>>())
    .then_ignore(super::expr::skip_semis())
    .then_ignore(just(TokenKind::Eof))
    .map(|stmts| Program { stmts })
}

pub(super) fn stmt_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SStmt, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let use_stmt = use_parser();
  let exported = just(TokenKind::Export).or_not().map(|e| e.is_some());

  let trait_stmt = trait_parser(expr.clone());
  let class_stmt = super::stmt_class::class_parser(expr.clone());
  let type_def = type_def_parser();
  let binding = binding_parser(expr.clone());
  let field_update = field_update_parser(expr.clone());
  let expr_stmt = expr.map_with(|e, ctx| SStmt::new(Stmt::Expr(e), ss(ctx.span())));

  choice((
    use_stmt,
    exported.clone().then(trait_stmt).map_with(|(exp, stmt), e| {
      let mut s = stmt;
      match &mut s {
        Stmt::TraitDecl(d) => d.exported = exp,
        Stmt::TraitUnion(d) => d.exported = exp,
        _ => {},
      }
      SStmt::new(s, ss(e.span()))
    }),
    exported.clone().then(class_stmt).map_with(|(exp, mut d), e| {
      d.exported = exp;
      SStmt::new(Stmt::ClassDecl(d), ss(e.span()))
    }),
    exported.clone().then(type_def).map_with(|(exp, (name, variants)), e| SStmt::new(Stmt::TypeDef { name, variants, exported: exp }, ss(e.span()))),
    exported.then(binding).map_with(|(exp, mut b), e| {
      b.exported = exp;
      SStmt::new(Stmt::Binding(b), ss(e.span()))
    }),
    field_update,
    expr_stmt,
  ))
}

fn use_parser<'a, I>() -> impl Parser<'a, I, SStmt, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let path_seg = select! {
      TokenKind::Ident(n) => n,
      TokenKind::Yield => intern("yield"),
  };

  let dotdot_prefix = just(TokenKind::DotDot).then_ignore(just(TokenKind::Slash)).to(intern(".."));

  let dot_prefix = just(TokenKind::Dot).then_ignore(just(TokenKind::Slash)).to(intern("."));

  let prefix_parts = dotdot_prefix.repeated().collect::<Vec<_>>().then(dot_prefix.or_not()).map(|(mut dd, dot)| {
    if let Some(d) = dot {
      dd.push(d);
    }
    dd
  });

  let segments = path_seg.separated_by(just(TokenKind::Slash)).at_least(1).collect::<Vec<_>>();

  let alias = just(TokenKind::Colon).ignore_then(ident()).map(UseKind::Alias);

  let selective = name_or_type()
    .separated_by(just(TokenKind::Semi).or_not())
    .collect::<Vec<_>>()
    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
    .map(UseKind::Selective);

  let kind = alias.or(selective).or_not().map(|k| k.unwrap_or(UseKind::Whole));

  just(TokenKind::Use).ignore_then(prefix_parts).then(segments).then(kind).map_with(|((mut prefix, segs), kind), e| {
    prefix.extend(segs);
    SStmt::new(Stmt::Use(UseStmt { path: prefix, kind }), ss(e.span()))
  })
}

fn binding_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, Binding, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let typed = ident()
    .then_ignore(just(TokenKind::Colon))
    .then(super::type_ann::type_parser())
    .then_ignore(just(TokenKind::Assign))
    .then(expr.clone())
    .map(|((name, type_ann), value)| Binding { exported: false, mutable: false, target: BindTarget::Name(name), type_ann: Some(type_ann), value });

  let reassign = ident().then_ignore(just(TokenKind::Reassign)).then(expr.clone()).map(|(name, value)| Binding {
    exported: false,
    mutable: false,
    target: BindTarget::Reassign(name),
    type_ann: None,
    value,
  });

  let simple = ident().then(just(TokenKind::DeclMut).to(true).or(just(TokenKind::Assign).to(false))).then(expr).map(|((name, mutable), value)| Binding {
    exported: false,
    mutable,
    target: BindTarget::Name(name),
    type_ann: None,
    value,
  });

  choice((typed, reassign, simple))
}

fn field_update_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SStmt, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  expr.clone().then_ignore(just(TokenKind::Reassign)).then(expr).try_map_with(|(target, value), e| {
    let (name, fields) = expr_to_field_chain(&target).map_err(|_| Rich::custom(e.span(), "'<-' requires name.field target"))?;
    Ok(SStmt::new(Stmt::FieldUpdate { name, fields, value }, ss(e.span())))
  })
}

fn expr_to_field_chain(expr: &SExpr) -> Result<(Sym, Vec<Sym>), ()> {
  match &expr.node {
    Expr::FieldAccess { expr: inner, field: FieldKind::Named(f) } => match &inner.node {
      Expr::Ident(name) => Ok((*name, vec![*f])),
      Expr::FieldAccess { .. } => {
        let (name, mut fields) = expr_to_field_chain(inner)?;
        fields.push(*f);
        Ok((name, fields))
      },
      _ => Err(()),
    },
    _ => Err(()),
  }
}

fn type_def_parser<'a, I>() -> impl Parser<'a, I, (Sym, Vec<(Sym, usize)>), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let variant = just(TokenKind::Pipe).ignore_then(type_name()).then(
    any()
      .filter(|k: &TokenKind| !matches!(k, TokenKind::Pipe | TokenKind::Semi | TokenKind::Eof | TokenKind::RBrace))
      .repeated()
      .collect::<Vec<_>>()
      .map(|toks| toks.len()),
  );

  type_name()
    .then(ident().ignored().repeated())
    .then_ignore(just(TokenKind::Assign))
    .then_ignore(super::expr::skip_semis())
    .then(variant.separated_by(super::expr::skip_semis()).at_least(1).collect::<Vec<_>>())
    .map(|((name, _), variants)| (name, variants))
}

fn trait_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, Stmt, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let trait_union = just(TokenKind::Trait)
    .ignore_then(just(TokenKind::Export).or_not())
    .ignore_then(type_name())
    .then_ignore(just(TokenKind::Assign))
    .then(type_name().separated_by(just(TokenKind::Pipe)).at_least(1).collect::<Vec<_>>())
    .map(|(name, variants)| Stmt::TraitUnion(TraitUnionDef { name, variants, exported: false }));

  let trait_decl = just(TokenKind::Trait)
    .ignore_then(just(TokenKind::Export).or_not())
    .ignore_then(type_name())
    .then_ignore(just(TokenKind::Assign))
    .then_ignore(just(TokenKind::LBrace))
    .then(trait_body(expr))
    .then_ignore(just(TokenKind::RBrace))
    .map(|(name, (entries, methods, defaults, requires, description, tags))| {
      Stmt::TraitDecl(TraitDeclData { name, entries, methods, defaults, requires, description, tags, exported: false })
    });

  trait_union.or(trait_decl)
}

type TraitBodyResult = (Vec<TraitEntry>, Vec<TraitMethodDecl>, Vec<AgentMethod>, Vec<Sym>, Option<Sym>, Vec<Sym>);

fn trait_body<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, TraitBodyResult, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
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
    .map(|((name, typ), default)| TraitBodyItem::Field(FieldDecl { name, type_name: typ, default, constraint: None }));

  let item = spread_entry.map(TraitBodyItem::Entry).or(default_method).or(field_entry);

  super::expr::skip_semis().ignore_then(item.separated_by(super::expr::skip_semis()).collect::<Vec<_>>()).then_ignore(super::expr::skip_semis()).map(|items| {
    let mut entries = Vec::new();
    let methods = Vec::new();
    let mut defaults = Vec::new();
    for item in items {
      match item {
        TraitBodyItem::Entry(e) => entries.push(e),
        TraitBodyItem::Default(m) => defaults.push(m),
        TraitBodyItem::Field(f) => entries.push(TraitEntry::Field(Box::new(f))),
      }
    }
    (entries, methods, defaults, Vec::new(), None, Vec::new())
  })
}

#[derive(Clone)]
enum TraitBodyItem {
  Entry(TraitEntry),
  Default(AgentMethod),
  Field(FieldDecl),
}
