use dioxus::prelude::*;

#[component]
pub fn VoiceBanner() -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container)] rounded-lg p-4 flex items-center gap-4",
      div { class: "bg-[var(--primary-container)]/20 rounded-full p-4 shrink-0",
        span { class: "text-2xl", "\u{1F3A4}" }
      }
      div { class: "flex-1 min-w-0",
        p { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]", "VOICE INTERACTION ACTIVE" }
        p { class: "text-xs text-[var(--outline)] mt-0.5", "LISTENING FOR COMMANDS..." }
      }
      div { class: "flex items-center gap-1 h-8",
        for i in 0..8 {
          {
              let height = match i % 4 {
                  0 => "h-3",
                  1 => "h-5",
                  2 => "h-7",
                  _ => "h-4",
              };
              let delay = format!("animation-delay: {}ms", i * 120);
              rsx! {
                div {
                  class: "w-1 bg-[var(--primary)] rounded-full animate-pulse {height}",
                  style: "{delay}",
                }
              }
          }
        }
      }
      div { class: "flex items-center gap-2 shrink-0",
        button { class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150",
          "PUSH TO TALK"
        }
        button { class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150",
          "CONFIGURE AUDIO"
        }
      }
    }
  }
}
