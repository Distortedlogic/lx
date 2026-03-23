use dioxus::prelude::*;

#[component]
pub fn StatusBar() -> Element {
  rsx! {
    div { class: "flex items-center justify-between h-6 px-3 bg-[var(--surface-container-lowest)] border-t-2 border-[var(--outline)] text-xs uppercase tracking-[0.05em] font-mono shrink-0",
      div { class: "flex items-center gap-3",
        span { class: "text-white font-bold", "SYSTEM_READY_V1.0.4" }
        span { class: "text-[var(--primary)]", "\u{25A0}" }
        span { class: "text-[var(--primary)]", "main*" }
        span { class: "text-[var(--outline)]", "Ln 1, Col 1" }
      }
      div { class: "flex items-center gap-3",
        span { class: "text-[var(--outline)]", "UTF-8" }
        span { class: "flex items-center gap-1 text-[var(--outline)]",
          span { class: "text-[var(--success)] text-[8px]", "\u{25CF}" }
          "Notifications (0)"
        }
      }
    }
  }
}
