use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use common_inference::InferenceClient as _;
use common_kokoro::SpeechRequest;
use common_whisper::TranscribeRequest;
use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;

#[component]
pub fn VoiceBanner() -> Element {
  let (voice_element_id, voice_widget) = use_ts_widget("voice", serde_json::json!({}));
  let (agent_element_id, agent_widget) = use_ts_widget("agent", serde_json::json!({}));
  let mut ctx = use_context::<VoiceContext>();
  use_effect(move || {
    ctx.widget.set(Some(voice_widget));
  });

  use_future(move || async move {
    loop {
      let Ok(msg) = voice_widget.recv::<serde_json::Value>().await else { break };
      match msg["type"].as_str() {
        Some("audio_chunk") => {
          if let Some(data) = msg["data"].as_str()
            && let Ok(bytes) = B64.decode(data)
          {
            ctx.pcm_buffer.write().extend_from_slice(&bytes);
          }
        },
        Some("silence_detected") => {
          let buffer = std::mem::take(&mut *ctx.pcm_buffer.write());
          if buffer.is_empty() {
            voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            continue;
          }
          spawn(async move {
            if let Err(e) = run_pipeline(buffer, voice_widget, agent_widget, ctx).await {
              ctx.transcript.write().push(TranscriptEntry { is_user: false, text: format!("Error: {e}") });
              ctx.pipeline_stage.set(PipelineStage::Idle);
              voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            }
          });
        },
        Some("audio_playing") => {
          if let Some(id) = msg["id"].as_str()
            && let Some(text) = ctx.pending.write().remove(id)
          {
            let mut t = ctx.transcript.write();
            match t.last_mut() {
              Some(entry) if !entry.is_user => entry.text.push_str(&format!(" {text}")),
              _ => t.push(TranscriptEntry { is_user: false, text }),
            }
          }
        },
        Some("rms") => {
          if let Some(level) = msg["level"].as_f64() {
            ctx.rms.set(level as f32);
          }
        },
        Some("status_change") => match msg["status"].as_str() {
          Some("idle") => ctx.status.set(VoiceStatus::Idle),
          Some("listening") => ctx.status.set(VoiceStatus::Listening),
          Some("processing") => ctx.status.set(VoiceStatus::Processing),
          Some("speaking") => ctx.status.set(VoiceStatus::Speaking),
          _ => {},
        },
        Some("start_standby") | Some("cancel") => ctx.pcm_buffer.write().clear(),
        Some("playback_complete") => {},
        _ => {},
      }
    }
  });

  use_future(move || async move {
    loop {
      let Ok(msg) = agent_widget.recv::<serde_json::Value>().await else { break };
      if let Some("user_message") = msg["type"].as_str() {
        let content = msg["content"].as_str().unwrap_or("").to_owned();
        if content.is_empty() {
          continue;
        }
        match crate::voice_backend::query_streaming(&content, |chunk| {
          agent_widget.send_update(serde_json::json!({ "type": "assistant_chunk", "text": chunk }));
        })
        .await
        {
          Ok(_) => {
            agent_widget.send_update(serde_json::json!({ "type": "assistant_done" }));
          },
          Err(e) => {
            agent_widget.send_update(serde_json::json!({ "type": "error", "message": format!("{e:#}") }));
          },
        }
      }
    }
  });

  let current_status = (ctx.status)();
  let is_active = current_status != VoiceStatus::Idle;
  let status_text = current_status.to_string();
  let bar_glow = if is_active { "shadow-[0_0_12px_var(--primary)]" } else { "" };
  let icon = if is_active { "\u{1F534}" } else { "\u{1F512}" };
  let volume = ((ctx.rms)() / 0.3).min(1.0);
  let stage = (ctx.pipeline_stage)();
  let entries = ctx.transcript.read();
  let turn_count = entries.iter().filter(|e| e.is_user).count();
  drop(entries);

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "bg-[var(--surface-container)] px-4 py-2 flex items-center gap-3 shrink-0 {bar_glow}",
        span { class: "text-[var(--primary)] text-sm", "{icon}" }
        span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
          "{status_text}"
        }
        if is_active {
          div { class: "flex items-end gap-[2px] h-4 ml-1",
            span {
              class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
              style: "height: {(volume * 40.0).max(2.0)}%;",
            }
            span {
              class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
              style: "height: {(volume * 70.0).max(2.0)}%;",
            }
            span {
              class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
              style: "height: {(volume * 100.0).max(2.0)}%;",
            }
            span {
              class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
              style: "height: {(volume * 60.0).max(2.0)}%;",
            }
          }
          span { class: "text-[10px] text-[var(--outline)] uppercase tracking-wider",
            "{stage}"
          }
          span { class: "text-[10px] text-[var(--outline)] uppercase tracking-wider",
            "TURNS: {turn_count}"
          }
        }
        div { class: "flex-1" }
        if is_active {
          button {
            class: "border border-[var(--outline)] text-[var(--on-surface)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--surface-container-high)] transition-colors duration-150 font-semibold",
            onclick: move |_| {
                voice_widget.send_update(serde_json::json!({ "type" : "stop_capture" }));
            },
            "STOP"
          }
        } else {
          button {
            class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150 font-semibold",
            onclick: move |_| {
                voice_widget.send_update(serde_json::json!({ "type" : "start_capture" }));
            },
            "PUSH TO TALK"
          }
        }
      }
      div {
        id: "{agent_element_id}",
        class: "flex-1 min-h-0 overflow-hidden",
      }
      div { id: "{voice_element_id}", class: "hidden" }
    }
  }
}

async fn tts(text: &str) -> anyhow::Result<Vec<u8>> {
  let req = SpeechRequest { text: text.to_owned(), voice: "am_michael".into(), lang_code: "a".into(), speed: 1.2 };
  common_kokoro::KOKORO.infer(&req).await
}

async fn run_pipeline(
  pcm: Vec<u8>,
  voice_widget: dioxus_widget_bridge::TsWidgetHandle,
  agent_widget: dioxus_widget_bridge::TsWidgetHandle,
  mut ctx: VoiceContext,
) -> anyhow::Result<()> {
  ctx.pipeline_stage.set(PipelineStage::Transcribing);
  let wav = common_audio::wrap_pcm_as_wav(&pcm, common_audio::SAMPLE_RATE, common_audio::CHANNELS, common_audio::BITS_PER_SAMPLE);
  let transcription = common_whisper::WHISPER.infer(&TranscribeRequest { audio_data: B64.encode(&wav), language: Some("en".into()) }).await?;
  let text = transcription.text.trim().to_owned();
  if text.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
    return Ok(());
  }
  ctx.transcript.write().push(TranscriptEntry { is_user: true, text: text.clone() });

  agent_widget.send_update(serde_json::json!({ "type": "user_display", "text": text }));

  ctx.pipeline_stage.set(PipelineStage::QueryingLlm);
  let response = crate::voice_backend::query_streaming(&text, |chunk| {
    agent_widget.send_update(serde_json::json!({ "type": "assistant_chunk", "text": chunk }));
  })
  .await?;
  agent_widget.send_update(serde_json::json!({ "type": "assistant_done" }));

  if response.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    return Ok(());
  }

  ctx.pipeline_stage.set(PipelineStage::SynthesizingSpeech);
  let wav_bytes = tts(&response).await?;
  ctx.pending.write().insert("r0".to_string(), response);
  voice_widget.send_update(serde_json::json!({
      "type": "audio_response",
      "data": B64.encode(&wav_bytes),
      "id": "r0",
  }));
  ctx.pipeline_stage.set(PipelineStage::Idle);
  Ok(())
}
