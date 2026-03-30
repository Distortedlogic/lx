use dioxus::prelude::*;

use crate::components::ui::select::{Select, SelectOption};
use crate::pages::agents::types::{ADAPTER_LABELS, ROLE_LABELS};

const INPUT_CLS: &str = "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]";
const SELECT_CLS: &str = "w-full bg-[var(--surface-container)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)]";

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
            class: INPUT_CLS,
            placeholder: "CEO",
            value: "{agent_name}",
            oninput: move |e| agent_name.set(e.value()),
            autofocus: true,
          }
        }
        div { class: "space-y-1",
          label { class: "text-xs text-[var(--outline)] block", "Role" }
          Select {
            class: SELECT_CLS.to_string(),
            value: agent_role.read().clone(),
            options: ROLE_LABELS.iter().map(|(k, l)| SelectOption::new(*k, *l)).collect::<Vec<_>>(),
            onchange: move |val: String| agent_role.set(val),
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Adapter" }
        Select {
          class: SELECT_CLS.to_string(),
          value: agent_adapter.read().clone(),
          options: ADAPTER_LABELS.iter().map(|(k, l)| SelectOption::new(*k, *l)).collect::<Vec<_>>(),
          onchange: move |val: String| agent_adapter.set(val),
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Model ID" }
        input {
            class: INPUT_CLS,
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
          class: "{INPUT_CLS} resize-none min-h-[80px]",
          placeholder: "What should this agent focus on?",
          value: "{agent_description}",
          oninput: move |e| agent_description.set(e.value()),
        }
      }
    }
  }
}
