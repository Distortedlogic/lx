use strum::Display;

use super::{Literal, SExpr, SPattern, SType};
use crate::sym::Sym;

#[derive(Debug, Clone)]
pub struct UseStmt {
  pub path: Vec<Sym>,
  pub kind: UseKind,
}

#[derive(Debug, Clone)]
pub enum UseKind {
  Whole,
  Alias(Sym),
  Selective(Vec<Sym>),
}

#[derive(Debug, Clone)]
pub enum Pattern {
  Literal(Literal),
  Bind(Sym),
  Wildcard,
  Tuple(Vec<SPattern>),
  List(PatternList),
  Record(PatternRecord),
  Constructor(PatternConstructor),
}

#[derive(Debug, Clone)]
pub enum TraitEntry {
  Field(Box<FieldDecl>),
  Spread(Sym),
}

#[derive(Debug, Clone)]
pub struct Field<D, C> {
  pub name: Sym,
  pub type_name: Sym,
  pub default: Option<D>,
  pub constraint: Option<C>,
}

pub type FieldDecl = Field<SExpr, SExpr>;

#[derive(Debug, Clone)]
pub struct TraitUnionDef {
  pub name: Sym,
  pub variants: Vec<Sym>,
  pub exported: bool,
}

#[derive(Debug, Clone)]
pub struct MethodSpec<F> {
  pub name: Sym,
  pub input: Vec<F>,
  pub output: Sym,
}

pub type TraitMethodDecl = MethodSpec<FieldDecl>;

#[derive(Debug, Clone)]
pub struct AgentMethod {
  pub name: Sym,
  pub handler: SExpr,
}

#[derive(Debug, Clone)]
pub struct ClassField {
  pub name: Sym,
  pub default: SExpr,
}

#[derive(Debug, Clone)]
pub struct TraitDeclData {
  pub name: Sym,
  pub entries: Vec<TraitEntry>,
  pub methods: Vec<TraitMethodDecl>,
  pub defaults: Vec<AgentMethod>,
  pub requires: Vec<Sym>,
  pub description: Option<Sym>,
  pub tags: Vec<Sym>,
  pub exported: bool,
}

#[derive(Debug, Clone)]
pub struct ClassDeclData {
  pub name: Sym,
  pub traits: Vec<Sym>,
  pub fields: Vec<ClassField>,
  pub methods: Vec<AgentMethod>,
  pub exported: bool,
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
  pub name: Sym,
  pub pattern: Option<SPattern>,
}

#[derive(Debug, Clone)]
pub enum TypeExpr {
  Named(Sym),
  Var(Sym),
  Applied(Sym, Vec<SType>),
  List(Box<SType>),
  Map { key: Box<SType>, value: Box<SType> },
  Record(Vec<TypeField>),
  Tuple(Vec<SType>),
  Func { param: Box<SType>, ret: Box<SType> },
  Fallible { ok: Box<SType>, err: Box<SType> },
}

#[derive(Debug, Clone)]
pub struct TypeField {
  pub name: Sym,
  pub ty: SType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum BinOp {
  #[strum(to_string = "+")]
  Add,
  #[strum(to_string = "-")]
  Sub,
  #[strum(to_string = "*")]
  Mul,
  #[strum(to_string = "/")]
  Div,
  #[strum(to_string = "%")]
  Mod,
  #[strum(to_string = "//")]
  IntDiv,
  #[strum(to_string = "++")]
  Concat,
  #[strum(to_string = "..")]
  Range,
  #[strum(to_string = "..=")]
  RangeInclusive,
  #[strum(to_string = "==")]
  Eq,
  #[strum(to_string = "!=")]
  NotEq,
  #[strum(to_string = "<")]
  Lt,
  #[strum(to_string = ">")]
  Gt,
  #[strum(to_string = "<=")]
  LtEq,
  #[strum(to_string = ">=")]
  GtEq,
  #[strum(to_string = "&&")]
  And,
  #[strum(to_string = "||")]
  Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum UnaryOp {
  #[strum(to_string = "-")]
  Neg,
  #[strum(to_string = "!")]
  Not,
}

#[derive(Debug, Clone)]
pub struct StmtTypeDef {
  pub name: Sym,
  pub variants: Vec<(Sym, usize)>,
  pub exported: bool,
}

#[derive(Debug, Clone)]
pub struct StmtFieldUpdate {
  pub name: Sym,
  pub fields: Vec<Sym>,
  pub value: SExpr,
}

#[derive(Debug, Clone)]
pub struct PatternList {
  pub elems: Vec<SPattern>,
  pub rest: Option<Sym>,
}

#[derive(Debug, Clone)]
pub struct PatternRecord {
  pub fields: Vec<FieldPattern>,
  pub rest: Option<Sym>,
}

#[derive(Debug, Clone)]
pub struct PatternConstructor {
  pub name: Sym,
  pub args: Vec<SPattern>,
}
