use dioxus::prelude::*;

use crate::components::identity::Identity;
use crate::contexts::activity_log::ActivityLog;

struct ActiveAgent {
  name: String,
  status: String,
  last_seen: String,
}

#[component]
pub fn ActiveAgentsPanel() -> Element {
  let log = use_context::<ActivityLog>();
  let events = log.events.read();

  let mut seen = std::collections::HashMap::<String, ActiveAgent>::new();
  for event in events.iter() {
    if event.kind == "agent_start" || event.kind == "agent_running" {
      let name = event.message.clone();
      seen.entry(name.clone()).or_insert_with(|| ActiveAgent {
        name,
        status: if event.kind == "agent_running" { "running".to_string() } else { "started".to_string() },
        last_seen: event.timestamp.clone(),
      });
    }
  }

  let active_agents: Vec<ActiveAgent> = seen.into_values().collect();
  let no_agents = active_agents.is_empty();

  rsx! {
    div {
      h3 { class: "mb-3 text-sm font-semibold uppercase tracking-wide text-[var(--on-surface-variant)]",
        "Agents"
      }
      if no_agents {
        div { class: "rounded-xl border border-[var(--outline-variant)] p-4",
          p { class: "text-sm text-[var(--on-surface-variant)]", "No recent agent runs." }
        }
      } else {
        div { class: "grid grid-cols-1 gap-2 sm:grid-cols-2 xl:grid-cols-4",
          for agent in active_agents.iter() {
            AgentRunCard {
              name: agent.name.clone(),
              status: agent.status.clone(),
              last_seen: agent.last_seen.clone(),
            }
          }
        }
      }
    }
  }
}

#[component]
fn AgentRunCard(name: String, status: String, last_seen: String) -> Element {
  rsx! {
    div { class: "flex h-[200px] flex-col overflow-hidden rounded-xl border border-[var(--outline-variant)] shadow-sm bg-[var(--surface-container)]",
      div { class: "border-b border-[var(--outline-variant)]/60 px-3 py-3",
        div { class: "flex items-center gap-2",
          if status == "running" {
            span { class: "relative flex h-2.5 w-2.5 shrink-0",
              span { class: "absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--tertiary)] opacity-70" }
              span { class: "relative inline-flex h-2.5 w-2.5 rounded-full bg-[var(--tertiary)]" }
            }
          } else {
            span { class: "inline-flex h-2.5 w-2.5 rounded-full bg-[var(--outline)]" }
          }
          Identity { name: name.clone(), size: "sm".to_string() }
        }
        div { class: "mt-2 text-[11px] text-[var(--on-surface-variant)]", "{last_seen}" }
      }
      div { class: "min-h-0 flex-1 overflow-y-auto p-3",
        p { class: "text-xs text-[var(--outline)]", "No transcript available." }
      }
    }
  }
}
