use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::{ident, name_or_type, type_name};
use super::{ArenaRef, ExprId, Span, StmtId, ss};
use crate::lexer::token::TokenKind;
use lx_ast::ast::{AstArena, BindTarget, Binding, Expr, ExprFieldAccess, FieldKind, Stmt, StmtFieldUpdate, StmtTypeDef, UseKind, UseStmt};
use lx_span::sym::{Sym, intern};

pub(super) fn program_parser<'a, I>(arena: ArenaRef) -> impl Parser<'a, I, Vec<StmtId>, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let expr = super::expr::expr_parser(arena.clone());

  let skip_to_semi = any().and_is(none_of([TokenKind::Semi, TokenKind::Eof])).repeated().at_least(1).then(just(TokenKind::Semi).or_not()).ignored();

  let recoverable_stmt = stmt_parser(expr, arena).map(Some).recover_with(via_parser(skip_to_semi.map(|_| None)));

  super::expr::skip_semis()
    .ignore_then(recoverable_stmt.separated_by(just(TokenKind::Semi).repeated().at_least(1)).allow_trailing().collect::<Vec<_>>())
    .then_ignore(super::expr::skip_semis())
    .then_ignore(just(TokenKind::Eof))
    .map(|stmts: Vec<Option<StmtId>>| stmts.into_iter().flatten().collect())
}

pub(super) fn stmt_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let a1 = arena.clone();
  let a2 = arena.clone();
  let a3 = arena.clone();
  let a4 = arena.clone();
  let a5 = arena.clone();
  let a6 = arena.clone();
  let a_chan = arena.clone();

  let use_stmt = use_parser(arena.clone());
  let exported = just(TokenKind::Export).or_not().map(|e| e.is_some());
  let keyword_stmt = super::stmt_keyword::keyword_parser(expr.clone());
  let trait_stmt = super::stmt_trait::trait_parser(expr.clone(), arena.clone());
  let class_stmt = super::stmt_class::class_parser(expr.clone());
  let type_def = type_def_parser();
  let binding = binding_parser(expr.clone(), arena.clone());
  let field_update = field_update_parser(expr.clone(), arena);

  let channel_decl = just(TokenKind::ChannelKw)
    .ignore_then(select! { TokenKind::Ident(name) => name })
    .map_with(move |name, e| a_chan.borrow_mut().alloc_stmt(Stmt::ChannelDecl(name), ss(e.span())));
  let expr_stmt = expr.map_with(move |eid, ctx| a1.borrow_mut().alloc_stmt(Stmt::Expr(eid), ss(ctx.span())));

  choice((
    use_stmt,
    exported.clone().then(keyword_stmt).map_with(move |(exp, mut d), e| {
      d.exported = exp || d.exported;
      a6.borrow_mut().alloc_stmt(Stmt::KeywordDecl(d), ss(e.span()))
    }),
    exported.clone().then(trait_stmt).map_with(move |(exp, mut stmt), e| {
      match &mut stmt {
        Stmt::TraitDecl(d) => d.exported = exp,
        Stmt::TraitUnion(d) => d.exported = exp,
        _ => {},
      }
      a2.borrow_mut().alloc_stmt(stmt, ss(e.span()))
    }),
    exported.clone().then(class_stmt).map_with(move |(exp, mut d), e| {
      d.exported = exp;
      a3.borrow_mut().alloc_stmt(Stmt::ClassDecl(d), ss(e.span()))
    }),
    exported.clone().then(type_def).map_with(move |(exp, (name, type_params, variants)), e| {
      a4.borrow_mut().alloc_stmt(Stmt::TypeDef(StmtTypeDef { name, type_params, variants, exported: exp }), ss(e.span()))
    }),
    exported.then(binding).map_with(move |(exp, mut b), e| {
      b.exported = exp;
      a5.borrow_mut().alloc_stmt(Stmt::Binding(b), ss(e.span()))
    }),
    field_update,
    channel_decl,
    expr_stmt,
  ))
}

fn use_parser<'a, I>(arena: ArenaRef) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let arena_tool = arena.clone();
  let raw_string = just(TokenKind::StrStart).ignore_then(select! { TokenKind::StrChunk(s) => s }).then_ignore(just(TokenKind::StrEnd));

  let use_tool =
    just(TokenKind::Use).ignore_then(just(TokenKind::ToolKw)).ignore_then(raw_string).then_ignore(just(TokenKind::As)).then(name_or_type()).map_with(
      move |(command_str, alias), e| {
        let command = intern(&command_str);
        let stmt = Stmt::Use(UseStmt { path: vec![], kind: UseKind::Tool { command, alias } });
        arena_tool.borrow_mut().alloc_stmt(stmt, ss(e.span()))
      },
    );

  let path_seg = super::expr::ident_or_keyword();

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
    .separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)).or_not())
    .collect::<Vec<_>>()
    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
    .map(UseKind::Selective);

  let kind = alias.or(selective).or_not().map(|k| k.unwrap_or(UseKind::Whole));

  let use_path = just(TokenKind::Use).ignore_then(prefix_parts).then(segments).then(kind).map_with(move |((mut prefix, segs), kind), e| {
    prefix.extend(segs);
    arena.borrow_mut().alloc_stmt(Stmt::Use(UseStmt { path: prefix, kind }), ss(e.span()))
  });

  use_tool.or(use_path)
}

fn binding_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, Binding, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let typed = ident()
    .then_ignore(just(TokenKind::Colon))
    .then(super::type_ann::type_parser(arena))
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
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  expr.clone().then_ignore(just(TokenKind::Reassign)).then(expr).map_with(move |(target, value), e| {
    let (name, fields) = expr_to_field_chain(target, &arena.borrow());
    arena.borrow_mut().alloc_stmt(Stmt::FieldUpdate(StmtFieldUpdate { name, fields, value }), ss(e.span()))
  })
}

fn type_def_parser<'a, I>() -> impl Parser<'a, I, (Sym, Vec<Sym>, Vec<(Sym, usize)>), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let variant = just(TokenKind::Pipe).ignore_then(type_name()).then(
    any()
      .filter(|k: &TokenKind| !matches!(k, TokenKind::Pipe | TokenKind::Semi | TokenKind::Comma | TokenKind::Eof | TokenKind::RBrace))
      .repeated()
      .collect::<Vec<_>>()
      .map(|toks| toks.len()),
  );

  type_name()
    .then(super::type_ann::generic_params())
    .then_ignore(just(TokenKind::Assign))
    .then_ignore(super::expr::skip_semis())
    .then(variant.separated_by(super::expr::skip_semis()).at_least(1).collect::<Vec<_>>())
    .map(|((name, type_params), variants)| (name, type_params, variants))
}

fn expr_to_field_chain(id: ExprId, arena: &AstArena) -> (Sym, Vec<Sym>) {
  match arena.expr(id) {
    Expr::FieldAccess(ExprFieldAccess { expr: inner_id, field: FieldKind::Named(f) }) => {
      let f = *f;
      let inner_id = *inner_id;
      match arena.expr(inner_id) {
        Expr::Ident(name) => (*name, vec![f]),
        Expr::FieldAccess(_) => {
          let (name, mut fields) = expr_to_field_chain(inner_id, arena);
          fields.push(f);
          (name, fields)
        },
        _ => panic!("'<-' requires name.field target"),
      }
    },
    _ => panic!("'<-' requires name.field target"),
  }
}
