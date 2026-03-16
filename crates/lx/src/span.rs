#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub offset: u32,
    pub len: u16,
}

impl Span {
    pub fn new(offset: u32, len: u16) -> Self {
        Self { offset, len }
    }

    pub fn from_range(start: u32, end: u32) -> Self {
        Self {
            offset: start,
            len: (end - start) as u16,
        }
    }

    pub fn end(&self) -> u32 {
        self.offset + self.len as u32
    }
}

impl From<Span> for miette::SourceSpan {
    fn from(s: Span) -> Self {
        (s.offset as usize, s.len as usize).into()
    }
}
