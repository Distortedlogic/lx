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
  let mut drag_active = use_signal(|| false);
  let pointer_start = use_signal(|| (0.0f64, 0.0f64));
  let mut pointer_pos = use_signal(|| (0.0f64, 0.0f64));
  let mut pending_drag_id = use_signal(|| Option::<String>::None);

  let columns: Vec<(&str, Vec<&Issue>)> = BOARD_STATUSES
    .iter()
    .map(|status| {
      let col_issues: Vec<&Issue> = issues.iter().filter(|i| &i.status == status).collect();
      (*status, col_issues)
    })
    .collect();

  rsx! {
    div {
      class: "relative",
      style: if *drag_active.read() { "user-select: none;" } else { "" },
      onmousemove: move |evt| {
          let coords = evt.client_coordinates();
          pointer_pos.set((coords.x, coords.y));

          if pending_drag_id.read().is_some() && !*drag_active.read() {
              let (sx, sy) = *pointer_start.read();
              let dx = coords.x - sx;
              let dy = coords.y - sy;
              if (dx * dx + dy * dy).sqrt() >= 5.0 {
                  drag_active.set(true);
                  dragging_issue_id.set(pending_drag_id.read().clone());
              }
          }
      },
      onmouseup: {
          let on_status_change = on_status_change.clone();
          move |_| {
              if *drag_active.read() {
                  if let Some(issue_id) = dragging_issue_id.read().clone() {
                      if let Some(target_status) = drag_over_column.read().clone() {
                          on_status_change.call((issue_id, target_status));
                      }
                  }
              }
              drag_active.set(false);
              dragging_issue_id.set(None);
              pending_drag_id.set(None);
              drag_over_column.set(None);
          }
      },
      onmouseleave: move |_| {
          drag_active.set(false);
          dragging_issue_id.set(None);
          pending_drag_id.set(None);
          drag_over_column.set(None);
      },
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
            drag_active,
            pending_drag_id,
            pointer_start,
            pointer_pos,
          }
        }
      }
      if *drag_active.read() {
        if let Some(ref active_id) = *dragging_issue_id.read() {
            if let Some(issue) = issues.iter().find(|i| &i.id == active_id) {
                {render_drag_overlay(issue, &agents, *pointer_pos.read())}
            }
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
  drag_active: Signal<bool>,
  pending_drag_id: Signal<Option<String>>,
  pointer_start: Signal<(f64, f64)>,
  pointer_pos: Signal<(f64, f64)>,
) -> Element {
  let label = status_label(&status);
  let count = issues.len();
  let is_drag_over = drag_over_column.read().as_deref() == Some(status.as_str());
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
        onmouseenter: move |_| {
            if *drag_active.read() {
                drag_over_column.set(Some(status_over.clone()));
            }
        },
        onmouseleave: move |_| {
            if drag_over_column.read().as_deref() == Some(status_leave.as_str()) {
                drag_over_column.set(None);
            }
        },
        for issue in issues.iter() {
          KanbanCard {
            issue: issue.clone(),
            agents: agents.clone(),
            dragging_issue_id,
            drag_active,
            pending_drag_id,
            pointer_start,
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
  drag_active: Signal<bool>,
  pending_drag_id: Signal<Option<String>>,
  pointer_start: Signal<(f64, f64)>,
  on_click: EventHandler<()>,
) -> Element {
  let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
  let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()));
  let is_dragging = *drag_active.read() && dragging_issue_id.read().as_deref() == Some(issue.id.as_str());
  let card_cls = if is_dragging {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left opacity-30 transition-opacity cursor-grabbing"
  } else {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left hover:shadow-sm transition-shadow cursor-grab"
  };
  let drag_id = issue.id.clone();

  rsx! {
    div {
      class: "{card_cls}",
      onmousedown: move |evt| {
          let coords = evt.client_coordinates();
          pointer_start.set((coords.x, coords.y));
          pending_drag_id.set(Some(drag_id.clone()));
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

fn render_drag_overlay(issue: &Issue, agents: &[AgentRef], pos: (f64, f64)) -> Element {
  let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
  let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()));
  let style =
    format!("position: fixed; left: {}px; top: {}px; width: 240px; pointer-events: none; z-index: 50; transform: translate(-50%, -50%);", pos.0, pos.1);

  rsx! {
    div {
      style: "{style}",
      div {
        class: "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 shadow-lg ring-1 ring-[var(--primary)]/20",
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
}
