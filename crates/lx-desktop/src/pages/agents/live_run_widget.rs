use dioxus::prelude::*;

use super::list::StatusBadge;
use super::run_types::source_label;
use super::transcript::TranscriptView;
use crate::runtime::use_desktop_runtime;
use crate::widgets::PiTranscript;

#[derive(Clone, Debug, PartialEq)]
pub struct LiveRunInfo {
  pub id: String,
  pub agent_id: String,
  pub agent_name: String,
  pub status: String,
  pub invocation_source: String,
  pub started_at: Option<String>,
  pub created_at: String,
}

#[component]
pub fn LiveRunWidget(runs: Vec<LiveRunInfo>, on_cancel: EventHandler<String>, on_open_run: EventHandler<(String, String)>) -> Element {
  if runs.is_empty() {
    return rsx! {};
  }

  rsx! {
    div { class: "overflow-hidden rounded-xl border border-cyan-500/25 bg-[var(--surface-container)]/80 shadow-lg",
      div { class: "border-b border-[var(--outline-variant)]/60 bg-cyan-500/[0.04] px-4 py-3",
        div { class: "text-xs font-semibold uppercase tracking-widest text-cyan-400",
          "Live Runs"
        }
      }
      div { class: "divide-y divide-[var(--outline-variant)]/60",
        for run in runs.iter() {
          LiveRunEntry {
            run: run.clone(),
            on_cancel: {
                let id = run.id.clone();
                move |_| on_cancel.call(id.clone())
            },
            on_open: {
                let agent_id = run.agent_id.clone();
                let run_id = run.id.clone();
                move |_| on_open_run.call((agent_id.clone(), run_id.clone()))
            },
          }
        }
      }
    }
  }
}

#[component]
fn LiveRunEntry(run: LiveRunInfo, on_cancel: EventHandler<()>, on_open: EventHandler<()>) -> Element {
  let runtime = use_desktop_runtime();
  let is_active = run.status == "running" || run.status == "queued";
  let short_id = &run.id[..8.min(run.id.len())];

  rsx! {
    section { class: "px-4 py-4",
      div { class: "mb-3 flex items-start justify-between",
        div { class: "min-w-0",
          span { class: "text-sm font-medium text-[var(--on-surface)]", "{run.agent_name}" }
          div { class: "mt-2 flex items-center gap-2 text-xs text-[var(--outline)]",
            span { class: "font-mono", "{short_id}" }
            StatusBadge { status: run.status.clone() }
            span { "{source_label(&run.invocation_source)}" }
          }
        }
        div { class: "flex items-center gap-2",
          if is_active {
            button {
              class: "inline-flex items-center gap-1 rounded-full border border-red-500/20 bg-red-500/[0.06] px-2.5 py-1 text-[11px] font-medium text-red-400 hover:bg-red-500/[0.12] transition-colors",
              onclick: move |_| on_cancel.call(()),
              "Stop"
            }
          }
          button {
            class: "inline-flex items-center gap-1 rounded-full border border-[var(--outline-variant)]/70 bg-[var(--surface-container)]/70 px-2.5 py-1 text-[11px] font-medium text-cyan-400 hover:border-cyan-500/30 transition-colors",
            onclick: move |_| on_open.call(()),
            "Open run"
          }
        }
      }
      div { class: "max-h-80 overflow-y-auto pr-1",
        if runtime.registry.find_agent(&run.agent_id).is_some() {
          PiTranscript { agent_id: run.agent_id.clone() }
        } else {
          TranscriptView { run_id: run.id.clone() }
        }
      }
    }
  }
}
