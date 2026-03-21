use std::sync::Arc;
use std::time::Instant;

use lx::backends::UserBackend;
use lx::value::LxVal;
use tokio::sync::oneshot;

use crate::event::{EventBus, RuntimeEvent, UserPromptKind, next_prompt_id};

pub struct DxUserBackend {
  bus: Arc<EventBus>,
  agent_id: String,
  response_tx: Arc<std::sync::Mutex<Option<oneshot::Sender<serde_json::Value>>>>,
}

impl DxUserBackend {
  pub fn new(bus: Arc<EventBus>, agent_id: String) -> Self {
    Self { bus, agent_id, response_tx: Arc::new(std::sync::Mutex::new(None)) }
  }

  pub fn response_sender(&self) -> Arc<std::sync::Mutex<Option<oneshot::Sender<serde_json::Value>>>> {
    self.response_tx.clone()
  }

  fn prompt_and_wait(&self, prompt_id: u64, kind: UserPromptKind) -> Result<serde_json::Value, String> {
    let (tx, rx) = oneshot::channel();
    {
      let mut guard = self.response_tx.lock().map_err(|e| format!("lock poisoned: {e}"))?;
      *guard = Some(tx);
    }

    self.bus.send(RuntimeEvent::UserPrompt { agent_id: self.agent_id.clone(), prompt_id, kind, ts: Instant::now() });

    let val = rx.blocking_recv().map_err(|_| "prompt cancelled".to_string())?;

    self.bus.send(RuntimeEvent::UserResponse { agent_id: self.agent_id.clone(), prompt_id, response: val.clone(), ts: Instant::now() });

    Ok(val)
  }
}

impl UserBackend for DxUserBackend {
  fn confirm(&self, message: &str) -> Result<bool, String> {
    let pid = next_prompt_id();
    let val = self.prompt_and_wait(pid, UserPromptKind::Confirm { message: message.to_string() })?;
    val.as_bool().ok_or_else(|| "expected bool response".to_string())
  }

  fn choose(&self, message: &str, options: &[String]) -> Result<usize, String> {
    let pid = next_prompt_id();
    let val = self.prompt_and_wait(pid, UserPromptKind::Choose { message: message.to_string(), options: options.to_vec() })?;
    val.as_u64().map(|n| n as usize).ok_or_else(|| "expected integer response".to_string())
  }

  fn ask(&self, message: &str, default: Option<&str>) -> Result<String, String> {
    let pid = next_prompt_id();
    let val = self.prompt_and_wait(pid, UserPromptKind::Ask { message: message.to_string(), default: default.map(|s| s.to_string()) })?;
    val.as_str().map(|s| s.to_string()).ok_or_else(|| "expected string response".to_string())
  }

  fn progress(&self, current: usize, total: usize, message: &str) {
    self.bus.send(RuntimeEvent::Progress { agent_id: self.agent_id.clone(), current, total, message: message.to_string(), ts: Instant::now() });
  }

  fn progress_pct(&self, pct: f64, message: &str) {
    let current = (pct * 100.0) as usize;
    self.bus.send(RuntimeEvent::Progress { agent_id: self.agent_id.clone(), current, total: 100, message: message.to_string(), ts: Instant::now() });
  }

  fn status(&self, level: &str, message: &str) {
    self.bus.send(RuntimeEvent::Log { agent_id: self.agent_id.clone(), level: level.to_string(), msg: message.to_string(), ts: Instant::now() });
  }

  fn table(&self, headers: &[String], rows: &[Vec<String>]) {
    let mut table_str = headers.join(" | ");
    table_str.push('\n');
    table_str.push_str(&"-".repeat(table_str.len()));
    table_str.push('\n');
    for row in rows {
      table_str.push_str(&row.join(" | "));
      table_str.push('\n');
    }
    self.bus.send(RuntimeEvent::Emit { agent_id: self.agent_id.clone(), value: table_str, ts: Instant::now() });
  }

  fn check_signal(&self) -> Option<LxVal> {
    None
  }
}
