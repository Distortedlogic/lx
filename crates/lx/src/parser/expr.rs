use chumsky::pratt::{infix, left, postfix, prefix, right};
use chumsky::prelude::*;

use super::{TInput, merge, ss};
use crate::ast::*;
use crate::lexer::token::TokenKind;

fn ident<'a>() -> impl Parser<'a, TInput<'a>, String, extra::Err<Rich<'a, TokenKind>>> + Clone {
  select! { TokenKind::Ident(n) => n }
}

fn type_name<'a>() -> impl Parser<'a, TInput<'a>, String, extra::Err<Rich<'a, TokenKind>>> + Clone {
  select! { TokenKind::TypeName(n) => n }
}

fn semi<'a>() -> impl Parser<'a, TInput<'a>, (), extra::Err<Rich<'a, TokenKind>>> + Clone {
  just(TokenKind::Semi).ignored()
}

fn skip_semis<'a>() -> impl Parser<'a, TInput<'a>, (), extra::Err<Rich<'a, TokenKind>>> + Clone {
  just(TokenKind::Semi).repeated().ignored()
}

pub(super) fn skip_semis_pub<'a>() -> impl Parser<'a, TInput<'a>, (), extra::Err<Rich<'a, TokenKind>>> + Clone {
  just(TokenKind::Semi).repeated().ignored()
}

pub(super) fn expr_parser<'a>() -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  recursive(|expr| {
    let literal = literal_parser();
    let string_lit = string_parser(expr.clone());
    let ident_expr = ident().map_with(|n, e| SExpr::new(Expr::Ident(n), ss(e.span())));
    let type_ctor = type_name().map_with(|n, e| SExpr::new(Expr::TypeConstructor(n), ss(e.span())));

    let list = list_parser(expr.clone());
    let block_or_record = block_or_record_parser(expr.clone());
    let map = map_parser(expr.clone());
    let paren = paren_parser(expr.clone());

    let loop_expr = just(TokenKind::Loop)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(stmts_until_rbrace(expr.clone()))
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|stmts, e| SExpr::new(Expr::Loop(stmts), ss(e.span())));

    let par_expr = just(TokenKind::Par)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(stmts_until_rbrace(expr.clone()))
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|stmts, e| SExpr::new(Expr::Par(stmts), ss(e.span())));

    let sel_arm = expr.clone().then_ignore(just(TokenKind::Arrow)).then(expr.clone()).map(|(expr, handler)| SelArm { expr, handler });

    let sel_expr = just(TokenKind::Sel)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(skip_semis())
      .ignore_then(sel_arm.separated_by(semi().then(skip_semis())).allow_trailing().collect::<Vec<_>>())
      .then_ignore(skip_semis())
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|arms, e| SExpr::new(Expr::Sel(arms), ss(e.span())));

    let break_expr =
      just(TokenKind::Break).ignore_then(atom_expr_start(expr.clone()).or_not()).map_with(|val, e| SExpr::new(Expr::Break(val.map(Box::new)), ss(e.span())));

    let assert_expr = just(TokenKind::Assert)
      .ignore_then(expr.clone())
      .then(expr.clone().and_is(just(TokenKind::Semi).or(just(TokenKind::Eof)).or(just(TokenKind::RBrace)).not()).or_not())
      .map_with(|(e, msg), ctx| SExpr::new(Expr::Assert { expr: Box::new(e), msg: msg.map(Box::new) }, ss(ctx.span())));

    let emit_expr = just(TokenKind::Emit).ignore_then(expr.clone()).map_with(|v, e| SExpr::new(Expr::Emit { value: Box::new(v) }, ss(e.span())));

    let yield_expr = just(TokenKind::Yield).ignore_then(expr.clone()).map_with(|v, e| SExpr::new(Expr::Yield { value: Box::new(v) }, ss(e.span())));

    let with_expr = with_parser(expr.clone());

    let atom = choice((
      literal,
      string_lit,
      paren,
      list,
      block_or_record,
      map,
      loop_expr,
      par_expr,
      sel_expr,
      break_expr,
      assert_expr,
      emit_expr,
      yield_expr,
      with_expr,
      type_ctor,
      ident_expr,
    ));

    pratt_expr(atom, expr.clone())
  })
}

fn literal_parser<'a>() -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  select! {
      TokenKind::Int(n) => Expr::Literal(Literal::Int(n)),
      TokenKind::Float(f) => Expr::Literal(Literal::Float(f)),
      TokenKind::True => Expr::Literal(Literal::Bool(true)),
      TokenKind::False => Expr::Literal(Literal::Bool(false)),
      TokenKind::Unit => Expr::Literal(Literal::Unit),
      TokenKind::RawStr(s) => Expr::Literal(Literal::RawStr(s)),
  }
  .map_with(|node, e| SExpr::new(node, ss(e.span())))
}

fn string_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let chunk = select! { TokenKind::StrChunk(s) => StrPart::Text(s) };
  let interp_braced = just(TokenKind::LBrace).ignore_then(expr.clone()).then_ignore(just(TokenKind::RBrace)).map(StrPart::Interp);
  let interp_bare = expr.clone().map(StrPart::Interp);
  let part = choice((chunk, interp_braced, interp_bare));

  just(TokenKind::StrStart)
    .ignore_then(part.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::StrEnd))
    .map_with(|parts, e| SExpr::new(Expr::Literal(Literal::Str(parts)), ss(e.span())))
}

fn list_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let spread = just(TokenKind::DotDot).ignore_then(expr.clone()).map(ListElem::Spread);
  let single = expr.clone().map(ListElem::Single);
  let elem = spread.or(single);

  just(TokenKind::LBracket)
    .ignore_then(elem.separated_by(semi().or_not()).allow_trailing().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RBracket))
    .map_with(|elems, e| SExpr::new(Expr::List(elems), ss(e.span())))
}

fn looks_like_record<'a>() -> impl Parser<'a, TInput<'a>, (), extra::Err<Rich<'a, TokenKind>>> + Clone {
  choice((
    ident().then_ignore(just(TokenKind::Colon)).ignored(),
    just(TokenKind::DotDot).ignored(),
    just(TokenKind::Colon).then(just(TokenKind::RBrace)).ignored(),
    ident().then(ident().or(semi().to("".to_string())).repeated().at_least(1)).then_ignore(just(TokenKind::RBrace).rewind()).ignored(),
  ))
}

fn record_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, Vec<RecordField>, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let spread_field = just(TokenKind::DotDot).ignore_then(expr.clone()).map(|value| RecordField { name: None, value, is_spread: true });

  let named_field = ident().then(just(TokenKind::Colon).ignore_then(expr.clone()).or_not()).map_with(|(name, val), e| {
    let value = val.unwrap_or_else(|| SExpr::new(Expr::Ident(name.clone()), ss(e.span())));
    RecordField { name: Some(name), value, is_spread: false }
  });

  let field = spread_field.or(named_field);

  skip_semis().ignore_then(field.separated_by(skip_semis()).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_semis())
}

fn block_or_record_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let empty_record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(just(TokenKind::Colon))
    .then(just(TokenKind::RBrace))
    .map_with(|_, e| SExpr::new(Expr::Record(vec![]), ss(e.span())));

  let record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(looks_like_record().rewind())
    .ignore_then(record_parser(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|fields, e| SExpr::new(Expr::Record(fields), ss(e.span())));

  let block = just(TokenKind::LBrace)
    .ignore_then(stmts_until_rbrace(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|stmts, e| SExpr::new(Expr::Block(stmts), ss(e.span())));

  choice((empty_record, record, block))
}

fn map_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let spread = just(TokenKind::DotDot).ignore_then(expr.clone()).map(|v| MapEntry { key: None, value: v, is_spread: true });

  let kv = expr.clone().then_ignore(just(TokenKind::Colon)).then(expr.clone()).map(|(k, v)| MapEntry { key: Some(k), value: v, is_spread: false });

  let entry = spread.or(kv);

  just(TokenKind::PercentLBrace)
    .ignore_then(entry.separated_by(semi().or_not()).allow_trailing().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|entries, e| SExpr::new(Expr::Map(entries), ss(e.span())))
}

pub(super) fn stmts_until_rbrace<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, Vec<SStmt>, extra::Err<Rich<'a, TokenKind>>> + Clone {
  skip_semis()
    .ignore_then(super::stmt::stmt_parser(expr).separated_by(semi().then(skip_semis())).allow_trailing().collect::<Vec<_>>().then_ignore(skip_semis()))
}

fn param_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, Param, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let typed = ident()
    .then(just(TokenKind::Colon).ignore_then(super::type_ann::type_parser()).or_not())
    .then(just(TokenKind::Assign).ignore_then(expr.clone()).or_not())
    .map(|((name, type_ann), default)| Param { name, type_ann, default });

  let underscore = just(TokenKind::Underscore).to(Param { name: "_".into(), type_ann: None, default: None });

  typed.or(underscore)
}

fn is_section_op(k: &TokenKind) -> bool {
  matches!(
    k,
    TokenKind::Plus
      | TokenKind::Star
      | TokenKind::Slash
      | TokenKind::Percent
      | TokenKind::IntDiv
      | TokenKind::PlusPlus
      | TokenKind::Eq
      | TokenKind::NotEq
      | TokenKind::Lt
      | TokenKind::Gt
      | TokenKind::LtEq
      | TokenKind::GtEq
      | TokenKind::And
      | TokenKind::Or
      | TokenKind::Minus
  )
}

fn section_op<'a>() -> impl Parser<'a, TInput<'a>, TokenKind, extra::Err<Rich<'a, TokenKind>>> + Clone {
  any().filter(|k: &TokenKind| is_section_op(k))
}

fn paren_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let unit = just(TokenKind::LParen).then(just(TokenKind::RParen)).map_with(|_, e| SExpr::new(Expr::Literal(Literal::Unit), ss(e.span())));

  let field_section = just(TokenKind::LParen)
    .ignore_then(just(TokenKind::Dot))
    .ignore_then(ident())
    .then_ignore(just(TokenKind::RParen))
    .map_with(|name, e| SExpr::new(Expr::Section(Section::Field(name)), ss(e.span())));

  let index_section =
    just(TokenKind::LParen).ignore_then(just(TokenKind::Dot)).ignore_then(select! { TokenKind::Int(n) => n }).then_ignore(just(TokenKind::RParen)).map_with(
      |n, e| {
        let idx: i64 = n.try_into().unwrap_or(0);
        SExpr::new(Expr::Section(Section::Index(idx)), ss(e.span()))
      },
    );

  let binop_section = just(TokenKind::LParen).ignore_then(section_op()).then_ignore(just(TokenKind::RParen)).map_with(|op_tok, e| {
    let op = super::token_to_binop(&op_tok).unwrap_or(BinOp::Add);
    SExpr::new(Expr::Section(Section::BinOp(op)), ss(e.span()))
  });

  let right_section =
    just(TokenKind::LParen).ignore_then(section_op()).then(expr.clone()).then_ignore(just(TokenKind::RParen)).map_with(|(op_tok, operand), e| {
      let op = super::token_to_binop(&op_tok).unwrap_or(BinOp::Add);
      SExpr::new(Expr::Section(Section::Right { op, operand: Box::new(operand) }), ss(e.span()))
    });

  let left_section =
    just(TokenKind::LParen).ignore_then(expr.clone()).then(section_op()).then_ignore(just(TokenKind::RParen)).map_with(|(operand, op_tok), e| {
      let op = super::token_to_binop(&op_tok).unwrap_or(BinOp::Add);
      SExpr::new(Expr::Section(Section::Left { operand: Box::new(operand), op }), ss(e.span()))
    });

  let func_def = just(TokenKind::LParen)
    .ignore_then(param_parser(expr.clone()).repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .then(just(TokenKind::Arrow).ignore_then(super::type_ann::type_parser()).or_not())
    .then(expr.clone())
    .map_with(|((params, ret_type), body), e| SExpr::new(Expr::Func { params, ret_type, body: Box::new(body) }, ss(e.span())));

  let tuple = just(TokenKind::LParen)
    .ignore_then(expr.clone().separated_by(semi().or_not()).at_least(2).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .map_with(|elems, e| SExpr::new(Expr::Tuple(elems), ss(e.span())));

  let grouped =
    just(TokenKind::LParen).ignore_then(expr.clone()).then_ignore(just(TokenKind::RParen)).map_with(|inner, e| SExpr::new(inner.node, ss(e.span())));

  choice((field_section, index_section, binop_section, right_section, func_def.clone(), unit, left_section, tuple, grouped))
}

fn with_parser<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let with_context = just(TokenKind::With)
    .ignore_then(just(TokenKind::Ident("context".into())))
    .ignore_then(ident().then_ignore(just(TokenKind::Colon)).then(expr.clone()).repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::LBrace))
    .then(stmts_until_rbrace(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(fields, body), e| SExpr::new(Expr::WithContext { fields, body }, ss(e.span())));

  let with_mut = just(TokenKind::With)
    .ignore_then(just(TokenKind::Ident("mut".into())).to(true).or_not().map(|x| x.unwrap_or(false)))
    .then(ident())
    .then(just(TokenKind::DeclMut).to(true).or(just(TokenKind::Assign).to(false)))
    .then(expr.clone())
    .then_ignore(just(TokenKind::LBrace))
    .then(stmts_until_rbrace(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|((((explicit_mut, name), is_decl_mut), value), body), e| {
      SExpr::new(Expr::With { name, value: Box::new(value), body, mutable: explicit_mut || is_decl_mut }, ss(e.span()))
    });

  let resource_binding = expr.clone().then_ignore(just(TokenKind::Ident("as".into()))).then(ident());

  let with_resource = just(TokenKind::With)
    .ignore_then(resource_binding.separated_by(just(TokenKind::Semi)).at_least(1).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::LBrace))
    .then(stmts_until_rbrace(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(resources, body), e| SExpr::new(Expr::WithResource { resources, body }, ss(e.span())));

  choice((with_context, with_mut, with_resource))
}

fn atom_expr_start<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  any()
    .filter(|k: &TokenKind| {
      matches!(
        k,
        TokenKind::Int(_)
          | TokenKind::Float(_)
          | TokenKind::StrStart
          | TokenKind::RawStr(_)
          | TokenKind::Ident(_)
          | TokenKind::TypeName(_)
          | TokenKind::LParen
          | TokenKind::LBracket
          | TokenKind::LBrace
          | TokenKind::True
          | TokenKind::False
          | TokenKind::Unit
          | TokenKind::Minus
          | TokenKind::Bang
          | TokenKind::PercentLBrace
      )
    })
    .rewind()
    .ignore_then(expr)
}

fn dot_access<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, FieldKind, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let named = ident().map(FieldKind::Named);
  let type_field = type_name().map(FieldKind::Named);
  let indexed = select! { TokenKind::Int(n) => n }.map(|n| {
    let idx: i64 = n.try_into().unwrap_or(0);
    FieldKind::Index(idx)
  });
  let neg_indexed = just(TokenKind::Minus).ignore_then(select! { TokenKind::Int(n) => n }).map(|n| {
    let idx: i64 = n.try_into().unwrap_or(0);
    FieldKind::Index(-idx)
  });
  let computed = just(TokenKind::LBracket).ignore_then(expr.clone()).then_ignore(just(TokenKind::RBracket)).map(|e| FieldKind::Computed(Box::new(e)));
  let str_key = string_parser(expr.clone()).map(|e| FieldKind::Computed(Box::new(e)));

  choice((named, type_field, indexed, neg_indexed, computed, str_key))
}

fn match_arms<'a>(
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, Vec<MatchArm>, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let arm = super::pattern::pattern_parser()
    .then(just(TokenKind::Amp).ignore_then(expr.clone()).or_not())
    .then_ignore(just(TokenKind::Arrow))
    .then(expr.clone())
    .map(|((pattern, guard), body)| MatchArm { pattern, guard, body });

  skip_semis().ignore_then(arm.separated_by(semi().then(skip_semis())).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_semis())
}

fn pratt_expr<'a>(
  atom: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
  expr: impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone,
) -> impl Parser<'a, TInput<'a>, SExpr, extra::Err<Rich<'a, TokenKind>>> + Clone {
  let match_block = just(TokenKind::LBrace).ignore_then(match_arms(expr.clone())).then_ignore(just(TokenKind::RBrace));

  let ternary_tail = expr.clone().then(just(TokenKind::Colon).ignore_then(expr.clone()).or_not());

  let question_rhs = match_block.map(QuestionRhs::Match).or(ternary_tail.map(|(then_, else_)| QuestionRhs::Ternary(then_, else_)));

  let named_arg = ident()
    .then_ignore(just(TokenKind::Colon))
    .then(expr.clone())
    .map_with(|(name, value), e| SExpr::new(Expr::NamedArg { name, value: Box::new(value) }, ss(e.span())));

  let app_arg = named_arg.or(expr.clone());

  let dot_rhs = dot_access(expr.clone());

  let slice_range = just(TokenKind::DotDot).ignore_then(select! { TokenKind::Int(n) => n }.or_not());

  atom
    .clone()
    .pratt((
      prefix(29, just(TokenKind::Minus), |_, operand: SExpr, e| SExpr::new(Expr::Unary { op: UnaryOp::Neg, operand: Box::new(operand) }, ss(e.span()))),
      prefix(29, just(TokenKind::Bang), |_, operand: SExpr, e| SExpr::new(Expr::Unary { op: UnaryOp::Not, operand: Box::new(operand) }, ss(e.span()))),
      postfix(10, just(TokenKind::Caret), |operand: SExpr, _, e| SExpr::new(Expr::Propagate(Box::new(operand)), ss(e.span()))),
      postfix(3, just(TokenKind::Question).then(question_rhs), |scrutinee: SExpr, (_, rhs), e| match rhs {
        QuestionRhs::Match(arms) => SExpr::new(Expr::Match { scrutinee: Box::new(scrutinee), arms }, ss(e.span())),
        QuestionRhs::Ternary(then_, else_) => {
          SExpr::new(Expr::Ternary { cond: Box::new(scrutinee), then_: Box::new(then_), else_: else_.map(Box::new) }, ss(e.span()))
        },
      }),
      postfix(
        33,
        just(TokenKind::Dot).then(dot_rhs.clone()).then(just(TokenKind::DotDot).ignore_then(select! { TokenKind::Int(n) => n }.or_not()).or_not()),
        |left: SExpr, ((_, field), slice_tail), e| {
          if let Some(end_opt) = slice_tail {
            if let FieldKind::Index(start_idx) = &field {
              let start_expr = SExpr::new(Expr::Literal(Literal::Int((*start_idx).into())), ss(e.span()));
              let end_expr = end_opt.map(|n| Box::new(SExpr::new(Expr::Literal(Literal::Int(n)), ss(e.span()))));
              return SExpr::new(Expr::Slice { expr: Box::new(left), start: Some(Box::new(start_expr)), end: end_expr }, ss(e.span()));
            }
          }
          SExpr::new(Expr::FieldAccess { expr: Box::new(left), field }, ss(e.span()))
        },
      ),
      infix(left(7), just(TokenKind::Amp), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::And, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(11), just(TokenKind::QQ), |l: SExpr, _, r: SExpr, e| SExpr::new(Expr::Coalesce { expr: Box::new(l), default: Box::new(r) }, ss(e.span()))),
      infix(left(13), just(TokenKind::Or), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Or, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(15), just(TokenKind::And), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::And, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(17), just(TokenKind::Eq), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Eq, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(17), just(TokenKind::NotEq), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::NotEq, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(17), just(TokenKind::Lt), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Lt, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(17), just(TokenKind::Gt), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Gt, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(17), just(TokenKind::LtEq), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::LtEq, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(17), just(TokenKind::GtEq), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::GtEq, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(19), just(TokenKind::Pipe), |l: SExpr, _, r: SExpr, e| SExpr::new(Expr::Pipe { left: Box::new(l), right: Box::new(r) }, ss(e.span()))),
      infix(left(21), just(TokenKind::PlusPlus), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Concat, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(23), just(TokenKind::DotDot), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Range, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(23), just(TokenKind::DotDotEq), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::RangeInclusive, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(25), just(TokenKind::Plus), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Add, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(25), just(TokenKind::Minus), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Sub, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(27), just(TokenKind::Star), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Mul, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(27), just(TokenKind::Slash), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Div, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(27), just(TokenKind::Percent), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::Mod, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
      infix(left(27), just(TokenKind::IntDiv), |l: SExpr, _, r: SExpr, e| {
        SExpr::new(Expr::Binary { op: BinOp::IntDiv, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
      }),
    ))
    .foldl(app_arg.repeated(), |func, arg| {
      let span = merge(
        SimpleSpan::new(func.span.offset(), func.span.offset() + func.span.len()),
        SimpleSpan::new(arg.span.offset(), arg.span.offset() + arg.span.len()),
      );
      SExpr::new(Expr::Apply { func: Box::new(func), arg: Box::new(arg) }, ss(span))
    })
}

enum QuestionRhs {
  Match(Vec<MatchArm>),
  Ternary(SExpr, Option<SExpr>),
}
