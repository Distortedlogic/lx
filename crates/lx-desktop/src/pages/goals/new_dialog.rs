use dioxus::prelude::*;
use uuid::Uuid;

use crate::pages::projects::types::{GOAL_LEVELS, GOAL_STATUSES, Goal};

fn level_label(level: &str) -> &'static str {
  match level {
    "company" => "Company",
    "team" => "Team",
    "agent" => "Agent",
    "task" => "Task",
    _ => "Task",
  }
}

#[component]
pub fn NewGoalDialog(open: Signal<bool>, goals: Signal<Vec<Goal>>, parent_id: Option<String>) -> Element {
  let mut title = use_signal(String::new);
  let mut description = use_signal(String::new);
  let mut status = use_signal(|| "planned".to_string());
  let mut level = use_signal(|| "task".to_string());
  let mut selected_parent: Signal<Option<String>> = use_signal(|| parent_id.clone());

  let existing_goals = goals();
  let header = if parent_id.is_some() { "NEW SUB-GOAL" } else { "NEW GOAL" };

  rsx! {
    div {
      class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
      onclick: move |_| open.set(false),
      div {
        class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg w-[480px] max-h-[80vh] overflow-y-auto",
        onclick: move |evt| evt.stop_propagation(),
        div { class: "flex items-center justify-between px-5 py-4 border-b border-[var(--outline-variant)]/20",
          span { class: "text-sm font-bold uppercase tracking-wider text-[var(--on-surface)]",
            "{header}"
          }
          button {
            class: "text-[var(--outline)] hover:text-[var(--on-surface)] text-sm",
            onclick: move |_| open.set(false),
            "X"
          }
        }
        div { class: "flex flex-col gap-4 px-5 py-4",
          input {
            class: "bg-[var(--surface-container-lowest)] text-sm px-3 py-2 rounded outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
            placeholder: "Goal title",
            value: "{title}",
            oninput: move |evt| title.set(evt.value()),
          }
          textarea {
            class: "bg-[var(--surface-container-lowest)] text-sm px-3 py-2 rounded outline-none text-[var(--on-surface)] placeholder-[var(--outline)] min-h-[60px] resize-none",
            placeholder: "Description",
            value: "{description}",
            oninput: move |evt| description.set(evt.value()),
          }
          div { class: "flex flex-col gap-1",
            span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
              "STATUS"
            }
            div { class: "flex gap-1 flex-wrap",
              for s in GOAL_STATUSES.iter() {
                {
                    let s_val = s.to_string();
                    let active = status() == *s;
                    rsx! {
                      button {
                        class: if active { "px-2 py-1 text-[10px] uppercase font-semibold rounded bg-[var(--primary)] text-[var(--on-primary)]" } else { "px-2 py-1 text-[10px] uppercase font-semibold rounded bg-[var(--surface-container-lowest)] text-[var(--outline)] hover:text-[var(--on-surface)]" },
                        onclick: move |_| status.set(s_val.clone()),
                        "{s}"
                      }
                    }
                }
              }
            }
          }
          div { class: "flex flex-col gap-1",
            span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
              "LEVEL"
            }
            div { class: "flex gap-1 flex-wrap",
              for l in GOAL_LEVELS.iter() {
                {
                    let l_val = l.to_string();
                    let active = level() == *l;
                    rsx! {
                      button {
                        class: if active { "px-2 py-1 text-[10px] uppercase font-semibold rounded bg-[var(--primary)] text-[var(--on-primary)]" } else { "px-2 py-1 text-[10px] uppercase font-semibold rounded bg-[var(--surface-container-lowest)] text-[var(--outline)] hover:text-[var(--on-surface)]" },
                        onclick: move |_| level.set(l_val.clone()),
                        "{level_label(l)}"
                      }
                    }
                }
              }
            }
          }
          div { class: "flex flex-col gap-1",
            span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
              "PARENT GOAL"
            }
            div { class: "flex flex-col gap-1 max-h-[120px] overflow-y-auto",
              button {
                class: if selected_parent().is_none() { "px-2 py-1 text-[10px] uppercase font-semibold rounded bg-[var(--primary)] text-[var(--on-primary)] text-left" } else { "px-2 py-1 text-[10px] uppercase font-semibold rounded bg-[var(--surface-container-lowest)] text-[var(--outline)] hover:text-[var(--on-surface)] text-left" },
                onclick: move |_| selected_parent.set(None),
                "No parent"
              }
              for g in existing_goals.iter() {
                {
                    let gid = g.id.clone();
                    let active = selected_parent() == Some(g.id.clone());
                    rsx! {
                      button {
                        class: if active { "px-2 py-1 text-[10px] font-semibold rounded bg-[var(--primary)] text-[var(--on-primary)] text-left truncate" } else { "px-2 py-1 text-[10px] font-semibold rounded bg-[var(--surface-container-lowest)] text-[var(--outline)] hover:text-[var(--on-surface)] text-left truncate" },
                        onclick: move |_| selected_parent.set(Some(gid.clone())),
                        "{g.title}"
                      }
                    }
                }
              }
            }
          }
        }
        div { class: "flex justify-end gap-2 px-5 py-4 border-t border-[var(--outline-variant)]/20",
          button {
            class: "px-4 py-2 text-xs uppercase font-semibold text-[var(--outline)] hover:text-[var(--on-surface)]",
            onclick: move |_| {
                title.set(String::new());
                description.set(String::new());
                status.set("planned".to_string());
                level.set("task".to_string());
                selected_parent.set(None);
                open.set(false);
            },
            "CANCEL"
          }
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150 rounded",
            onclick: move |_| {
                let t = title().trim().to_string();
                if t.is_empty() {
                    return;
                }
                let desc = description().trim().to_string();
                let now = "2026-03-28T00:00:00Z".to_string();
                goals
                    .write()
                    .push(Goal {
                        id: Uuid::new_v4().to_string(),
                        title: t,
                        description: if desc.is_empty() { None } else { Some(desc) },
                        status: status(),
                        level: level(),
                        parent_id: selected_parent(),
                        owner_agent_id: None,
                        created_at: now.clone(),
                        updated_at: now,
                    });
                title.set(String::new());
                description.set(String::new());
                status.set("planned".to_string());
                level.set("task".to_string());
                selected_parent.set(None);
                open.set(false);
            },
            "CREATE"
          }
        }
      }
    }
  }
}
