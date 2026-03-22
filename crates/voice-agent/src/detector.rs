use std::sync::LazyLock;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use regex::Regex;
use whisper_client::{InferenceClient, TranscribeRequest, WhisperClient};

const SAMPLE_RATE: u32 = 16000;
const BYTES_PER_SECOND: usize = (SAMPLE_RATE * 2) as usize;
const CHECK_INTERVAL_BYTES: usize = BYTES_PER_SECOND * 2;
const MAX_DETECTION_WINDOW: usize = BYTES_PER_SECOND * 4;
const OVERLAP_BYTES: usize = BYTES_PER_SECOND;

pub const TRIGGER_ACTIVATE: &str = "computer activate";
pub const TRIGGER_RESPOND: &str = "computer respond";

static TRIGGER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)computer activate|computer respond").expect("trigger regex"));

pub struct TriggerDetector {
  detection_buffer: Vec<u8>,
  bytes_since_check: usize,
}

impl Default for TriggerDetector {
  fn default() -> Self {
    Self::new()
  }
}

impl TriggerDetector {
  pub fn new() -> Self {
    Self { detection_buffer: Vec::new(), bytes_since_check: 0 }
  }

  pub fn feed(&mut self, pcm: &[u8]) {
    self.detection_buffer.extend_from_slice(pcm);
    self.bytes_since_check += pcm.len();
    if self.detection_buffer.len() > MAX_DETECTION_WINDOW {
      let trim = self.detection_buffer.len() - MAX_DETECTION_WINDOW;
      self.detection_buffer.drain(..trim);
    }
  }

  pub fn should_check(&self) -> bool {
    self.bytes_since_check >= CHECK_INTERVAL_BYTES
  }

  pub async fn check_trigger(&mut self, trigger: &str, backend: &WhisperClient) -> anyhow::Result<bool> {
    self.bytes_since_check = 0;
    if self.detection_buffer.is_empty() {
      return Ok(false);
    }
    let wav = audio_core::wrap_pcm_as_wav(&self.detection_buffer, SAMPLE_RATE, 1, 16);
    let audio_data = STANDARD.encode(&wav);
    let req = TranscribeRequest { audio_data, language: None };
    let result = backend.infer(&req).await?;
    let transcript = result.text.to_lowercase();
    let found = contains_trigger(&transcript, trigger);
    if !found {
      let keep_from = self.detection_buffer.len().saturating_sub(OVERLAP_BYTES);
      self.detection_buffer.drain(..keep_from);
    }
    Ok(found)
  }

  pub fn reset(&mut self) {
    self.detection_buffer.clear();
    self.bytes_since_check = 0;
  }

  pub fn take_buffer(&mut self) -> Vec<u8> {
    self.bytes_since_check = 0;
    std::mem::take(&mut self.detection_buffer)
  }
}

fn contains_trigger(transcript: &str, trigger: &str) -> bool {
  let normalized = transcript.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect::<String>();
  let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
  normalized.contains(trigger)
}

pub fn strip_triggers(text: &str) -> String {
  TRIGGER_RE.replace_all(text, " ").split_whitespace().collect::<Vec<_>>().join(" ")
}
