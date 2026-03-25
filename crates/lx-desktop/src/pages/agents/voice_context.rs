use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum VoiceStatus {
  Idle,
  Listening,
  Processing,
  Speaking,
}

impl std::fmt::Display for VoiceStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Idle => write!(f, "IDLE"),
      Self::Listening => write!(f, "LISTENING"),
      Self::Processing => write!(f, "PROCESSING"),
      Self::Speaking => write!(f, "SPEAKING"),
    }
  }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PipelineStage {
  Idle,
  Transcribing,
  QueryingLlm,
  SynthesizingSpeech,
}

impl std::fmt::Display for PipelineStage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Idle => write!(f, ""),
      Self::Transcribing => write!(f, "TRANSCRIBING"),
      Self::QueryingLlm => write!(f, "QUERYING_LLM"),
      Self::SynthesizingSpeech => write!(f, "SYNTHESIZING_SPEECH"),
    }
  }
}

#[derive(Clone, PartialEq)]
pub struct TranscriptEntry {
  pub is_user: bool,
  pub text: String,
}

#[derive(Clone, Copy)]
pub struct VoiceContext {
  pub status: Signal<VoiceStatus>,
  pub transcript: Signal<Vec<TranscriptEntry>>,
  pub pending: Signal<HashMap<String, String>>,
  pub pcm_buffer: Signal<Vec<u8>>,
  pub rms: Signal<f32>,
  pub pipeline_stage: Signal<PipelineStage>,
  pub widget: Signal<Option<dioxus_widget_bridge::TsWidgetHandle>>,
}

impl VoiceContext {
  pub fn provide() -> Self {
    let ctx = Self {
      status: Signal::new(VoiceStatus::Idle),
      transcript: Signal::new(Vec::new()),
      pending: Signal::new(HashMap::new()),
      pcm_buffer: Signal::new(Vec::new()),
      rms: Signal::new(0.0),
      pipeline_stage: Signal::new(PipelineStage::Idle),
      widget: Signal::new(None),
    };
    use_context_provider(|| ctx);
    ctx
  }
}
