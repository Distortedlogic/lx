use dioxus::prelude::*;

use crate::routes::Route;

use super::payload::PayloadRenderer;
use super::types::{Approval, approval_type_icon, approval_type_label};

#[component]
pub fn ApprovalCard(approval: Approval, on_approve: EventHandler<()>, on_reject: EventHandler<()>, is_pending: bool) -> Element {
  let icon = approval_type_icon(&approval.approval_type);
  let label = approval_type_label(&approval.approval_type);
  let context_name = approval.payload.name.as_ref().map(|n| format!(": {n}")).unwrap_or_default();

  let (status_icon, status_color, status_text) = match approval.status.as_str() {
    "approved" => ("check_circle", "text-green-500", "Approved"),
    "rejected" => ("cancel", "text-red-500", "Rejected"),
    "revision_requested" => ("schedule", "text-amber-500", "Revision Requested"),
    _ => ("schedule", "text-yellow-500", "Pending"),
  };

  let is_actionable = (approval.status == "pending" || approval.status == "revision_requested") && approval.approval_type != "budget_override_required";

  let detail_id = approval.id.clone();

  rsx! {
    div { class: "border border-[var(--outline-variant)] rounded-lg p-4 space-y-3",
      div { class: "flex items-center justify-between",
        div { class: "flex items-center gap-2",
          span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
            "{icon}"
          }
          span { class: "text-sm font-semibold text-[var(--on-surface)]",
            "{label}{context_name}"
          }
        }
        div { class: "flex items-center gap-1.5",
          span { class: "material-symbols-outlined text-sm {status_color}",
            "{status_icon}"
          }
          span { class: "text-xs {status_color}", "{status_text}" }
          span { class: "text-xs text-[var(--outline)] ml-2", "{approval.created_at}" }
        }
      }
      PayloadRenderer {
        approval_type: approval.approval_type.clone(),
        payload: approval.payload.clone(),
      }
      if let Some(note) = &approval.decision_note {
        p { class: "text-xs italic text-[var(--outline)]", "{note}" }
      }
      if is_actionable {
        div { class: "flex items-center gap-2 pt-1",
          button {
            class: "bg-green-700 hover:bg-green-600 text-white px-4 py-1.5 text-xs uppercase font-semibold rounded disabled:opacity-40",
            disabled: is_pending,
            onclick: move |_| on_approve.call(()),
            "APPROVE"
          }
          button {
            class: "bg-red-700 hover:bg-red-600 text-white px-4 py-1.5 text-xs uppercase font-semibold rounded disabled:opacity-40",
            disabled: is_pending,
            onclick: move |_| on_reject.call(()),
            "REJECT"
          }
        }
      }
      Link {
        to: Route::ApprovalDetail {
            approval_id: detail_id,
        },
        class: "text-xs text-[var(--primary)] hover:underline uppercase tracking-wider font-semibold",
        "VIEW DETAILS"
      }
    }
  }
}
