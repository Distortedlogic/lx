use super::types::ADAPTER_LABELS;
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};
use dioxus::prelude::*;

#[component]
pub fn NewAgentDialog(open: bool, on_close: EventHandler<()>, on_create: EventHandler<NewAgentPayload>) -> Element {
  let mut name = use_signal(String::new);
  let mut title = use_signal(String::new);
  let mut role = use_signal(|| "general".to_string());
  let mut adapter_type = use_signal(|| "claude_local".to_string());
  let mut show_advanced = use_signal(|| false);

  if !open {
    return rsx! {};
  }

  rsx! {
    div {
      class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
      onclick: move |_| on_close.call(()),
      div {
        class: "bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg w-full max-w-md overflow-hidden",
        onclick: move |evt| evt.stop_propagation(),
        div { class: "flex items-center justify-between px-4 py-2.5 border-b border-[var(--outline-variant)]",
          span { class: "text-sm text-[var(--outline)]", "New Agent" }
          button {
            class: "text-[var(--outline)] hover:text-[var(--on-surface)] text-lg",
            onclick: move |_| on_close.call(()),
            "x"
          }
        }
        div { class: "p-6 space-y-4",
          if !*show_advanced.read() {
            div { class: "space-y-4",
              input {
                class: INPUT_FIELD,
                placeholder: "Agent name",
                value: "{name}",
                oninput: move |evt| name.set(evt.value().to_string()),
              }
              input {
                class: INPUT_FIELD,
                placeholder: "Title (e.g. VP of Engineering)",
                value: "{title}",
                oninput: move |evt| title.set(evt.value().to_string()),
              }
              div {
                label { class: "text-xs text-[var(--outline)] block mb-1",
                  "Role"
                }
                select {
                  class: INPUT_FIELD,
                  value: "{role}",
                  onchange: move |evt| role.set(evt.value().to_string()),
                  option { value: "ceo", "CEO" }
                  option { value: "executive", "Executive" }
                  option { value: "manager", "Manager" }
                  option { value: "general", "General" }
                  option { value: "specialist", "Specialist" }
                }
              }
              div {
                label { class: "text-xs text-[var(--outline)] block mb-1",
                  "Adapter"
                }
                select {
                  class: INPUT_FIELD,
                  value: "{adapter_type}",
                  onchange: move |evt| adapter_type.set(evt.value().to_string()),
                  for (key , label) in ADAPTER_LABELS {
                    option { value: *key, "{label}" }
                  }
                }
              }
            }
            button {
              class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)] underline",
              onclick: move |_| show_advanced.set(true),
              "Show advanced options"
            }
          } else {
            button {
              class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)]",
              onclick: move |_| show_advanced.set(false),
              "< Back"
            }
            p { class: "text-sm text-[var(--outline)]",
              "Advanced configuration available after creation."
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
            disabled: name.read().trim().is_empty(),
            onclick: {
                move |_| {
                    on_create.call(NewAgentPayload);
                }
            },
            "Create Agent"
          }
        }
      }
    }
  }
}

#[derive(Clone, Debug)]
pub struct NewAgentPayload;
