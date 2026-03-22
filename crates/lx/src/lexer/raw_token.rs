use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r]+")]
pub(super) enum RawToken {
  #[token("(")]
  LParen,
  #[token(")")]
  RParen,
  #[token("[")]
  LBracket,
  #[token("]")]
  RBracket,
  #[token("{")]
  LBrace,
  #[token("}")]
  RBrace,
  #[token("%{")]
  PercentLBrace,
  #[token(";")]
  Semi,
  #[token(",")]
  Comma,
  #[token("^")]
  Caret,
  #[token("~")]
  Tilde,
  #[token("??")]
  QQ,
  #[token("?")]
  Question,
  #[token("&&")]
  And,
  #[token("&")]
  Amp,
  #[token("||")]
  Or,
  #[token("|")]
  Pipe,
  #[token("!=")]
  NotEq,
  #[token("!")]
  BangExcl,
  #[token("==")]
  Eq,
  #[token("=")]
  Assign,
  #[token(":=")]
  DeclMut,
  #[token(":")]
  Colon,
  #[token("*")]
  Star,
  #[token("%")]
  Percent,
  #[token("++")]
  PlusPlus,
  #[token("+")]
  Plus,
  #[token("->")]
  Arrow,
  #[token("-")]
  Minus,
  #[token("//")]
  IntDiv,
  #[token("/")]
  Slash,
  #[token("<-")]
  Reassign,
  #[token("<=")]
  LtEq,
  #[token("<")]
  Lt,
  #[token(">=")]
  GtEq,
  #[token(">")]
  Gt,
  #[token("..=")]
  DotDotEq,
  #[token("..")]
  DotDot,
  #[token(".")]
  Dot,
  #[token("\"")]
  Quote,
  #[token("`")]
  Backtick,
  #[token("#")]
  Hash,
  #[token("\n")]
  Newline,
  #[regex("--[^\n]*", allow_greedy = true)]
  Comment,
  #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*", priority = 10)]
  HexInt,
  #[regex(r"0[bB][01][01_]*", priority = 10)]
  BinInt,
  #[regex(r"0[oO][0-7][0-7_]*", priority = 10)]
  OctInt,
  #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+\-]?[0-9][0-9_]*)?", priority = 8)]
  FloatLit,
  #[regex(r"[0-9][0-9_]*[eE][+\-]?[0-9][0-9_]*", priority = 7)]
  FloatExp,
  #[regex(r"[0-9][0-9_]*", priority = 5)]
  DecInt,
  #[regex(r"[A-Z][a-zA-Z0-9]*")]
  TypeName,
  #[regex(r"[a-z_][a-zA-Z0-9_']*\??")]
  Ident,
}
