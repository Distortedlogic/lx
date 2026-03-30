use dioxus::prelude::*;

use crate::pages::agents::types::{ADAPTER_LABELS, ROLE_LABELS};

const INPUT_CLS: &str = "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]";
const SELECT_CLS: &str = "w-full border border-[var(--outline-variant)] bg-[var(--surface-container)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)]";

#[component]
pub fn StepAgent(agent_name: Signal<String>, agent_role: Signal<String>, agent_description: Signal<String>, agent_adapter: Signal<String>) -> Element {
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
            "Configure the agent that will handle your first task."
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
          select {
            class: SELECT_CLS,
            value: "{agent_role}",
            onchange: move |e| agent_role.set(e.value()),
            for (key , label) in ROLE_LABELS {
              option { value: *key, "{label}" }
            }
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Adapter" }
        select {
          class: SELECT_CLS,
          value: "{agent_adapter}",
          onchange: move |e| agent_adapter.set(e.value()),
          for (key , label) in ADAPTER_LABELS {
            option { value: *key, "{label}" }
          }
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
