use std::sync::atomic::{AtomicU32, Ordering};

use crate::sym::Sym;
mod display;
mod expr_types;
mod types;

pub use expr_types::*;
pub use types::*;

use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u32);

static NEXT_NODE_ID: AtomicU32 = AtomicU32::new(0);

pub fn reset_node_ids() {
  NEXT_NODE_ID.store(0, Ordering::Relaxed);
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
  pub node: T,
  pub span: SourceSpan,
  pub id: NodeId,
}

impl<T> Spanned<T> {
  pub fn new(node: T, span: SourceSpan) -> Self {
    let id = NodeId(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed));
    Self { node, span, id }
  }

  pub fn with_id(node: T, span: SourceSpan, id: NodeId) -> Self {
    Self { node, span, id }
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
  TypeDef(StmtTypeDef),
  TraitUnion(TraitUnionDef),
  TraitDecl(TraitDeclData),
  ClassDecl(ClassDeclData),
  FieldUpdate(StmtFieldUpdate),
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
  Name(Sym),
  Reassign(Sym),
  Pattern(SPattern),
}

#[derive(Debug, Clone)]
pub enum Expr {
  Literal(Literal),
  Ident(Sym),
  TypeConstructor(Sym),

  Binary(ExprBinary),
  Unary(ExprUnary),
  Pipe(ExprPipe),

  Apply(ExprApply),
  Section(Section),

  FieldAccess(ExprFieldAccess),

  Block(Vec<SStmt>),
  Tuple(Vec<SExpr>),

  List(Vec<ListElem>),
  Record(Vec<RecordField>),
  Map(Vec<MapEntry>),

  Func(ExprFunc),
  Match(ExprMatch),
  Ternary(ExprTernary),

  Propagate(Box<SExpr>),
  Coalesce(ExprCoalesce),

  Slice(ExprSlice),
  NamedArg(ExprNamedArg),

  Loop(Vec<SStmt>),
  Break(Option<Box<SExpr>>),
  Assert(ExprAssert),

  Par(Vec<SStmt>),
  Sel(Vec<SelArm>),
  Timeout(ExprTimeout),

  Emit(ExprEmit),
  Yield(ExprYield),
  With(ExprWith),
}

#[derive(Debug, Clone)]
pub enum WithKind {
  Binding { name: Sym, value: Box<SExpr>, mutable: bool },
  Resources { resources: Vec<(SExpr, Sym)> },
  Context { fields: Vec<(Sym, SExpr)> },
}
