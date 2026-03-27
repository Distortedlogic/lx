mod pane_area;
mod voice_banner;
mod voice_context;
mod voice_pipeline;
mod voice_porcupine;

use dioxus::prelude::*;

use self::pane_area::PaneArea;
use self::voice_banner::VoiceBanner;
use self::voice_context::{PipelineStage, VoiceContext, VoiceData, VoiceStatus};

#[component]
pub fn Agents() -> Element {
  let data = use_store(|| VoiceData {
    status: VoiceStatus::Idle,
    transcript: Vec::new(),
    pcm_buffer: Vec::new(),
    rms: 0.0,
    pipeline_stage: PipelineStage::Idle,
    always_listen: false,
    barge_in: false,
  });
  let ctx = VoiceContext { data, widget: Signal::new(None) };
  use_context_provider(|| ctx);

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "flex-1 min-h-0 border-b border-[var(--outline-variant)]/15",
        VoiceBanner {}
      }
      div { class: "flex-1 min-h-0", PaneArea {} }
    }
  }
}
