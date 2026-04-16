use dioxus::prelude::*;

use crate::runtime::use_desktop_runtime;
use crate::widgets::PiWidget;

use super::controller::use_flow_editor_state;

#[component]
pub fn FlowRuntimeBar() -> Element {
  let state = use_flow_editor_state();
  let runtime = use_desktop_runtime();
  let mut last_agent = use_signal(|| Option::<String>::None);

  let flow_id = state.flow_id.read().clone();
  let document = state.document.read().clone();
  let flow_agents = runtime.registry.agents_for_flow(&flow_id);
  let launch_prompt = flow_prompt(&document.title, document.metadata.notes.as_deref());

  rsx! {
    div { class: "mb-4 rounded-2xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-4",
      div { class: "flex flex-wrap items-center justify-between gap-4",
        div {
          div { class: "text-xs font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
            "Runtime"
          }
          p { class: "text-sm text-[var(--on-surface-variant)]",
            "Launch Pi-backed runtime sessions grouped by flow_id while keeping desktop state lx-shaped."
          }
        }
        button {
          class: "btn-outline-sm",
          onclick: move |_| {
              let name = format!("Flow {}", document.title);
              let launched = runtime.launch_flow_pi_agent(flow_id.clone(), &name, launch_prompt.clone(), std::env::current_dir().ok());
              last_agent.set(Some(launched));
          },
          "Launch Pi Flow Run"
        }
      }
      if !flow_agents.is_empty() {
        div { class: "mt-3 flex flex-wrap gap-2",
          for agent in flow_agents.iter() {
            button {
              key: "{agent.id}",
              class: "rounded-full border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-3 py-1.5 text-xs text-[var(--on-surface)]",
              onclick: {
                  let id = agent.id.clone();
                  move |_| last_agent.set(Some(id.clone()))
              },
              "{agent.name}"
            }
          }
        }
      }
      if let Some(agent_id) = last_agent.read().clone() {
        div { class: "mt-4",
          PiWidget { agent_id }
        }
      }
    }
  }
}

fn flow_prompt(title: &str, notes: Option<&str>) -> String {
  match notes {
    Some(notes) if !notes.trim().is_empty() => format!("Work on the flow \"{title}\". Flow notes: {notes}"),
    _ => format!("Work on the flow \"{title}\" and report the next execution steps."),
  }
}
