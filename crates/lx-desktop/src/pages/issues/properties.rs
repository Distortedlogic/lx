use dioxus::prelude::*;

use super::types::{AgentRef, Issue, PRIORITY_ORDER, STATUS_ORDER, status_icon_class, status_label};
#[component]
pub fn IssuePropertiesPanel(issue: Issue, agents: Vec<AgentRef>, on_update: EventHandler<(String, String)>) -> Element {
  let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()));

  rsx! {
    div { class: "space-y-4",
      div { class: "space-y-1",
        PropertyRow { label: "Status",
          StatusPicker {
            current: issue.status.clone(),
            on_change: move |s: String| on_update.call(("status".to_string(), s)),
          }
        }
        PropertyRow { label: "Priority",
          PriorityPicker {
            current: issue.priority.clone(),
            on_change: move |p: String| on_update.call(("priority".to_string(), p)),
          }
        }
        PropertyRow { label: "Assignee",
          AssigneePicker {
            current_agent_id: issue.assignee_agent_id.clone(),
            agents: agents.clone(),
            on_change: move |id: String| on_update.call(("assignee_agent_id".to_string(), id)),
          }
        }
        if !issue.labels.is_empty() {
          PropertyRow { label: "Labels",
            div { class: "flex flex-wrap gap-1",
              for label in issue.labels.iter() {
                span {
                  class: "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium border",
                  style: "border-color: {label.color}; background: {label.color}22;",
                  "{label.name}"
                }
              }
            }
          }
        }
      }
      div { class: "border-t border-[var(--outline-variant)]/30 pt-4 space-y-1",
        if let Some(name) = assignee_name {
          PropertyRow { label: "Assigned to",
            span { class: "text-sm text-[var(--on-surface)]", "{name}" }
          }
        }
        PropertyRow { label: "Created",
          span { class: "text-sm text-[var(--on-surface)]", "{issue.created_at}" }
        }
        PropertyRow { label: "Updated",
          span { class: "text-sm text-[var(--on-surface)]", "{issue.updated_at}" }
        }
        if issue.request_depth > 0 {
          PropertyRow { label: "Depth",
            span { class: "text-sm font-mono text-[var(--on-surface)]",
              "{issue.request_depth}"
            }
          }
        }
      }
    }
  }
}

#[component]
fn PropertyRow(label: &'static str, children: Element) -> Element {
  rsx! {
    div { class: "flex items-center gap-3 py-1.5",
      span { class: "property-label", "{label}" }
      div { class: "flex items-center gap-1.5 min-w-0 flex-1", {children} }
    }
  }
}

#[component]
fn StatusPicker(current: String, on_change: EventHandler<String>) -> Element {
  let mut open = use_signal(|| false);
  rsx! {
    div { class: "relative",
      button {
        class: "inline-flex items-center gap-1.5 cursor-pointer hover:bg-[var(--surface-container)] rounded px-1 py-0.5 transition-colors",
        onclick: move |_| {
            let v = *open.read();
            open.set(!v)
        },
        span { class: "material-symbols-outlined text-sm {status_icon_class(&current)}",
          "circle"
        }
        span { class: "text-sm text-[var(--on-surface)]", "{status_label(&current)}" }
      }
      if *open.read() {
        div { class: "absolute left-0 top-full mt-1 z-50 w-40 border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-lg rounded p-1",
          for status in STATUS_ORDER {
            button {
              class: "flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-[var(--surface-container-high)]",
              onclick: {
                  let s = status.to_string();
                  move |_| {
                      on_change.call(s.clone());
                      open.set(false);
                  }
              },
              span { class: "material-symbols-outlined text-xs {status_icon_class(status)}",
                "circle"
              }
              "{status_label(status)}"
            }
          }
        }
      }
    }
  }
}

#[component]
fn PriorityPicker(current: String, on_change: EventHandler<String>) -> Element {
  let mut open = use_signal(|| false);
  rsx! {
    div { class: "relative",
      button {
        class: "inline-flex items-center gap-1.5 cursor-pointer hover:bg-[var(--surface-container)] rounded px-1 py-0.5 transition-colors",
        onclick: move |_| {
            let v = *open.read();
            open.set(!v)
        },
        span { class: "text-sm text-[var(--on-surface)]", "{current}" }
      }
      if *open.read() {
        div { class: "absolute left-0 top-full mt-1 z-50 w-36 border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-lg rounded p-1",
          for priority in PRIORITY_ORDER {
            button {
              class: "flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-[var(--surface-container-high)]",
              onclick: {
                  let p = priority.to_string();
                  move |_| {
                      on_change.call(p.clone());
                      open.set(false);
                  }
              },
              "{priority}"
            }
          }
        }
      }
    }
  }
}

#[component]
fn AssigneePicker(current_agent_id: Option<String>, agents: Vec<AgentRef>, on_change: EventHandler<String>) -> Element {
  let mut open = use_signal(|| false);
  let current_name =
    current_agent_id.as_ref().and_then(|id| agents.iter().find(|a| &a.id == id).map(|a| a.name.clone())).unwrap_or_else(|| "Unassigned".to_string());

  rsx! {
    div { class: "relative",
      button {
        class: "inline-flex items-center gap-1.5 cursor-pointer hover:bg-[var(--surface-container)] rounded px-1 py-0.5 transition-colors",
        onclick: move |_| {
            let v = *open.read();
            open.set(!v)
        },
        span { class: "text-sm text-[var(--on-surface)]", "{current_name}" }
      }
      if *open.read() {
        div { class: "absolute left-0 top-full mt-1 z-50 w-44 border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-lg rounded p-1",
          button {
            class: "flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-[var(--surface-container-high)]",
            onclick: move |_| {
                on_change.call(String::new());
                open.set(false);
            },
            "Unassigned"
          }
          for agent in agents.iter() {
            button {
              class: "flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-[var(--surface-container-high)]",
              onclick: {
                  let id = agent.id.clone();
                  move |_| {
                      on_change.call(id.clone());
                      open.set(false);
                  }
              },
              "{agent.name}"
            }
          }
        }
      }
    }
  }
}
