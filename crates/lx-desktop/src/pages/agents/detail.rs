use super::config_form::AgentConfigPanel;
use super::list::StatusBadge;
use super::overview::AgentOverview;
use super::types::{AgentDetail as AgentDetailData, AgentDetailTab, role_label};
use crate::styles::{BTN_OUTLINE_SM, TAB_ACTIVE, TAB_INACTIVE};
use dioxus::prelude::*;

#[component]
pub fn AgentDetailShell(
  agent: AgentDetailData,
  on_back: EventHandler<()>,
  on_run: EventHandler<()>,
  on_pause: EventHandler<()>,
  on_resume: EventHandler<()>,
  on_terminate: EventHandler<()>,
) -> Element {
  let mut active_tab = use_signal(|| AgentDetailTab::Overview);

  let role_text = role_label(&agent.role);
  let subtitle = match &agent.title {
    Some(t) => format!("{role_text} - {t}"),
    None => role_text.to_string(),
  };
  let is_paused = agent.status == "paused";

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-6",
      div { class: "flex items-center justify-between gap-2",
        div { class: "flex items-center gap-3 min-w-0",
          button {
            class: "shrink-0 text-xs text-[var(--outline)] hover:text-[var(--on-surface)]",
            onclick: move |_| on_back.call(()),
            "< Back"
          }
          AgentIconDisplay { icon: agent.icon.clone() }
          div { class: "min-w-0",
            h2 { class: "text-2xl font-bold text-[var(--on-surface)] truncate",
              "{agent.name}"
            }
            p { class: "text-sm text-[var(--outline)] truncate", "{subtitle}" }
          }
        }
        div { class: "flex items-center gap-2 shrink-0",
          button {
            class: BTN_OUTLINE_SM,
            onclick: move |_| on_run.call(()),
            "Run"
          }
          if is_paused {
            button {
              class: BTN_OUTLINE_SM,
              onclick: move |_| on_resume.call(()),
              "Resume"
            }
          } else {
            button {
              class: BTN_OUTLINE_SM,
              onclick: move |_| on_pause.call(()),
              "Pause"
            }
          }
          StatusBadge { status: agent.status.clone() }
        }
      }
      div { class: "flex gap-1 border-b border-[var(--outline-variant)]/30",
        for tab in AgentDetailTab::all() {
          button {
            class: if *active_tab.read() == *tab { TAB_ACTIVE } else { TAB_INACTIVE },
            onclick: {
                let tab = tab.clone();
                move |_| active_tab.set(tab.clone())
            },
            "{tab.label()}"
          }
        }
      }
      match *active_tab.read() {
          AgentDetailTab::Overview => rsx! {
            AgentOverview { agent: agent.clone() }
          },
          AgentDetailTab::Config => rsx! {
            AgentConfigPanel { agent: agent.clone() }
          },
          AgentDetailTab::Runs => rsx! {
            p { class: "text-sm text-[var(--outline)]", "Runs tab (Unit 8)" }
          },
          AgentDetailTab::Skills => rsx! {
            p { class: "text-sm text-[var(--outline)]", "Skills tab (Unit 8)" }
          },
          AgentDetailTab::Budget => rsx! {
            p { class: "text-sm text-[var(--outline)]", "Budget tab (Unit 8)" }
          },
      }
    }
  }
}

#[component]
fn AgentIconDisplay(icon: Option<String>) -> Element {
  let icon_char = icon.as_deref().unwrap_or("smart_toy");
  rsx! {
    div { class: "shrink-0 flex items-center justify-center h-12 w-12 rounded-lg bg-[var(--surface-container-high)]",
      span { class: "material-symbols-outlined text-xl", "{icon_char}" }
    }
  }
}
