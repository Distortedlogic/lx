use dioxus::prelude::*;

use super::card::ApprovalCard;
use super::types::{Approval, ApprovalPayload};

pub fn default_approvals() -> Vec<Approval> {
  vec![
    Approval {
      id: "apr-001".into(),
      approval_type: "hire_agent".into(),
      status: "pending".into(),
      requested_by: Some("CTO Agent".into()),
      payload: ApprovalPayload {
        name: Some("Designer".into()),
        role: Some("UI/UX Designer".into()),
        title: Some("Senior Product Designer".into()),
        description: None,
        amount: None,
      },
      decision_note: None,
      created_at: "2026-03-28T10:00:00Z".into(),
    },
    Approval {
      id: "apr-002".into(),
      approval_type: "approve_ceo_strategy".into(),
      status: "approved".into(),
      requested_by: Some("CEO Agent".into()),
      payload: ApprovalPayload {
        name: None,
        role: None,
        title: Some("Q2 Growth Strategy".into()),
        description: Some("Expand into European markets with localized offerings.".into()),
        amount: None,
      },
      decision_note: Some("Approved by board consensus.".into()),
      created_at: "2026-03-27T14:30:00Z".into(),
    },
    Approval {
      id: "apr-003".into(),
      approval_type: "budget_override_required".into(),
      status: "pending".into(),
      requested_by: Some("Finance Agent".into()),
      payload: ApprovalPayload {
        name: Some("Code Review Bot".into()),
        role: None,
        title: None,
        description: Some("Agent exceeded $20.00 budget limit.".into()),
        amount: Some(2500),
      },
      decision_note: None,
      created_at: "2026-03-28T08:15:00Z".into(),
    },
  ]
}

#[component]
pub fn Approvals() -> Element {
  let approvals = dioxus_storage::use_persistent("lx_approvals", default_approvals);
  let mut status_filter = use_signal(|| "pending");
  let mut action_error: Signal<Option<String>> = use_signal(|| None);

  let all = approvals();
  let pending_count = all.iter().filter(|a| a.status == "pending" || a.status == "revision_requested").count();

  let mut filtered: Vec<Approval> = if status_filter() == "pending" {
    all.iter().filter(|a| a.status == "pending" || a.status == "revision_requested").cloned().collect()
  } else {
    all.clone()
  };
  filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex-between",
        h1 { class: "page-heading", "APPROVALS" }
      }
      div { class: "flex gap-2",
        button {
          class: if status_filter() == "pending" { "px-4 py-2 text-xs font-semibold uppercase tracking-wider rounded bg-[var(--primary)] text-[var(--on-primary)] flex items-center gap-1.5" } else { "px-4 py-2 text-xs font-semibold uppercase tracking-wider rounded text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container)] flex items-center gap-1.5" },
          onclick: move |_| status_filter.set("pending"),
          "PENDING"
          if pending_count > 0 {
            span { class: "ml-1 rounded-full px-1.5 py-0.5 text-[10px] font-medium bg-yellow-500/20 text-yellow-500",
              "{pending_count}"
            }
          }
        }
        button {
          class: if status_filter() == "all" { "px-4 py-2 text-xs font-semibold uppercase tracking-wider rounded bg-[var(--primary)] text-[var(--on-primary)]" } else { "px-4 py-2 text-xs font-semibold uppercase tracking-wider rounded text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container)]" },
          onclick: move |_| status_filter.set("all"),
          "ALL"
        }
      }
      if let Some(err) = action_error() {
        p { class: "text-sm text-red-500", "{err}" }
      }
      if filtered.is_empty() {
        div { class: "flex flex-col items-center justify-center py-16 text-center",
          span { class: "material-symbols-outlined text-xl text-[var(--outline)]/30 mb-3",
            "verified_user"
          }
          p { class: "text-sm text-[var(--outline)]",
            if status_filter() == "pending" {
              "No pending approvals."
            } else {
              "No approvals yet."
            }
          }
        }
      }
      if !filtered.is_empty() {
        div { class: "grid gap-3",
          for approval in filtered.iter() {
            {
                let approve_id = approval.id.clone();
                let reject_id = approval.id.clone();
                rsx! {
                  ApprovalCard {
                    key: "{approval.id}",
                    approval: approval.clone(),
                    on_approve: move |_| {
                        let mut a = approvals;
                        let mut all = a();
                        if let Some(ap) = all.iter_mut().find(|x| x.id == approve_id) {
                            ap.status = "approved".into();
                        }
                        a.set(all);
                        action_error.set(None);
                    },
                    on_reject: move |_| {
                        let mut a = approvals;
                        let mut all = a();
                        if let Some(ap) = all.iter_mut().find(|x| x.id == reject_id) {
                            ap.status = "rejected".into();
                        }
                        a.set(all);
                        action_error.set(None);
                    },
                    is_pending: false,
                  }
                }
            }
          }
        }
      }
    }
  }
}
