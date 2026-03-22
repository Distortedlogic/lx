use num_bigint::BigInt;

use super::{BinOp, SExpr, SPattern, SStmt, SType, UnaryOp, WithKind};
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
  Interp(SExpr),
}

#[derive(Debug, Clone)]
pub enum Section {
  Right { op: BinOp, operand: Box<SExpr> },
  Left { operand: Box<SExpr>, op: BinOp },
  Field(Sym),
  Index(i64),
  BinOp(BinOp),
}

#[derive(Debug, Clone)]
pub enum FieldKind {
  Named(Sym),
  Index(i64),
  Computed(Box<SExpr>),
}

#[derive(Debug, Clone)]
pub enum ListElem {
  Single(SExpr),
  Spread(SExpr),
}

#[derive(Debug, Clone)]
pub struct RecordField {
  pub name: Option<Sym>,
  pub value: SExpr,
  pub is_spread: bool,
}

#[derive(Debug, Clone)]
pub struct MapEntry {
  pub key: Option<SExpr>,
  pub value: SExpr,
  pub is_spread: bool,
}

#[derive(Debug, Clone)]
pub struct Param {
  pub name: Sym,
  pub type_ann: Option<SType>,
  pub default: Option<SExpr>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
  pub pattern: SPattern,
  pub guard: Option<SExpr>,
  pub body: SExpr,
}

#[derive(Debug, Clone)]
pub struct SelArm {
  pub expr: SExpr,
  pub handler: SExpr,
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
  pub op: BinOp,
  pub left: Box<SExpr>,
  pub right: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprUnary {
  pub op: UnaryOp,
  pub operand: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprPipe {
  pub left: Box<SExpr>,
  pub right: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprApply {
  pub func: Box<SExpr>,
  pub arg: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprFieldAccess {
  pub expr: Box<SExpr>,
  pub field: FieldKind,
}

#[derive(Debug, Clone)]
pub struct ExprFunc {
  pub params: Vec<Param>,
  pub ret_type: Option<SType>,
  pub guard: Option<Box<SExpr>>,
  pub body: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprMatch {
  pub scrutinee: Box<SExpr>,
  pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct ExprTernary {
  pub cond: Box<SExpr>,
  pub then_: Box<SExpr>,
  pub else_: Option<Box<SExpr>>,
}

#[derive(Debug, Clone)]
pub struct ExprCoalesce {
  pub expr: Box<SExpr>,
  pub default: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprSlice {
  pub expr: Box<SExpr>,
  pub start: Option<Box<SExpr>>,
  pub end: Option<Box<SExpr>>,
}

#[derive(Debug, Clone)]
pub struct ExprNamedArg {
  pub name: Sym,
  pub value: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprAssert {
  pub expr: Box<SExpr>,
  pub msg: Option<Box<SExpr>>,
}

#[derive(Debug, Clone)]
pub struct ExprTimeout {
  pub ms: Box<SExpr>,
  pub body: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprEmit {
  pub value: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprYield {
  pub value: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ExprWith {
  pub kind: WithKind,
  pub body: Vec<SStmt>,
}
