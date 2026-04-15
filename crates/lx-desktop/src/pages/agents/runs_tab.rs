use dioxus::prelude::*;

use super::list::StatusBadge;
use super::run_detail::RunDetailPanel;
use super::run_types::{HeartbeatRun, source_label};

#[component]
pub fn RunsTab(runs: Vec<HeartbeatRun>, agent_route_id: String) -> Element {
  let mut selected_run_id = use_signal(|| Option::<String>::None);

  if runs.is_empty() {
    return rsx! {
      p { class: "text-sm text-[var(--outline)]", "No runs yet." }
    };
  }

  let mut sorted = runs.clone();
  sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));

  let effective_id = selected_run_id.read().clone().or_else(|| sorted.first().map(|r| r.id.clone()));
  let selected_run = effective_id.as_ref().and_then(|id| sorted.iter().find(|r| &r.id == id));

  rsx! {
    div { class: "flex gap-0",
      div {
        class: "shrink-0 border border-[var(--outline-variant)]/30 rounded-lg w-72 overflow-y-auto",
        style: "max-height: calc(100vh - 2rem);",
        for run in sorted.iter() {
          RunListItem {
            run: run.clone(),
            is_selected: effective_id.as_ref() == Some(&run.id),
            on_select: {
                let id = run.id.clone();
                move |_| selected_run_id.set(Some(id.clone()))
            },
          }
        }
      }
      if let Some(run) = selected_run {
        div { class: "flex-1 min-w-0 pl-4",
          RunDetailPanel { run: run.clone() }
        }
      }
    }
  }
}

#[component]
fn RunListItem(run: HeartbeatRun, is_selected: bool, on_select: EventHandler<()>) -> Element {
  let is_live = run.status == "running" || run.status == "queued";
  let short_id = &run.id[..8.min(run.id.len())];
  let bg = if is_selected { "bg-[var(--surface-container-high)]" } else { "" };

  rsx! {
    button {
      class: "flex items-center gap-2 w-full px-3 py-2.5 text-left border-b border-[var(--outline-variant)]/15 hover:bg-[var(--surface-container)] transition-colors",
      class: "{bg}",
      onclick: move |_| on_select.call(()),
      if is_live {
        span { class: "relative flex h-2 w-2 shrink-0",
          span { class: "animate-pulse absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75" }
          span { class: "relative inline-flex rounded-full h-2 w-2 bg-cyan-400" }
        }
      }
      div { class: "flex-1 min-w-0",
        span { class: "text-xs font-mono text-[var(--on-surface)]", "{short_id}" }
        span { class: "text-xs text-[var(--outline)] ml-2",
          "{source_label(&run.invocation_source)}"
        }
      }
      StatusBadge { status: run.status.clone() }
    }
  }
}
