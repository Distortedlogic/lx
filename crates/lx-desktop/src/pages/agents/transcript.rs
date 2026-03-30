use super::transcript_blocks;
use dioxus::prelude::*;
use lx_api::types::ActivityEvent;

#[derive(Clone, Debug, PartialEq)]
pub enum ToolStatus {
  Running,
  Completed,
  Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolItem {
  pub ts: String,
  pub name: String,
  pub input: String,
  pub result: Option<String>,
  pub is_error: bool,
  pub status: ToolStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StderrLine {
  pub ts: String,
  pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TranscriptBlock {
  Message { role: String, text: String, ts: String },
  Thinking { text: String, ts: String },
  Tool { name: String, input: String, result: Option<String>, is_error: bool, status: ToolStatus, ts: String },
  Activity { name: String, status: ToolStatus, ts: String },
  CommandGroup { items: Vec<ToolItem>, ts: String },
  ToolGroup { items: Vec<ToolItem>, ts: String },
  StderrGroup { lines: Vec<StderrLine>, ts: String },
  Stdout { text: String, ts: String },
  Event { label: String, text: String, detail: Option<String>, tone: String, ts: String },
}

fn event_to_block(event: &ActivityEvent) -> TranscriptBlock {
  match event.kind.as_str() {
    "log" | "agent_message" | "message" | "assistant" | "user" => {
      let role = if event.kind == "user" { "user" } else { "assistant" };
      TranscriptBlock::Message { role: role.into(), text: event.message.clone(), ts: event.timestamp.clone() }
    },
    "thinking" => TranscriptBlock::Thinking { text: event.message.clone(), ts: event.timestamp.clone() },
    "tool_call" => TranscriptBlock::Tool {
      name: event.kind.clone(),
      input: event.message.clone(),
      result: None,
      is_error: false,
      status: ToolStatus::Running,
      ts: event.timestamp.clone(),
    },
    "tool_result" => TranscriptBlock::Tool {
      name: "tool".into(),
      input: String::new(),
      result: Some(event.message.clone()),
      is_error: false,
      status: ToolStatus::Completed,
      ts: event.timestamp.clone(),
    },
    "stderr" => {
      TranscriptBlock::StderrGroup { lines: vec![StderrLine { ts: event.timestamp.clone(), text: event.message.clone() }], ts: event.timestamp.clone() }
    },
    "stdout" => TranscriptBlock::Stdout { text: event.message.clone(), ts: event.timestamp.clone() },
    "activity" => TranscriptBlock::Activity { name: event.message.clone(), status: ToolStatus::Running, ts: event.timestamp.clone() },
    k if k.contains("tool") => TranscriptBlock::Tool {
      name: event.kind.clone(),
      input: event.message.clone(),
      result: None,
      is_error: false,
      status: ToolStatus::Running,
      ts: event.timestamp.clone(),
    },
    k if k.contains("error") => {
      TranscriptBlock::Event { label: "error".into(), text: event.message.clone(), detail: None, tone: "error".into(), ts: event.timestamp.clone() }
    },
    _ => TranscriptBlock::Event { label: event.kind.clone(), text: event.message.clone(), detail: None, tone: "info".into(), ts: event.timestamp.clone() },
  }
}

#[component]
pub fn TranscriptView(run_id: String, #[props(optional)] events: Option<Vec<ActivityEvent>>) -> Element {
  let entries: Vec<TranscriptBlock> = match events {
    Some(evts) => evts.iter().map(event_to_block).collect(),
    None => vec![],
  };

  if entries.is_empty() {
    return rsx! {
      div { class: "border border-[var(--outline-variant)]/30 rounded-lg p-4",
        p { class: "text-sm text-[var(--outline)] text-center",
          "No transcript data available."
        }
      }
    };
  }

  rsx! {
    div { class: "space-y-2",
      for entry in entries.iter() {
        transcript_blocks::TranscriptBlockView { block: entry.clone() }
      }
    }
  }
}
