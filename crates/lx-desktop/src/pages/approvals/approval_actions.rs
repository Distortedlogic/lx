use dioxus::prelude::*;

#[component]
pub fn ApprovalActions(
  approval_type: String,
  status: String,
  on_approve: EventHandler<()>,
  on_reject: EventHandler<()>,
  on_request_revision: EventHandler<()>,
  on_resubmit: EventHandler<()>,
) -> Element {
  let is_actionable = status == "pending" || status == "revision_requested";
  let is_budget = approval_type == "budget_override_required";

  if !is_actionable {
    return rsx! {};
  }

  rsx! {
    div { class: "flex flex-wrap items-center gap-2 pt-1",
      if !is_budget {
        button {
          class: "bg-green-700 hover:bg-green-600 text-white px-4 py-1.5 text-xs uppercase font-semibold rounded",
          onclick: move |_| on_approve.call(()),
          "APPROVE"
        }
        button {
          class: "bg-red-700 hover:bg-red-600 text-white px-4 py-1.5 text-xs uppercase font-semibold rounded",
          onclick: move |_| on_reject.call(()),
          "REJECT"
        }
      }
      if is_budget && status == "pending" {
        p { class: "text-sm text-[var(--outline)]",
          "Resolve this budget stop from the budget controls on /costs"
        }
      }
      if status == "pending" {
        button {
          class: "border border-[var(--outline-variant)] text-[var(--on-surface)] px-4 py-1.5 text-xs uppercase font-semibold rounded hover:bg-[var(--surface-container)]",
          onclick: move |_| on_request_revision.call(()),
          "REQUEST REVISION"
        }
      }
      if status == "revision_requested" {
        button {
          class: "border border-[var(--outline-variant)] text-[var(--on-surface)] px-4 py-1.5 text-xs uppercase font-semibold rounded hover:bg-[var(--surface-container)]",
          onclick: move |_| on_resubmit.call(()),
          "MARK RESUBMITTED"
        }
      }
    }
  }
}
