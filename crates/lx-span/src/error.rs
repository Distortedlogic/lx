use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("parse error: {msg}")]
#[diagnostic(code(lx::parse))]
pub struct ParseError {
  pub msg: String,
  #[label("{msg}")]
  pub span: SourceSpan,
  #[help]
  pub help: Option<String>,
}

impl ParseError {
  pub fn new(msg: impl Into<String>, span: SourceSpan, help: Option<String>) -> Self {
    Self { msg: msg.into(), span, help }
  }
}
