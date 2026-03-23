use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::prelude::*;
use kokoro_client::SpeechRequest;
use voice_agent::AgentBackend as _;
use whisper_client::InferenceClient as _;
use whisper_client::TranscribeRequest;
use widget_bridge::use_ts_widget;

#[component]
pub fn VoiceView(voice_id: String) -> Element {
  let (element_id, widget) = use_ts_widget("voice", serde_json::json!({}));
  let mut pcm_buffer: Signal<Vec<u8>> = use_signal(Vec::new);

  let eid_rsx = element_id.clone();
  use_future(move || async move {
    loop {
      let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };

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
            continue;
          }
          if let Err(e) = process_voice_pipeline(&buffer, widget).await {
            widget.send_update(serde_json::json!({
                "type": "error",
                "message": e.to_string(),
            }));
          }
        },
        Some("start_standby") | Some("cancel") => {
          pcm_buffer.write().clear();
        },
        Some("playback_complete") => {},
        _ => {},
      }
    }
  });

  rsx! {
    div {
      id: "{eid_rsx}",
      class: "w-full h-full bg-[var(--surface-container-lowest)]",
    }
  }
}

async fn process_voice_pipeline(pcm: &[u8], widget: widget_bridge::TsWidgetHandle) -> anyhow::Result<()> {
  let wav = audio_core::wrap_pcm_as_wav(pcm, audio_core::SAMPLE_RATE, audio_core::CHANNELS, audio_core::BITS_PER_SAMPLE);
  let audio_data = B64.encode(&wav);

  let transcription = whisper_client::WHISPER.infer(&TranscribeRequest { audio_data, language: None }).await?;

  let text = transcription.text.trim().to_owned();
  widget.send_update(serde_json::json!({
      "type": "transcript",
      "text": text,
  }));

  if text.is_empty() {
    return Ok(());
  }

  let response = crate::voice_backend::ClaudeCliBackend.query(&text).await?;
  widget.send_update(serde_json::json!({
      "type": "agent_response",
      "text": response,
  }));

  let speech_req = SpeechRequest { text: response, voice: "af_heart".into(), lang_code: "a".into(), speed: 1.0 };
  let wav_bytes = kokoro_client::KOKORO.infer(&speech_req).await?;
  let chunks = audio_core::chunk_wav(&wav_bytes, 32768);
  for chunk in chunks {
    widget.send_update(serde_json::json!({
        "type": "audio_response",
        "data": B64.encode(&chunk),
    }));
  }
  Ok(())
}
