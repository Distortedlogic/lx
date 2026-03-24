use dioxus::prelude::*;

use crate::terminal::status_badge::{BadgeVariant, StatusBadge};

#[derive(Clone, Copy, PartialEq)]
pub enum AgentStatus {
  Idle,
  Active,
}

impl AgentStatus {
  pub fn label(self) -> &'static str {
    match self {
      Self::Idle => "IDLE",
      Self::Active => "ACTIVE",
    }
  }

  pub fn badge_variant(self) -> BadgeVariant {
    match self {
      Self::Idle => BadgeVariant::Idle,
      Self::Active => BadgeVariant::Active,
    }
  }
}

#[component]
pub fn AgentCard(
  agent_name: &'static str,
  status: AgentStatus,
  current_task: Option<&'static str>,
  resources: Option<&'static str>,
  live_output: Option<Vec<&'static str>>,
  last_active: Option<&'static str>,
  memory_load: Option<f64>,
) -> Element {
  let border_class = match status {
    AgentStatus::Active => "border border-[var(--primary)]/60",
    _ => "border border-[var(--primary)]/30",
  };
  let card_class = format!("bg-[var(--surface-container)] rounded-lg p-4 flex-1 min-w-0 {border_class}");

  let load_percent = (memory_load.unwrap_or(0.0) * 100.0) as u32;

  rsx! {
    div { class: "{card_class}",
      div { class: "flex items-center gap-3 mb-3",
        span { class: "text-[var(--primary)]", "\u{25CF}" }
        span { class: "font-semibold uppercase text-sm tracking-wider text-[var(--on-surface)]",
          "{agent_name}"
        }
        StatusBadge {
          label: status.label().to_string(),
          variant: status.badge_variant(),
        }
      }
      if let Some(current_task_val) = current_task {
        div { class: "flex gap-4 text-xs mb-3",
          div {
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
              "CURRENT TASK"
            }
            p { class: "text-[var(--on-surface-variant)] uppercase", "{current_task_val}" }
          }
          if let Some(resources_val) = resources {
            div {
              p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
                "RESOURCES"
              }
              p { class: "text-[var(--on-surface-variant)]", "{resources_val}" }
            }
          }
        }
        if let Some(output_lines) = &live_output {
          div { class: "mb-3",
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-2",
              "LIVE OUTPUT"
            }
            div { class: "bg-[var(--surface-container-low)] rounded p-3 font-mono text-xs text-[var(--success)]",
              for line in output_lines.iter() {
                p { class: "mb-0.5", "\u{203A} {line}" }
              }
            }
          }
        }
        div { class: "flex gap-3",
          button { class: "flex-1 border border-[var(--outline)] text-[var(--on-surface)] rounded py-2 text-xs uppercase tracking-wider hover:bg-[var(--surface-container-high)] transition-colors duration-150",
            "INTERCEPT"
          }
          button { class: "flex-1 border border-[var(--outline)] text-[var(--on-surface)] rounded py-2 text-xs uppercase tracking-wider hover:bg-[var(--surface-container-high)] transition-colors duration-150",
            "TERMINATE"
          }
        }
      }
      if let Some(last_active_val) = last_active {
        div { class: "flex gap-4 text-xs mb-3",
          div {
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
              "STATUS"
            }
            p { class: "text-[var(--on-surface-variant)] uppercase", "AWAITING_COMMAND" }
          }
          div {
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
              "LAST ACTIVE"
            }
            p { class: "text-[var(--on-surface-variant)]", "{last_active_val}" }
          }
        }
        div { class: "mb-3",
          p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-2",
            "MEMORY SNAP"
          }
          div { class: "h-2 bg-[var(--surface-container-low)] rounded-full overflow-hidden mb-1",
            div {
              class: "h-full bg-[var(--primary)] rounded-full",
              style: "width: {load_percent}%;",
            }
          }
          p { class: "text-[10px] text-[var(--outline)]", "{load_percent}% LOAD" }
        }
        button { class: "w-full bg-[var(--success)] text-[var(--on-primary)] rounded py-2 text-sm uppercase tracking-wider font-semibold hover:brightness-110 transition-all duration-150",
          "DEPLOY MISSION"
        }
      }
    }
  }
}
