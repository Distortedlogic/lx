use num_bigint::BigInt;

use super::{BinOp, ExprId, PatternId, StmtId, TypeExprId, UnaryOp, WithKind};
use crate::sym::Sym;

#[derive(Debug, Clone)]
pub enum Literal {
  Int(BigInt),
  Float(f64),
  Str(Vec<StrPart>),
  RawStr(String),
  Bool(bool),
  Unit,
}

#[derive(Debug, Clone)]
pub enum StrPart {
  Text(String),
  Interp(ExprId),
}

impl PartialEq for StrPart {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (StrPart::Text(a), StrPart::Text(b)) => a == b,
      (StrPart::Interp(a), StrPart::Interp(b)) => a == b,
      _ => false,
    }
  }
}

impl Eq for StrPart {}

impl PartialEq for Literal {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Literal::Int(a), Literal::Int(b)) => a == b,
      (Literal::Float(a), Literal::Float(b)) => a.to_bits() == b.to_bits(),
      (Literal::Str(a), Literal::Str(b)) => a == b,
      (Literal::RawStr(a), Literal::RawStr(b)) => a == b,
      (Literal::Bool(a), Literal::Bool(b)) => a == b,
      (Literal::Unit, Literal::Unit) => true,
      _ => false,
    }
  }
}

impl Eq for Literal {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Section {
  Right { op: BinOp, operand: ExprId },
  Left { operand: ExprId, op: BinOp },
  Field(Sym),
  Index(i64),
  BinOp(BinOp),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldKind {
  Named(Sym),
  Index(i64),
  Computed(ExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListElem {
  Single(ExprId),
  Spread(ExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordField {
  Named { name: Sym, value: ExprId },
  Spread(ExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapEntry {
  Keyed { key: ExprId, value: ExprId },
  Spread(ExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
  pub name: Sym,
  pub type_ann: Option<TypeExprId>,
  pub default: Option<ExprId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
  pub pattern: PatternId,
  pub guard: Option<ExprId>,
  pub body: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelArm {
  pub expr: ExprId,
  pub handler: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprBinary {
  pub op: BinOp,
  pub left: ExprId,
  pub right: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprUnary {
  pub op: UnaryOp,
  pub operand: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprPipe {
  pub left: ExprId,
  pub right: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprApply {
  pub func: ExprId,
  pub arg: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprFieldAccess {
  pub expr: ExprId,
  pub field: FieldKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprFunc {
  pub params: Vec<Param>,
  pub ret_type: Option<TypeExprId>,
  pub guard: Option<ExprId>,
  pub body: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprMatch {
  pub scrutinee: ExprId,
  pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprTernary {
  pub cond: ExprId,
  pub then_: ExprId,
  pub else_: Option<ExprId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprCoalesce {
  pub expr: ExprId,
  pub default: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprSlice {
  pub expr: ExprId,
  pub start: Option<ExprId>,
  pub end: Option<ExprId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprNamedArg {
  pub name: Sym,
  pub value: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprAssert {
  pub expr: ExprId,
  pub msg: Option<ExprId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprTimeout {
  pub ms: ExprId,
  pub body: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprEmit {
  pub value: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprYield {
  pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprWith {
  pub kind: WithKind,
  pub body: Vec<StmtId>,
}
