use dioxus::prelude::*;

#[component]
pub fn StatusBar() -> Element {
  rsx! {
    div { class: "flex items-center justify-between h-6 px-3 bg-[var(--surface-container-low)] text-xs uppercase tracking-[0.05em] font-[var(--font-body)] shrink-0",
      div { class: "flex items-center gap-3",
        span { class: "text-[var(--primary)] font-semibold", "LX ENGINE // DEV BUILD" }
        span { class: "text-[var(--outline)]", "Main+" }
        span { class: "text-[var(--outline)]", "Ln 12, Col 45" }
      }
      div { class: "flex items-center gap-3",
        span { class: "text-[var(--outline)]", "UTF-8" }
        span { class: "text-[var(--outline)]",
          span { class: "text-[var(--success)]", "\u{25CF} " }
          "Prettier"
        }
        span { class: "text-[var(--outline)]", "\u{1F514}" }
      }
    }
  }
}
