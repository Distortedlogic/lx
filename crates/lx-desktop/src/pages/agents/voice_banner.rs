use std::mem;

use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceDataStoreExt, VoiceStatus};
use super::{voice_pipeline, voice_porcupine};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::logger::tracing::error;
use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;

#[component]
pub fn VoiceBanner() -> Element {
  let (voice_element_id, voice_widget) = use_ts_widget("voice", serde_json::json!({}));
  let (agent_element_id, agent_widget) = use_ts_widget("agent", serde_json::json!({}));
  let mut ctx = use_context::<VoiceContext>();
  use_hook(|| ctx.widget.set(Some(voice_widget)));

  use_future(move || async move {
    loop {
      let Ok(msg) = voice_widget.recv::<serde_json::Value>().await else {
        break;
      };
      match msg["type"].as_str() {
        Some("audio_chunk") => {
          if let Some(data) = msg["data"].as_str()
            && let Ok(bytes) = B64.decode(data)
          {
            if voice_porcupine::is_available() {
              let samples: Vec<i16> = bytes.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();
              if voice_porcupine::feed_samples(&samples) {
                handle_keyword_detected(voice_widget, ctx);
              }
            }
            if ctx.data.status().cloned() == VoiceStatus::Listening {
              ctx.data.pcm_buffer().write().extend_from_slice(&bytes);
            }
          }
        },
        Some("silence_detected") => {
          let stage = ctx.data.pipeline_stage().cloned();
          if stage != PipelineStage::Idle {
            ctx.data.pcm_buffer().write().clear();
            continue;
          }

          let buffer = mem::take(&mut *ctx.data.pcm_buffer().write());
          if buffer.is_empty() {
            if ctx.data.always_listen().cloned() {
              voice_widget.send_update(serde_json::json!({ "type": "resume_standby" }));
            } else {
              voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            }
            continue;
          }

          spawn(async move {
            let result = handle_utterance(buffer, agent_widget, ctx).await;
            if ctx.data.barge_in().cloned() {
              ctx.data.barge_in().set(false);
              return;
            }
            if let Err(e) = &result {
              error!("voice: pipeline error: {e}");
              ctx.data.transcript().push(TranscriptEntry { is_user: false, text: format!("Error: {e}") });
              ctx.data.pipeline_stage().set(PipelineStage::Idle);
            }
            if ctx.data.always_listen().cloned() {
              voice_widget.send_update(serde_json::json!({ "type": "resume_standby" }));
            } else {
              voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            }
          });
        },
        Some("rms") => {
          if let Some(level) = msg["level"].as_f64() {
            ctx.data.rms().set(level as f32);
          }
        },
        Some("status_change") => match msg["status"].as_str() {
          Some("idle") => ctx.data.status().set(VoiceStatus::Idle),
          Some("standby") => ctx.data.status().set(VoiceStatus::Standby),
          Some("listening") => ctx.data.status().set(VoiceStatus::Listening),
          Some("processing") => ctx.data.status().set(VoiceStatus::Processing),
          Some("speaking") => ctx.data.status().set(VoiceStatus::Speaking),
          _ => {},
        },
        Some("start_standby") | Some("cancel") => {
          ctx.data.pcm_buffer().write().clear();
        },
        _ => {},
      }
    }
  });

  use_future(move || async move {
    loop {
      let Ok(msg) = agent_widget.recv::<serde_json::Value>().await else {
        break;
      };
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

  let current_status = ctx.data.status().cloned();
  let is_active = current_status != VoiceStatus::Idle;
  let status_text = current_status.to_string();
  let bar_glow = if is_active { "shadow-[0_0_12px_var(--primary)]" } else { "" };
  let icon = match current_status {
    VoiceStatus::Standby => "\u{1F7E2}",
    _ if is_active => "\u{1F534}",
    _ => "\u{1F512}",
  };
  let volume = (ctx.data.rms().cloned() / 0.3).min(1.0);
  let stage = ctx.data.pipeline_stage().cloned();
  let transcript_store = ctx.data.transcript();
  let entries = transcript_store.read();
  let turn_count = entries.iter().filter(|e| e.is_user).count();
  drop(entries);
  let always_listen = ctx.data.always_listen().cloned();
  let porcupine_available = voice_porcupine::is_available();

  rsx! {
    div { class: "flex flex-col h-full",
      div {
        class: "bg-[var(--surface-container)] px-4 py-2 flex items-center gap-3 shrink-0",
        class: "{bar_glow}",
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
        if porcupine_available {
          button {
            class: "border border-[var(--outline-variant)] text-[var(--on-surface-variant)] rounded px-3 py-1.5 text-xs uppercase hover:bg-[var(--surface-container-high)] transition-colors duration-150 font-semibold",
            onclick: move |_| {
                if always_listen {
                    ctx.data.always_listen().set(false);
                    voice_widget.send_update(serde_json::json!({ "type" : "stop_capture" }));
                } else {
                    ctx.data.always_listen().set(true);
                    voice_widget
                        .send_update(serde_json::json!({ "type" : "start_standby_listen" }));
                }
            },
            if always_listen {
              "ALWAYS LISTEN: ON"
            } else {
              "ALWAYS LISTEN: OFF"
            }
          }
        }
        if is_active {
          button {
            class: "border border-[var(--outline)] text-[var(--on-surface)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--surface-container-high)] transition-colors duration-150 font-semibold",
            onclick: move |_| {
                ctx.data.always_listen().set(false);
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

fn handle_keyword_detected(voice_widget: dioxus_widget_bridge::TsWidgetHandle, ctx: VoiceContext) {
  let status = ctx.data.status().cloned();
  if status != VoiceStatus::Standby && status != VoiceStatus::Speaking {
    return;
  }
  if status == VoiceStatus::Speaking {
    ctx.data.barge_in().set(true);
    voice_pipeline::stop_active_playback();
  }
  ctx.data.pcm_buffer().write().clear();
  voice_porcupine::reset_buffer();
  voice_widget.send_update(serde_json::json!({ "type": "start_recording" }));
  spawn(async move {
    let _ = voice_pipeline::play_wav(voice_pipeline::generate_ack_tone()).await;
  });
}

async fn handle_utterance(pcm: Vec<u8>, agent_widget: dioxus_widget_bridge::TsWidgetHandle, ctx: VoiceContext) -> anyhow::Result<()> {
  ctx.data.pipeline_stage().set(PipelineStage::Transcribing);
  let text = voice_pipeline::transcribe(&pcm).await?;
  if text.is_empty() {
    ctx.data.pipeline_stage().set(PipelineStage::Idle);
    return Ok(());
  }
  voice_pipeline::run_pipeline(&text, agent_widget, ctx).await
}
