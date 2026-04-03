use super::transcript_blocks;
use crate::components::scroll_to_bottom::ScrollToBottom;
use dioxus::prelude::*;
use lx_api::types::ActivityEvent;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum TranscriptMode {
  Nice,
  Raw,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum TranscriptDensity {
  Comfortable,
  Compact,
}

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
  pub token_count: Option<u32>,
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
  Tool { name: String, input: String, result: Option<String>, is_error: bool, status: ToolStatus, ts: String, token_count: Option<u32> },
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
      token_count: event.token_count,
    },
    "tool_result" => TranscriptBlock::Tool {
      name: "tool".into(),
      input: String::new(),
      result: Some(event.message.clone()),
      is_error: false,
      status: ToolStatus::Completed,
      ts: event.timestamp.clone(),
      token_count: event.token_count,
    },
    "tool_error" => TranscriptBlock::Tool {
      name: "tool".into(),
      input: String::new(),
      result: Some(event.message.clone()),
      is_error: true,
      status: ToolStatus::Error,
      ts: event.timestamp.clone(),
      token_count: event.token_count,
    },
    "command_group" => TranscriptBlock::CommandGroup {
      items: vec![ToolItem {
        ts: event.timestamp.clone(),
        name: "command".into(),
        input: event.message.clone(),
        result: None,
        is_error: false,
        status: ToolStatus::Running,
        token_count: event.token_count,
      }],
      ts: event.timestamp.clone(),
    },
    "tool_group" => TranscriptBlock::ToolGroup {
      items: vec![ToolItem {
        ts: event.timestamp.clone(),
        name: "tool".into(),
        input: event.message.clone(),
        result: None,
        is_error: false,
        status: ToolStatus::Running,
        token_count: event.token_count,
      }],
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
      token_count: event.token_count,
    },
    k if k.contains("error") => {
      TranscriptBlock::Event { label: "error".into(), text: event.message.clone(), detail: None, tone: "error".into(), ts: event.timestamp.clone() }
    },
    _ => TranscriptBlock::Event { label: event.kind.clone(), text: event.message.clone(), detail: None, tone: "info".into(), ts: event.timestamp.clone() },
  }
}

fn strip_shell_wrapper(input: &str) -> &str {
  let s = input.trim();
  for prefix in ["bash -lc ", "sh -c ", "bash -c "] {
    if let Some(rest) = s.strip_prefix(prefix) {
      let rest = rest.trim();
      if (rest.starts_with('"') && rest.ends_with('"')) || (rest.starts_with('\'') && rest.ends_with('\'')) {
        if rest.len() <= 2 {
          return "";
        }
        return &rest[1..rest.len() - 1];
      }
      return rest;
    }
  }
  s
}

fn extract_file_paths(input: &str) -> Vec<&str> {
  input.split_whitespace().filter(|w| w.starts_with('/') || w.starts_with("./") || w.starts_with("~/")).collect()
}

pub fn summarize_tool_input(input: &str, max_len: usize) -> String {
  let stripped = strip_shell_wrapper(input);
  let paths = extract_file_paths(stripped);
  if !paths.is_empty() {
    let summary = paths.join(", ");
    if summary.len() <= max_len {
      return summary;
    }
  }
  let trimmed = stripped.trim();
  if trimmed.len() <= max_len {
    return trimmed.to_string();
  }
  let mut end = max_len;
  while !trimmed.is_char_boundary(end) {
    end -= 1;
  }
  format!("{}...", &trimmed[..end])
}

#[component]
pub fn TranscriptView(run_id: String, #[props(optional)] events: Option<Vec<ActivityEvent>>, #[props(optional)] limit: Option<usize>) -> Element {
  let mut mode = use_signal(|| TranscriptMode::Nice);
  let mut density = use_signal(|| TranscriptDensity::Comfortable);

  let all_entries: Vec<TranscriptBlock> = match events {
    Some(evts) => evts.iter().map(event_to_block).collect(),
    None => vec![],
  };
  let entries: Vec<TranscriptBlock> = match limit {
    Some(n) if n < all_entries.len() => {
      let skip = all_entries.len().saturating_sub(n);
      all_entries.into_iter().skip(skip).collect()
    },
    _ => all_entries,
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

  let active_btn = "text-xs px-2 py-1 rounded transition-colors bg-[var(--primary)] text-[var(--on-primary)]";
  let inactive_btn =
    "text-xs px-2 py-1 rounded transition-colors bg-[var(--surface-container)] text-[var(--on-surface-variant)] hover:bg-[var(--surface-container-high)]";
  let cur_mode = mode();
  let cur_density = density();

  rsx! {
    div { class: "flex items-center gap-3 mb-2",
      div { class: "flex gap-0.5 rounded-md bg-[var(--surface-container)]/50 p-0.5",
        button {
          class: if cur_mode == TranscriptMode::Nice { active_btn } else { inactive_btn },
          onclick: move |_| mode.set(TranscriptMode::Nice),
          "Nice"
        }
        button {
          class: if cur_mode == TranscriptMode::Raw { active_btn } else { inactive_btn },
          onclick: move |_| mode.set(TranscriptMode::Raw),
          "Raw"
        }
      }
      div { class: "flex gap-0.5 rounded-md bg-[var(--surface-container)]/50 p-0.5",
        button {
          class: if cur_density == TranscriptDensity::Comfortable { active_btn } else { inactive_btn },
          onclick: move |_| density.set(TranscriptDensity::Comfortable),
          "Comfortable"
        }
        button {
          class: if cur_density == TranscriptDensity::Compact { active_btn } else { inactive_btn },
          onclick: move |_| density.set(TranscriptDensity::Compact),
          "Compact"
        }
      }
    }
    ScrollToBottom { class: "max-h-[60vh]".to_string(),
      div { class: if cur_density == TranscriptDensity::Compact { "space-y-1" } else { "space-y-2" },
        for entry in entries.iter() {
          transcript_blocks::TranscriptBlockView {
            block: entry.clone(),
            mode: cur_mode,
            density: cur_density,
          }
        }
      }
    }
  }
}
