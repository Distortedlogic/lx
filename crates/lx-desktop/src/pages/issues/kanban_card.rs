use dioxus::prelude::*;

use super::types::{AgentRef, Issue, priority_icon_class};

#[component]
pub fn KanbanCard(
  issue: Issue,
  agents: Vec<AgentRef>,
  dragging_issue_id: Signal<Option<String>>,
  drag_active: Signal<bool>,
  pending_drag_id: Signal<Option<String>>,
  pointer_start: Signal<(f64, f64)>,
  #[props(default)] is_active: bool,
  on_click: EventHandler<()>,
) -> Element {
  let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
  let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()));
  let is_dragging = *drag_active.read() && dragging_issue_id.read().as_deref() == Some(issue.id.as_str());
  let card_cls = if is_dragging {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left opacity-30 transition-opacity cursor-grabbing"
  } else if is_active {
    "rounded-md border border-[var(--tertiary)]/40 bg-[var(--surface-container)] p-2.5 w-full text-left hover:shadow-sm transition-shadow cursor-grab ring-1 ring-[var(--tertiary)]/30 animate-pulse"
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
        if is_active {
          span { class: "material-symbols-outlined text-xs text-[var(--tertiary)] shrink-0", "bolt" }
        }
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

pub fn render_drag_overlay(issue: &Issue, agents: &[AgentRef], pos: (f64, f64)) -> Element {
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
