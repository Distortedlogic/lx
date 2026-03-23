use dioxus::prelude::*;

#[component]
pub fn TaskPriorityPanel() -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4",
      div { class: "flex items-center gap-3 mb-4",
        div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
        span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
          "TASK_PRIORITY"
        }
        div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      }
      div { class: "flex items-center justify-between mb-2",
        span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
          "WEIGHTING_INDEX"
        }
        span { class: "text-sm font-semibold text-[var(--on-surface)]", "0.84" }
      }
      input {
        r#type: "range",
        class: "w-full accent-[var(--primary)] mb-3",
      }
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
    div { class: "bg-[var(--surface-container)] border border-[var(--success)] rounded-lg p-4",
      div { class: "flex items-center gap-2 mb-2",
        span { class: "text-[var(--success)]", "\u{25CF}" }
        span { class: "text-sm font-bold uppercase tracking-wider text-[var(--on-surface)]",
          "ARCHITECT_01"
        }
      }
      p { class: "text-[10px] text-[var(--outline)] mb-3", "ID: 0x8.AP3.003" }
      div { class: "flex justify-between text-xs mb-1",
        span { class: "text-[var(--outline)] uppercase tracking-wider", "RUNTIME" }
        span { class: "text-[var(--on-surface-variant)]", "284:12:05" }
      }
      div { class: "flex justify-between text-xs",
        span { class: "text-[var(--outline)] uppercase tracking-wider", "LOAD_FACTOR" }
        span { class: "text-[var(--on-surface-variant)]", "OPTIMAL" }
      }
    }
  }
}

#[component]
pub fn SystemNotice() -> Element {
  rsx! {
    div { class: "flex items-start gap-3 mt-2",
      span { class: "text-[var(--warning)] text-lg shrink-0", "\u{26A0}" }
      p { class: "text-xs text-[var(--outline)] leading-relaxed",
        "SYSTEM_NOTICE: All configuration changes require manual validation before persisting to the blockchain ledger. Expect a 120ms latency injection during the verification cycle."
      }
    }
  }
}
