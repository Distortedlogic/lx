use dioxus::prelude::*;

#[component]
pub fn StepTask(task_title: Signal<String>, task_description: Signal<String>) -> Element {
  rsx! {
    div { class: "space-y-5",
      div { class: "flex items-center gap-3 mb-1",
        div { class: "bg-[var(--surface-container-highest)] p-2",
          span { class: "material-symbols-outlined text-xl text-[var(--outline)]",
            "checklist"
          }
        }
        div {
          h3 { class: "text-sm font-medium text-[var(--on-surface)]",
            "Define your first flow"
          }
          p { class: "text-xs text-[var(--outline)]",
            "An lx flow orchestrates agents, channels, and tools to complete work."
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Flow name" }
        input {
          class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]",
          placeholder: "e.g. analyze-codebase, draft-proposal, review-pr",
          value: "{task_title}",
          oninput: move |e| task_title.set(e.value()),
          autofocus: true,
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Flow description (optional)" }
        textarea {
          class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)] resize-none min-h-[120px]",
          placeholder: "Describe what this flow should accomplish. The agent will use this to plan its steps.",
          value: "{task_description}",
          oninput: move |e| task_description.set(e.value()),
        }
      }
    }
  }
}
