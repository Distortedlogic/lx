use std::sync::Arc;
use std::time::Instant;

use lx::backends::ProcessShellBackend;
use lx::backends::ShellBackend;
use lx::error::LxError;
use lx::span::Span;
use lx::value::LxVal;

use crate::event::{EventBus, RuntimeEvent};

pub struct DxShellBackend {
  pub inner: ProcessShellBackend,
  pub bus: Arc<EventBus>,
  pub agent_id: String,
}

impl ShellBackend for DxShellBackend {
  fn exec(&self, cmd: &str, span: Span) -> Result<LxVal, LxError> {
    self.bus.send(RuntimeEvent::ShellExec { agent_id: self.agent_id.clone(), cmd: cmd.to_string(), ts: Instant::now() });

    let result = self.inner.exec(cmd, span)?;
    emit_shell_result(&self.bus, &self.agent_id, cmd, &result);
    Ok(result)
  }

  fn exec_capture(&self, cmd: &str, span: Span) -> Result<LxVal, LxError> {
    self.bus.send(RuntimeEvent::ShellExec { agent_id: self.agent_id.clone(), cmd: cmd.to_string(), ts: Instant::now() });

    let result = self.inner.exec_capture(cmd, span);
    match &result {
      Ok(val) => {
        self.bus.send(RuntimeEvent::ShellResult {
          agent_id: self.agent_id.clone(),
          cmd: cmd.to_string(),
          exit_code: 0,
          stdout: format!("{val}"),
          stderr: String::new(),
          ts: Instant::now(),
        });
      },
      Err(e) => {
        self.bus.send(RuntimeEvent::ShellResult {
          agent_id: self.agent_id.clone(),
          cmd: cmd.to_string(),
          exit_code: 1,
          stdout: String::new(),
          stderr: format!("{e}"),
          ts: Instant::now(),
        });
      },
    }
    result
  }
}

fn emit_shell_result(bus: &EventBus, agent_id: &str, cmd: &str, val: &LxVal) {
  let (exit_code, stdout, stderr) = extract_shell_fields(val);
  bus.send(RuntimeEvent::ShellResult { agent_id: agent_id.to_string(), cmd: cmd.to_string(), exit_code, stdout, stderr, ts: Instant::now() });
}

fn extract_shell_fields(val: &LxVal) -> (i32, String, String) {
  match val {
    LxVal::Ok(inner) => {
      let code = inner.int_field("code").map(|n| format!("{n}").parse::<i32>().unwrap_or(0)).unwrap_or(0);
      let stdout = inner.str_field("out").unwrap_or("").to_string();
      let stderr = inner.str_field("err").unwrap_or("").to_string();
      (code, stdout, stderr)
    },
    LxVal::Err(inner) => {
      let msg = inner.str_field("msg").unwrap_or("").to_string();
      (1, String::new(), msg)
    },
    other => (0, format!("{other}"), String::new()),
  }
}
