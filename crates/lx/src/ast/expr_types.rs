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

#[derive(Debug, Clone)]
pub enum Section {
  Right { op: BinOp, operand: ExprId },
  Left { operand: ExprId, op: BinOp },
  Field(Sym),
  Index(i64),
  BinOp(BinOp),
}

#[derive(Debug, Clone)]
pub enum FieldKind {
  Named(Sym),
  Index(i64),
  Computed(ExprId),
}

#[derive(Debug, Clone)]
pub enum ListElem {
  Single(ExprId),
  Spread(ExprId),
}

#[derive(Debug, Clone)]
pub enum RecordField {
  Named { name: Sym, value: ExprId },
  Spread(ExprId),
}

#[derive(Debug, Clone)]
pub enum MapEntry {
  Keyed { key: ExprId, value: ExprId },
  Spread(ExprId),
}

#[derive(Debug, Clone)]
pub struct Param {
  pub name: Sym,
  pub type_ann: Option<TypeExprId>,
  pub default: Option<ExprId>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
  pub pattern: PatternId,
  pub guard: Option<ExprId>,
  pub body: ExprId,
}

#[derive(Debug, Clone)]
pub struct SelArm {
  pub expr: ExprId,
  pub handler: ExprId,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprBinary {
  pub op: BinOp,
  pub left: ExprId,
  pub right: ExprId,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprUnary {
  pub op: UnaryOp,
  pub operand: ExprId,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprPipe {
  pub left: ExprId,
  pub right: ExprId,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprApply {
  pub func: ExprId,
  pub arg: ExprId,
}

#[derive(Debug, Clone)]
pub struct ExprFieldAccess {
  pub expr: ExprId,
  pub field: FieldKind,
}

#[derive(Debug, Clone)]
pub struct ExprFunc {
  pub params: Vec<Param>,
  pub ret_type: Option<TypeExprId>,
  pub guard: Option<ExprId>,
  pub body: ExprId,
}

#[derive(Debug, Clone)]
pub struct ExprMatch {
  pub scrutinee: ExprId,
  pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprTernary {
  pub cond: ExprId,
  pub then_: ExprId,
  pub else_: Option<ExprId>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprCoalesce {
  pub expr: ExprId,
  pub default: ExprId,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprSlice {
  pub expr: ExprId,
  pub start: Option<ExprId>,
  pub end: Option<ExprId>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprNamedArg {
  pub name: Sym,
  pub value: ExprId,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprAssert {
  pub expr: ExprId,
  pub msg: Option<ExprId>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprTimeout {
  pub ms: ExprId,
  pub body: ExprId,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprEmit {
  pub value: ExprId,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprYield {
  pub value: ExprId,
}

#[derive(Debug, Clone)]
pub struct ExprWith {
  pub kind: WithKind,
  pub body: Vec<StmtId>,
}
