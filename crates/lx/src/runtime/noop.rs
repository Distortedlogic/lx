use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::{AiBackend, AiOpts, EmitBackend, LogBackend, LogLevel};

pub struct NoopEmitBackend;

impl EmitBackend for NoopEmitBackend {
  fn emit(&self, _value: &LxVal, _span: SourceSpan) -> Result<(), LxError> {
    Ok(())
  }
}

pub struct NoopLogBackend;

impl LogBackend for NoopLogBackend {
  fn log(&self, _level: LogLevel, _msg: &str) {}
}

pub struct NoopAiBackend;

impl AiBackend for NoopAiBackend {
  fn prompt(&self, _text: &str, _span: SourceSpan) -> Result<LxVal, LxError> {
    Ok(LxVal::err_str("ai backend not configured"))
  }

  fn prompt_with(&self, _opts: &AiOpts, _span: SourceSpan) -> Result<LxVal, LxError> {
    Ok(LxVal::err_str("ai backend not configured"))
  }
}
