use crate::ast::{AstArena, BinOp, Expr};
use crate::sym::Sym;

pub enum ExprMatcher {
  Any,
  Ident(Option<Sym>),
  Literal,
  Binary { op: Option<BinOp>, left: Box<ExprMatcher>, right: Box<ExprMatcher> },
  Apply { func: Box<ExprMatcher>, arg: Box<ExprMatcher> },
  Match { arms_count: Option<usize> },
  Propagate(Box<ExprMatcher>),
  Block,
  Func,
}

impl ExprMatcher {
  pub fn matches(&self, expr: &Expr, arena: &AstArena) -> bool {
    match (self, expr) {
      (ExprMatcher::Any, _) => true,
      (ExprMatcher::Ident(None), Expr::Ident(_)) => true,
      (ExprMatcher::Ident(Some(expected)), Expr::Ident(actual)) => *expected == *actual,
      (ExprMatcher::Literal, Expr::Literal(_)) => true,
      (ExprMatcher::Binary { op, left, right }, Expr::Binary(b)) => {
        op.is_none_or(|expected| expected == b.op) && left.matches(arena.expr(b.left), arena) && right.matches(arena.expr(b.right), arena)
      },
      (ExprMatcher::Apply { func, arg }, Expr::Apply(a)) => func.matches(arena.expr(a.func), arena) && arg.matches(arena.expr(a.arg), arena),
      (ExprMatcher::Match { arms_count }, Expr::Match(m)) => arms_count.is_none_or(|expected| m.arms.len() == expected),
      (ExprMatcher::Propagate(inner), Expr::Propagate(id)) => inner.matches(arena.expr(*id), arena),
      (ExprMatcher::Block, Expr::Block(_)) => true,
      (ExprMatcher::Func, Expr::Func(_)) => true,
      _ => false,
    }
  }

  pub fn any() -> ExprMatcher {
    ExprMatcher::Any
  }

  pub fn ident(name: Sym) -> ExprMatcher {
    ExprMatcher::Ident(Some(name))
  }

  pub fn any_ident() -> ExprMatcher {
    ExprMatcher::Ident(None)
  }

  pub fn literal() -> ExprMatcher {
    ExprMatcher::Literal
  }

  pub fn binary(op: BinOp, left: ExprMatcher, right: ExprMatcher) -> ExprMatcher {
    ExprMatcher::Binary { op: Some(op), left: Box::new(left), right: Box::new(right) }
  }

  pub fn any_binary() -> ExprMatcher {
    ExprMatcher::Binary { op: None, left: Box::new(Self::any()), right: Box::new(Self::any()) }
  }

  pub fn apply(func: ExprMatcher, arg: ExprMatcher) -> ExprMatcher {
    ExprMatcher::Apply { func: Box::new(func), arg: Box::new(arg) }
  }

  pub fn empty_match() -> ExprMatcher {
    ExprMatcher::Match { arms_count: Some(0) }
  }

  pub fn propagate(inner: ExprMatcher) -> ExprMatcher {
    ExprMatcher::Propagate(Box::new(inner))
  }
}
