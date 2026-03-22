use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::{EmitBackend, LogBackend, LogLevel};

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
