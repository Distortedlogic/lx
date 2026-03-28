use std::sync::Arc;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::value::LxVal;

pub type LxResult<T> = Result<T, LxError>;
pub type EvalResult<T> = Result<T, EvalSignal>;

#[derive(Debug, Clone)]
pub enum EvalSignal {
  Error(LxError),
  Break(LxVal),
  AgentStop,
}

impl From<LxError> for EvalSignal {
  fn from(e: LxError) -> Self {
    EvalSignal::Error(e)
  }
}

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("assertion failed: {expr}")]
#[diagnostic(code(lx::assert))]
pub struct AssertError {
  pub expr: String,
  pub message: Option<String>,
  pub expected: Option<String>,
  pub actual: Option<String>,
  #[help]
  pub help: Option<String>,
  #[label("assertion failed")]
  pub span: SourceSpan,
}

#[derive(Debug, Clone, Error, Diagnostic)]
pub enum LxError {
  #[error("parse error: {msg}")]
  #[diagnostic(code(lx::parse))]
  Parse {
    msg: String,
    #[label("{msg}")]
    span: SourceSpan,
    #[help]
    help: Option<String>,
  },

  #[error("runtime error: {msg}")]
  #[diagnostic(code(lx::runtime))]
  Runtime {
    msg: String,
    #[label("{msg}")]
    span: SourceSpan,
  },

  #[error(transparent)]
  #[diagnostic(transparent)]
  Assert(Box<AssertError>),

  #[error("type error: {msg}")]
  #[diagnostic(code(lx::type_error))]
  Type {
    msg: String,
    #[label("{msg}")]
    span: SourceSpan,
    #[help]
    help: Option<String>,
  },

  #[error("propagated error: {value}")]
  #[diagnostic(code(lx::propagate))]
  Propagate {
    value: Box<crate::value::LxVal>,
    #[label("error propagated here")]
    span: SourceSpan,
  },

  #[error("{inner}")]
  Sourced { source_name: String, source_text: Arc<str>, inner: Box<LxError> },
}

impl LxError {
  pub fn parse(msg: impl Into<String>, span: SourceSpan, help: Option<String>) -> Self {
    Self::Parse { msg: msg.into(), span, help }
  }

  pub fn runtime(msg: impl Into<String>, span: SourceSpan) -> Self {
    Self::Runtime { msg: msg.into(), span }
  }

  pub fn assert_fail(expr: impl Into<String>, message: Option<String>, expected: Option<String>, actual: Option<String>, span: SourceSpan) -> Self {
    let mut help_parts: Vec<String> = Vec::new();
    if let Some(ref msg) = message {
      help_parts.push(msg.clone());
    }
    if let (Some(exp), Some(act)) = (&expected, &actual) {
      help_parts.push(format!("expected: {exp}"));
      help_parts.push(format!("  actual: {act}"));
    }
    let help = if help_parts.is_empty() { None } else { Some(help_parts.join("\n")) };
    Self::Assert(Box::new(AssertError { expr: expr.into(), message, expected, actual, help, span }))
  }

  pub fn type_err(msg: impl Into<String>, span: SourceSpan, help: Option<String>) -> Self {
    Self::Type { msg: msg.into(), span, help }
  }

  pub fn division_by_zero(span: SourceSpan) -> Self {
    Self::runtime("division by zero", span)
  }

  pub fn propagate(value: crate::value::LxVal, span: SourceSpan) -> Self {
    Self::Propagate { value: Box::new(value), span }
  }

  pub fn with_source(self, name: String, text: Arc<str>) -> Self {
    Self::Sourced { source_name: name, source_text: text, inner: Box::new(self) }
  }
}
