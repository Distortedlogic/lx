use num_bigint::BigInt;

use super::{BinOp, SExpr, SPattern, SType};
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
