pub mod arena;
mod display;
mod expr_types;
mod parent_map;
mod types;
mod walk_impls;

use std::marker::PhantomData;

use lx_macros::AstWalk;

use crate::sym::Sym;

pub use arena::{AstArena, AstNode, ExprId, NodeId, PatternId, Spanned, StmtId, TypeExprId};
pub use expr_types::*;
pub use parent_map::build_parent_map;
pub use types::*;

pub struct Surface;
pub struct Core;

#[derive(Debug, Clone)]
pub struct Program<Phase = Surface> {
  pub stmts: Vec<StmtId>,
  pub arena: AstArena,
  pub _phase: PhantomData<Phase>,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum Stmt {
  Binding(Binding),
  #[walk(skip)]
  TypeDef(StmtTypeDef),
  #[walk(skip)]
  TraitUnion(TraitUnionDef),
  TraitDecl(TraitDeclData),
  ClassDecl(ClassDeclData),
  FieldUpdate(StmtFieldUpdate),
  #[walk(skip)]
  Use(UseStmt),
  Expr(ExprId),
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct Binding {
  pub exported: bool,
  pub mutable: bool,
  pub target: BindTarget,
  pub type_ann: Option<TypeExprId>,
  pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum BindTarget {
  Name(Sym),
  Reassign(Sym),
  Pattern(PatternId),
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
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

  Block(Vec<StmtId>),
  Tuple(Vec<ExprId>),

  List(Vec<ListElem>),
  Record(Vec<RecordField>),
  Map(Vec<MapEntry>),

  Func(ExprFunc),
  Match(ExprMatch),
  Ternary(ExprTernary),

  Propagate(ExprId),
  Coalesce(ExprCoalesce),

  Slice(ExprSlice),
  NamedArg(ExprNamedArg),

  Loop(Vec<StmtId>),
  Break(Option<ExprId>),
  Assert(ExprAssert),

  Par(Vec<StmtId>),
  Sel(Vec<SelArm>),
  Timeout(ExprTimeout),

  Emit(ExprEmit),
  Yield(ExprYield),
  With(ExprWith),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WithKind {
  Binding { name: Sym, value: ExprId, mutable: bool },
  Resources { resources: Vec<(ExprId, Sym)> },
  Context { fields: Vec<(Sym, ExprId)> },
}
