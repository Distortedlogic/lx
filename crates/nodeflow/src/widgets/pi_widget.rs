use dioxus::prelude::*;

use crate::runtime::use_desktop_runtime;

use super::{PiInput, PiToolActivity, PiTranscript};

#[component]
pub fn PiWidget(agent_id: String) -> Element {
  let runtime = use_desktop_runtime();
  let Some(agent) = runtime.registry.find_agent(&agent_id) else {
    return rsx! {
      div { class: "rounded-xl border border-[var(--outline-variant)]/30 p-4 text-sm text-[var(--outline)]",
        "Runtime agent not found."
      }
    };
  };

  rsx! {
    div { class: "flex flex-col gap-4",
      div { class: "rounded-xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-4",
        div { class: "flex items-center justify-between gap-4",
          div {
            h2 { class: "text-lg font-semibold text-[var(--on-surface)]",
              "{agent.name}"
            }
            p { class: "text-sm text-[var(--outline)]", "{agent.task_summary}" }
          }
          div { class: "text-right text-xs text-[var(--outline)]",
            div { "Session: {agent.session_id}" }
            if let Some(model) = agent.model.clone() {
              div { "Model: {model}" }
            }
            div { "Status: {format_status(&agent.status)}" }
          }
        }
      }
      div { class: "grid grid-cols-1 xl:grid-cols-[minmax(0,1.5fr)_minmax(18rem,1fr)] gap-4",
        PiTranscript { agent_id: agent_id.clone() }
        div { class: "flex flex-col gap-4",
          PiToolActivity { agent_id: agent_id.clone() }
          PiInput { agent_id }
        }
      }
    }
  }
}

fn format_status(status: &crate::runtime::types::DesktopAgentStatus) -> &'static str {
  match status {
    crate::runtime::types::DesktopAgentStatus::Idle => "Idle",
    crate::runtime::types::DesktopAgentStatus::Starting => "Starting",
    crate::runtime::types::DesktopAgentStatus::Running => "Running",
    crate::runtime::types::DesktopAgentStatus::Paused => "Paused",
    crate::runtime::types::DesktopAgentStatus::Completed => "Completed",
    crate::runtime::types::DesktopAgentStatus::Error => "Error",
    crate::runtime::types::DesktopAgentStatus::Aborted => "Aborted",
  }
}
