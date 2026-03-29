use super::types::{AgentSummary, FilterTab, adapter_label, role_label, status_dot_class};
use crate::styles::{BTN_OUTLINE_SM, FLEX_BETWEEN, TAB_ACTIVE, TAB_INACTIVE};
use dioxus::prelude::*;

#[component]
pub fn AgentList(agents: Vec<AgentSummary>, on_select: EventHandler<String>, on_new_agent: EventHandler<()>) -> Element {
  let mut active_tab = use_signal(|| FilterTab::All);
  let filtered: Vec<&AgentSummary> = agents.iter().filter(|a| active_tab.read().matches(&a.status)).collect();
  let count_label = if filtered.len() == 1 { "1 agent".to_string() } else { format!("{} agents", filtered.len()) };

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: FLEX_BETWEEN,
        div { class: "flex gap-1",
          for tab in [FilterTab::All, FilterTab::Active, FilterTab::Paused, FilterTab::Error] {
            button {
              class: if *active_tab.read() == tab { TAB_ACTIVE } else { TAB_INACTIVE },
              onclick: {
                  let tab = tab.clone();
                  move |_| active_tab.set(tab.clone())
              },
              "{tab.label()}"
            }
          }
        }
        button {
          class: BTN_OUTLINE_SM,
          onclick: move |_| on_new_agent.call(()),
          "+ New Agent"
        }
      }
      if filtered.is_empty() {
        div { class: "flex-1 flex items-center justify-center",
          p { class: "text-sm text-[var(--outline)]", "No agents match this filter." }
        }
      }
      p { class: "text-xs text-[var(--outline)]", "{count_label}" }
      div { class: "border border-[var(--outline-variant)]/30 overflow-hidden",
        for agent in filtered.iter() {
          AgentRow {
            agent: (*agent).clone(),
            on_click: {
                let id = agent.id.clone();
                move |_| on_select.call(id.clone())
            },
          }
        }
      }
    }
  }
}

#[component]
fn AgentRow(agent: AgentSummary, on_click: EventHandler<()>) -> Element {
  let subtitle = {
    let role = role_label(&agent.role);
    match &agent.title {
      Some(t) => format!("{role} - {t}"),
      None => role.to_string(),
    }
  };
  let adapter = adapter_label(&agent.adapter_type);

  rsx! {
    button {
      class: "flex items-center gap-3 px-3 py-2.5 w-full text-left border-b border-[var(--outline-variant)]/15 hover:bg-[var(--surface-container)] transition-colors",
      onclick: move |_| on_click.call(()),
      span { class: "{status_dot_class(&agent.status)}" }
      div { class: "flex-1 min-w-0",
        span { class: "text-sm font-medium text-[var(--on-surface)]", "{agent.name}" }
        span { class: "text-xs text-[var(--outline)] ml-2", "{subtitle}" }
      }
      span { class: "text-xs text-[var(--outline)] font-mono w-14 text-right",
        "{adapter}"
      }
      StatusBadge { status: agent.status.clone() }
    }
  }
}

#[component]
pub fn StatusBadge(status: String) -> Element {
  let (bg, text) = match status.as_str() {
    "active" | "running" | "idle" => ("bg-green-500/10 text-green-600", "Active"),
    "paused" => ("bg-yellow-500/10 text-yellow-600", "Paused"),
    "error" => ("bg-red-500/10 text-red-600", "Error"),
    "terminated" => ("bg-neutral-500/10 text-neutral-500", "Terminated"),
    "pending_approval" => ("bg-amber-500/10 text-amber-600", "Pending"),
    other => ("bg-neutral-500/10 text-neutral-400", other),
  };
  let label = text.to_string();
  rsx! {
    span { class: "inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium {bg}",
      "{label}"
    }
  }
}
