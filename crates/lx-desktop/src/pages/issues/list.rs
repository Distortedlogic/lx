use dioxus::prelude::*;

use super::kanban::KanbanBoardView;
use super::types::{AgentRef, Issue, IssueViewMode, IssueViewState, QUICK_FILTER_PRESETS, filter_issues, priority_icon_class, status_icon_class};
use crate::pages::agents::list::StatusBadge;
use crate::styles::{BTN_OUTLINE_SM, FLEX_BETWEEN, INPUT_FIELD, TAB_ACTIVE, TAB_INACTIVE};

#[component]
pub fn IssuesList(
  issues: Vec<Issue>,
  agents: Vec<AgentRef>,
  on_select: EventHandler<String>,
  on_new_issue: EventHandler<()>,
  on_update: EventHandler<(String, String, String)>,
) -> Element {
  let mut view_state = use_signal(IssueViewState::default);
  let filtered = filter_issues(&issues, &view_state.read());

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: FLEX_BETWEEN,
        h1 { class: "text-lg font-semibold text-[var(--on-surface)]", "Issues" }
        div { class: "flex items-center gap-2",
          div { class: "flex items-center border border-[var(--outline-variant)]/30",
            button {
              class: if view_state.read().view_mode == IssueViewMode::List { "p-1.5 bg-[var(--surface-container-high)]" } else { "p-1.5 hover:bg-[var(--surface-container)]" },
              onclick: move |_| view_state.write().view_mode = IssueViewMode::List,
              span { class: "material-symbols-outlined text-sm", "list" }
            }
            button {
              class: if view_state.read().view_mode == IssueViewMode::Board { "p-1.5 bg-[var(--surface-container-high)]" } else { "p-1.5 hover:bg-[var(--surface-container)]" },
              onclick: move |_| view_state.write().view_mode = IssueViewMode::Board,
              span { class: "material-symbols-outlined text-sm", "view_column" }
            }
          }
          button {
            class: BTN_OUTLINE_SM,
            onclick: move |_| on_new_issue.call(()),
            "+ New Issue"
          }
        }
      }
      div { class: "flex gap-1",
        for (label , statuses) in QUICK_FILTER_PRESETS {
          {
              let statuses_vec: Vec<String> = statuses.iter().map(|s| s.to_string()).collect();
              let is_active = view_state.read().statuses == statuses_vec;
              rsx! {
                button {
                  class: if is_active { TAB_ACTIVE } else { TAB_INACTIVE },
                  onclick: {
                      let sv = statuses_vec.clone();
                      move |_| view_state.write().statuses = sv.clone()
                  },
                  "{label}"
                }
              }
          }
        }
      }
      input {
        class: INPUT_FIELD,
        placeholder: "Search issues...",
        value: "{view_state.read().search}",
        oninput: move |evt| view_state.write().search = evt.value().to_string(),
      }
      {
          let suffix = if filtered.len() != 1 { "s" } else { "" };
          let count_text = format!("{} issue{}", filtered.len(), suffix);
          rsx! {
            p { class: "text-xs text-[var(--outline)]", "{count_text}" }
          }
      }
      match view_state.read().view_mode {
          IssueViewMode::List => rsx! {
            IssueListView { issues: filtered.clone(), agents: agents.clone(), on_select }
          },
          IssueViewMode::Board => rsx! {
            KanbanBoardView {
              issues: filtered.clone(),
              agents: agents.clone(),
              on_select,
              on_status_change: move |(id, status): (String, String)| {
                  on_update.call((id, "status".to_string(), status));
              },
            }
          },
      }
    }
  }
}

#[component]
fn IssueListView(issues: Vec<Issue>, agents: Vec<AgentRef>, on_select: EventHandler<String>) -> Element {
  if issues.is_empty() {
    return rsx! {
      div { class: "flex-1 flex items-center justify-center py-8",
        p { class: "text-sm text-[var(--outline)]", "No issues match the current filters." }
      }
    };
  }

  rsx! {
    div { class: "border border-[var(--outline-variant)]/30 overflow-hidden",
      for issue in issues.iter() {
        IssueRow {
          issue: issue.clone(),
          agents: agents.clone(),
          on_click: {
              let id = issue.identifier.clone().unwrap_or_else(|| issue.id.clone());
              move |_| on_select.call(id.clone())
          },
        }
      }
    }
  }
}

#[component]
fn IssueRow(issue: Issue, agents: Vec<AgentRef>, on_click: EventHandler<()>) -> Element {
  let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
  let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()));

  rsx! {
    button {
      class: "flex items-center gap-3 px-3 py-2.5 w-full text-left border-b border-[var(--outline-variant)]/15 hover:bg-[var(--surface-container)] transition-colors",
      onclick: move |_| on_click.call(()),
      span { class: "material-symbols-outlined text-sm {status_icon_class(&issue.status)}",
        match issue.status.as_str() {
            "done" => "check_circle",
            "cancelled" => "cancel",
            "blocked" => "block",
            "in_progress" => "pending",
            "in_review" => "rate_review",
            _ => "circle",
        }
      }
      span { class: "material-symbols-outlined text-sm {priority_icon_class(&issue.priority)}",
        match issue.priority.as_str() {
            "critical" => "priority_high",
            "high" => "arrow_upward",
            "low" => "arrow_downward",
            _ => "remove",
        }
      }
      span { class: "text-xs font-mono text-[var(--outline)] shrink-0 w-16", "{id_display}" }
      span { class: "flex-1 text-sm text-[var(--on-surface)] truncate min-w-0",
        "{issue.title}"
      }
      if let Some(name) = assignee_name {
        span { class: "text-xs text-[var(--outline)] shrink-0", "{name}" }
      }
      StatusBadge { status: issue.status.clone() }
    }
  }
}
