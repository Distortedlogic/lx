use num_bigint::BigInt;

use super::{BinOp, SExpr, SPattern, SType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMode {
    Normal,
    Propagate,
    Block,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(BigInt),
    Float(f64),
    Str(Vec<StrPart>),
    RawStr(String),
    Regex(String),
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
    Field(String),
    Index(i64),
    BinOp(BinOp),
}

#[derive(Debug, Clone)]
pub enum FieldKind {
    Named(String),
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
    pub name: Option<String>,
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
    pub name: String,
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
pub struct ReceiveArm {
    pub action: String,
    pub handler: SExpr,
}
