use miette::SourceSpan;
use std::sync::Arc;

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
