mod inbox_rows;
mod inbox_state;

use self::inbox_rows::{ApprovalRow, FailedRunRow, JoinRequestRow};
use self::inbox_state::{InboxApprovalItem, InboxCategoryFilter, InboxFailedRun, InboxIssueItem, InboxJoinRequest, InboxTab};
use dioxus::prelude::*;

#[component]
fn InboxTabBar(active: InboxTab, on_change: EventHandler<InboxTab>) -> Element {
  rsx! {
    div { class: "flex border-b border-[var(--outline-variant)]",
      for tab in InboxTab::all() {
        {
            let t = *tab;
            let is_active = t == active;
            let cls = if is_active {
                "px-4 py-2 text-xs font-semibold uppercase tracking-wider border-b-2 border-[var(--primary)] text-[var(--on-surface)]"
            } else {
                "px-4 py-2 text-xs uppercase tracking-wider text-[var(--outline)] hover:text-[var(--on-surface)] cursor-pointer"
            };
            rsx! {
              button { class: cls, onclick: move |_| on_change.call(t), "{tab.label()}" }
            }
        }
      }
    }
  }
}

#[component]
fn CategoryFilterSelect(value: InboxCategoryFilter, on_change: EventHandler<InboxCategoryFilter>) -> Element {
  rsx! {
    select {
      class: "bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded px-2 py-1 text-xs text-[var(--on-surface)]",
      value: "{value.label()}",
      onchange: move |evt| {
          let selected = match evt.value().as_str() {
              "Everything" => InboxCategoryFilter::Everything,
              "Issues I Touched" => InboxCategoryFilter::IssuesTouched,
              "Join Requests" => InboxCategoryFilter::JoinRequests,
              "Approvals" => InboxCategoryFilter::Approvals,
              "Failed Runs" => InboxCategoryFilter::FailedRuns,
              "Alerts" => InboxCategoryFilter::Alerts,
              _ => InboxCategoryFilter::Everything,
          };
          on_change.call(selected);
      },
      for filter in InboxCategoryFilter::all() {
        option { value: "{filter.label()}", "{filter.label()}" }
      }
    }
  }
}

#[component]
pub fn Inbox() -> Element {
  let mut active_tab = use_signal(|| InboxTab::Mine);
  let mut category_filter = use_signal(|| InboxCategoryFilter::Everything);
  let action_error: Signal<Option<String>> = use_signal(|| None);

  let demo_approvals: Vec<InboxApprovalItem> = vec![];
  let demo_failed_runs: Vec<InboxFailedRun> = vec![];
  let demo_join_requests: Vec<InboxJoinRequest> = vec![];
  let demo_issues: Vec<InboxIssueItem> = vec![];

  let tab = active_tab();
  let filter = category_filter();

  let show_approvals = filter == InboxCategoryFilter::Everything || filter == InboxCategoryFilter::Approvals;
  let show_failed_runs = filter == InboxCategoryFilter::Everything || filter == InboxCategoryFilter::FailedRuns;
  let show_join_requests = filter == InboxCategoryFilter::Everything || filter == InboxCategoryFilter::JoinRequests;
  let show_issues = filter == InboxCategoryFilter::Everything || filter == InboxCategoryFilter::IssuesTouched;

  let is_empty = demo_approvals.is_empty() && demo_failed_runs.is_empty() && demo_join_requests.is_empty() && demo_issues.is_empty();

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "flex items-center gap-2 px-4 py-3",
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
          "inbox"
        }
        h1 { class: "text-lg font-semibold text-[var(--on-surface)]", "Inbox" }
      }
      InboxTabBar { active: tab, on_change: move |t| active_tab.set(t) }
      if tab == InboxTab::All {
        div { class: "px-4 py-2 border-b border-[var(--outline-variant)]",
          CategoryFilterSelect {
            value: filter,
            on_change: move |f| category_filter.set(f),
          }
        }
      }
      if let Some(ref err) = action_error() {
        div { class: "mx-4 mt-2 rounded-md border border-red-500/40 bg-red-500/5 px-3 py-2 text-sm text-red-500",
          "{err}"
        }
      }
      div { class: "flex-1 overflow-auto",
        if is_empty {
          div { class: "flex flex-col items-center justify-center py-16 text-[var(--outline)]",
            span { class: "material-symbols-outlined text-xl mb-4", "inbox" }
            p { class: "text-sm", "Your inbox is empty." }
          }
        } else {
          if show_issues && !demo_issues.is_empty() {
            div { class: "border-b border-[var(--outline-variant)]",
              div { class: "px-4 py-2 text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                "Issues"
              }
              for issue in demo_issues.iter() {
                div { class: "px-4 py-2 border-b border-[var(--outline-variant)]/30 text-sm",
                  div { class: "flex items-center gap-2",
                    if let Some(ref ident) = issue.identifier {
                      span { class: "font-mono text-[var(--outline)]",
                        "{ident}"
                      }
                    }
                    span { class: "font-medium", "{issue.title}" }
                    span { class: "ml-auto text-xs text-[var(--outline)]",
                      "{issue.status}"
                    }
                  }
                }
              }
            }
          }
          if show_approvals && !demo_approvals.is_empty() {
            div { class: "border-b border-[var(--outline-variant)]",
              div { class: "px-4 py-2 text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                "Approvals"
              }
              for approval in demo_approvals.iter() {
                ApprovalRow {
                  key: "{approval.id}",
                  approval: approval.clone(),
                  on_approve: move |id: String| {
                      let _ = &id;
                  },
                  on_reject: move |id: String| {
                      let _ = &id;
                  },
                  is_pending: false,
                }
              }
            }
          }
          if show_failed_runs && !demo_failed_runs.is_empty() {
            div { class: "border-b border-[var(--outline-variant)]",
              div { class: "px-4 py-2 text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                "Failed Runs"
              }
              for run in demo_failed_runs.iter() {
                FailedRunRow {
                  key: "{run.id}",
                  run: run.clone(),
                  on_dismiss: move |id: String| {
                      let _ = &id;
                  },
                  on_retry: move |id: String| {
                      let _ = &id;
                  },
                  is_retrying: false,
                }
              }
            }
          }
          if show_join_requests && !demo_join_requests.is_empty() {
            div {
              div { class: "px-4 py-2 text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                "Join Requests"
              }
              for jr in demo_join_requests.iter() {
                JoinRequestRow {
                  key: "{jr.id}",
                  join_request: jr.clone(),
                  on_approve: move |id: String| {
                      let _ = &id;
                  },
                  on_reject: move |id: String| {
                      let _ = &id;
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
