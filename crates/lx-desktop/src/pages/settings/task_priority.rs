use dioxus::prelude::*;

#[component]
pub fn TaskPriorityPanel() -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container-low)] border-2 border-[var(--outline-variant)] p-6",
      span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--warning)] mb-4",
        "TASK_PRIORITY"
      }
      div { class: "flex items-center justify-between mb-2",
        span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
          "WEIGHTING_INDEX"
        }
        span { class: "text-sm font-semibold text-[var(--on-surface)]", "0.84" }
      }
      input { r#type: "range", class: "w-full accent-[var(--warning)] mb-3" }
      div { class: "flex justify-between text-[10px] text-[var(--outline)] mb-4",
        span { "LOW_LATENCY" }
        span { "HIGH_THROUGHPUT" }
      }
      div { class: "flex flex-col gap-2",
        label { class: "flex items-center gap-2 text-xs text-[var(--on-surface-variant)] cursor-pointer",
          div { class: "w-4 h-4 rounded bg-[var(--warning)] flex items-center justify-center text-[var(--on-primary)] text-[10px]",
            "\u{2713}"
          }
          "AUTO-SCALE_RESOURCES"
        }
        label { class: "flex items-center gap-2 text-xs text-[var(--on-surface-variant)] cursor-pointer",
          div { class: "w-4 h-4 rounded border border-[var(--outline)] flex items-center justify-center" }
          "REDUNDANT_VERIFICATION"
        }
      }
    }
  }
}

#[component]
pub fn ArchitectCard() -> Element {
  rsx! {
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
        span { class: "text-[var(--outline)] uppercase tracking-wider", "RUNTIME" }
        span { class: "text-[var(--on-surface-variant)]", "284:12:05" }
      }
      div { class: "flex justify-between text-xs",
        span { class: "text-[var(--outline)] uppercase tracking-wider", "LOAD_FACTOR" }
        span { class: "text-[var(--primary)] font-mono", "OPTIMAL" }
      }
    }
  }
}

#[component]
pub fn SystemNotice() -> Element {
  rsx! {
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
