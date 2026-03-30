# UNIT 11: Kanban Board Drag-and-Drop

## Goal

Add HTML5 drag-and-drop to the KanbanBoard so users can drag cards between status columns.
Currently cards are click-only buttons. After this work, dragging a card to a different column
calls `on_status_change` with `(issue_id, new_status)`.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/pages/issues/kanban.rs` | Rewrite |

## Current State

`kanban.rs` (100 lines) has three components:

- `KanbanBoardView` -- renders columns, passes `on_status_change` prop but never wires it to anything
- `KanbanColumn` -- renders header + card list, does NOT accept `on_status_change`
- `KanbanCard` -- a `button` element with `onclick` only

## Step 1: Replace `crates/lx-desktop/src/pages/issues/kanban.rs` entirely

Replace the full file content with the following:

```rust
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
  let mut dragging_issue_id = use_signal(|| Option::<String>::None);
  let mut drag_over_column = use_signal(|| Option::<String>::None);

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
          on_status_change,
          dragging_issue_id,
          drag_over_column,
        }
      }
    }
  }
}

#[component]
fn KanbanColumn(
  status: String,
  issues: Vec<Issue>,
  agents: Vec<AgentRef>,
  on_select: EventHandler<String>,
  on_status_change: EventHandler<(String, String)>,
  dragging_issue_id: Signal<Option<String>>,
  drag_over_column: Signal<Option<String>>,
) -> Element {
  let label = status_label(&status);
  let count = issues.len();
  let is_drag_over = drag_over_column.read().as_deref() == Some(status.as_str());
  let status_drop = status.clone();
  let status_over = status.clone();
  let status_leave = status.clone();

  let drop_highlight = if is_drag_over {
    "flex-1 min-h-[120px] rounded-md p-1 space-y-1 bg-[var(--primary)]/10 ring-1 ring-[var(--primary)]/40 transition-colors"
  } else {
    "flex-1 min-h-[120px] rounded-md p-1 space-y-1 bg-[var(--surface-container)]/20 transition-colors"
  };

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
      div {
        class: "{drop_highlight}",
        ondragover: move |evt| {
            evt.prevent_default();
            drag_over_column.set(Some(status_over.clone()));
        },
        ondragleave: move |_| {
            if drag_over_column.read().as_deref() == Some(status_leave.as_str()) {
                drag_over_column.set(None);
            }
        },
        ondrop: move |evt| {
            evt.prevent_default();
            drag_over_column.set(None);
            if let Some(issue_id) = dragging_issue_id.read().clone() {
                on_status_change.call((issue_id, status_drop.clone()));
            }
            dragging_issue_id.set(None);
        },
        for issue in issues.iter() {
          KanbanCard {
            issue: issue.clone(),
            agents: agents.clone(),
            dragging_issue_id,
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
fn KanbanCard(
  issue: Issue,
  agents: Vec<AgentRef>,
  dragging_issue_id: Signal<Option<String>>,
  on_click: EventHandler<()>,
) -> Element {
  let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
  let assignee_name = issue
    .assignee_agent_id
    .as_ref()
    .and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()));
  let is_dragging = dragging_issue_id.read().as_deref() == Some(issue.id.as_str());
  let card_cls = if is_dragging {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left opacity-30 transition-opacity cursor-grabbing"
  } else {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left hover:shadow-sm transition-shadow cursor-grab"
  };
  let drag_id = issue.id.clone();

  rsx! {
    div {
      class: "{card_cls}",
      draggable: "true",
      ondragstart: move |_| {
          dragging_issue_id.set(Some(drag_id.clone()));
      },
      ondragend: move |_| {
          dragging_issue_id.set(None);
      },
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
```

## Behavior Summary

- `dragging_issue_id: Signal<Option<String>>` -- set on `ondragstart`, cleared on `ondragend` and `ondrop`
- `drag_over_column: Signal<Option<String>>` -- set on `ondragover`, cleared on `ondragleave` and `ondrop`
- Source card gets `opacity-30` class while dragging
- Target column gets `bg-[var(--primary)]/10 ring-1 ring-[var(--primary)]/40` highlight
- `ondragover` calls `evt.prevent_default()` (required for HTML5 drop to work)
- `ondrop` reads `dragging_issue_id`, calls `on_status_change.call((issue_id, new_status))`
- Card element changed from `button` to `div` with `draggable: "true"` (buttons have conflicting drag behavior)
- Card keeps `onclick` for navigation; HTML5 drag only fires on actual drag movement, not click

## Verification

Run `just diagnose` and confirm no compiler errors in `crates/lx-desktop`.
