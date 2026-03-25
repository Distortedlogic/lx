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

#[derive(Clone)]
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
  pub fn new() -> Self {
    Self {
      status: use_signal(|| VoiceStatus::Idle),
      transcript: use_signal(Vec::new),
      pending: use_signal(HashMap::new),
      pcm_buffer: use_signal(Vec::new),
      rms: use_signal(|| 0.0),
      pipeline_stage: use_signal(|| PipelineStage::Idle),
      widget: use_signal(|| None),
    }
  }
}
