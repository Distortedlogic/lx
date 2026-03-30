use dioxus::prelude::*;

#[component]
pub fn StepLaunch(company_name: String, agent_name: String, agent_role: String, agent_adapter: String, task_title: String) -> Element {
  rsx! {
    div { class: "space-y-5",
      div { class: "flex items-center gap-3 mb-1",
        div { class: "bg-[var(--surface-container-highest)] p-2",
          span { class: "material-symbols-outlined text-xl text-[var(--outline)]",
            "rocket_launch"
          }
        }
        div {
          h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Ready to launch" }
          p { class: "text-xs text-[var(--outline)]",
            "Everything is set up. Launch will create the task and open it."
          }
        }
      }
      div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)]",
        SummaryRow {
          icon: "apartment",
          label: "Company",
          value: company_name,
        }
        SummaryRow { icon: "smart_toy", label: "Agent", value: agent_name }
        SummaryRow { icon: "badge", label: "Role", value: agent_role }
        SummaryRow { icon: "memory", label: "Adapter", value: agent_adapter }
        SummaryRow { icon: "checklist", label: "Task", value: task_title }
      }
    }
  }
}

#[component]
fn SummaryRow(icon: &'static str, label: &'static str, value: String) -> Element {
  rsx! {
    div { class: "flex items-center gap-3 px-3 py-2.5",
      span { class: "material-symbols-outlined text-base text-[var(--outline)] shrink-0",
        "{icon}"
      }
      div { class: "flex-1 min-w-0",
        p { class: "text-sm font-medium text-[var(--on-surface)] truncate",
          "{value}"
        }
        p { class: "text-xs text-[var(--outline)]", "{label}" }
      }
      span { class: "material-symbols-outlined text-base text-green-500 shrink-0",
        "check_circle"
      }
    }
  }
}
