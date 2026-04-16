use std::collections::BTreeMap;

use lx_api::types::ActivityEvent;

use super::types::{DesktopRuntimeEvent, DesktopRuntimeEventKind, payload_text, result_preview};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeTranscriptRow {
  pub ts: String,
  pub role: String,
  pub text: String,
}

pub fn activity_events(events: &[DesktopRuntimeEvent]) -> Vec<ActivityEvent> {
  events
    .iter()
    .filter_map(|event| match event.kind {
      DesktopRuntimeEventKind::MessageComplete => Some(ActivityEvent {
        timestamp: event.ts.clone(),
        kind: event.payload.get("role").and_then(serde_json::Value::as_str).unwrap_or("assistant").to_string(),
        message: payload_text(&event.payload)?,
        token_count: None,
        adapter: Some("pi_rpc".to_string()),
      }),
      DesktopRuntimeEventKind::ToolCall => Some(ActivityEvent {
        timestamp: event.ts.clone(),
        kind: "tool_call".to_string(),
        message: format!(
          "{} {}",
          event.payload.get("tool_name").and_then(serde_json::Value::as_str).unwrap_or("tool"),
          result_preview(&event.payload.get("args").cloned().unwrap_or_default()).unwrap_or_default()
        ),
        token_count: None,
        adapter: Some("pi_rpc".to_string()),
      }),
      DesktopRuntimeEventKind::ToolResult => Some(ActivityEvent {
        timestamp: event.ts.clone(),
        kind: "tool_result".to_string(),
        message: payload_text(&event.payload).unwrap_or_else(|| "tool completed".to_string()),
        token_count: None,
        adapter: Some("pi_rpc".to_string()),
      }),
      DesktopRuntimeEventKind::ToolError | DesktopRuntimeEventKind::BackendError => Some(ActivityEvent {
        timestamp: event.ts.clone(),
        kind: "tool_error".to_string(),
        message: payload_text(&event.payload).unwrap_or_else(|| "runtime error".to_string()),
        token_count: None,
        adapter: Some("pi_rpc".to_string()),
      }),
      DesktopRuntimeEventKind::RuntimeEmit | DesktopRuntimeEventKind::ControlState => Some(ActivityEvent {
        timestamp: event.ts.clone(),
        kind: "activity".to_string(),
        message: payload_text(&event.payload)?,
        token_count: None,
        adapter: Some("pi_rpc".to_string()),
      }),
      DesktopRuntimeEventKind::AgentSpawn | DesktopRuntimeEventKind::AgentStop | DesktopRuntimeEventKind::MessageDelta => None,
    })
    .collect()
}

pub fn transcript_rows(events: &[DesktopRuntimeEvent]) -> Vec<RuntimeTranscriptRow> {
  let mut rows = Vec::new();
  let mut streaming = BTreeMap::<String, (String, String, String)>::new();
  for event in events {
    match event.kind {
      DesktopRuntimeEventKind::MessageDelta => {
        let message_id = event.payload.get("message_id").and_then(serde_json::Value::as_str).unwrap_or("assistant-message").to_string();
        let role = event.payload.get("role").and_then(serde_json::Value::as_str).unwrap_or("assistant").to_string();
        let delta = event.payload.get("delta").and_then(serde_json::Value::as_str).unwrap_or_default();
        let entry = streaming.entry(message_id).or_insert_with(|| (event.ts.clone(), role, String::new()));
        entry.2.push_str(delta);
      },
      DesktopRuntimeEventKind::MessageComplete => rows.push(RuntimeTranscriptRow {
        ts: event.ts.clone(),
        role: event.payload.get("role").and_then(serde_json::Value::as_str).unwrap_or("assistant").to_string(),
        text: payload_text(&event.payload).unwrap_or_default(),
      }),
      DesktopRuntimeEventKind::ToolCall => rows.push(RuntimeTranscriptRow {
        ts: event.ts.clone(),
        role: "tool".to_string(),
        text: format!("Calling {}", event.payload.get("tool_name").and_then(serde_json::Value::as_str).unwrap_or("tool")),
      }),
      DesktopRuntimeEventKind::ToolError | DesktopRuntimeEventKind::BackendError => rows.push(RuntimeTranscriptRow {
        ts: event.ts.clone(),
        role: "error".to_string(),
        text: payload_text(&event.payload).unwrap_or_else(|| "Runtime error".to_string()),
      }),
      DesktopRuntimeEventKind::ControlState | DesktopRuntimeEventKind::RuntimeEmit => {
        if let Some(text) = payload_text(&event.payload) {
          rows.push(RuntimeTranscriptRow { ts: event.ts.clone(), role: "system".to_string(), text });
        }
      },
      DesktopRuntimeEventKind::AgentSpawn | DesktopRuntimeEventKind::AgentStop | DesktopRuntimeEventKind::ToolResult => {},
    }
  }
  rows.extend(streaming.into_values().map(|(ts, role, text)| RuntimeTranscriptRow { ts, role, text }));
  rows.sort_by(|left, right| left.ts.cmp(&right.ts));
  rows
}
