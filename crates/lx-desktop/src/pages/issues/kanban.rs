use dioxus::prelude::*;

use super::types::{AgentRef, Issue, priority_icon_class, status_icon_class, status_label};

const BOARD_STATUSES: &[&str] = &["backlog", "todo", "in_progress", "in_review", "blocked", "done", "cancelled"];

#[component]
pub fn KanbanBoardView(
  issues: Vec<Issue>,
  agents: Vec<AgentRef>,
  on_select: EventHandler<String>,
  on_status_change: EventHandler<(String, String)>,
) -> Element {
  let columns: Vec<(&str, Vec<&Issue>)> = BOARD_STATUSES
    .iter()
    .map(|status| {
      let col_issues: Vec<&Issue> = issues.iter().filter(|i| &i.status == status).collect();
      (*status, col_issues)
    })
    .collect();

  rsx! {
    div { class: "flex gap-3 overflow-x-auto pb-4 -mx-2 px-2",
      for (status , col_issues) in columns.iter() {
        KanbanColumn {
          status: status.to_string(),
          issues: col_issues.iter().map(|i| (*i).clone()).collect(),
          agents: agents.clone(),
          on_select,
        }
      }
    }
  }
}

#[component]
fn KanbanColumn(status: String, issues: Vec<Issue>, agents: Vec<AgentRef>, on_select: EventHandler<String>) -> Element {
  let label = status_label(&status);
  let count = issues.len();

  rsx! {
    div { class: "flex flex-col min-w-[260px] w-[260px] shrink-0",
      div { class: "flex items-center gap-2 px-2 py-2 mb-1",
        span { class: "material-symbols-outlined text-sm {status_icon_class(&status)}",
          "circle"
        }
        span { class: "text-xs font-semibold uppercase tracking-wide text-[var(--outline)]",
          "{label}"
        }
        span { class: "text-xs text-[var(--outline)]/60 ml-auto tabular-nums",
          "{count}"
        }
      }
      div { class: "flex-1 min-h-[120px] rounded-md p-1 space-y-1 bg-[var(--surface-container)]/20",
        for issue in issues.iter() {
          KanbanCard {
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
}

#[component]
fn KanbanCard(issue: Issue, agents: Vec<AgentRef>, on_click: EventHandler<()>) -> Element {
  let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
  let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()));

  rsx! {
    button {
      class: "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left hover:shadow-sm transition-shadow",
      onclick: move |_| on_click.call(()),
      div { class: "flex items-start gap-1.5 mb-1.5",
        span { class: "text-xs text-[var(--outline)] font-mono shrink-0", "{id_display}" }
      }
      p { class: "text-sm leading-snug text-[var(--on-surface)] line-clamp-2 mb-2",
        "{issue.title}"
      }
      div { class: "flex items-center gap-2",
        span { class: "material-symbols-outlined text-xs {priority_icon_class(&issue.priority)}",
          match issue.priority.as_str() {
              "critical" => "priority_high",
              "high" => "arrow_upward",
              "low" => "arrow_downward",
              _ => "remove",
          }
        }
        if let Some(name) = assignee_name {
          span { class: "text-xs text-[var(--outline)]", "{name}" }
        }
      }
    }
  }
}
