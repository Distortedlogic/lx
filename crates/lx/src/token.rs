use crate::span::Span;
use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
  pub kind: TokenKind,
  pub span: Span,
}

impl Token {
  pub fn new(kind: TokenKind, span: Span) -> Self {
    Self { kind, span }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
  Int(BigInt),
  Float(f64),
  StrStart,
  StrChunk(String),
  StrEnd,
  RawStr(String),
  Regex(String),
  True,
  False,
  Unit,

  Ident(String),
  TypeName(String),

  Plus,
  Minus,
  Star,
  Slash,
  Percent,
  IntDiv,
  PlusPlus,
  Eq,
  NotEq,
  Lt,
  Gt,
  LtEq,
  GtEq,
  And,
  Or,
  Pipe,
  QQ,
  Caret,
  Amp,
  Arrow,
  Question,
  Bang,
  Dot,
  DotDot,
  DotDotEq,
  Assign,
  DeclMut,
  Reassign,
  Colon,

  LParen,
  RParen,
  LBracket,
  RBracket,
  LBrace,
  RBrace,
  PercentLBrace,

  Dollar,
  DollarCaret,
  DollarBrace,
  ShellText(String),
  ShellEnd,

  Use,
  Loop,
  Break,
  Par,
  Sel,
  Assert,
  Underscore,

  TildeArrow,
  TildeArrowQ,
  Protocol,
  Mcp,
  Yield,
  With,

  Export,
  Semi,
  Eof,

  Error,
}
