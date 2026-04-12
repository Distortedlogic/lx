use super::types::{AgentDetail, adapter_label, role_label};
use dioxus::prelude::*;

#[component]
pub fn AgentOverview(agent: AgentDetail) -> Element {
  rsx! {
    div { class: "space-y-8",
      AgentPropertiesPanel { agent: agent.clone() }
      div { class: "space-y-3",
        h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Latest Run" }
        p { class: "text-sm text-[var(--outline)]", "No runs yet." }
      }
      div { class: "space-y-3",
        h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Costs" }
        CostsGrid {
          budget_cents: agent.budget_monthly_cents,
          spent_cents: agent.spent_monthly_cents,
        }
      }
    }
  }
}

#[component]
fn AgentPropertiesPanel(agent: AgentDetail) -> Element {
  rsx! {
    div { class: "space-y-4",
      div { class: "space-y-1",
        PropertyRow { label: "Status",
          span { class: "text-sm text-[var(--on-surface)]", "{agent.status}" }
        }
        PropertyRow { label: "Role",
          span { class: "text-sm text-[var(--on-surface)]", "{role_label(&agent.role)}" }
        }
        if let Some(title) = &agent.title {
          PropertyRow { label: "Title",
            span { class: "text-sm text-[var(--on-surface)]", "{title}" }
          }
        }
        PropertyRow { label: "Adapter",
          span { class: "text-sm font-mono text-[var(--on-surface)]",
            "{adapter_label(&agent.adapter_type)}"
          }
        }
        if let Some(hb) = &agent.last_heartbeat_at {
          PropertyRow { label: "Heartbeat",
            span { class: "text-sm text-[var(--on-surface)]", "{hb}" }
          }
        }
        PropertyRow { label: "Created",
          span { class: "text-sm text-[var(--on-surface)]", "{agent.created_at}" }
        }
      }
    }
  }
}

#[component]
fn PropertyRow(label: &'static str, children: Element) -> Element {
  rsx! {
    div { class: "flex items-center gap-3 py-1.5",
      span { class: "property-label", "{label}" }
      div { class: "flex items-center gap-1.5 min-w-0", {children} }
    }
  }
}

#[component]
fn CostsGrid(budget_cents: i64, spent_cents: i64) -> Element {
  let budget_str = format_cents(budget_cents);
  let spent_str = format_cents(spent_cents);
  let pct = if budget_cents > 0 { format!("{}%", (spent_cents as f64 / budget_cents as f64 * 100.0) as i64) } else { "No cap".to_string() };

  rsx! {
    div { class: "border border-[var(--outline-variant)]/30 rounded-lg p-4",
      div { class: "grid grid-cols-2 gap-4",
        div {
          span { class: "text-xs text-[var(--outline)] block", "Spent" }
          span { class: "text-lg font-semibold text-[var(--on-surface)]", "{spent_str}" }
          span { class: "text-xs text-[var(--outline)] block", "{pct} of limit" }
        }
        div {
          span { class: "text-xs text-[var(--outline)] block", "Budget" }
          span { class: "text-lg font-semibold text-[var(--on-surface)]", "{budget_str}" }
        }
      }
    }
  }
}

fn format_cents(cents: i64) -> String {
  if cents == 0 {
    return "Disabled".to_string();
  }
  format!("${:.2}", cents as f64 / 100.0)
}
