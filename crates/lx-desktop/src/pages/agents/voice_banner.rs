use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use common_inference::InferenceClient as _;
use common_kokoro::SpeechRequest;
use common_voice::AgentBackend as _;
use common_whisper::TranscribeRequest;
use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;

#[derive(Clone, Copy, PartialEq)]
enum VoiceStatus {
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

#[derive(Clone)]
struct TranscriptEntry {
  is_user: bool,
  text: String,
}

#[component]
pub fn VoiceBanner() -> Element {
  let (element_id, widget) = use_ts_widget("voice", serde_json::json!({}));
  let mut status: Signal<VoiceStatus> = use_signal(|| VoiceStatus::Idle);
  let mut pcm_buffer: Signal<Vec<u8>> = use_signal(Vec::new);
  let mut transcript: Signal<Vec<TranscriptEntry>> = use_signal(Vec::new);

  use_future(move || async move {
    loop {
      let Ok(msg) = widget.recv::<serde_json::Value>().await else {
        break;
      };
      match msg["type"].as_str() {
        Some("audio_chunk") => {
          if let Some(data) = msg["data"].as_str()
            && let Ok(bytes) = B64.decode(data)
          {
            pcm_buffer.write().extend_from_slice(&bytes);
          }
        },
        Some("silence_detected") => {
          let buffer = std::mem::take(&mut *pcm_buffer.write());
          if buffer.is_empty() {
            widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            continue;
          }
          match process_voice_pipeline(&buffer, widget, transcript).await {
            Ok(true) => {},
            Ok(false) => {
              widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            },
            Err(e) => {
              transcript.write().push(TranscriptEntry { is_user: false, text: format!("Error: {e}") });
              widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            },
          }
        },
        Some("status_change") => match msg["status"].as_str() {
          Some("idle") => status.set(VoiceStatus::Idle),
          Some("listening") => status.set(VoiceStatus::Listening),
          Some("processing") => status.set(VoiceStatus::Processing),
          Some("speaking") => status.set(VoiceStatus::Speaking),
          _ => {},
        },
        Some("start_standby") | Some("cancel") => {
          pcm_buffer.write().clear();
        },
        Some("playback_complete") => {},
        _ => {},
      }
    }
  });

  let current_status = status();
  let is_active = current_status != VoiceStatus::Idle;
  let status_text = current_status.to_string();
  let entries = transcript.read().clone();
  let bar_glow = if is_active { "shadow-[0_0_12px_var(--primary)]" } else { "" };
  let icon = if is_active { "\u{1F534}" } else { "\u{1F512}" };
  let button_label = if status() == VoiceStatus::Idle { "PUSH TO TALK" } else { "STOP" };

  rsx! {
    div { class: "flex flex-col gap-2",
      div { class: "bg-[var(--surface-container)] rounded-lg px-4 py-2 flex items-center gap-3 {bar_glow}",
        span { class: "text-[var(--primary)] text-sm", "{icon}" }
        span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
          "{status_text}"
        }
        if is_active {
          span { class: "text-[var(--primary)] text-sm ml-1 animate-pulse",
            "\u{2581}\u{2582}\u{2583}\u{2584}"
          }
        }
        div { class: "flex-1" }
        button {
          class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150 font-semibold",
          onclick: move |_| {
              if status() == VoiceStatus::Idle {
                  widget.send_update(serde_json::json!({ "type" : "start_capture" }));
              } else {
                  widget.send_update(serde_json::json!({ "type" : "stop_capture" }));
              }
          },
          "{button_label}"
        }
      }
      if !entries.is_empty() {
        div { class: "bg-[var(--surface-container-lowest)] rounded-lg px-4 py-3 max-h-48 overflow-y-auto text-sm space-y-1",
          for entry in entries.iter() {
            div { class: if entry.is_user { "text-[#64b5f6]" } else { "text-[#81c784]" },
              if entry.is_user {
                "You: {entry.text}"
              } else {
                "Agent: {entry.text}"
              }
            }
          }
        }
      }
      div { id: "{element_id}", class: "hidden" }
    }
  }
}

async fn process_voice_pipeline(
  pcm: &[u8],
  widget: dioxus_widget_bridge::TsWidgetHandle,
  mut transcript: Signal<Vec<TranscriptEntry>>,
) -> anyhow::Result<bool> {
  let wav = common_audio::wrap_pcm_as_wav(pcm, common_audio::SAMPLE_RATE, common_audio::CHANNELS, common_audio::BITS_PER_SAMPLE);
  let audio_data = B64.encode(&wav);
  let transcription = common_whisper::WHISPER.infer(&TranscribeRequest { audio_data, language: None }).await?;
  let text = transcription.text.trim().to_owned();
  if text.is_empty() {
    return Ok(false);
  }
  transcript.write().push(TranscriptEntry { is_user: true, text: text.clone() });
  let response = crate::voice_backend::ClaudeCliBackend.query(&text).await?;
  transcript.write().push(TranscriptEntry { is_user: false, text: response.clone() });
  let speech_req = SpeechRequest { text: response, voice: "af_heart".into(), lang_code: "a".into(), speed: 1.0 };
  let wav_bytes = common_kokoro::KOKORO.infer(&speech_req).await?;
  let chunks = common_audio::chunk_wav(&wav_bytes, 32768);
  for chunk in chunks {
    widget.send_update(serde_json::json!({
        "type": "audio_response",
        "data": B64.encode(&chunk),
    }));
  }
  Ok(true)
}
