use chumsky::prelude::*;

use super::TInput;
use super::ss;
use crate::ast::*;
use crate::lexer::token::TokenKind;

pub(super) fn program_parser<'a>() -> impl Parser<'a, TInput<'a>, Program, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let expr = super::expr::expr_parser();
  just(TokenKind::Semi)
    .repeated()
    .ignore_then(stmt_parser(expr).separated_by(just(TokenKind::Semi).repeated().at_least(1)).allow_trailing().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::Semi).repeated())
    .then_ignore(just(TokenKind::Eof))
    .map(|stmts| Program { stmts })
}

pub(super) fn stmt_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SStmt, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let use_stmt = use_parser();
  let exported = just(TokenKind::Export).or_not().map(|e| e.is_some());

  let trait_decl = trait_parser(expr.clone());
  let class_decl = class_parser(expr.clone());
  let type_def = type_def_parser();
  let binding = binding_parser(expr.clone());
  let field_update = field_update_parser(expr.clone());
  let pattern_binding = pattern_binding_parser(expr.clone());
  let expr_stmt = expr.clone().map_with(|e, ctx| SStmt::new(Stmt::Expr(e), ss(ctx.span())));

  choice((
    use_stmt,
    exported
      .clone()
      .then(trait_decl)
      .map_with(|(exp, mut f), e| {
        f(exp);
        SStmt::new(Stmt::Expr(SExpr::new(Expr::Literal(Literal::Unit), ss(e.span()))), ss(e.span()))
      })
      .labelled("trait-hack"),
    exported.clone().then(class_decl).map_with(|(_, _), e| SStmt::new(Stmt::Expr(SExpr::new(Expr::Literal(Literal::Unit), ss(e.span()))), ss(e.span()))),
    exported.clone().then(type_def).map_with(|(exp, (name, variants)), e| SStmt::new(Stmt::TypeDef { name, variants, exported: exp }, ss(e.span()))),
    exported.then(binding).map_with(|(exp, mut b), e| {
      b.exported = exp;
      SStmt::new(Stmt::Binding(b), ss(e.span()))
    }),
    field_update,
    pattern_binding,
    expr_stmt,
  ))
}

fn use_parser<'a>() -> impl Parser<'a, TInput<'a>, SStmt, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let path_seg = select! {
      TokenKind::Ident(n) => n,
      TokenKind::Yield => "yield".to_string(),
  };

  let dotdot_prefix = just(TokenKind::DotDot).then_ignore(just(TokenKind::Slash)).to("..".to_string());

  let dot_prefix = just(TokenKind::Dot).then_ignore(just(TokenKind::Slash)).to(".".to_string());

  let prefix = dotdot_prefix.repeated().collect::<Vec<_>>().then(dot_prefix.or_not()).map(|(mut dotdots, dot)| {
    if let Some(d) = dot {
      dotdots.push(d);
    }
    dotdots
  });

  let segments = path_seg.separated_by(just(TokenKind::Slash)).at_least(1).collect::<Vec<_>>();

  let alias = just(TokenKind::Colon).ignore_then(select! { TokenKind::Ident(n) => n }).map(UseKind::Alias);

  let selective = just(TokenKind::LBrace)
    .ignore_then(select! { TokenKind::Ident(n) => n, TokenKind::TypeName(n) => n }.separated_by(just(TokenKind::Semi).or_not()).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RBrace))
    .map(UseKind::Selective);

  let kind = alias.or(selective).or_not().map(|k| k.unwrap_or(UseKind::Whole));

  just(TokenKind::Use).ignore_then(prefix).then(segments).then(kind).map_with(|((mut prefix, segs), kind), e| {
    prefix.extend(segs);
    SStmt::new(Stmt::Use(UseStmt { path: prefix, kind }), ss(e.span()))
  })
}

fn binding_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, Binding, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let simple = select! { TokenKind::Ident(n) => n }
    .then(just(TokenKind::DeclMut).to(true).or(just(TokenKind::Assign).to(false)))
    .then(expr.clone())
    .map(|((name, mutable), value)| Binding { exported: false, mutable, target: BindTarget::Name(name), type_ann: None, value });

  let reassign = select! { TokenKind::Ident(n) => n }.then_ignore(just(TokenKind::Reassign)).then(expr.clone()).map(|(name, value)| Binding {
    exported: false,
    mutable: false,
    target: BindTarget::Reassign(name),
    type_ann: None,
    value,
  });

  let typed = select! { TokenKind::Ident(n) => n }
    .then_ignore(just(TokenKind::Colon))
    .then(super::type_ann::type_parser())
    .then_ignore(just(TokenKind::Assign))
    .then(expr.clone())
    .map(|((name, type_ann), value)| Binding { exported: false, mutable: false, target: BindTarget::Name(name), type_ann: Some(type_ann), value });

  choice((typed, reassign, simple))
}

fn pattern_binding_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SStmt, extra::Err<Rich<'a, TokenKind>>> + Clone {
  expr.clone().then(just(TokenKind::Assign).to(false).or(just(TokenKind::DeclMut).to(true))).then(expr.clone()).map_with(|((lhs, mutable), value), e| {
    let pat = expr_to_pattern(&lhs);
    SStmt::new(Stmt::Binding(Binding { exported: false, mutable, target: BindTarget::Pattern(pat), type_ann: None, value }), ss(e.span()))
  })
}

fn field_update_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SStmt, extra::Err<Rich<'a, TokenKind>>> + Clone {
  expr.clone().then_ignore(just(TokenKind::Reassign)).then(expr.clone()).try_map_with(|(target, value), e| {
    let (name, fields) = expr_to_field_chain(&target).map_err(|_| Rich::custom(e.span(), "expected name.field target for '<-'"))?;
    Ok(SStmt::new(Stmt::FieldUpdate { name, fields, value }, ss(e.span())))
  })
}

fn expr_to_field_chain(expr: &SExpr) -> Result<(String, Vec<String>), ()> {
  match &expr.node {
    Expr::FieldAccess { expr: inner, field: FieldKind::Named(f) } => match &inner.node {
      Expr::Ident(name) => Ok((name.clone(), vec![f.clone()])),
      Expr::FieldAccess { .. } => {
        let (name, mut fields) = expr_to_field_chain(inner)?;
        fields.push(f.clone());
        Ok((name, fields))
      },
      _ => Err(()),
    },
    _ => Err(()),
  }
}

fn expr_to_pattern(expr: &SExpr) -> SPattern {
  let span = expr.span;
  match &expr.node {
    Expr::Ident(name) => SPattern::new(Pattern::Bind(name.clone()), span),
    Expr::Literal(Literal::Unit) => SPattern::new(Pattern::Wildcard, span),
    Expr::Tuple(elems) => {
      let pats = elems.iter().map(expr_to_pattern).collect();
      SPattern::new(Pattern::Tuple(pats), span)
    },
    Expr::List(elems) => {
      let mut pats = Vec::new();
      let mut rest = None;
      for e in elems {
        match e {
          ListElem::Single(e) => pats.push(expr_to_pattern(e)),
          ListElem::Spread(e) => {
            if let Expr::Ident(name) = &e.node {
              rest = Some(name.clone());
            }
          },
        }
      }
      SPattern::new(Pattern::List { elems: pats, rest }, span)
    },
    Expr::Record(fields) => {
      let fps = fields
        .iter()
        .filter(|f| !f.is_spread)
        .filter_map(|f| {
          f.name.as_ref().map(|name| {
            let pattern = if let Expr::Ident(id) = &f.value.node {
              if id == name { None } else { Some(expr_to_pattern(&f.value)) }
            } else {
              Some(expr_to_pattern(&f.value))
            };
            FieldPattern { name: name.clone(), pattern }
          })
        })
        .collect();
      SPattern::new(Pattern::Record { fields: fps, rest: None }, span)
    },
    _ => SPattern::new(Pattern::Wildcard, span),
  }
}

fn type_def_parser<'a>() -> impl Parser<'a, TInput<'a>, (String, Vec<(String, usize)>), extra::Err<Rich<'a, TokenKind>>> + Clone {
  let variant = just(TokenKind::Pipe).ignore_then(select! { TokenKind::TypeName(n) => n }).then(
    any()
      .filter(|k: &TokenKind| !matches!(k, TokenKind::Pipe | TokenKind::Semi | TokenKind::Eof | TokenKind::RBrace))
      .repeated()
      .collect::<Vec<_>>()
      .map(|toks| toks.len()),
  );

  select! { TokenKind::TypeName(n) => n }
    .then(select! { TokenKind::Ident(_) => () }.repeated())
    .then_ignore(just(TokenKind::Assign))
    .then_ignore(just(TokenKind::Semi).repeated())
    .then(variant.repeated().at_least(1).collect::<Vec<_>>())
    .map(|((name, _), variants)| (name, variants))
}

fn trait_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, Box<dyn FnMut(bool) + 'a>, extra::Err<Rich<'a, TokenKind>>> + Clone {
  just(TokenKind::Trait)
    .ignore_then(just(TokenKind::Export).or_not())
    .ignore_then(select! { TokenKind::TypeName(n) => n })
    .ignore_then(just(TokenKind::Assign))
    .ignore_then(any().filter(|k: &TokenKind| !matches!(k, TokenKind::Eof)).repeated().collect::<Vec<_>>())
    .map(|_| -> Box<dyn FnMut(bool)> { Box::new(|_| {}) })
}

fn class_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, (), extra::Err<Rich<'a, TokenKind>>> + Clone {
  just(TokenKind::ClassKw)
    .ignore_then(just(TokenKind::Export).or_not())
    .ignore_then(select! { TokenKind::TypeName(n) => n })
    .ignore_then(any().filter(|k: &TokenKind| !matches!(k, TokenKind::Eof)).repeated())
    .ignored()
}
