use std::process::Command;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::ast::{ShellMode, StrPart};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::Interpreter;

impl Interpreter {
  pub(super) fn eval_shell(
    &mut self,
    mode: &ShellMode,
    parts: &[StrPart],
    span: Span,
  ) -> Result<Value, LxError> {
    let cmd_str = self.build_shell_string(parts)?;
    match mode {
      ShellMode::Normal | ShellMode::Raw | ShellMode::Block => self.exec_shell_full(&cmd_str, span),
      ShellMode::Propagate => self.exec_shell_propagate(&cmd_str, span),
    }
  }

  fn build_shell_string(&mut self, parts: &[StrPart]) -> Result<String, LxError> {
    let mut cmd = String::new();
    for part in parts {
      match part {
        StrPart::Text(s) => cmd.push_str(s),
        StrPart::Interp(expr) => {
          let val = self.eval_expr(expr)?;
          cmd.push_str(&format!("{val}"));
        },
      }
    }
    Ok(cmd)
  }

  fn exec_shell_full(&self, cmd: &str, _span: Span) -> Result<Value, LxError> {
    let cmd_trimmed = cmd.trim();
    match Command::new("sh").arg("-c").arg(cmd_trimmed).output() {
      Ok(output) => {
        let out = String::from_utf8_lossy(&output.stdout).into_owned();
        let err = String::from_utf8_lossy(&output.stderr).into_owned();
        let code = output.status.code().unwrap_or(-1);
        let mut fields = IndexMap::new();
        fields.insert("out".into(), Value::Str(Arc::from(out.as_str())));
        fields.insert("err".into(), Value::Str(Arc::from(err.as_str())));
        fields.insert("code".into(), Value::Int(code.into()));
        Ok(Value::Ok(Box::new(Value::Record(Arc::new(fields)))))
      },
      Err(e) => {
        let mut fields = IndexMap::new();
        fields.insert("cmd".into(), Value::Str(Arc::from(cmd_trimmed)));
        fields.insert("msg".into(), Value::Str(Arc::from(e.to_string().as_str())));
        Ok(Value::Err(Box::new(Value::Record(Arc::new(fields)))))
      },
    }
  }

  fn exec_shell_propagate(&self, cmd: &str, span: Span) -> Result<Value, LxError> {
    let cmd_trimmed = cmd.trim();
    match Command::new("sh").arg("-c").arg(cmd_trimmed).output() {
      Ok(output) => {
        let code = output.status.code().unwrap_or(-1);
        if code == 0 {
          let out = String::from_utf8_lossy(&output.stdout).into_owned();
          Ok(Value::Str(Arc::from(out.as_str())))
        } else {
          let err = String::from_utf8_lossy(&output.stderr).into_owned();
          let mut fields = IndexMap::new();
          fields.insert("cmd".into(), Value::Str(Arc::from(cmd_trimmed)));
          fields.insert("msg".into(), Value::Str(Arc::from(err.as_str())));
          let shell_err = Value::Err(Box::new(Value::Record(Arc::new(fields))));
          Err(LxError::propagate(shell_err, span))
        }
      },
      Err(e) => {
        let mut fields = IndexMap::new();
        fields.insert("cmd".into(), Value::Str(Arc::from(cmd_trimmed)));
        fields.insert("msg".into(), Value::Str(Arc::from(e.to_string().as_str())));
        let shell_err = Value::Err(Box::new(Value::Record(Arc::new(fields))));
        Err(LxError::propagate(shell_err, span))
      },
    }
  }
}
