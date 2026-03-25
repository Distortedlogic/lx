mod agent_card;
mod pane_area;
mod voice_banner;
mod voice_context;

use dioxus::prelude::*;

use self::agent_card::AgentCard;
use self::pane_area::PaneArea;
use self::voice_banner::VoiceBanner;
use self::voice_context::VoiceContext;

#[component]
pub fn Agents() -> Element {
  let ctx = VoiceContext::provide();
  let session_short = &crate::voice_backend::SESSION_ID[..8];
  let status_text = (ctx.status)().to_string();

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "shrink-0 p-4 flex flex-col gap-4 border-b border-[var(--outline-variant)]/15 max-h-[40%] overflow-auto",
        div { class: "flex items-center justify-between",
          div {
            h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]",
              "AGENT_MANAGER"
            }
            p { class: "text-xs text-[var(--outline)] uppercase tracking-wider mt-1",
              "SESSION: {session_short}"
            }
          }
          span { class: "text-xs text-[var(--outline)] uppercase tracking-wider",
            "STATUS: {status_text}"
          }
        }
        VoiceBanner {}
        AgentCard {}
      }
      div { class: "flex-1 min-h-0", PaneArea {} }
    }
  }
}
