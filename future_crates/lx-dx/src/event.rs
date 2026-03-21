use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_call_id() -> u64 {
  NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn next_prompt_id() -> u64 {
  NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
  pub start_line: usize,
  pub start_col: usize,
  pub end_line: usize,
  pub end_col: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserPromptKind {
  Confirm { message: String },
  Choose { message: String, options: Vec<String> },
  Ask { message: String, default: Option<String> },
}

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
  AgentSpawned {
    agent_id: String,
    name: String,
    config: serde_json::Value,
    ts: Instant,
  },
  AgentKilled {
    agent_id: String,
    ts: Instant,
  },
  AiCallStart {
    agent_id: String,
    call_id: u64,
    prompt: String,
    model: Option<String>,
    system: Option<String>,
    ts: Instant,
  },
  AiCallComplete {
    agent_id: String,
    call_id: u64,
    response: String,
    cost_usd: Option<f64>,
    duration_ms: u64,
    model: String,
    langfuse_trace_id: Option<String>,
    ts: Instant,
  },
  AiCallError {
    agent_id: String,
    call_id: u64,
    error: String,
    ts: Instant,
  },
  MessageSend {
    from_agent: String,
    to_agent: String,
    msg: serde_json::Value,
    ts: Instant,
  },
  MessageAsk {
    from_agent: String,
    to_agent: String,
    msg: serde_json::Value,
    ts: Instant,
  },
  MessageResponse {
    from_agent: String,
    to_agent: String,
    response: serde_json::Value,
    duration_ms: u64,
    ts: Instant,
  },
  Emit {
    agent_id: String,
    value: String,
    ts: Instant,
  },
  Log {
    agent_id: String,
    level: String,
    msg: String,
    ts: Instant,
  },
  ShellExec {
    agent_id: String,
    cmd: String,
    ts: Instant,
  },
  ShellResult {
    agent_id: String,
    cmd: String,
    exit_code: i32,
    stdout: String,
    stderr: String,
    ts: Instant,
  },
  UserPrompt {
    agent_id: String,
    prompt_id: u64,
    kind: UserPromptKind,
    ts: Instant,
  },
  UserResponse {
    agent_id: String,
    prompt_id: u64,
    response: serde_json::Value,
    ts: Instant,
  },
  TraceSpanRecorded {
    agent_id: String,
    span_id: u64,
    name: String,
    input: String,
    output: String,
    score: Option<f64>,
    ts: Instant,
  },
  Progress {
    agent_id: String,
    current: usize,
    total: usize,
    message: String,
    ts: Instant,
  },
  Error {
    agent_id: String,
    error: String,
    span_info: Option<SpanInfo>,
    ts: Instant,
  },
  ProgramStarted {
    source_path: String,
    ts: Instant,
  },
  ProgramFinished {
    result: Result<String, String>,
    duration_ms: u64,
    ts: Instant,
  },
}

impl RuntimeEvent {
  pub fn agent_id(&self) -> Option<&str> {
    match self {
      Self::AgentSpawned { agent_id, .. }
      | Self::AgentKilled { agent_id, .. }
      | Self::AiCallStart { agent_id, .. }
      | Self::AiCallComplete { agent_id, .. }
      | Self::AiCallError { agent_id, .. }
      | Self::Emit { agent_id, .. }
      | Self::Log { agent_id, .. }
      | Self::ShellExec { agent_id, .. }
      | Self::ShellResult { agent_id, .. }
      | Self::UserPrompt { agent_id, .. }
      | Self::UserResponse { agent_id, .. }
      | Self::TraceSpanRecorded { agent_id, .. }
      | Self::Progress { agent_id, .. }
      | Self::Error { agent_id, .. } => Some(agent_id),
      Self::MessageSend { from_agent, .. } | Self::MessageAsk { from_agent, .. } | Self::MessageResponse { from_agent, .. } => Some(from_agent),
      Self::ProgramStarted { .. } | Self::ProgramFinished { .. } => None,
    }
  }
}

const BUS_CAPACITY: usize = 4096;

#[derive(Clone)]
pub struct EventBus {
  tx: broadcast::Sender<RuntimeEvent>,
}

impl EventBus {
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(BUS_CAPACITY);
    Self { tx }
  }

  pub fn send(&self, event: RuntimeEvent) {
    let _ = self.tx.send(event);
  }

  pub fn subscribe(&self) -> broadcast::Receiver<RuntimeEvent> {
    self.tx.subscribe()
  }
}

impl PartialEq for EventBus {
  fn eq(&self, _other: &Self) -> bool {
    false
  }
}

impl Default for EventBus {
  fn default() -> Self {
    Self::new()
  }
}
