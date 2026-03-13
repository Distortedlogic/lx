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
  Compose { left: Box<SExpr>, right: Box<SExpr> },

  FieldAccess { expr: Box<SExpr>, field: FieldKind },

  Block(Vec<SStmt>),
  Tuple(Vec<SExpr>),

  List(Vec<ListElem>),
  Record(Vec<RecordField>),
  Map(Vec<MapEntry>),
  Set(Vec<SetElem>),

  Func { params: Vec<Param>, body: Box<SExpr> },
  Match { scrutinee: Box<SExpr>, arms: Vec<MatchArm> },
  Ternary { cond: Box<SExpr>, then_: Box<SExpr>, else_: Option<Box<SExpr>> },

  Propagate(Box<SExpr>),
  Coalesce { expr: Box<SExpr>, default: Box<SExpr> },

  Loop(Vec<SStmt>),
  Break(Option<Box<SExpr>>),
  Assert { expr: Box<SExpr>, msg: Option<Box<SExpr>> },
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
pub enum SetElem {
  Single(SExpr),
  Spread(SExpr),
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
pub struct FieldPattern {
  pub name: String,
  pub pattern: Option<SPattern>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  IntDiv,
  Concat,
  Range,
  RangeInclusive,
  Eq,
  NotEq,
  Lt,
  Gt,
  LtEq,
  GtEq,
  And,
  Or,
}

impl std::fmt::Display for BinOp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      BinOp::Add => "+",
      BinOp::Sub => "-",
      BinOp::Mul => "*",
      BinOp::Div => "/",
      BinOp::Mod => "%",
      BinOp::IntDiv => "//",
      BinOp::Concat => "++",
      BinOp::Range => "..",
      BinOp::RangeInclusive => "..=",
      BinOp::Eq => "==",
      BinOp::NotEq => "!=",
      BinOp::Lt => "<",
      BinOp::Gt => ">",
      BinOp::LtEq => "<=",
      BinOp::GtEq => ">=",
      BinOp::And => "&&",
      BinOp::Or => "||",
    };
    f.write_str(s)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
  Neg,
  Not,
}

impl std::fmt::Display for UnaryOp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      UnaryOp::Neg => "-",
      UnaryOp::Not => "!",
    };
    f.write_str(s)
  }
}
