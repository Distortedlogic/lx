use dioxus::prelude::*;

#[component]
pub fn StepCompany(company_name: Signal<String>, company_goal: Signal<String>) -> Element {
  rsx! {
    div { class: "space-y-5",
      div { class: "flex items-center gap-3 mb-1",
        div { class: "bg-[var(--surface-container-highest)] p-2",
          span { class: "material-symbols-outlined text-xl text-[var(--outline)]",
            "apartment"
          }
        }
        div {
          h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Name your company" }
          p { class: "text-xs text-[var(--outline)]",
            "This is the organization your agents will work for."
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Company name" }
        input {
          class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]",
          placeholder: "Acme Corp",
          value: "{company_name}",
          oninput: move |e| company_name.set(e.value()),
          autofocus: true,
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Mission / goal (optional)" }
        textarea {
          class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)] resize-none min-h-[60px]",
          placeholder: "What is this company trying to achieve?",
          value: "{company_goal}",
          oninput: move |e| company_goal.set(e.value()),
        }
      }
    }
  }
}
