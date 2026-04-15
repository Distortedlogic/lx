use dioxus::prelude::*;
use uuid::Uuid;

use super::types::{PROJECT_COLORS, PROJECT_STATUSES, Project};

#[component]
pub fn NewProjectDialog(open: Signal<bool>, projects: Signal<Vec<Project>>) -> Element {
  let mut name = use_signal(String::new);
  let mut description = use_signal(String::new);
  let mut status = use_signal(|| "planned".to_string());
  let mut target_date = use_signal(String::new);
  let mut color = use_signal(|| PROJECT_COLORS[0].to_string());

  rsx! {
    div {
      class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
      onclick: move |_| open.set(false),
      div {
        class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg w-[480px] max-h-[80vh] overflow-y-auto",
        onclick: move |evt| evt.stop_propagation(),
        div { class: "flex items-center justify-between px-5 py-4 border-b border-[var(--outline-variant)]/20",
          span { class: "text-sm font-bold uppercase tracking-wider text-[var(--on-surface)]",
            "NEW PROJECT"
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
            placeholder: "Project name",
            value: "{name}",
            oninput: move |evt| name.set(evt.value()),
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
              for s in PROJECT_STATUSES.iter() {
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
              "COLOR"
            }
            div { class: "flex gap-2 flex-wrap",
              for c in PROJECT_COLORS.iter() {
                {
                    let c_val = c.to_string();
                    let active = color() == *c;
                    let ring = if active { "ring-2 ring-white" } else { "" };
                    rsx! {
                      button {
                        class: "w-6 h-6 rounded",
                        class: "{ring}",
                        style: "background-color: {c}",
                        onclick: move |_| color.set(c_val.clone()),
                      }
                    }
                }
              }
            }
          }
          div { class: "flex flex-col gap-1",
            span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
              "TARGET DATE"
            }
            input {
              class: "bg-[var(--surface-container-lowest)] text-sm px-3 py-2 rounded outline-none text-[var(--on-surface)]",
              r#type: "date",
              value: "{target_date}",
              oninput: move |evt| target_date.set(evt.value()),
            }
          }
        }
        div { class: "flex justify-end gap-2 px-5 py-4 border-t border-[var(--outline-variant)]/20",
          button {
            class: "px-4 py-2 text-xs uppercase font-semibold text-[var(--outline)] hover:text-[var(--on-surface)]",
            onclick: move |_| {
                name.set(String::new());
                description.set(String::new());
                status.set("planned".to_string());
                target_date.set(String::new());
                color.set(PROJECT_COLORS[0].to_string());
                open.set(false);
            },
            "CANCEL"
          }
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150 rounded",
            onclick: move |_| {
                let n = name().trim().to_string();
                if n.is_empty() {
                    return;
                }
                let desc = description().trim().to_string();
                let td = target_date().trim().to_string();
                projects
                    .write()
                    .push(Project {
                        id: Uuid::new_v4().to_string(),
                        name: n,
                        description: if desc.is_empty() { None } else { Some(desc) },
                        status: status(),
                        color: color(),
                        target_date: if td.is_empty() { None } else { Some(td) },
                        goal_ids: Vec::new(),
                        archived_at: None,
                    });
                name.set(String::new());
                description.set(String::new());
                status.set("planned".to_string());
                target_date.set(String::new());
                color.set(PROJECT_COLORS[0].to_string());
                open.set(false);
            },
            "CREATE"
          }
        }
      }
    }
  }
}
