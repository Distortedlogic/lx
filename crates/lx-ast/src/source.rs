use std::collections::HashMap;

use lx_span::source::{CommentPlacement, FileId};

use crate::ast::{ExprId, NodeId, PatternId, StmtId, TypeExprId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalExprId {
  pub file: FileId,
  pub local: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalStmtId {
  pub file: FileId,
  pub local: StmtId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalPatternId {
  pub file: FileId,
  pub local: PatternId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalTypeExprId {
  pub file: FileId,
  pub local: TypeExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalNodeId {
  Expr(GlobalExprId),
  Stmt(GlobalStmtId),
  Pattern(GlobalPatternId),
  TypeExpr(GlobalTypeExprId),
}

impl GlobalExprId {
  pub fn new(file: FileId, local: ExprId) -> Self {
    Self { file, local }
  }
}

impl GlobalStmtId {
  pub fn new(file: FileId, local: StmtId) -> Self {
    Self { file, local }
  }
}

impl GlobalPatternId {
  pub fn new(file: FileId, local: PatternId) -> Self {
    Self { file, local }
  }
}

impl GlobalTypeExprId {
  pub fn new(file: FileId, local: TypeExprId) -> Self {
    Self { file, local }
  }
}

#[derive(Debug, Clone)]
pub struct AttachedComment {
  pub comment_idx: usize,
  pub placement: CommentPlacement,
}

pub type CommentMap = HashMap<NodeId, Vec<AttachedComment>>;

impl NodeId {
  pub fn in_file(self, file: FileId) -> GlobalNodeId {
    match self {
      NodeId::Expr(id) => GlobalNodeId::Expr(GlobalExprId::new(file, id)),
      NodeId::Stmt(id) => GlobalNodeId::Stmt(GlobalStmtId::new(file, id)),
      NodeId::Pattern(id) => GlobalNodeId::Pattern(GlobalPatternId::new(file, id)),
      NodeId::TypeExpr(id) => GlobalNodeId::TypeExpr(GlobalTypeExprId::new(file, id)),
    }
  }
}
