use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::{EmitBackend, LlmBackend, LlmOpts, LogBackend, LogLevel};

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

pub struct NoopLlmBackend;

impl LlmBackend for NoopLlmBackend {
  fn prompt(&self, _text: &str, _span: SourceSpan) -> Result<LxVal, LxError> {
    Ok(LxVal::err_str("llm backend not configured"))
  }

  fn prompt_with(&self, _opts: &LlmOpts, _span: SourceSpan) -> Result<LxVal, LxError> {
    Ok(LxVal::err_str("llm backend not configured"))
  }
}
