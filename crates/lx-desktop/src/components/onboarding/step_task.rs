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
            "Give it something to do"
          }
          p { class: "text-xs text-[var(--outline)]",
            "Give your agent a small task to start with."
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Task title" }
        input {
          class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]",
          placeholder: "e.g. Research competitor pricing",
          value: "{task_title}",
          oninput: move |e| task_title.set(e.value()),
          autofocus: true,
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Description (optional)" }
        textarea {
          class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)] resize-none min-h-[120px]",
          placeholder: "Add more detail about what the agent should do...",
          value: "{task_description}",
          oninput: move |e| task_description.set(e.value()),
        }
      }
    }
  }
}
