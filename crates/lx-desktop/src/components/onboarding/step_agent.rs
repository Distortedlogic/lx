use dioxus::prelude::*;

#[component]
pub fn StepAgent(agent_name: Signal<String>) -> Element {
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
            "Name the agent that will handle your first task."
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Agent name" }
        input {
          class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]",
          placeholder: "CEO",
          value: "{agent_name}",
          oninput: move |e| agent_name.set(e.value()),
          autofocus: true,
        }
      }
    }
  }
}
