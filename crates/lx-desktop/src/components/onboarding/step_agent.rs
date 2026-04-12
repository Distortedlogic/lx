use dioxus::prelude::*;

use crate::components::ui::select::{Select, SelectOption};
use crate::pages::agents::types::{ADAPTER_LABELS, ROLE_LABELS};

fn adapter_icon(key: &str) -> &'static str {
  match key {
    "claude_local" => "psychology",
    "codex_local" => "code",
    "gemini_local" => "auto_awesome",
    "opencode_local" => "terminal",
    "cursor" => "edit",
    "hermes_local" => "send",
    "openclaw_gateway" => "cloud",
    "process" => "memory",
    "http" => "language",
    _ => "smart_toy",
  }
}

#[component]
pub fn StepAgent(
  agent_name: Signal<String>,
  agent_role: Signal<String>,
  agent_description: Signal<String>,
  agent_adapter: Signal<String>,
  agent_model_id: Signal<String>,
) -> Element {
  rsx! {
    div { class: "space-y-5",
      div { class: "flex items-center gap-3 mb-1",
        div { class: "bg-[var(--surface-container-highest)] p-2",
          span { class: "material-symbols-outlined text-xl text-[var(--outline)]",
            "smart_toy"
          }
        }
        div {
          h3 { class: "text-sm font-medium text-[var(--on-surface)]",
            "Create your first agent"
          }
          p { class: "text-xs text-[var(--outline)]",
            "Configure the agent that will run your first lx flow."
          }
        }
      }
      div { class: "grid grid-cols-2 gap-3",
        div { class: "space-y-1",
          label { class: "text-xs text-[var(--outline)] block", "Agent name" }
          input {
            class: "onboarding-input",
            placeholder: "CEO",
            value: "{agent_name}",
            oninput: move |e| agent_name.set(e.value()),
            autofocus: true,
          }
        }
        div { class: "space-y-1",
          label { class: "text-xs text-[var(--outline)] block", "Role" }
          Select {
            class: "onboarding-select".to_string(),
            value: agent_role.read().clone(),
            options: ROLE_LABELS.iter().map(|(k, l)| SelectOption::new(*k, *l)).collect::<Vec<_>>(),
            onchange: move |val: String| agent_role.set(val),
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Adapter" }
        div { class: "grid grid-cols-3 gap-2",
          for (key , label) in ADAPTER_LABELS.iter() {
            {
                let k = *key;
                let l = *label;
                let selected = *agent_adapter.read() == k;
                let icon = adapter_icon(k);
                let cls = if selected {
                    "flex flex-col items-center gap-1 px-2 py-2.5 border-2 border-[var(--primary)] bg-[var(--primary)]/10 rounded cursor-pointer transition-colors"
                } else {
                    "flex flex-col items-center gap-1 px-2 py-2.5 border border-[var(--outline-variant)] rounded cursor-pointer hover:border-[var(--on-surface-variant)] transition-colors"
                };
                rsx! {
                  div {
                    key: "{k}",
                    class: cls,
                    onclick: move |_| agent_adapter.set(k.to_string()),
                    span { class: "material-symbols-outlined text-lg text-[var(--on-surface-variant)]", "{icon}" }
                    span { class: "text-[11px] text-[var(--on-surface)] leading-tight text-center", "{l}" }
                  }
                }
            }
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Model ID" }
        input {
          class: "onboarding-input",
          placeholder: "claude-sonnet-4-20250514",
          value: "{agent_model_id}",
          oninput: move |e| agent_model_id.set(e.value()),
        }
        p { class: "text-[10px] text-[var(--outline)]/60 mt-0.5",
          "The model identifier your adapter will use (e.g. claude-sonnet-4-20250514, gpt-4o, gemini-2.0-flash)"
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Description (optional)" }
        textarea {
          class: "onboarding-input resize-none min-h-[80px]",
          placeholder: "What should this agent focus on?",
          value: "{agent_description}",
          oninput: move |e| agent_description.set(e.value()),
        }
      }
    }
  }
}
