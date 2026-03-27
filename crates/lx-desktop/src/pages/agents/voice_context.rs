use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum VoiceStatus {
  Idle,
  Standby,
  Listening,
  Processing,
  Speaking,
}

impl std::fmt::Display for VoiceStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Idle => write!(f, "IDLE"),
      Self::Standby => write!(f, "STANDBY"),
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

#[derive(Store, Clone, PartialEq)]
pub struct VoiceData {
  pub status: VoiceStatus,
  pub transcript: Vec<TranscriptEntry>,
  pub pcm_buffer: Vec<u8>,
  pub rms: f32,
  pub pipeline_stage: PipelineStage,
  pub always_listen: bool,
  pub barge_in: bool,
}

#[derive(Clone, Copy)]
pub struct VoiceContext {
  pub data: Store<VoiceData>,
  pub widget: Signal<Option<dioxus_widget_bridge::TsWidgetHandle>>,
}
