mod display;
mod expr_types;
mod types;

pub use expr_types::*;
pub use types::*;

use crate::span::Span;

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
pub type SType = Spanned<TypeExpr>;

#[derive(Debug, Clone)]
pub struct Program {
  pub stmts: Vec<SStmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
  Binding(Binding),
  TypeDef { name: String, variants: Vec<(String, usize)>, exported: bool },
  TraitUnion(TraitUnionDef),
  TraitDecl(TraitDeclData),
  ClassDecl(ClassDeclData),
  FieldUpdate { name: String, fields: Vec<String>, value: SExpr },
  Use(UseStmt),
  Expr(SExpr),
}

#[derive(Debug, Clone)]
pub struct Binding {
  pub exported: bool,
  pub mutable: bool,
  pub target: BindTarget,
  pub type_ann: Option<SType>,
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

  Func { params: Vec<Param>, ret_type: Option<SType>, body: Box<SExpr> },
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

  Emit { value: Box<SExpr> },
  Yield { value: Box<SExpr> },
  With { name: String, value: Box<SExpr>, body: Vec<SStmt>, mutable: bool },
  WithResource { resources: Vec<(SExpr, String)>, body: Vec<SStmt> },
  WithContext { fields: Vec<(String, SExpr)>, body: Vec<SStmt> },
  Shell { mode: ShellMode, parts: Vec<StrPart> },
}
