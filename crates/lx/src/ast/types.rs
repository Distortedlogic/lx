use lx_macros::AstWalk;
use strum::Display;

use super::{ExprId, Literal, PatternId, TypeExprId};
use crate::sym::Sym;

#[derive(Debug, Clone, PartialEq)]
pub struct UseStmt {
  pub path: Vec<Sym>,
  pub kind: UseKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
  Whole,
  Alias(Sym),
  Selective(Vec<Sym>),
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum Pattern {
  Literal(Literal),
  Bind(Sym),
  Wildcard,
  Tuple(Vec<PatternId>),
  List(PatternList),
  Record(PatternRecord),
  Constructor(PatternConstructor),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitEntry {
  Field(Box<FieldDecl>),
  Spread(Sym),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<D, C> {
  pub name: Sym,
  pub type_name: Sym,
  pub default: Option<D>,
  pub constraint: Option<C>,
}

pub type FieldDecl = Field<ExprId, ExprId>;

#[derive(Debug, Clone, PartialEq)]
pub struct TraitUnionDef {
  pub name: Sym,
  pub type_params: Vec<Sym>,
  pub variants: Vec<Sym>,
  pub exported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodSpec<F> {
  pub name: Sym,
  pub input: Vec<F>,
  pub output: Sym,
}

pub type TraitMethodDecl = MethodSpec<FieldDecl>;

#[derive(Debug, Clone, PartialEq)]
pub struct AgentMethod {
  pub name: Sym,
  pub handler: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassField {
  pub name: Sym,
  pub default: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDeclData {
  pub name: Sym,
  pub type_params: Vec<Sym>,
  pub entries: Vec<TraitEntry>,
  pub methods: Vec<TraitMethodDecl>,
  pub defaults: Vec<AgentMethod>,
  pub requires: Vec<Sym>,
  pub description: Option<Sym>,
  pub tags: Vec<Sym>,
  pub exported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclData {
  pub name: Sym,
  pub type_params: Vec<Sym>,
  pub traits: Vec<Sym>,
  pub fields: Vec<ClassField>,
  pub methods: Vec<AgentMethod>,
  pub exported: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
  Agent,
  Tool,
  Prompt,
  Connector,
  Store,
  Session,
  Guard,
  Workflow,
  Schema,
  Mcp,
  Cli,
  Http,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeywordDeclData {
  pub keyword: KeywordKind,
  pub name: Sym,
  pub type_params: Vec<Sym>,
  pub fields: Vec<ClassField>,
  pub methods: Vec<AgentMethod>,
  pub trait_entries: Option<Vec<TraitEntry>>,
  pub exported: bool,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct FieldPattern {
  pub name: Sym,
  pub pattern: Option<PatternId>,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum TypeExpr {
  Named(Sym),
  Var(Sym),
  Applied(Sym, Vec<TypeExprId>),
  List(TypeExprId),
  Map { key: TypeExprId, value: TypeExprId },
  Record(Vec<TypeField>),
  Tuple(Vec<TypeExprId>),
  Func { param: TypeExprId, ret: TypeExprId },
  Fallible { ok: TypeExprId, err: TypeExprId },
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct TypeField {
  pub name: Sym,
  pub ty: TypeExprId,
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

#[derive(Debug, Clone, PartialEq)]
pub struct StmtTypeDef {
  pub name: Sym,
  pub type_params: Vec<Sym>,
  pub variants: Vec<(Sym, usize)>,
  pub exported: bool,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct StmtFieldUpdate {
  pub name: Sym,
  pub fields: Vec<Sym>,
  pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct PatternList {
  pub elems: Vec<PatternId>,
  pub rest: Option<Sym>,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct PatternRecord {
  pub fields: Vec<FieldPattern>,
  pub rest: Option<Sym>,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct PatternConstructor {
  pub name: Sym,
  pub args: Vec<PatternId>,
}
