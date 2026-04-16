use dioxus::prelude::*;

use crate::runtime::{DesktopAgentLaunchSpec, use_desktop_runtime};
use crate::widgets::PiWidget;

#[component]
pub fn PiPage(#[props(optional)] agent_id: Option<String>) -> Element {
  let runtime = use_desktop_runtime();
  let agents = runtime.registry.all_agents();
  let mut selected = use_signal(|| agent_id.clone());
  let mut prompt = use_signal(|| "Inspect this repository and summarize the active work.".to_string());

  let start = move |_| {
    let mut spec = DesktopAgentLaunchSpec::new("Pi Session", "Standalone Pi runtime session", prompt.read().clone());
    spec.cwd = std::env::current_dir().ok();
    let launched = runtime.launch_pi_agent(&spec);
    selected.set(Some(launched));
  };

  rsx! {
    div { class: "flex flex-col gap-4 p-4 h-full overflow-auto",
      div { class: "flex items-center justify-between gap-4",
        div {
          h1 { class: "page-heading", "PI RUNTIME" }
          p { class: "text-sm text-[var(--outline)]",
            "Pi sessions are adapted into the desktop runtime registry."
          }
        }
        button { class: "btn-outline-sm", onclick: start, "Launch Pi Session" }
      }
      textarea {
        class: "min-h-24 w-full rounded-xl border border-[var(--outline-variant)]/30 bg-[var(--surface)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
        value: "{prompt}",
        oninput: move |event| prompt.set(event.value()),
      }
      div { class: "grid grid-cols-1 xl:grid-cols-[18rem_minmax(0,1fr)] gap-4 min-h-0",
        div { class: "rounded-xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-low)] p-3",
          div { class: "mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
            "Agents"
          }
          if agents.is_empty() {
            p { class: "text-sm text-[var(--outline)]", "No runtime sessions yet." }
          } else {
            div { class: "space-y-2",
              for agent in agents {
                button {
                  key: "{agent.id}",
                  class: if selected.read().as_ref() == Some(&agent.id) { "w-full rounded-lg border border-cyan-500/30 bg-cyan-500/10 p-3 text-left" } else { "w-full rounded-lg border border-[var(--outline-variant)]/20 bg-[var(--surface-container)] p-3 text-left" },
                  onclick: {
                      let id = agent.id.clone();
                      move |_| selected.set(Some(id.clone()))
                  },
                  div { class: "text-sm font-medium text-[var(--on-surface)]",
                    "{agent.name}"
                  }
                  div { class: "text-xs text-[var(--outline)]",
                    "{agent.task_summary}"
                  }
                }
              }
            }
          }
        }
        if let Some(agent_id) = selected.read().clone() {
          PiWidget { agent_id }
        } else {
          div { class: "rounded-xl border border-[var(--outline-variant)]/30 p-4 text-sm text-[var(--outline)]",
            "Select or launch a Pi session."
          }
        }
      }
    }
  }
}
