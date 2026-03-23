use dioxus::prelude::*;

struct EnvVar {
  key: &'static str,
  value: &'static str,
}

const VARS: &[EnvVar] = &[
  EnvVar { key: "API_ENDPOINT_ROOT", value: "https://core.monolith.io/v2" },
  EnvVar { key: "MAX_CONCURRENCY", value: "512" },
  EnvVar { key: "RETRY_POLICY", value: "EXPONENTIAL_BACKOFF" },
];

#[component]
pub fn EnvVarsPanel() -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-0 overflow-hidden",
      div { class: "bg-[var(--surface-container-high)] px-4 py-2 border-b-2 border-[var(--outline-variant)] flex justify-between items-center",
        span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
          "ENVIRONMENT_VARIABLES"
        }
        span { class: "text-[10px] uppercase tracking-wider text-[var(--tertiary)] font-mono",
          "STATUS: MUTABLE"
        }
      }
      div { class: "flex text-[10px] uppercase tracking-wider text-[var(--on-surface-variant)] py-3 px-4 border-b border-[var(--outline-variant)]",
        span { class: "flex-[3]", "KEY" }
        span { class: "flex-[5]", "VALUE" }
        span { class: "flex-[1] text-right", "ACTIONS" }
      }
      div { class: "flex flex-col gap-1",
        for var_entry in VARS {
          div { class: "flex items-center px-4 py-3 border-b border-[var(--outline-variant)]/30 hover:bg-[var(--surface-container)] transition-colors duration-150",
            span { class: "flex-[3] text-xs font-semibold text-[var(--warning)] uppercase",
              "{var_entry.key}"
            }
            span { class: "flex-[5] text-xs text-[var(--on-surface-variant)]",
              "{var_entry.value}"
            }
            span { class: "flex-[1] text-right",
              span { class: "material-symbols-outlined text-sm text-[var(--outline)] cursor-pointer hover:text-[var(--tertiary)]",
                "edit"
              }
            }
          }
        }
      }
      div { class: "flex items-center gap-2 px-4 py-3",
        input {
          class: "flex-[3] bg-[var(--surface-container-low)] text-xs px-3 py-1.5 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
          placeholder: "NEW_KEY",
        }
        input {
          class: "flex-[5] bg-[var(--surface-container-low)] text-xs px-3 py-1.5 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
          placeholder: "VALUE",
        }
        button { class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-1.5 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150",
          "ADD"
        }
      }
    }
  }
}
