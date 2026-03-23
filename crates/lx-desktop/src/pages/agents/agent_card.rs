use dioxus::prelude::*;

use crate::terminal::status_badge::{BadgeVariant, StatusBadge};

#[derive(Clone, Copy, PartialEq)]
pub enum AgentStatus {
  Idle,
  Active,
  Running,
}

impl AgentStatus {
  fn label(self) -> &'static str {
    match self {
      Self::Idle => "IDLE",
      Self::Active => "ACTIVE",
      Self::Running => "RUNNING",
    }
  }

  fn badge_variant(self) -> BadgeVariant {
    match self {
      Self::Idle => BadgeVariant::Idle,
      Self::Active => BadgeVariant::Active,
      Self::Running => BadgeVariant::Running,
    }
  }
}

#[component]
pub fn AgentCard(
  agent_name: &'static str,
  status: AgentStatus,
  pid: &'static str,
  memory: &'static str,
  action_text: &'static str,
  task_items: Vec<(&'static str, &'static str)>,
) -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container)] rounded-lg p-4",
      div { class: "flex items-center gap-3 mb-3",
        span { class: "text-[var(--primary)]", "\u{25CF}" }
        span { class: "font-semibold uppercase text-sm tracking-wider text-[var(--on-surface)]", "{agent_name}" }
        StatusBadge { label: status.label().to_string(), variant: status.badge_variant() }
        div { class: "flex-1" }
        span { class: "text-xs text-[var(--outline)]", "PID {pid} \u{00B7} {memory}" }
        div { class: "flex items-center gap-1 ml-2",
          button { class: "px-1.5 py-0.5 text-xs text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container-high)] rounded transition-colors duration-150", "\u{25B6}" }
          button { class: "px-1.5 py-0.5 text-xs text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container-high)] rounded transition-colors duration-150", "\u{25A0}" }
          button { class: "px-1.5 py-0.5 text-xs text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container-high)] rounded transition-colors duration-150", "\u{2699}" }
        }
      }
      if status == AgentStatus::Idle {
        div { class: "flex items-center justify-center py-6 text-sm text-[var(--outline)]",
          "WAITING FOR TASK ALLOCATION..."
        }
      } else {
        div { class: "flex gap-3",
          div { class: "flex-1 bg-[var(--surface-container-low)] rounded p-3",
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-2", "CURRENT ACTION" }
            p { class: "text-xs text-[var(--on-surface-variant)] font-mono",
              span { class: "text-[var(--primary)]", "{action_text}" }
            }
          }
          if !task_items.is_empty() {
            div { class: "flex-1 bg-[var(--surface-container-low)] rounded p-3",
              p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-2", "TASK STACK" }
              for (status_icon , task_label) in task_items.iter() {
                {
                    let icon = match *status_icon {
                        "done" => "\u{25CF}",
                        "active" => "\u{25C7}",
                        _ => "\u{25CB}",
                    };
                    let color = match *status_icon {
                        "done" => "text-[var(--success)]",
                        "active" => "text-[var(--primary)]",
                        _ => "text-[var(--outline)]",
                    };
                    rsx! {
                      p { class: "text-xs mb-1",
                        span { class: "{color} mr-1.5", "{icon}" }
                        "{task_label}"
                      }
                    }
                }
              }
            }
          }
        }
      }
    }
  }
}
