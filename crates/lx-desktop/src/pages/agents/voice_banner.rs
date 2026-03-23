use dioxus::prelude::*;

#[component]
pub fn VoiceBanner() -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container)] rounded-lg px-4 py-2 flex items-center gap-3",
      span { class: "text-[var(--primary)] text-sm", "\u{1F512}" }
      span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
        "SYSTEM_LISTENING"
      }
      span { class: "text-[var(--primary)] text-sm ml-1", "\u{2581}\u{2582}\u{2583}\u{2584}" }
      div { class: "flex-1" }
      button { class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150 font-semibold",
        "PUSH TO TALK"
      }
    }
  }
}
