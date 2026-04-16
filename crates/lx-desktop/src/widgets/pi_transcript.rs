use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::runtime::types::{DesktopRuntimeEventKind, payload_text};
use crate::runtime::use_desktop_runtime;

#[derive(Clone, Debug, PartialEq, Eq)]
struct TranscriptRow {
  ts: String,
  role: String,
  text: String,
}

#[component]
pub fn PiTranscript(agent_id: String) -> Element {
  let runtime = use_desktop_runtime();
  let rows = transcript_rows(&runtime.registry.events_for_agent(&agent_id));

  if rows.is_empty() {
    return rsx! {
      div { class: "rounded-xl border border-[var(--outline-variant)]/30 p-4 text-sm text-[var(--outline)]",
        "No transcript yet."
      }
    };
  }

  rsx! {
    div { class: "space-y-3 rounded-xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-low)] p-3 max-h-[28rem] overflow-y-auto",
      for row in rows {
        TranscriptBubble { key: "{row.ts}:{row.role}", row }
      }
    }
  }
}

fn transcript_rows(events: &[crate::runtime::types::DesktopRuntimeEvent]) -> Vec<TranscriptRow> {
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
        let role = event.payload.get("role").and_then(serde_json::Value::as_str).unwrap_or("assistant").to_string();
        rows.push(TranscriptRow { ts: event.ts.clone(), role, text: payload_text(&event.payload).unwrap_or_default() });
      },
      DesktopRuntimeEventKind::ToolCall => rows.push(TranscriptRow {
        ts: event.ts.clone(),
        role: "tool".to_string(),
        text: format!("Calling {}", event.payload.get("tool_name").and_then(serde_json::Value::as_str).unwrap_or("tool")),
      }),
      DesktopRuntimeEventKind::ToolError | DesktopRuntimeEventKind::BackendError => rows.push(TranscriptRow {
        ts: event.ts.clone(),
        role: "error".to_string(),
        text: payload_text(&event.payload).unwrap_or_else(|| "Runtime error".to_string()),
      }),
      DesktopRuntimeEventKind::ControlState | DesktopRuntimeEventKind::RuntimeEmit => {
        if let Some(text) = payload_text(&event.payload) {
          rows.push(TranscriptRow { ts: event.ts.clone(), role: "system".to_string(), text });
        }
      },
      DesktopRuntimeEventKind::AgentSpawn | DesktopRuntimeEventKind::AgentStop | DesktopRuntimeEventKind::ToolResult => {},
    }
  }
  rows.extend(streaming.into_values().map(|(ts, role, text)| TranscriptRow { ts, role, text }));
  rows.sort_by(|left, right| left.ts.cmp(&right.ts));
  rows
}

#[component]
fn TranscriptBubble(row: TranscriptRow) -> Element {
  let class = match row.role.as_str() {
    "user" => "ml-auto bg-[var(--primary)] text-[var(--on-primary)]",
    "assistant" => "mr-auto bg-[var(--surface-container-high)] text-[var(--on-surface)]",
    "tool" => "mr-auto bg-cyan-500/10 text-cyan-300 border border-cyan-500/20",
    "error" => "mr-auto bg-red-500/10 text-red-300 border border-red-500/20",
    _ => "mx-auto bg-[var(--surface-container)] text-[var(--outline)] border border-[var(--outline-variant)]/30",
  };

  rsx! {
    div { class: "max-w-[85%] rounded-xl px-3 py-2 text-sm whitespace-pre-wrap {class}",
      "{row.text}"
    }
  }
}
