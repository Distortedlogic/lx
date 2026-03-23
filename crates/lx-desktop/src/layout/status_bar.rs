use dioxus::prelude::*;

#[component]
pub fn StatusBar() -> Element {
  rsx! {
    div { class: "flex items-center justify-between h-6 px-3 bg-[var(--surface-container-low)] text-xs uppercase tracking-[0.05em] font-[var(--font-body)] shrink-0",
      div { class: "flex items-center gap-3",
        span { class: "text-[var(--primary)] font-semibold", "SYSTEM_READY_V1.0.4" }
        span { class: "text-[var(--outline)]", "main*" }
        span { class: "text-[var(--outline)]", "Ln 1, Col 1" }
      }
      div { class: "flex items-center gap-3",
        span { class: "text-[var(--outline)]", "Spaces: 4" }
        span { class: "text-[var(--outline)]", "UTF-8" }
        span { class: "text-[var(--outline)]", "Notifications" }
      }
    }
  }
}
