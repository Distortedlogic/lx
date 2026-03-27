mod pane_area;
mod voice_banner;
mod voice_context;
mod voice_pipeline;
mod voice_porcupine;

use dioxus::prelude::*;

use self::pane_area::PaneArea;
use self::voice_banner::VoiceBanner;
use self::voice_context::VoiceContext;

#[component]
pub fn Agents() -> Element {
  let _ctx = VoiceContext::provide();

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "flex-1 min-h-0 border-b border-[var(--outline-variant)]/15",
        VoiceBanner {}
      }
      div { class: "flex-1 min-h-0", PaneArea {} }
    }
  }
}
