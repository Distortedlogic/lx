use dioxus::prelude::*;

use super::types::{AgentRef, PRIORITY_ORDER, STATUS_ORDER};
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};

#[derive(Clone, Debug)]
pub struct NewIssuePayload;

#[component]
pub fn NewIssueDialog(open: bool, agents: Vec<AgentRef>, on_close: EventHandler<()>, on_create: EventHandler<NewIssuePayload>) -> Element {
  let mut title = use_signal(String::new);
  let mut description = use_signal(String::new);
  let mut status = use_signal(|| "todo".to_string());
  let mut priority = use_signal(|| "medium".to_string());
  let mut assignee = use_signal(|| Option::<String>::None);

  if !open {
    return rsx! {};
  }

  rsx! {
    div {
      class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
      onclick: move |_| on_close.call(()),
      div {
        class: "bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg w-full max-w-lg overflow-hidden",
        onclick: move |evt| evt.stop_propagation(),
        div { class: "flex items-center justify-between px-4 py-2.5 border-b border-[var(--outline-variant)]",
          span { class: "text-sm text-[var(--outline)]", "New Issue" }
          button {
            class: "text-[var(--outline)] hover:text-[var(--on-surface)] text-lg",
            onclick: move |_| on_close.call(()),
            "x"
          }
        }
        div { class: "p-4 space-y-4",
          input {
            class: "w-full text-lg font-semibold bg-transparent outline-none text-[var(--on-surface)] placeholder:text-[var(--outline)]/40",
            placeholder: "Issue title",
            value: "{title}",
            oninput: move |evt| title.set(evt.value().to_string()),
          }
          textarea {
            class: "w-full rounded border border-[var(--outline-variant)] px-3 py-2 bg-transparent outline-none text-sm min-h-[100px] resize-y placeholder:text-[var(--outline)]/40",
            placeholder: "Description (optional)",
            value: "{description}",
            oninput: move |evt| description.set(evt.value().to_string()),
          }
          div { class: "grid grid-cols-3 gap-3",
            div {
              label { class: "text-xs text-[var(--outline)] block mb-1",
                "Status"
              }
              select {
                class: INPUT_FIELD,
                value: "{status}",
                onchange: move |evt| status.set(evt.value().to_string()),
                for s in STATUS_ORDER {
                  option { value: *s, "{s}" }
                }
              }
            }
            div {
              label { class: "text-xs text-[var(--outline)] block mb-1",
                "Priority"
              }
              select {
                class: INPUT_FIELD,
                value: "{priority}",
                onchange: move |evt| priority.set(evt.value().to_string()),
                for p in PRIORITY_ORDER {
                  option { value: *p, "{p}" }
                }
              }
            }
            div {
              label { class: "text-xs text-[var(--outline)] block mb-1",
                "Assignee"
              }
              select {
                class: INPUT_FIELD,
                value: assignee.read().as_deref().unwrap_or(""),
                onchange: move |evt| {
                    let v = evt.value().to_string();
                    assignee.set(if v.is_empty() { None } else { Some(v) });
                },
                option { value: "", "Unassigned" }
                for agent in agents.iter() {
                  option { value: "{agent.id}", "{agent.name}" }
                }
              }
            }
          }
        }
        div { class: "border-t border-[var(--outline-variant)] px-4 py-3 flex justify-end gap-2",
          button {
            class: BTN_OUTLINE_SM,
            onclick: move |_| on_close.call(()),
            "Cancel"
          }
          button {
            class: BTN_PRIMARY_SM,
            disabled: title.read().trim().is_empty(),
            onclick: {
                move |_| {
                    on_create.call(NewIssuePayload);
                }
            },
            "Create Issue"
          }
        }
      }
    }
  }
}
