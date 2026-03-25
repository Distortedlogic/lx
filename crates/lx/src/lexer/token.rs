use crate::sym::Sym;
use miette::SourceSpan;
use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
  pub kind: TokenKind,
  pub span: SourceSpan,
}

impl Token {
  pub fn new(kind: TokenKind, span: SourceSpan) -> Self {
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
  True,
  False,
  Unit,

  Ident(Sym),
  TypeName(Sym),

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

  Use,
  Loop,
  Break,
  Par,
  Sel,
  Assert,
  Underscore,

  Trait,
  ClassKw,
  Emit,
  Yield,
  With,
  Timeout,
  As,

  AgentKw,
  ToolKw,
  PromptKw,
  ConnectorKw,
  StoreKw,
  SessionKw,
  GuardKw,
  WorkflowKw,
  SchemaKw,
  McpKw,
  CliKw,
  HttpKw,

  Export,
  Semi,
  Eof,

  Error,
}
