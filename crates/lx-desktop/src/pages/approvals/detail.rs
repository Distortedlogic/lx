use std::collections::HashMap;

use dioxus::prelude::*;

use super::approval_actions::ApprovalActions;
use super::list::default_approvals;
use super::payload::PayloadRenderer;
use super::types::{ApprovalComment, approval_type_icon, approval_type_label};
use crate::components::page_skeleton::PageSkeleton;

#[component]
pub fn ApprovalDetail(approval_id: String) -> Element {
  rsx! {
    SuspenseBoundary {
      fallback: |_| rsx! {
        PageSkeleton { variant: "detail".to_string() }
      },
      ApprovalDetailInner { approval_id }
    }
  }
}

#[component]
fn ApprovalDetailInner(approval_id: String) -> Element {
  let approvals = dioxus_storage::use_persistent("lx_approvals", default_approvals);
  let comments_store = dioxus_storage::use_persistent("lx_approval_comments", HashMap::<String, Vec<ApprovalComment>>::new);
  let mut comment_body = use_signal(String::new);
  let mut error: Signal<Option<String>> = use_signal(|| None);
  let mut show_raw = use_signal(|| false);

  let all = approvals();
  let approval = all.iter().find(|a| a.id == approval_id);

  let Some(approval) = approval.cloned() else {
    return rsx! {
      div { class: "p-4 text-sm text-[var(--outline)]", "Approval not found" }
    };
  };

  let icon = approval_type_icon(&approval.approval_type);
  let label = approval_type_label(&approval.approval_type);
  let id_short = if approval.id.len() > 8 { &approval.id[..8] } else { &approval.id };

  let (status_icon, status_color, status_text) = match approval.status.as_str() {
    "approved" => ("check_circle", "text-green-500", "Approved"),
    "rejected" => ("cancel", "text-red-500", "Rejected"),
    "revision_requested" => ("schedule", "text-amber-500", "Revision Requested"),
    _ => ("schedule", "text-yellow-500", "Pending"),
  };

  let requester = approval.requested_by.as_deref().unwrap_or("Unknown");

  let all_comments = comments_store();
  let approval_comments: Vec<ApprovalComment> = all_comments.get(&approval_id).cloned().unwrap_or_default();
  let comment_count = approval_comments.len();

  let payload_debug = format!("{:#?}", approval.payload);
  let aid_approve = approval_id.clone();
  let aid_reject = approval_id.clone();
  let aid_revision = approval_id.clone();
  let aid_resubmit = approval_id.clone();
  let aid_comment = approval_id.clone();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-6 max-w-3xl",
      div { class: "border border-[var(--outline-variant)] rounded-lg p-4 space-y-3",
        div { class: "flex items-center justify-between",
          div { class: "flex items-center gap-2",
            span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
              "{icon}"
            }
            div {
              p { class: "text-lg font-semibold text-[var(--on-surface)]",
                "{label}"
              }
              p { class: "text-xs font-mono text-[var(--outline)]",
                "{id_short}"
              }
            }
          }
          div { class: "flex items-center gap-1.5",
            span {
              class: "material-symbols-outlined text-sm",
              class: "{status_color}",
              "{status_icon}"
            }
            span {
              class: "text-xs font-semibold",
              class: "{status_color}",
              "{status_text}"
            }
          }
        }
        p { class: "text-xs text-[var(--outline)]", "Requested by {requester}" }
        PayloadRenderer {
          approval_type: approval.approval_type.clone(),
          payload: approval.payload.clone(),
        }
        button {
          class: "flex items-center gap-1 text-xs text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
          onclick: move |_| show_raw.set(!show_raw()),
          span { class: "material-symbols-outlined text-sm",
            if show_raw() {
              "expand_more"
            } else {
              "chevron_right"
            }
          }
          "See full request"
        }
        if show_raw() {
          pre { class: "text-xs bg-[var(--surface-container)] rounded-md p-3 overflow-x-auto font-mono text-[var(--outline)]",
            "{payload_debug}"
          }
        }
        if let Some(note) = &approval.decision_note {
          p { class: "text-xs italic text-[var(--outline)]", "{note}" }
        }
        if let Some(err) = error() {
          p { class: "text-sm text-red-500", "{err}" }
        }
        ApprovalActions {
          approval_type: approval.approval_type.clone(),
          status: approval.status.clone(),
          on_approve: move |_| {
              let mut a = approvals;
              let mut all = a();
              if let Some(ap) = all.iter_mut().find(|x| x.id == aid_approve) {
                  ap.status = "approved".into();
              }
              a.set(all);
              error.set(None);
          },
          on_reject: move |_| {
              let mut a = approvals;
              let mut all = a();
              if let Some(ap) = all.iter_mut().find(|x| x.id == aid_reject) {
                  ap.status = "rejected".into();
              }
              a.set(all);
              error.set(None);
          },
          on_request_revision: move |_| {
              let mut a = approvals;
              let mut all = a();
              if let Some(ap) = all.iter_mut().find(|x| x.id == aid_revision) {
                  ap.status = "revision_requested".into();
              }
              a.set(all);
              error.set(None);
          },
          on_resubmit: move |_| {
              let mut a = approvals;
              let mut all = a();
              if let Some(ap) = all.iter_mut().find(|x| x.id == aid_resubmit) {
                  ap.status = "pending".into();
              }
              a.set(all);
              error.set(None);
          },
        }
      }
      div { class: "border border-[var(--outline-variant)] rounded-lg p-4 space-y-3",
        h3 { class: "text-sm font-semibold text-[var(--on-surface)] uppercase tracking-wider",
          "COMMENTS ({comment_count})"
        }
        div { class: "space-y-2",
          for comment in approval_comments.iter() {
            div { class: "border border-[var(--outline-variant)]/60 rounded-md p-3",
              div { class: "flex items-center justify-between mb-1",
                span { class: "text-xs font-semibold text-[var(--on-surface)]",
                  "{comment.author.as_deref().unwrap_or(\"Board\")}"
                }
                span { class: "text-xs text-[var(--outline)]", "{comment.created_at}" }
              }
              p { class: "text-sm text-[var(--on-surface-variant)]",
                "{comment.body}"
              }
            }
          }
        }
        textarea {
          class: "w-full bg-[var(--surface-container-lowest)] border border-[var(--outline-variant)] text-sm px-3 py-2 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)] resize-y min-h-[4rem]",
          placeholder: "Add a comment...",
          value: "{comment_body}",
          oninput: move |evt| comment_body.set(evt.value()),
        }
        div { class: "flex justify-end",
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold rounded disabled:opacity-40",
            disabled: comment_body().trim().is_empty(),
            onclick: move |_| {
                let body = comment_body().trim().to_string();
                if body.is_empty() {
                    return;
                }
                let new_comment = ApprovalComment {
                    id: uuid::Uuid::new_v4().to_string(),
                    body,
                    author: Some("Board".into()),
                    created_at: "2026-03-28T12:00:00Z".into(),
                };
                let mut store = comments_store;
                let mut all = store();
                all.entry(aid_comment.clone()).or_default().push(new_comment);
                store.set(all);
                comment_body.set(String::new());
            },
            "POST COMMENT"
          }
        }
      }
    }
  }
}
