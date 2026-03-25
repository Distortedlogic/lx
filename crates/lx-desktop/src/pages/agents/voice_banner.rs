use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
use crate::layout::shell::TerminalSpawnRequest;
use crate::panes::DesktopPane;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use common_inference::InferenceClient as _;
use common_kokoro::SpeechRequest;
use common_pane_tree::PaneNode;
use common_whisper::TranscribeRequest;
use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;
use tokio::sync::mpsc;

#[component]
pub fn VoiceBanner() -> Element {
  let (element_id, widget) = use_ts_widget("voice", serde_json::json!({}));
  let mut ctx = use_context::<VoiceContext>();
  let spawn_tx = use_context::<mpsc::UnboundedSender<TerminalSpawnRequest>>();
  use_effect(move || {
    ctx.widget.set(Some(widget));
  });

  use_future(move || async move {
    loop {
      let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };
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
            widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            continue;
          }
          spawn(async move {
            if let Err(e) = run_pipeline(buffer, widget, ctx, spawn_tx.clone()).await {
              ctx.transcript.write().push(TranscriptEntry { is_user: false, text: format!("Error: {e}") });
              ctx.pipeline_stage.set(PipelineStage::Idle);
              widget.send_update(serde_json::json!({ "type": "stop_capture" }));
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

  let current_status = (ctx.status)();
  let is_active = current_status != VoiceStatus::Idle;
  let status_text = current_status.to_string();
  let bar_glow = if is_active { "shadow-[0_0_12px_var(--primary)]" } else { "" };
  let icon = if is_active { "\u{1F534}" } else { "\u{1F512}" };
  let button_label = if (ctx.status)() == VoiceStatus::Idle { "PUSH TO TALK" } else { "STOP" };
  let volume = ((ctx.rms)() / 0.3).min(1.0);

  rsx! {
    div { class: "flex flex-col gap-2",
      div { class: "bg-[var(--surface-container)] rounded-lg px-4 py-2 flex items-center gap-3 {bar_glow}",
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
        }
        div { class: "flex-1" }
        button {
          class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150 font-semibold",
          onclick: move |_| {
              if (ctx.status)() == VoiceStatus::Idle {
                  widget.send_update(serde_json::json!({ "type" : "start_capture" }));
              } else {
                  widget.send_update(serde_json::json!({ "type" : "stop_capture" }));
              }
          },
          "{button_label}"
        }
      }
      div { id: "{element_id}", class: "hidden" }
    }
  }
}

fn split_sentences(text: &str) -> Vec<String> {
  let mut sentences = Vec::new();
  let mut current = String::new();
  for ch in text.chars() {
    current.push(ch);
    if matches!(ch, '.' | '!' | '?') {
      let trimmed = current.trim().to_owned();
      if !trimmed.is_empty() {
        sentences.push(trimmed);
      }
      current.clear();
    }
  }
  let trimmed = current.trim().to_owned();
  if !trimmed.is_empty() {
    sentences.push(trimmed);
  }
  sentences
}

async fn tts(text: &str) -> anyhow::Result<Vec<u8>> {
  let req = SpeechRequest { text: text.to_owned(), voice: "am_michael".into(), lang_code: "a".into(), speed: 1.0 };
  common_kokoro::KOKORO.infer(&req).await
}

fn shell_escape(s: &str) -> String {
  let mut escaped = String::with_capacity(s.len() + 10);
  escaped.push('\'');
  for ch in s.chars() {
    if ch == '\'' {
      escaped.push_str("'\\''");
    } else {
      escaped.push(ch);
    }
  }
  escaped.push('\'');
  escaped
}

async fn run_pipeline(
  pcm: Vec<u8>,
  widget: dioxus_widget_bridge::TsWidgetHandle,
  mut ctx: VoiceContext,
  spawn_tx: mpsc::UnboundedSender<TerminalSpawnRequest>,
) -> anyhow::Result<()> {
  ctx.pipeline_stage.set(PipelineStage::Transcribing);
  let wav = common_audio::wrap_pcm_as_wav(&pcm, common_audio::SAMPLE_RATE, common_audio::CHANNELS, common_audio::BITS_PER_SAMPLE);
  let transcription = common_whisper::WHISPER.infer(&TranscribeRequest { audio_data: B64.encode(&wav), language: Some("en".into()) }).await?;
  let text = transcription.text.trim().to_owned();
  if text.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    widget.send_update(serde_json::json!({ "type": "stop_capture" }));
    return Ok(());
  }
  ctx.transcript.write().push(TranscriptEntry { is_user: true, text: text.clone() });

  ctx.pipeline_stage.set(PipelineStage::QueryingLlm);

  let escaped = shell_escape(&text);
  let system_escaped = shell_escape(crate::voice_backend::SYSTEM_PROMPT);
  let session_id = &*crate::voice_backend::SESSION_ID;
  let session_created = crate::voice_backend::SESSION_CREATED.load(std::sync::atomic::Ordering::Relaxed);
  let cmd = if session_created {
    format!("claude -p {escaped} --system-prompt {system_escaped} --resume {session_id}")
  } else {
    format!("claude -p {escaped} --system-prompt {system_escaped} --session-id {session_id}")
  };

  let terminal_id = uuid::Uuid::new_v4().to_string();
  let session = common_pty::get_or_create(&terminal_id, 80, 24, Some("."), Some(&cmd)).map_err(|e| anyhow::anyhow!("pty create failed: {e}"))?;
  let (_initial, mut rx) = session.subscribe();

  let mut turn_count = ctx.voice_turn_count;
  let count = turn_count() + 1;
  turn_count.set(count);
  let title = format!("Voice Turn {count}");
  let pane = PaneNode::Leaf(DesktopPane::Terminal { id: terminal_id.clone(), working_dir: ".".into(), command: Some(cmd.clone()), name: Some(title.clone()) });

  let _ = spawn_tx.send(TerminalSpawnRequest { id: terminal_id.clone(), title, pane });

  let mut accumulated = Vec::new();
  loop {
    match rx.recv().await {
      Ok(bytes) => accumulated.extend_from_slice(&bytes),
      Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
      Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
    }
  }

  crate::voice_backend::SESSION_CREATED.store(true, std::sync::atomic::Ordering::Relaxed);

  let stripped = strip_ansi_escapes::strip(&accumulated);
  let response = String::from_utf8_lossy(&stripped).trim().to_owned();
  if response.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    return Ok(());
  }

  let sentences = split_sentences(&response);
  if sentences.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    return Ok(());
  }

  ctx.pipeline_stage.set(PipelineStage::SynthesizingSpeech);
  let mut next_tts: Option<tokio::task::JoinHandle<anyhow::Result<Vec<u8>>>> = {
    let s = sentences[0].clone();
    Some(tokio::spawn(async move { tts(&s).await }))
  };

  for (i, sentence) in sentences.iter().enumerate() {
    let Some(handle) = next_tts.take() else { break };
    let wav_bytes = handle.await??;

    if i + 1 < sentences.len() {
      let s = sentences[i + 1].clone();
      next_tts = Some(tokio::spawn(async move { tts(&s).await }));
    }

    let id = format!("s{i}");
    ctx.pending.write().insert(id.clone(), sentence.clone());
    widget.send_update(serde_json::json!({
        "type": "audio_response",
        "data": B64.encode(&wav_bytes),
        "id": id,
    }));
  }
  ctx.pipeline_stage.set(PipelineStage::Idle);
  Ok(())
}
