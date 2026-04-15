use super::inbox_state::{InboxApprovalItem, InboxFailedRun, InboxJoinRequest};
use dioxus::prelude::*;

#[component]
pub fn FailedRunRow(run: InboxFailedRun, on_dismiss: EventHandler<String>, on_retry: EventHandler<String>, is_retrying: bool) -> Element {
  let display_error = run.error_message.clone();
  let issue_display = if let (Some(ident), Some(title)) = (&run.issue_identifier, &run.issue_title) {
    rsx! {
      span { class: "font-mono text-[var(--outline)] mr-1.5", "{ident}" }
      "{title}"
    }
  } else {
    let label = match &run.agent_name {
      Some(name) => format!("Failed run - {name}"),
      None => "Failed run".to_string(),
    };
    rsx! { "{label}" }
  };

  let run_id = run.id.clone();
  let run_id2 = run.id.clone();
  rsx! {
    div { class: "group border-b border-[var(--outline-variant)] px-2 py-2.5 last:border-b-0",
      div { class: "flex items-start gap-2",
        div { class: "mt-0.5 shrink-0 rounded-md bg-red-500/20 p-1.5",
          span { class: "material-symbols-outlined text-red-500 text-sm", "cancel" }
        }
        div { class: "min-w-0 flex-1",
          div { class: "text-sm font-medium truncate", {issue_display} }
          div { class: "mt-1 flex flex-wrap items-center gap-x-2 text-xs text-[var(--outline)]",
            span { class: "px-1.5 py-0.5 rounded border border-[var(--outline-variant)] text-[10px]",
              "{run.status}"
            }
            if let Some(ref name) = run.agent_name {
              span { "{name}" }
            }
            span { class: "truncate max-w-[300px]", "{display_error}" }
            span { "{run.created_at}" }
          }
        }
        div { class: "flex shrink-0 items-center gap-2",
          button {
            class: "border border-[var(--outline-variant)] rounded px-2.5 py-1 text-xs hover:bg-[var(--surface-container)]",
            disabled: is_retrying,
            onclick: move |_| on_retry.call(run_id.clone()),
            if is_retrying {
              "Retrying..."
            } else {
              "Retry"
            }
          }
          button {
            class: "rounded-md p-1 text-[var(--outline)] hover:bg-[var(--surface-container)] hover:text-[var(--on-surface)]",
            onclick: move |_| on_dismiss.call(run_id2.clone()),
            span { class: "material-symbols-outlined text-sm", "close" }
          }
        }
      }
    }
  }
}

#[component]
pub fn ApprovalRow(approval: InboxApprovalItem, on_approve: EventHandler<String>, on_reject: EventHandler<String>, is_pending: bool) -> Element {
  let id1 = approval.id.clone();
  let id2 = approval.id.clone();
  let show_buttons = approval.status == super::inbox_state::ApprovalStatus::Pending;
  rsx! {
    div { class: "group border-b border-[var(--outline-variant)] px-2 py-2.5 last:border-b-0",
      div { class: "flex items-start gap-2",
        div { class: "mt-0.5 shrink-0 rounded-md bg-[var(--surface-container)] p-1.5",
          span { class: "material-symbols-outlined text-[var(--outline)] text-sm",
            "approval"
          }
        }
        div { class: "min-w-0 flex-1",
          div { class: "text-sm font-medium truncate", "{approval.approval_type}" }
          div { class: "mt-1 flex flex-wrap items-center gap-x-2 text-xs text-[var(--outline)]",
            span { class: "capitalize", "{approval.status}" }
            if let Some(ref name) = approval.requester_name {
              span { "requested by {name}" }
            }
            span { "updated {approval.updated_at}" }
          }
        }
        if show_buttons {
          div { class: "flex shrink-0 items-center gap-2",
            button {
              class: "bg-green-700 text-white rounded px-3 py-1 text-xs hover:bg-green-600",
              disabled: is_pending,
              onclick: move |_| on_approve.call(id1.clone()),
              "Approve"
            }
            button {
              class: "bg-red-600 text-white rounded px-3 py-1 text-xs hover:bg-red-500",
              disabled: is_pending,
              onclick: move |_| on_reject.call(id2.clone()),
              "Reject"
            }
          }
        }
      }
    }
  }
}

#[component]
pub fn JoinRequestRow(join_request: InboxJoinRequest, on_approve: EventHandler<String>, on_reject: EventHandler<String>, is_pending: bool) -> Element {
  let label = if join_request.request_type == "human" {
    "Human join request".to_string()
  } else {
    match &join_request.agent_name {
      Some(name) => format!("Agent join request: {name}"),
      None => "Agent join request".to_string(),
    }
  };
  let id1 = join_request.id.clone();
  let id2 = join_request.id.clone();
  rsx! {
    div { class: "group border-b border-[var(--outline-variant)] px-2 py-2.5 last:border-b-0",
      div { class: "flex items-start gap-2",
        div { class: "mt-0.5 shrink-0 rounded-md bg-[var(--surface-container)] p-1.5",
          span { class: "material-symbols-outlined text-[var(--outline)] text-sm",
            "person_add"
          }
        }
        div { class: "min-w-0 flex-1",
          div { class: "text-sm font-medium truncate", "{label}" }
          div { class: "mt-1 flex flex-wrap items-center gap-x-2 text-xs text-[var(--outline)]",
            span { "requested {join_request.created_at} from IP {join_request.request_ip}" }
            if let Some(ref adapter) = join_request.adapter_type {
              span { "adapter: {adapter}" }
            }
          }
        }
        div { class: "flex shrink-0 items-center gap-2",
          button {
            class: "bg-green-700 text-white rounded px-3 py-1 text-xs hover:bg-green-600",
            disabled: is_pending,
            onclick: move |_| on_approve.call(id1.clone()),
            "Approve"
          }
          button {
            class: "bg-red-600 text-white rounded px-3 py-1 text-xs hover:bg-red-500",
            disabled: is_pending,
            onclick: move |_| on_reject.call(id2.clone()),
            "Reject"
          }
        }
      }
    }
  }
}
