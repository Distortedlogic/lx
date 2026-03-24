pub mod arena;
mod comment_attach;
mod display;
mod expr_types;
mod parent_map;
mod types;
mod walk_impls;

use std::marker::PhantomData;

use lx_macros::AstWalk;

use crate::sym::Sym;

pub use arena::{AstArena, AstNode, ExprId, NodeId, PatternId, Spanned, StmtId, TypeExprId};
pub use comment_attach::attach_comments;
pub use expr_types::*;
pub use parent_map::build_parent_map;
pub use types::*;

pub struct Surface;
pub struct Core;

#[derive(Debug, Clone)]
pub struct Program<Phase = Surface> {
  pub stmts: Vec<StmtId>,
  pub arena: AstArena,
  pub comments: crate::source::CommentStore,
  pub comment_map: crate::source::CommentMap,
  pub file: crate::source::FileId,
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

impl<P> Program<P> {
  pub fn leading_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
    self.attached_comments(node, crate::source::CommentPlacement::Leading)
  }

  pub fn trailing_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
    self.attached_comments(node, crate::source::CommentPlacement::Trailing)
  }

  pub fn dangling_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
    self.attached_comments(node, crate::source::CommentPlacement::Dangling)
  }

  fn attached_comments(&self, node: NodeId, placement: crate::source::CommentPlacement) -> Vec<&crate::source::Comment> {
    let all = self.comments.all();
    self.comment_map.get(&node).map(|attached| attached.iter().filter(|a| a.placement == placement).map(|a| &all[a.comment_idx]).collect()).unwrap_or_default()
  }
}
