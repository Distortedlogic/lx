use dioxus::prelude::*;
use lx_api::types::ActivityEvent;

#[derive(Clone, Debug, PartialEq)]
pub enum TranscriptBlock {
  Message { role: String, text: String, ts: String },
  Thinking { text: String, ts: String },
  ToolUse { name: String, input_summary: String, result: Option<String>, is_error: bool, ts: String },
  Event { label: String, text: String, tone: String, ts: String },
}

fn event_to_block(event: &ActivityEvent) -> TranscriptBlock {
  match event.kind.as_str() {
    "log" | "agent_message" | "message" => TranscriptBlock::Message { role: "assistant".into(), text: event.message.clone(), ts: event.timestamp.clone() },
    "thinking" => TranscriptBlock::Thinking { text: event.message.clone(), ts: event.timestamp.clone() },
    k if k.contains("tool") => {
      TranscriptBlock::ToolUse { name: event.kind.clone(), input_summary: event.message.clone(), result: None, is_error: false, ts: event.timestamp.clone() }
    },
    k if k.contains("error") => {
      TranscriptBlock::Event { label: "error".into(), text: event.message.clone(), tone: "error".into(), ts: event.timestamp.clone() }
    },
    _ => TranscriptBlock::Event { label: event.kind.clone(), text: event.message.clone(), tone: "info".into(), ts: event.timestamp.clone() },
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
        TranscriptBlockView { block: entry.clone() }
      }
    }
  }
}

#[component]
fn TranscriptBlockView(block: TranscriptBlock) -> Element {
  match block {
    TranscriptBlock::Message { role, text, .. } => {
      let icon = if role == "assistant" { "smart_toy" } else { "person" };
      let bg = if role == "assistant" { "bg-[var(--surface-container)]" } else { "bg-[var(--surface-container-high)]" };
      rsx! {
        div { class: "flex gap-3 p-3 rounded-lg {bg}",
          span { class: "material-symbols-outlined text-sm text-[var(--outline)] shrink-0 mt-0.5",
            "{icon}"
          }
          div { class: "flex-1 min-w-0 text-sm text-[var(--on-surface)] whitespace-pre-wrap break-words",
            "{text}"
          }
        }
      }
    },
    TranscriptBlock::Thinking { text, .. } => {
      rsx! {
        div { class: "flex gap-3 p-3 rounded-lg bg-amber-500/5 border border-amber-500/10",
          span { class: "material-symbols-outlined text-sm text-amber-600 shrink-0 mt-0.5",
            "psychology"
          }
          div { class: "flex-1 min-w-0 text-xs text-[var(--outline)] italic whitespace-pre-wrap",
            "{text}"
          }
        }
      }
    },
    TranscriptBlock::ToolUse { name, input_summary, result, is_error, .. } => {
      let border = if is_error { "border-red-500/20" } else { "border-[var(--outline-variant)]/20" };
      rsx! {
        div { class: "border {border} rounded-lg p-3 space-y-2",
          div { class: "flex items-center gap-2",
            span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
              "build"
            }
            span { class: "text-xs font-medium text-[var(--on-surface)]",
              "{name}"
            }
          }
          if !input_summary.is_empty() {
            p { class: "text-xs text-[var(--outline)] font-mono truncate",
              "{input_summary}"
            }
          }
          if let Some(res) = result {
            div { class: "text-xs font-mono whitespace-pre-wrap max-h-32 overflow-y-auto p-2 bg-[var(--surface-container)] rounded",
              "{res}"
            }
          }
        }
      }
    },
    TranscriptBlock::Event { label, text, tone, .. } => {
      let color = match tone.as_str() {
        "error" => "text-red-600",
        "warn" => "text-amber-600",
        "info" => "text-cyan-600",
        _ => "text-[var(--outline)]",
      };
      rsx! {
        div { class: "flex items-center gap-2 py-1",
          span { class: "text-[10px] font-semibold uppercase tracking-wider {color}",
            "{label}"
          }
          span { class: "text-xs text-[var(--outline)]", "{text}" }
        }
      }
    },
  }
}
