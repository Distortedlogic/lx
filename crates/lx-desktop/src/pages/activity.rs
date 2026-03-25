use crate::contexts::activity_log::ActivityLog;
use dioxus::prelude::*;

#[component]
pub fn Activity() -> Element {
  let log = use_context::<ActivityLog>();
  let events = log.events.read();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex items-center justify-between",
        h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]",
          "ACTIVITY_LOG"
        }
        span { class: "text-xs text-[var(--outline)] uppercase tracking-wider",
          "{events.len()} EVENTS"
        }
      }
      if events.is_empty() {
        div { class: "flex-1 flex items-center justify-center",
          p { class: "text-sm text-[var(--outline)]", "No activity recorded yet" }
        }
      } else {
        div { class: "bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] overflow-hidden",
          div { class: "flex text-[10px] uppercase tracking-wider text-[var(--on-surface-variant)] py-3 px-4 border-b border-[var(--outline-variant)] bg-[var(--surface-container-high)]",
            span { class: "w-32 shrink-0", "TIMESTAMP" }
            span { class: "w-24 shrink-0", "KIND" }
            span { class: "flex-1", "MESSAGE" }
          }
          div { class: "flex flex-col max-h-[calc(100vh-12rem)] overflow-y-auto",
            for event in events.iter() {
              div { class: "flex items-center px-4 py-2.5 border-b border-[var(--outline-variant)]/15 hover:bg-[var(--surface-container)] transition-colors duration-150 text-xs",
                span { class: "w-32 shrink-0 text-[var(--outline)] font-mono",
                  "{event.timestamp}"
                }
                span { class: "w-24 shrink-0 text-[var(--primary)] uppercase font-semibold",
                  "{event.kind}"
                }
                span { class: "flex-1 text-[var(--on-surface-variant)]",
                  "{event.message}"
                }
              }
            }
          }
        }
      }
    }
  }
}
