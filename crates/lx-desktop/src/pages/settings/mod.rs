mod env_vars;
mod quotas;
pub mod state;
mod task_priority;

use dioxus::prelude::*;

use self::env_vars::EnvVarsPanel;
use self::quotas::QuotasPanel;
use self::state::SettingsState;
use self::task_priority::TaskPriorityPanel;

#[component]
pub fn Settings() -> Element {
  let mut settings = SettingsState::provide();
  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex items-center justify-between",
        span { class: "text-sm text-[var(--outline)] uppercase tracking-wider",
          "SYSTEM / AGENT_CONFIG_V2"
        }
        div { class: "flex gap-2",
          button {
            class: "border border-[var(--outline)] text-[var(--on-surface)] rounded px-4 py-2 text-xs uppercase tracking-wider hover:bg-[var(--surface-container-high)] transition-colors duration-150",
            onclick: move |_| settings.discard(),
            "DISCARD CHANGES"
          }
          button {
            class: "bg-[var(--warning)] text-[var(--on-primary)] rounded px-4 py-2 text-xs uppercase tracking-wider font-semibold hover:brightness-110 transition-all duration-150",
            onclick: move |_| settings.execute(),
            "APPLY SETTINGS"
          }
        }
      }
      div { class: "flex gap-4 flex-1",
        div { class: "flex-[6] flex flex-col gap-4",
          EnvVarsPanel {}
          QuotasPanel {}
        }
        div { class: "flex-[4] flex flex-col gap-4",
          TaskPriorityPanel {}
          div { class: "relative bg-[var(--surface-container-lowest)] border-2 border-white p-4",
            span { class: "absolute -top-3 -right-3 bg-[var(--warning)] text-black text-[10px] px-2 py-1 font-black uppercase tracking-wider",
              "LIVE"
            }
            div { class: "flex items-center gap-4 mb-4",
              div { class: "w-12 h-12 border-2 border-[var(--outline)] p-1",
                div { class: "w-full h-full bg-[var(--warning)] flex items-center justify-center",
                  span { class: "material-symbols-outlined text-black font-bold",
                    "smart_toy"
                  }
                }
              }
              div {
                span { class: "text-sm font-bold uppercase tracking-wider text-[var(--on-surface)]",
                  "ARCHITECT_01"
                }
                p { class: "text-[10px] text-[var(--on-surface-variant)] uppercase font-mono",
                  "ID: 948-XFF-001"
                }
              }
            }
            div { class: "pt-2 border-t border-[var(--outline-variant)]" }
            div { class: "flex justify-between text-xs mb-1 pt-2",
              span { class: "text-[var(--outline)] uppercase tracking-wider",
                "RUNTIME"
              }
              span { class: "text-[var(--on-surface-variant)]", "284:12:05" }
            }
            div { class: "flex justify-between text-xs",
              span { class: "text-[var(--outline)] uppercase tracking-wider",
                "LOAD_FACTOR"
              }
              span { class: "text-[var(--primary)] font-mono", "OPTIMAL" }
            }
          }
          div { class: "bg-[var(--surface-container)] p-4 border-l-4 border-[var(--tertiary)]",
            div { class: "flex items-start gap-3",
              span { class: "material-symbols-outlined text-[var(--tertiary)] text-lg",
                "info"
              }
              p { class: "text-[10px] text-[var(--on-surface-variant)] leading-relaxed",
                span { class: "text-white font-bold", "SYSTEM_NOTICE: " }
                "All configuration changes require manual validation before persisting to the blockchain ledger. Expect a 120ms latency injection during the verification cycle."
              }
            }
          }
        }
      }
    }
  }
}
