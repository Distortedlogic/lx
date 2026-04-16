use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopBackendKind {
  Pi,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopAgentStatus {
  Idle,
  Starting,
  Running,
  Paused,
  Completed,
  Error,
  Aborted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopRuntimeEventKind {
  AgentSpawn,
  AgentStop,
  MessageDelta,
  MessageComplete,
  ToolCall,
  ToolResult,
  ToolError,
  RuntimeEmit,
  ControlState,
  BackendError,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopToolStatus {
  Running,
  Completed,
  Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopAgentRuntime {
  pub id: String,
  pub backend_kind: DesktopBackendKind,
  pub name: String,
  pub status: DesktopAgentStatus,
  pub parent_id: Option<String>,
  pub flow_id: Option<String>,
  pub session_id: String,
  pub task_summary: String,
  pub model: Option<String>,
  pub cwd: Option<String>,
  pub created_at: String,
  pub last_event_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DesktopRuntimeEvent {
  pub id: String,
  pub agent_id: String,
  pub kind: DesktopRuntimeEventKind,
  pub ts: String,
  pub payload: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DesktopToolActivity {
  pub call_id: String,
  pub agent_id: String,
  pub tool_name: String,
  pub args: Value,
  pub status: DesktopToolStatus,
  pub result_preview: Option<String>,
  pub is_error: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopFlowRun {
  pub id: String,
  pub flow_id: String,
  pub root_agent_id: String,
  pub created_at: String,
}

#[derive(Clone, Debug)]
pub struct DesktopAgentLaunchSpec {
  pub name: String,
  pub task_summary: String,
  pub prompt: String,
  pub flow_id: Option<String>,
  pub parent_id: Option<String>,
  pub cwd: Option<PathBuf>,
}

impl DesktopAgentLaunchSpec {
  pub fn new(name: impl Into<String>, task_summary: impl Into<String>, prompt: impl Into<String>) -> Self {
    Self { name: name.into(), task_summary: task_summary.into(), prompt: prompt.into(), flow_id: None, parent_id: None, cwd: None }
  }
}

impl DesktopAgentRuntime {
  pub fn new(spec: &DesktopAgentLaunchSpec) -> Self {
    let now = now_ts();
    Self {
      id: new_id("agent"),
      backend_kind: DesktopBackendKind::Pi,
      name: spec.name.clone(),
      status: DesktopAgentStatus::Starting,
      parent_id: spec.parent_id.clone(),
      flow_id: spec.flow_id.clone(),
      session_id: String::new(),
      task_summary: spec.task_summary.clone(),
      model: None,
      cwd: spec.cwd.as_ref().map(|path| path.display().to_string()),
      created_at: now.clone(),
      last_event_at: now,
    }
  }
}

impl DesktopRuntimeEvent {
  pub fn new(agent_id: impl Into<String>, kind: DesktopRuntimeEventKind, payload: Value) -> Self {
    Self { id: new_id("event"), agent_id: agent_id.into(), kind, ts: now_ts(), payload }
  }
}

impl DesktopToolActivity {
  pub fn running(agent_id: impl Into<String>, call_id: impl Into<String>, tool_name: impl Into<String>, args: Value) -> Self {
    Self {
      call_id: call_id.into(),
      agent_id: agent_id.into(),
      tool_name: tool_name.into(),
      args,
      status: DesktopToolStatus::Running,
      result_preview: None,
      is_error: false,
    }
  }
}

pub fn new_id(prefix: &str) -> String {
  format!("{prefix}-{}", Uuid::new_v4())
}

pub fn now_ts() -> String {
  SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_millis()).unwrap_or(0).to_string()
}

pub fn payload_text(payload: &Value) -> Option<String> {
  payload.get("text").and_then(Value::as_str).map(ToOwned::to_owned).or_else(|| payload.get("delta").and_then(Value::as_str).map(ToOwned::to_owned))
}

pub fn result_preview(value: &Value) -> Option<String> {
  if let Some(text) = value.as_str() {
    return Some(truncate(text));
  }
  if let Some(text) = value.get("text").and_then(Value::as_str) {
    return Some(truncate(text));
  }
  if let Some(content) = value.get("content").and_then(Value::as_array)
    && let Some(text) = content.iter().find_map(|item| item.get("text").and_then(Value::as_str))
  {
    return Some(truncate(text));
  }
  serde_json::to_string(value).ok().map(|text| truncate(&text))
}

pub fn text_payload(role: &str, text: impl Into<String>) -> Value {
  json!({ "role": role, "text": text.into() })
}

fn truncate(text: &str) -> String {
  const MAX: usize = 280;
  if text.chars().count() <= MAX {
    return text.to_string();
  }
  text.chars().take(MAX).collect::<String>() + "..."
}
