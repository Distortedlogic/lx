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
      DesktopRuntimeEventKind::MessageComplete => {
        let message_id = event.payload.get("message_id").and_then(serde_json::Value::as_str).unwrap_or("assistant-message");
        let streamed = streaming.remove(message_id);
        let role = event
          .payload
          .get("role")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .or_else(|| streamed.as_ref().map(|(_, role, _)| role.clone()))
          .unwrap_or_else(|| "assistant".to_string());
        let text = payload_text(&event.payload).or_else(|| streamed.map(|(_, _, text)| text)).unwrap_or_default();
        rows.push(RuntimeTranscriptRow { ts: event.ts.clone(), role, text });
      },
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

#[cfg(test)]
mod tests {
  use super::super::types::DesktopRuntimeEvent;
  use super::*;

  #[test]
  fn completed_messages_replace_streaming_rows() {
    let rows = transcript_rows(&[
      DesktopRuntimeEvent {
        id: "event-1".to_string(),
        agent_id: "agent-1".to_string(),
        kind: DesktopRuntimeEventKind::MessageDelta,
        ts: "1".to_string(),
        payload: serde_json::json!({ "role": "assistant", "message_id": "msg-1", "delta": "Hello" }),
      },
      DesktopRuntimeEvent {
        id: "event-2".to_string(),
        agent_id: "agent-1".to_string(),
        kind: DesktopRuntimeEventKind::MessageDelta,
        ts: "2".to_string(),
        payload: serde_json::json!({ "role": "assistant", "message_id": "msg-1", "delta": " world" }),
      },
      DesktopRuntimeEvent {
        id: "event-3".to_string(),
        agent_id: "agent-1".to_string(),
        kind: DesktopRuntimeEventKind::MessageComplete,
        ts: "3".to_string(),
        payload: serde_json::json!({ "role": "assistant", "message_id": "msg-1", "text": "Hello world" }),
      },
    ]);

    assert_eq!(rows, vec![RuntimeTranscriptRow { ts: "3".to_string(), role: "assistant".to_string(), text: "Hello world".to_string() }]);
  }

  #[test]
  fn incomplete_streaming_messages_are_retained() {
    let rows = transcript_rows(&[DesktopRuntimeEvent {
      id: "event-1".to_string(),
      agent_id: "agent-1".to_string(),
      kind: DesktopRuntimeEventKind::MessageDelta,
      ts: "1".to_string(),
      payload: serde_json::json!({ "role": "assistant", "message_id": "msg-1", "delta": "Hello" }),
    }]);

    assert_eq!(rows, vec![RuntimeTranscriptRow { ts: "1".to_string(), role: "assistant".to_string(), text: "Hello".to_string() }]);
  }
}
