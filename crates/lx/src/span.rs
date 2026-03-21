pub type Span = miette::SourceSpan;

pub trait SpanExt {
  fn new(offset: usize, len: usize) -> Span;
  fn from_range(start: usize, end: usize) -> Span;
  fn end(&self) -> usize;
}

impl SpanExt for Span {
  fn new(offset: usize, len: usize) -> Span {
    (offset, len).into()
  }
  fn from_range(start: usize, end: usize) -> Span {
    (start, end - start).into()
  }
  fn end(&self) -> usize {
    self.offset() + self.len()
  }
}
