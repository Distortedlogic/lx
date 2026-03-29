use dioxus::prelude::*;

use super::types::ApprovalPayload;
use crate::pages::costs::types::format_cents;

#[component]
pub fn PayloadRenderer(approval_type: String, payload: ApprovalPayload) -> Element {
  match approval_type.as_str() {
    "hire_agent" => rsx! {
      div { class: "space-y-1",
        PayloadField { label: "Name", value: payload.name }
        PayloadField { label: "Role", value: payload.role }
        PayloadField { label: "Title", value: payload.title }
      }
    },
    "budget_override_required" => rsx! {
      div { class: "space-y-1",
        PayloadField { label: "Scope", value: payload.name }
        PayloadField { label: "Amount", value: payload.amount.map(format_cents) }
        if let Some(desc) = &payload.description {
          pre { class: "text-xs text-[var(--outline)] bg-[var(--surface-container)] rounded p-2 mt-2 overflow-x-auto font-mono",
            "{desc}"
          }
        }
      }
    },
    _ => rsx! {
      div { class: "space-y-1",
        PayloadField { label: "Title", value: payload.title }
        if let Some(desc) = &payload.description {
          pre { class: "text-xs text-[var(--outline)] bg-[var(--surface-container)] rounded p-2 mt-2 overflow-x-auto whitespace-pre-wrap",
            "{desc}"
          }
        }
      }
    },
  }
}

#[component]
fn PayloadField(label: &'static str, value: Option<String>) -> Element {
  match value {
    None => rsx! {},
    Some(v) => rsx! {
      div { class: "flex items-center gap-2 text-xs",
        span { class: "w-20 text-[var(--outline)] shrink-0", "{label}" }
        span { class: "text-[var(--on-surface)]", "{v}" }
      }
    },
  }
}
