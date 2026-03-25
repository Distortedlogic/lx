use super::state::{EnvEntry, SettingsState};
use dioxus::prelude::*;

#[component]
pub fn EnvVarsPanel() -> Element {
  let settings = use_context::<SettingsState>();
  let mut data = settings.data;
  let mut new_key = use_signal(String::new);
  let mut new_value = use_signal(String::new);

  let env_vars = settings.data.read().env_vars.clone();

  rsx! {
    div { class: "bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-0 overflow-hidden",
      div { class: "bg-[var(--surface-container-high)] px-4 py-2 border-b-2 border-[var(--outline-variant)] flex justify-between items-center",
        span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
          "ENVIRONMENT_VARIABLES"
        }
        span { class: "text-[10px] uppercase tracking-wider text-[var(--tertiary)] font-mono",
          "COUNT: {env_vars.len()}"
        }
      }
      div { class: "flex text-[10px] uppercase tracking-wider text-[var(--on-surface-variant)] py-3 px-4 border-b border-[var(--outline-variant)]",
        span { class: "flex-[3]", "KEY" }
        span { class: "flex-[5]", "VALUE" }
        span { class: "flex-[1] text-right", "ACTIONS" }
      }
      div { class: "flex flex-col gap-1",
        for (i , entry) in env_vars.iter().enumerate() {
          {
              let key = entry.key.clone();
              let value = entry.value.clone();
              rsx! {
                div { class: "flex items-center px-4 py-3 border-b border-[var(--outline-variant)]/30 hover:bg-[var(--surface-container)] transition-colors duration-150",
                  span { class: "flex-[3] text-xs font-semibold text-[var(--warning)] uppercase", "{key}" }
                  span { class: "flex-[5] text-xs text-[var(--on-surface-variant)]", "{value}" }
                  span { class: "flex-[1] text-right",
                    span {
                      class: "material-symbols-outlined text-sm text-[var(--outline)] cursor-pointer hover:text-[var(--error)]",
                      onclick: move |_| {
                          data.write().env_vars.remove(i);
                      },
                      "delete"
                    }
                  }
                }
              }
          }
        }
      }
      div { class: "flex items-center gap-2 px-4 py-3",
        input {
          class: "flex-[3] bg-[var(--surface-container-low)] text-xs px-3 py-1.5 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
          placeholder: "NEW_KEY",
          value: "{new_key}",
          oninput: move |evt| new_key.set(evt.value()),
        }
        input {
          class: "flex-[5] bg-[var(--surface-container-low)] text-xs px-3 py-1.5 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
          placeholder: "VALUE",
          value: "{new_value}",
          oninput: move |evt| new_value.set(evt.value()),
        }
        button {
          class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-1.5 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150",
          onclick: move |_| {
              let k = new_key().trim().to_string();
              let v = new_value().trim().to_string();
              if !k.is_empty() {
                  data.write().env_vars.push(EnvEntry { key: k, value: v });
                  new_key.set(String::new());
                  new_value.set(String::new());
              }
          },
          "ADD"
        }
      }
    }
  }
}
