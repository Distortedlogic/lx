use miette::SourceSpan;
use std::sync::Arc;

use crate::ast::{ExprId, PatternId, StmtId, TypeExprId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u32);

impl FileId {
  pub fn new(id: u32) -> Self {
    Self(id)
  }
  pub fn index(self) -> u32 {
    self.0
  }
}

pub struct SourceDb {
  files: Vec<SourceFile>,
}

struct SourceFile {
  path: String,
  source: Arc<str>,
}

impl Default for SourceDb {
  fn default() -> Self {
    Self::new()
  }
}

impl SourceDb {
  pub fn new() -> Self {
    Self { files: Vec::new() }
  }

  pub fn add_file(&mut self, path: String, source: Arc<str>) -> FileId {
    let id = FileId(self.files.len() as u32);
    self.files.push(SourceFile { path, source });
    id
  }

  pub fn source(&self, id: FileId) -> &str {
    &self.files[id.0 as usize].source
  }

  pub fn path(&self, id: FileId) -> &str {
    &self.files[id.0 as usize].path
  }
}

#[derive(Debug, Clone, Copy)]
pub struct FullSpan {
  pub file: FileId,
  pub span: SourceSpan,
}

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
pub struct Comment {
  pub span: SourceSpan,
  pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct CommentStore {
  comments: Vec<Comment>,
}

impl CommentStore {
  pub fn from_vec(comments: Vec<Comment>) -> Self {
    Self { comments }
  }

  pub fn push(&mut self, comment: Comment) {
    self.comments.push(comment);
  }

  pub fn all(&self) -> &[Comment] {
    &self.comments
  }

  pub fn comments_in_range(&self, start: usize, end: usize) -> &[Comment] {
    let lo = self.comments.partition_point(|c| c.span.offset() < start);
    let hi = self.comments.partition_point(|c| c.span.offset() < end);
    &self.comments[lo..hi]
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentPlacement {
  Leading,
  Trailing,
  Dangling,
}

#[derive(Debug, Clone)]
pub struct AttachedComment {
  pub comment_idx: usize,
  pub placement: CommentPlacement,
}

pub type CommentMap = std::collections::HashMap<crate::ast::NodeId, Vec<AttachedComment>>;

impl crate::ast::NodeId {
  pub fn in_file(self, file: FileId) -> GlobalNodeId {
    match self {
      crate::ast::NodeId::Expr(id) => GlobalNodeId::Expr(GlobalExprId::new(file, id)),
      crate::ast::NodeId::Stmt(id) => GlobalNodeId::Stmt(GlobalStmtId::new(file, id)),
      crate::ast::NodeId::Pattern(id) => GlobalNodeId::Pattern(GlobalPatternId::new(file, id)),
      crate::ast::NodeId::TypeExpr(id) => GlobalNodeId::TypeExpr(GlobalTypeExprId::new(file, id)),
    }
  }
}
