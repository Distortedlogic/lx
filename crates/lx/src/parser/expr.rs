use crate::sym::intern;
use chumsky::input::ValueInput;
use chumsky::pratt::{Operator, infix, left, postfix, prefix};
use chumsky::prelude::*;

use super::{Span, ss, token_to_binop};
use crate::ast::{BinOp, Expr, FieldKind, ListElem, Literal, MapEntry, MatchArm, Param, RecordField, SExpr, SStmt, Section, SelArm, StrPart, UnaryOp};
use crate::lexer::token::TokenKind;

fn ident<'a, I>() -> impl Parser<'a, I, String, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! { TokenKind::Ident(n) => n }
}

fn type_name<'a, I>() -> impl Parser<'a, I, String, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  select! { TokenKind::TypeName(n) => n }
}

pub(super) fn skip_semis<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  just(TokenKind::Semi).repeated().ignored()
}

fn semi_sep<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  just(TokenKind::Semi).repeated().at_least(1).ignored()
}

pub(super) fn expr_parser<'a, I>() -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  recursive(|expr| {
    let literal = select! {
        TokenKind::Int(n) => Expr::Literal(Literal::Int(n)),
        TokenKind::Float(f) => Expr::Literal(Literal::Float(f)),
        TokenKind::True => Expr::Literal(Literal::Bool(true)),
        TokenKind::False => Expr::Literal(Literal::Bool(false)),
        TokenKind::Unit => Expr::Literal(Literal::Unit),
        TokenKind::RawStr(s) => Expr::Literal(Literal::RawStr(s)),
    }
    .map_with(|node, e| SExpr::new(node, ss(e.span())));

    let string_lit = string_parser(expr.clone());
    let ident_expr = ident().map_with(|n, e| SExpr::new(Expr::Ident(n), ss(e.span())));
    let type_ctor = type_name().map_with(|n, e| SExpr::new(Expr::TypeConstructor(n), ss(e.span())));
    let list = list_parser(expr.clone());
    let block_or_record = block_or_record_parser(expr.clone());
    let map = map_parser(expr.clone());
    let paren = paren_parser(expr.clone());

    let loop_expr = just(TokenKind::Loop)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(stmts_block(expr.clone()))
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|stmts, e| SExpr::new(Expr::Loop(stmts), ss(e.span())));

    let par_expr = just(TokenKind::Par)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(stmts_block(expr.clone()))
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|stmts, e| SExpr::new(Expr::Par(stmts), ss(e.span())));

    let sel_arm = expr.clone().then_ignore(just(TokenKind::Arrow)).then(expr.clone()).map(|(ex, handler)| SelArm { expr: ex, handler });

    let sel_expr = just(TokenKind::Sel)
      .ignore_then(just(TokenKind::LBrace))
      .ignore_then(skip_semis())
      .ignore_then(sel_arm.separated_by(semi_sep()).allow_trailing().collect::<Vec<_>>())
      .then_ignore(skip_semis())
      .then_ignore(just(TokenKind::RBrace))
      .map_with(|arms, e| SExpr::new(Expr::Sel(arms), ss(e.span())));

    let break_expr = just(TokenKind::Break).ignore_then(expr.clone().or_not()).map_with(|val, e| SExpr::new(Expr::Break(val.map(Box::new)), ss(e.span())));

    let assert_expr = just(TokenKind::Assert)
      .ignore_then(expr.clone())
      .then(expr.clone().or_not())
      .map_with(|(ex, msg), e| SExpr::new(Expr::Assert { expr: Box::new(ex), msg: msg.map(Box::new) }, ss(e.span())));

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
      emit_expr,
      yield_expr,
      with_expr,
      break_expr,
      assert_expr,
      type_ctor,
      ident_expr,
    ))
    .boxed();

    pratt_expr(atom, expr)
  })
}

fn string_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let chunk = select! { TokenKind::StrChunk(s) => StrPart::Text(s) };
  let interp_braced = just(TokenKind::LBrace).ignore_then(expr.clone()).then_ignore(just(TokenKind::RBrace)).map(StrPart::Interp);
  let interp_bare = expr.map(StrPart::Interp);
  let part = choice((chunk, interp_braced, interp_bare));

  just(TokenKind::StrStart)
    .ignore_then(part.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::StrEnd))
    .map_with(|parts, e| SExpr::new(Expr::Literal(Literal::Str(parts)), ss(e.span())))
}

fn list_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread = just(TokenKind::DotDot).ignore_then(expr.clone()).map(ListElem::Spread);
  let single = expr.map(ListElem::Single);
  let elem = spread.or(single);

  elem
    .separated_by(just(TokenKind::Semi).or_not())
    .allow_trailing()
    .collect::<Vec<_>>()
    .delimited_by(just(TokenKind::LBracket), just(TokenKind::RBracket))
    .map_with(|elems, e| SExpr::new(Expr::List(elems), ss(e.span())))
}

fn block_or_record_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let empty_record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(just(TokenKind::Colon))
    .then(just(TokenKind::RBrace))
    .map_with(|_, e| SExpr::new(Expr::Record(vec![]), ss(e.span())));

  let record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(looks_like_record().rewind())
    .ignore_then(record_fields(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|fields, e| SExpr::new(Expr::Record(fields), ss(e.span())));

  let block = just(TokenKind::LBrace)
    .ignore_then(stmts_block(expr))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|stmts, e| SExpr::new(Expr::Block(stmts), ss(e.span())));

  choice((empty_record, record, block))
}

fn looks_like_record<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  choice((ident().then_ignore(just(TokenKind::Colon)).ignored(), just(TokenKind::DotDot).ignored()))
}

fn record_fields<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, Vec<RecordField>, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread_field = just(TokenKind::DotDot).ignore_then(expr.clone()).map(|value| RecordField { name: None, value, is_spread: true });

  let named_field = ident().then(just(TokenKind::Colon).ignore_then(expr).or_not()).map_with(|(name, val), e| {
    let value = val.unwrap_or_else(|| SExpr::new(Expr::Ident(name.clone()), ss(e.span())));
    RecordField { name: Some(name), value, is_spread: false }
  });

  let field = spread_field.or(named_field);

  skip_semis().ignore_then(field.separated_by(skip_semis()).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_semis())
}

fn map_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let spread = just(TokenKind::DotDot).ignore_then(expr.clone()).map(|v| MapEntry { key: None, value: v, is_spread: true });

  let kv = expr.clone().then_ignore(just(TokenKind::Colon)).then(expr).map(|(k, v)| MapEntry { key: Some(k), value: v, is_spread: false });

  let entry = spread.or(kv);

  entry
    .separated_by(just(TokenKind::Semi).or_not())
    .allow_trailing()
    .collect::<Vec<_>>()
    .delimited_by(just(TokenKind::PercentLBrace), just(TokenKind::RBrace))
    .map_with(|entries, e| SExpr::new(Expr::Map(entries), ss(e.span())))
}

pub(super) fn stmts_block<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, Vec<SStmt>, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  skip_semis().ignore_then(super::stmt::stmt_parser(expr).separated_by(semi_sep()).allow_trailing().collect::<Vec<_>>().then_ignore(skip_semis()))
}

fn paren_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
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
    let op = token_to_binop(&op_tok).unwrap_or(BinOp::Add);
    SExpr::new(Expr::Section(Section::BinOp(op)), ss(e.span()))
  });

  let right_section =
    just(TokenKind::LParen).ignore_then(section_op()).then(expr.clone()).then_ignore(just(TokenKind::RParen)).map_with(|(op_tok, operand), e| {
      let op = token_to_binop(&op_tok).unwrap_or(BinOp::Add);
      SExpr::new(Expr::Section(Section::Right { op, operand: Box::new(operand) }), ss(e.span()))
    });

  let left_section =
    just(TokenKind::LParen).ignore_then(expr.clone()).then(section_op()).then_ignore(just(TokenKind::RParen)).map_with(|(operand, op_tok), e| {
      let op = token_to_binop(&op_tok).unwrap_or(BinOp::Add);
      SExpr::new(Expr::Section(Section::Left { operand: Box::new(operand), op }), ss(e.span()))
    });

  let param = param_parser(expr.clone());
  let func_def = just(TokenKind::LParen)
    .ignore_then(param.repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .then(just(TokenKind::Arrow).ignore_then(super::type_ann::type_parser()).or_not())
    .then(expr.clone())
    .map_with(|((params, ret_type), body), e| SExpr::new(Expr::Func { params, ret_type, body: Box::new(body) }, ss(e.span())));

  let tuple = just(TokenKind::LParen)
    .ignore_then(expr.clone().separated_by(just(TokenKind::Semi).or_not()).at_least(2).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::RParen))
    .map_with(|elems, e| SExpr::new(Expr::Tuple(elems), ss(e.span())));

  let grouped = just(TokenKind::LParen).ignore_then(expr).then_ignore(just(TokenKind::RParen)).map_with(|inner, e| SExpr::new(inner.node, ss(e.span())));

  choice((field_section, index_section, binop_section, right_section, func_def, unit, left_section, tuple, grouped))
}

fn param_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, Param, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let typed = ident()
    .then(just(TokenKind::Colon).ignore_then(super::type_ann::type_parser()).or_not())
    .then(just(TokenKind::Assign).ignore_then(expr).or_not())
    .map(|((name, type_ann), default)| Param { name, type_ann, default });

  let underscore = just(TokenKind::Underscore).to(Param { name: "_".into(), type_ann: None, default: None });

  typed.or(underscore)
}

fn section_op<'a, I>() -> impl Parser<'a, I, TokenKind, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  any().filter(|k: &TokenKind| {
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
  })
}

fn with_parser<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let with_context = just(TokenKind::With)
    .ignore_then(just(TokenKind::Ident(intern("context"))))
    .ignore_then(ident().then_ignore(just(TokenKind::Colon)).then(expr.clone()).repeated().collect::<Vec<_>>())
    .then_ignore(just(TokenKind::LBrace))
    .then(stmts_block(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(fields, body), e| SExpr::new(Expr::WithContext { fields, body }, ss(e.span())));

  let with_binding = just(TokenKind::With)
    .ignore_then(just(TokenKind::Ident("mut".into())).to(true).or_not().map(|x| x.unwrap_or(false)))
    .then(ident())
    .then(just(TokenKind::DeclMut).to(true).or(just(TokenKind::Assign).to(false)))
    .then(expr.clone())
    .then_ignore(just(TokenKind::LBrace))
    .then(stmts_block(expr.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|((((explicit_mut, name), is_decl_mut), value), body), e| {
      SExpr::new(Expr::With { name, value: Box::new(value), body, mutable: explicit_mut || is_decl_mut }, ss(e.span()))
    });

  let resource = expr.clone().then_ignore(just(TokenKind::Ident(intern("as")))).then(ident());

  let with_resource = just(TokenKind::With)
    .ignore_then(resource.separated_by(just(TokenKind::Semi)).at_least(1).collect::<Vec<_>>())
    .then_ignore(just(TokenKind::LBrace))
    .then(stmts_block(expr))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(|(resources, body), e| SExpr::new(Expr::WithResource { resources, body }, ss(e.span())));

  choice((with_context, with_binding, with_resource))
}

fn dot_rhs<'a, I>(
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, FieldKind, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
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
  let computed = expr.clone().delimited_by(just(TokenKind::LBracket), just(TokenKind::RBracket)).map(|e| FieldKind::Computed(Box::new(e)));
  let str_key = string_parser(expr).map(|e| FieldKind::Computed(Box::new(e)));

  choice((named, type_field, neg_indexed, indexed, computed, str_key))
}

fn pratt_expr<'a, I>(
  atom: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone + 'a,
  expr: impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone + 'a,
) -> impl Parser<'a, I, SExpr, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let match_arms = skip_semis()
    .ignore_then(
      super::pattern::pattern_parser()
        .then(just(TokenKind::Amp).ignore_then(expr.clone()).or_not())
        .then_ignore(just(TokenKind::Arrow))
        .then(expr.clone())
        .map(|((pattern, guard), body)| MatchArm { pattern, guard, body })
        .separated_by(semi_sep())
        .allow_trailing()
        .collect::<Vec<_>>(),
    )
    .then_ignore(skip_semis())
    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace));

  let ternary_tail = expr.clone().then(just(TokenKind::Colon).ignore_then(expr.clone()).or_not());

  let question_rhs = match_arms.map(QRhs::Match).or(ternary_tail.map(|(t, e)| QRhs::Ternary(t, e)));

  let dot_field = dot_rhs(expr.clone());

  let named_arg = ident()
    .then_ignore(just(TokenKind::Colon))
    .then(expr.clone())
    .map_with(|(name, value), e| SExpr::new(Expr::NamedArg { name, value: Box::new(value) }, ss(e.span())));

  let _app_arg = named_arg.or(expr);

  macro_rules! binop {
    ($assoc:ident($bp:expr), $tok:expr, $op:expr) => {
      infix($assoc($bp), just($tok), |l: SExpr, _, r: SExpr, e| SExpr::new(Expr::Binary { op: $op, left: Box::new(l), right: Box::new(r) }, ss(e.span())))
        .boxed()
    };
  }

  atom.pratt(vec![
    prefix(29, just(TokenKind::Minus), |_, operand: SExpr, e| SExpr::new(Expr::Unary { op: UnaryOp::Neg, operand: Box::new(operand) }, ss(e.span()))).boxed(),
    prefix(29, just(TokenKind::Bang), |_, operand: SExpr, e| SExpr::new(Expr::Unary { op: UnaryOp::Not, operand: Box::new(operand) }, ss(e.span()))).boxed(),
    postfix(33, just(TokenKind::Dot).then(dot_field), |left: SExpr, (_, field), e| SExpr::new(Expr::FieldAccess { expr: Box::new(left), field }, ss(e.span())))
      .boxed(),
    postfix(10, just(TokenKind::Caret), |operand: SExpr, _, e| SExpr::new(Expr::Propagate(Box::new(operand)), ss(e.span()))).boxed(),
    postfix(3, just(TokenKind::Question).then(question_rhs), |scrutinee: SExpr, (_, rhs), e| match rhs {
      QRhs::Match(arms) => SExpr::new(Expr::Match { scrutinee: Box::new(scrutinee), arms }, ss(e.span())),
      QRhs::Ternary(then_, else_) => SExpr::new(Expr::Ternary { cond: Box::new(scrutinee), then_: Box::new(then_), else_: else_.map(Box::new) }, ss(e.span())),
    })
    .boxed(),
    binop!(left(27), TokenKind::Star, BinOp::Mul),
    binop!(left(27), TokenKind::Slash, BinOp::Div),
    binop!(left(27), TokenKind::Percent, BinOp::Mod),
    binop!(left(27), TokenKind::IntDiv, BinOp::IntDiv),
    binop!(left(25), TokenKind::Plus, BinOp::Add),
    binop!(left(25), TokenKind::Minus, BinOp::Sub),
    binop!(left(23), TokenKind::DotDot, BinOp::Range),
    binop!(left(23), TokenKind::DotDotEq, BinOp::RangeInclusive),
    binop!(left(21), TokenKind::PlusPlus, BinOp::Concat),
    infix(left(19), just(TokenKind::Pipe), |l: SExpr, _, r: SExpr, e| SExpr::new(Expr::Pipe { left: Box::new(l), right: Box::new(r) }, ss(e.span()))).boxed(),
    binop!(left(17), TokenKind::Eq, BinOp::Eq),
    binop!(left(17), TokenKind::NotEq, BinOp::NotEq),
    binop!(left(17), TokenKind::Lt, BinOp::Lt),
    binop!(left(17), TokenKind::Gt, BinOp::Gt),
    binop!(left(17), TokenKind::LtEq, BinOp::LtEq),
    binop!(left(17), TokenKind::GtEq, BinOp::GtEq),
    binop!(left(15), TokenKind::And, BinOp::And),
    binop!(left(13), TokenKind::Or, BinOp::Or),
    infix(left(11), just(TokenKind::QQ), |l: SExpr, _, r: SExpr, e| SExpr::new(Expr::Coalesce { expr: Box::new(l), default: Box::new(r) }, ss(e.span())))
      .boxed(),
    infix(left(7), just(TokenKind::Amp), |l: SExpr, _, r: SExpr, e| {
      SExpr::new(Expr::Binary { op: BinOp::And, left: Box::new(l), right: Box::new(r) }, ss(e.span()))
    })
    .boxed(),
    infix(left(31), empty(), |func: SExpr, _, arg: SExpr, e| SExpr::new(Expr::Apply { func: Box::new(func), arg: Box::new(arg) }, ss(e.span()))).boxed(),
  ])
}

enum QRhs {
  Match(Vec<MatchArm>),
  Ternary(SExpr, Option<SExpr>),
}
