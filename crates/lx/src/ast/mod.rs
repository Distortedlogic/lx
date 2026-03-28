pub mod arena;
mod comment_attach;
mod display;
mod expr_types;
mod types;
mod walk_impls;

use std::marker::PhantomData;

use lx_macros::AstWalk;

use crate::source::{Comment, CommentMap, CommentPlacement, CommentStore, FileId};
use crate::sym::Sym;

pub use arena::{AstArena, AstNode, ExprId, NodeId, PatternId, Spanned, StmtId, TypeExprId};
pub use comment_attach::attach_comments;
pub use expr_types::*;
pub use types::*;

pub struct Surface;
pub struct Core;

#[derive(Debug, Clone)]
pub struct Program<Phase = Surface> {
  pub stmts: Vec<StmtId>,
  pub arena: AstArena,
  pub comments: CommentStore,
  pub comment_map: CommentMap,
  pub file: FileId,
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
  #[walk(skip)]
  KeywordDecl(KeywordDeclData),
  FieldUpdate(StmtFieldUpdate),
  #[walk(skip)]
  Use(UseStmt),
  ChannelDecl(Sym),
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
  Tell(ExprTell),
  Ask(ExprAsk),

  Apply(ExprApply),
  Section(Section),

  FieldAccess(ExprFieldAccess),

  Block(ExprBlock),
  Tuple(ExprTuple),

  List(Vec<ListElem>),
  Record(Vec<RecordField>),
  Map(Vec<MapEntry>),

  Func(ExprFunc),
  Match(ExprMatch),
  Ternary(ExprTernary),

  Propagate(ExprPropagate),
  Coalesce(ExprCoalesce),

  Slice(ExprSlice),
  NamedArg(ExprNamedArg),

  Loop(ExprLoop),
  Break(ExprBreak),
  Assert(ExprAssert),

  Par(ExprPar),
  Sel(Vec<SelArm>),
  Timeout(ExprTimeout),
  Spawn(ExprId),
  Stop,

  Emit(ExprEmit),
  Yield(ExprYield),
  With(ExprWith),

  Grouped(ExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WithKind {
  Binding { name: Sym, value: ExprId, mutable: bool },
  Resources { resources: Vec<(ExprId, Sym)> },
  Context { fields: Vec<(Sym, ExprId)> },
}

impl<P> Program<P> {
  pub fn leading_comments(&self, node: NodeId) -> Vec<&Comment> {
    self.attached_comments(node, CommentPlacement::Leading)
  }

  pub fn trailing_comments(&self, node: NodeId) -> Vec<&Comment> {
    self.attached_comments(node, CommentPlacement::Trailing)
  }

  pub fn dangling_comments(&self, node: NodeId) -> Vec<&Comment> {
    self.attached_comments(node, CommentPlacement::Dangling)
  }

  fn attached_comments(&self, node: NodeId, placement: CommentPlacement) -> Vec<&Comment> {
    let all = self.comments.all();
    self.comment_map.get(&node).map(|attached| attached.iter().filter(|a| a.placement == placement).map(|a| &all[a.comment_idx]).collect()).unwrap_or_default()
  }
}
