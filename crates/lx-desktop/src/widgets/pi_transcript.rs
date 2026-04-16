use dioxus::prelude::*;

use crate::runtime::events::{RuntimeTranscriptRow, transcript_rows};
use crate::runtime::use_desktop_runtime;

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

#[component]
fn TranscriptBubble(row: RuntimeTranscriptRow) -> Element {
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
