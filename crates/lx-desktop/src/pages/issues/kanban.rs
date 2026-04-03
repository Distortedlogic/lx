use dioxus::prelude::*;

use super::kanban_card::{KanbanCard, render_drag_overlay};
use super::types::{AgentRef, Issue, status_icon_class, status_label};

const BOARD_STATUSES: &[&str] = &["backlog", "todo", "in_progress", "in_review", "blocked", "done", "cancelled"];

#[component]
pub fn KanbanBoardView(
  issues: Vec<Issue>,
  agents: Vec<AgentRef>,
  on_select: EventHandler<String>,
  on_status_change: EventHandler<(String, String)>,
  #[props(optional)] on_reorder: Option<EventHandler<(String, String, usize)>>,
  #[props(default)] active_issue_ids: Vec<String>,
) -> Element {
  let mut dragging_issue_id = use_signal(|| Option::<String>::None);
  let mut drag_over_column = use_signal(|| Option::<String>::None);
  let mut drag_active = use_signal(|| false);
  let pointer_start = use_signal(|| (0.0f64, 0.0f64));
  let mut pointer_pos = use_signal(|| (0.0f64, 0.0f64));
  let mut pending_drag_id = use_signal(|| Option::<String>::None);
  let mut drag_over_index = use_signal(|| Option::<usize>::None);

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
      ontouchmove: move |evt: TouchEvent| {
          if *drag_active.read() || pending_drag_id.read().is_some() {
              evt.prevent_default();
          }
          if let Some(touch) = evt.touches().first() {
              let coords = touch.client_coordinates();
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
          }
      },
      onmouseup: {
          let issues = issues.clone();
          let on_status_change = on_status_change;
          let on_reorder = on_reorder;
          move |_| {
              if *drag_active.read()
                  && let Some(issue_id) = dragging_issue_id.read().clone()
                  && let Some(target_status) = drag_over_column.read().clone()
              {
                  let source_status = issues
                      .iter()
                      .find(|i| i.id == issue_id)
                      .map(|i| i.status.clone());
                  if source_status.as_deref() == Some(target_status.as_str()) {
                      if let Some(idx) = *drag_over_index.read()
                          && let Some(ref handler) = on_reorder
                      {
                          handler.call((issue_id, target_status, idx));
                      }
                  } else {
                      on_status_change.call((issue_id, target_status));
                  }
              }
              drag_active.set(false);
              dragging_issue_id.set(None);
              pending_drag_id.set(None);
              drag_over_column.set(None);
              drag_over_index.set(None);
          }
      },
      ontouchend: {
          let on_status_change = on_status_change;
          let on_reorder = on_reorder;
          move |_: TouchEvent| {
              if *drag_active.read()
                  && let Some(issue_id) = dragging_issue_id.read().clone()
                  && let Some(target_status) = drag_over_column.read().clone()
              {
                  let source_status = issues
                      .iter()
                      .find(|i| i.id == issue_id)
                      .map(|i| i.status.clone());
                  if source_status.as_deref() == Some(target_status.as_str()) {
                      if let Some(idx) = *drag_over_index.read()
                          && let Some(ref handler) = on_reorder
                      {
                          handler.call((issue_id, target_status, idx));
                      }
                  } else {
                      on_status_change.call((issue_id, target_status));
                  }
              }
              drag_active.set(false);
              dragging_issue_id.set(None);
              pending_drag_id.set(None);
              drag_over_column.set(None);
              drag_over_index.set(None);
          }
      },
      onmouseleave: move |_| {
          drag_active.set(false);
          dragging_issue_id.set(None);
          pending_drag_id.set(None);
          drag_over_column.set(None);
          drag_over_index.set(None);
      },
      ontouchcancel: move |_: TouchEvent| {
          drag_active.set(false);
          dragging_issue_id.set(None);
          pending_drag_id.set(None);
          drag_over_column.set(None);
          drag_over_index.set(None);
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
            drag_over_index,
            active_issue_ids: active_issue_ids.clone(),
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
  active_issue_ids: Vec<String>,
  on_select: EventHandler<String>,
  on_status_change: EventHandler<(String, String)>,
  dragging_issue_id: Signal<Option<String>>,
  drag_over_column: Signal<Option<String>>,
  drag_active: Signal<bool>,
  pending_drag_id: Signal<Option<String>>,
  pointer_start: Signal<(f64, f64)>,
  pointer_pos: Signal<(f64, f64)>,
  drag_over_index: Signal<Option<usize>>,
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
                drag_over_index.set(None);
            }
        },
        for (idx , issue) in issues.iter().enumerate() {
          if *drag_active.read() && is_drag_over && drag_over_index.read().as_ref() == Some(&idx) {
            div { class: "h-0.5 rounded bg-[var(--primary)] mx-1 my-0.5" }
          }
          div {
            onmouseenter: {
                let status_for_enter = status.clone();
                move |_| {
                    if *drag_active.read()
                        && drag_over_column.read().as_deref() == Some(status_for_enter.as_str())
                    {
                        drag_over_index.set(Some(idx));
                    }
                }
            },
            KanbanCard {
              issue: issue.clone(),
              agents: agents.clone(),
              dragging_issue_id,
              drag_active,
              pending_drag_id,
              pointer_start,
              is_active: active_issue_ids.contains(&issue.id),
              on_click: {
                  let id = issue.identifier.clone().unwrap_or_else(|| issue.id.clone());
                  move |_| on_select.call(id.clone())
              },
            }
          }
        }
        if *drag_active.read() && is_drag_over
            && drag_over_index.read().as_ref() == Some(&issues.len())
        {
          div { class: "h-0.5 rounded bg-[var(--primary)] mx-1 my-0.5" }
        }
      }
    }
  }
}
