use std::sync::Arc;

use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::{HttpBackend, HttpOpts, ShellBackend};

pub struct DenyShellBackend;

impl ShellBackend for DenyShellBackend {
  fn exec(&self, _cmd: &str, _span: Span) -> Result<LxVal, LxError> {
    Ok(LxVal::Err(Box::new(LxVal::str("shell access denied by sandbox policy"))))
  }

  fn exec_capture(&self, _cmd: &str, _span: Span) -> Result<LxVal, LxError> {
    Ok(LxVal::Err(Box::new(LxVal::str("shell access denied by sandbox policy"))))
  }
}

pub struct DenyHttpBackend;

impl HttpBackend for DenyHttpBackend {
  fn request(&self, _method: &str, _url: &str, _opts: &HttpOpts, _span: Span) -> Result<LxVal, LxError> {
    Ok(LxVal::Err(Box::new(LxVal::str("network access denied by sandbox policy"))))
  }
}

pub struct RestrictedShellBackend {
  pub inner: Arc<dyn ShellBackend>,
  pub allowed_cmds: Vec<String>,
}

impl ShellBackend for RestrictedShellBackend {
  fn exec(&self, cmd: &str, span: Span) -> Result<LxVal, LxError> {
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    if self.allowed_cmds.iter().any(|c| c == first_word) {
      self.inner.exec(cmd, span)
    } else {
      Ok(LxVal::Err(Box::new(LxVal::str(format!("command '{first_word}' not allowed by sandbox policy")))))
    }
  }

  fn exec_capture(&self, cmd: &str, span: Span) -> Result<LxVal, LxError> {
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    if self.allowed_cmds.iter().any(|c| c == first_word) {
      self.inner.exec_capture(cmd, span)
    } else {
      Ok(LxVal::Err(Box::new(LxVal::str(format!("command '{first_word}' not allowed by sandbox policy")))))
    }
  }
}
