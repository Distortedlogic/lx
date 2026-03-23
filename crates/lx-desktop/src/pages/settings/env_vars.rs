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
    div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4",
      div { class: "flex items-center justify-between mb-4",
        div { class: "flex items-center gap-3",
          div { class: "h-px w-8 bg-[var(--outline-variant)]" }
          span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
            "ENVIRONMENT_VARIABLES"
          }
          div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
        }
        span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
          "STATUS: MUTABLE"
        }
      }
      div { class: "flex text-[10px] uppercase tracking-wider text-[var(--outline)] mb-2 px-4",
        span { class: "flex-[3]", "KEY" }
        span { class: "flex-[5]", "VALUE" }
        span { class: "flex-[1] text-right", "ACTIONS" }
      }
      div { class: "flex flex-col gap-1",
        for var_entry in VARS {
          div { class: "flex items-center px-4 py-2 rounded hover:bg-[var(--surface-container-high)] transition-colors duration-150",
            div { class: "w-1 h-6 bg-[var(--success)] rounded-full mr-3 shrink-0" }
            span { class: "flex-[3] text-xs font-semibold text-[var(--primary)] uppercase",
              "{var_entry.key}"
            }
            span { class: "flex-[5] text-xs text-[var(--on-surface-variant)]",
              "{var_entry.value}"
            }
            span { class: "flex-[1] text-right text-[var(--success)]",
              "\u{2713}"
            }
          }
        }
      }
      div { class: "flex items-center gap-2 mt-3 px-4",
        input {
          class: "flex-[3] bg-[var(--surface-container-low)] text-xs px-3 py-1.5 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
          placeholder: "NEW_KEY",
        }
        input {
          class: "flex-[5] bg-[var(--surface-container-low)] text-xs px-3 py-1.5 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
          placeholder: "VALUE",
        }
        button { class: "bg-[var(--success)] text-[var(--on-primary)] rounded px-4 py-1.5 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150",
          "ADD"
        }
      }
    }
  }
}
