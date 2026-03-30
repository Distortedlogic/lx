use miette::SourceSpan;

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
