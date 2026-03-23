use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::prelude::*;
use kokoro_client::SpeechRequest;
use pane_tree::TabsState;
use pane_tree::{NotificationLevel, PaneNotification};
use voice_agent::AgentBackend as _;
use whisper_client::InferenceClient as _;
use whisper_client::TranscribeRequest;
use widget_bridge::use_ts_widget;

use super::use_tabs_state;
use crate::panes::DesktopPane;

#[component]
pub fn TerminalView(terminal_id: String, working_dir: String, command: Option<String>) -> Element {
  let (element_id, widget) = use_ts_widget("terminal", serde_json::json!({}));
  let mut tabs_state: Signal<TabsState<DesktopPane>> = use_tabs_state();
  let tid_notif = terminal_id.clone();

  let eid_rsx = element_id.clone();
  use_future(move || {
    let element_id = element_id.clone();
    let tid_notif = tid_notif.clone();
    let wd = working_dir.clone();
    let cmd = command.clone();
    async move {
      let session = match pty_mux::get_or_create(&element_id, 80, 24, Some(&wd), cmd.as_deref()) {
        Ok(s) => s,
        Err(_e) => return,
      };

      let (initial, mut rx) = session.subscribe();
      if !initial.is_empty() {
        widget.send_update(B64.encode(&initial));
      }

      loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(bytes) => {
                        widget.send_update(B64.encode(&bytes));
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tabs_state.write().set_notification(
                            &tid_notif,
                            PaneNotification { level: NotificationLevel::Success, message: None },
                        );
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                }
            }
            result = widget.recv::<serde_json::Value>() => {
                match result {
                    Ok(msg) => match msg["type"].as_str() {
                        Some("input") => {
                            if let Some(data) = msg["data"].as_str() {
                                let _ = session.send_input(data.as_bytes().to_vec()).await;
                            }
                        }
                        Some("resize") => {
                            let cols = msg["cols"].as_u64().unwrap_or(80) as u16;
                            let rows = msg["rows"].as_u64().unwrap_or(24) as u16;
                            let _ = session.resize(cols, rows);
                            widget.send_resize();
                        }
                        _ => {}
                    },
                    Err(_) => break,
                }
            }
        }
      }
    }
  });

  rsx! {
    div {
      id: "{eid_rsx}",
      class: "w-full h-full bg-[var(--surface-container-lowest)] overflow-hidden p-[1.1rem]",
    }
  }
}

#[component]
pub fn BrowserView(browser_id: String, url: String, devtools: bool) -> Element {
  let (element_id, _widget) = use_ts_widget("browser", serde_json::json!({ "url": url, "mode": "cdp" }));

  rsx! {
    div {
      id: "{element_id}",
      class: "w-full h-full bg-[var(--surface-container)]",
    }
  }
}

#[component]
pub fn EditorView(editor_id: String, file_path: String, language: Option<String>) -> Element {
  let lang = language.unwrap_or_else(|| "plaintext".into());
  let content = if file_path.is_empty() { String::new() } else { std::fs::read_to_string(&file_path).unwrap_or_default() };
  let (element_id, _widget) = use_ts_widget(
    "editor",
    serde_json::json!({
        "content": content,
        "language": lang,
        "filePath": file_path,
    }),
  );

  rsx! {
    div {
      id: "{element_id}",
      class: "w-full h-full bg-[var(--surface-container-lowest)]",
    }
  }
}

#[component]
pub fn AgentView(agent_id: String, session_id: String, model: String) -> Element {
  let (element_id, _widget) = use_ts_widget("agent", serde_json::json!({}));

  rsx! {
    div {
      id: "{element_id}",
      class: "w-full h-full bg-[var(--surface-container)]",
    }
  }
}

#[component]
pub fn CanvasView(canvas_id: String, widget_type: String, config: serde_json::Value) -> Element {
  let (element_id, _widget) = use_ts_widget(&widget_type, &config);

  rsx! {
    div {
      id: "{element_id}",
      class: "w-full h-full bg-[var(--surface-container)]",
    }
  }
}

#[component]
pub fn ChartView(chart_id: String, chart_json: String, title: Option<String>) -> Element {
  let div_id = use_hook(|| format!("chart-{}", uuid::Uuid::new_v4().simple()));
  let id = div_id.clone();
  use_effect(move || {
    let json = chart_json.clone();
    if json.is_empty() {
      return;
    }
    document::eval(&format!("DxCharts.initChart('{id}', {json})"));
  });
  let id_drop = div_id.clone();
  use_drop(move || {
    document::eval(&format!("DxCharts.disposeChart('{id_drop}')"));
  });
  rsx! {
    div {
      id: "{div_id}",
      class: "w-full h-full min-h-32 bg-[var(--surface-container)]",
    }
  }
}

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
