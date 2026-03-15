use strum::Display;

use crate::span::Span;
use num_bigint::BigInt;

#[derive(Debug, Clone)]
pub struct Spanned<T> {
  pub node: T,
  pub span: Span,
}

impl<T> Spanned<T> {
  pub fn new(node: T, span: Span) -> Self {
    Self { node, span }
  }
}

pub type SExpr = Spanned<Expr>;
pub type SStmt = Spanned<Stmt>;
pub type SPattern = Spanned<Pattern>;

#[derive(Debug, Clone)]
pub struct Program {
  pub stmts: Vec<SStmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
  Binding(Binding),
  TypeDef { name: String, variants: Vec<(String, usize)>, exported: bool },
  Protocol { name: String, fields: Vec<ProtocolField>, exported: bool },
  McpDecl { name: String, tools: Vec<McpToolDecl>, exported: bool },
  FieldUpdate { name: String, fields: Vec<String>, value: SExpr },
  Use(UseStmt),
  Expr(SExpr),
}

#[derive(Debug, Clone)]
pub struct Binding {
  pub exported: bool,
  pub mutable: bool,
  pub target: BindTarget,
  pub value: SExpr,
}

#[derive(Debug, Clone)]
pub enum BindTarget {
  Name(String),
  Reassign(String),
  Pattern(SPattern),
}

#[derive(Debug, Clone)]
pub enum Expr {
  Literal(Literal),
  Ident(String),
  TypeConstructor(String),

  Binary { op: BinOp, left: Box<SExpr>, right: Box<SExpr> },
  Unary { op: UnaryOp, operand: Box<SExpr> },
  Pipe { left: Box<SExpr>, right: Box<SExpr> },

  Apply { func: Box<SExpr>, arg: Box<SExpr> },
  Section(Section),

  FieldAccess { expr: Box<SExpr>, field: FieldKind },

  Block(Vec<SStmt>),
  Tuple(Vec<SExpr>),

  List(Vec<ListElem>),
  Record(Vec<RecordField>),
  Map(Vec<MapEntry>),

  Func { params: Vec<Param>, body: Box<SExpr> },
  Match { scrutinee: Box<SExpr>, arms: Vec<MatchArm> },
  Ternary { cond: Box<SExpr>, then_: Box<SExpr>, else_: Option<Box<SExpr>> },

  Propagate(Box<SExpr>),
  Coalesce { expr: Box<SExpr>, default: Box<SExpr> },

  Slice { expr: Box<SExpr>, start: Option<Box<SExpr>>, end: Option<Box<SExpr>> },
  NamedArg { name: String, value: Box<SExpr> },

  Loop(Vec<SStmt>),
  Break(Option<Box<SExpr>>),
  Assert { expr: Box<SExpr>, msg: Option<Box<SExpr>> },

  Par(Vec<SStmt>),
  Sel(Vec<SelArm>),

  AgentSend { target: Box<SExpr>, msg: Box<SExpr> },
  AgentAsk { target: Box<SExpr>, msg: Box<SExpr> },

  Yield { value: Box<SExpr> },
  With { name: String, value: Box<SExpr>, body: Vec<SStmt>, mutable: bool },

  Shell { mode: ShellMode, parts: Vec<StrPart> },
}

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
pub struct UseStmt {
  pub path: Vec<String>,
  pub kind: UseKind,
}

#[derive(Debug, Clone)]
pub enum UseKind {
  Whole,
  Alias(String),
  Selective(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Pattern {
  Literal(Literal),
  Bind(String),
  Wildcard,
  Tuple(Vec<SPattern>),
  List { elems: Vec<SPattern>, rest: Option<String> },
  Record { fields: Vec<FieldPattern>, rest: Option<String> },
  Constructor { name: String, args: Vec<SPattern> },
}

#[derive(Debug, Clone)]
pub struct ProtocolField {
  pub name: String,
  pub type_name: String,
  pub default: Option<SExpr>,
}

#[derive(Debug, Clone)]
pub struct McpToolDecl {
  pub name: String,
  pub input: Vec<ProtocolField>,
  pub output: McpOutputType,
}

#[derive(Debug, Clone)]
pub enum McpOutputType {
  Named(String),
  List(Box<McpOutputType>),
  Record(Vec<ProtocolField>),
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
  pub name: String,
  pub pattern: Option<SPattern>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum BinOp {
  #[strum(to_string = "+")] Add,
  #[strum(to_string = "-")] Sub,
  #[strum(to_string = "*")] Mul,
  #[strum(to_string = "/")] Div,
  #[strum(to_string = "%")] Mod,
  #[strum(to_string = "//")] IntDiv,
  #[strum(to_string = "++")] Concat,
  #[strum(to_string = "..")] Range,
  #[strum(to_string = "..=")] RangeInclusive,
  #[strum(to_string = "==")] Eq,
  #[strum(to_string = "!=")] NotEq,
  #[strum(to_string = "<")] Lt,
  #[strum(to_string = ">")] Gt,
  #[strum(to_string = "<=")] LtEq,
  #[strum(to_string = ">=")] GtEq,
  #[strum(to_string = "&&")] And,
  #[strum(to_string = "||")] Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum UnaryOp {
  #[strum(to_string = "-")] Neg,
  #[strum(to_string = "!")] Not,
}
