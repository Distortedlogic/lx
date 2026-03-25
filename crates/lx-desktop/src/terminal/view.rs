use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use common_pane_tree::TabsState;
use common_pane_tree::{NotificationLevel, PaneNotification};
use common_voice::AgentBackend as _;
use dioxus::logger::tracing::error;
use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use super::use_tabs_state;
use crate::panes::DesktopPane;

pub use super::browser_view::{BrowserNavCtx, BrowserView};

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
      let session = match common_pty::get_or_create(&element_id, 80, 24, Some(&wd), cmd.as_deref()) {
        Ok(s) => s,
        Err(e) => {
          error!("pty session create failed: {e}");
          return;
        },
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
                    Err(RecvError::Closed) => {
                        tabs_state.write().set_notification(
                            &tid_notif,
                            PaneNotification { level: NotificationLevel::Success, message: None },
                        );
                        break;
                    }
                    Err(RecvError::Lagged(_)) => {}
                }
            }
            result = widget.recv::<Value>() => {
                match result {
                    Ok(msg) => match msg["type"].as_str() {
                        Some("input") => {
                            if let Some(data) = msg["data"].as_str()
                                && let Err(e) = session.send_input(data.as_bytes().to_vec()).await { error!("pty send_input failed: {e}"); break; }
                        }
                        Some("resize") => {
                            let cols = msg["cols"].as_u64().unwrap_or(80) as u16;
                            let rows = msg["rows"].as_u64().unwrap_or(24) as u16;
                            if let Err(e) = session.resize(cols, rows) { error!("pty resize failed: {e}"); break; }
                            widget.send_resize();
                        }
                        _ => {}
                    },
                    Err(e) => { error!("terminal widget recv failed: {e}"); break; }
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
pub fn EditorView(editor_id: String, file_path: String, language: Option<String>) -> Element {
  let fp = file_path.clone();
  let content = use_resource(move || {
    let fp = fp.clone();
    async move { if fp.is_empty() { String::new() } else { tokio::fs::read_to_string(&fp).await.unwrap_or_default() } }
  });

  let (element_id, widget) = use_ts_widget("editor", serde_json::json!({}));

  use_effect(move || {
    if let Some(text) = content.value().read().as_ref() {
      widget.send_update(serde_json::json!({ "content": text }));
    }
  });

  let file_path_save = file_path.clone();
  use_future(move || {
    let file_path = file_path_save.clone();
    async move {
      loop {
        let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };
        match msg["type"].as_str() {
          Some("cursor") => {
            let line = msg["line"].as_u64().unwrap_or(1) as u32;
            let col = msg["col"].as_u64().unwrap_or(1) as u32;
            let ctx = use_context::<crate::contexts::status_bar::StatusBarState>();
            ctx.update_cursor(line, col);
          },
          Some("save") => {
            if let Some(text) = msg["content"].as_str() {
              let fp = file_path.clone();
              if !fp.is_empty() {
                let text = text.to_owned();
                let _ = tokio::fs::write(&fp, &text).await;
              }
            }
          },
          _ => {},
        }
      }
    }
  });

  rsx! { div { id: "{element_id}", class: "w-full h-full bg-[var(--surface-container-lowest)]" } }
}

#[component]
pub fn AgentView(agent_id: String, session_id: String, model: String) -> Element {
  let (element_id, widget) = use_ts_widget("agent", serde_json::json!({ "sessionId": session_id, "model": model }));

  use_future(move || async move {
    loop {
      let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };
      match msg["type"].as_str() {
        Some("user_message") => {
          let content = msg["content"].as_str().unwrap_or("").to_owned();
          if content.is_empty() {
            continue;
          }
          match crate::voice_backend::ClaudeCliBackend.query(&content).await {
            Ok(response) => {
              widget.send_update(serde_json::json!({
                "type": "assistant_chunk",
                "text": response,
              }));
              widget.send_update(serde_json::json!({ "type": "assistant_done" }));
            },
            Err(e) => {
              widget.send_update(serde_json::json!({
                "type": "error",
                "message": format!("{e:#}"),
              }));
            },
          }
        },
        Some("tool_decision") => {},
        _ => {},
      }
    }
  });

  rsx! { div { id: "{element_id}", class: "w-full h-full bg-[var(--surface-container)]" } }
}

#[component]
pub fn CanvasView(canvas_id: String, widget_type: String, config: Value) -> Element {
  let (element_id, widget) = use_ts_widget(&widget_type, &config);

  use_future(move || async move {
    loop {
      let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };
      match msg["type"].as_str() {
        Some("content_update") => {},
        Some("interaction") => {},
        _ => {},
      }
    }
  });

  rsx! { div { id: "{element_id}", class: "w-full h-full bg-[var(--surface-container)]" } }
}

#[component]
pub fn ChartView(chart_id: String, chart_json: String, title: Option<String>) -> Element {
  let div_id = use_hook(|| format!("chart-{}", Uuid::new_v4().simple()));
  let id = div_id.clone();
  use_effect(move || {
    let json = chart_json.clone();
    if json.is_empty() {
      return;
    }
    document::eval(&format!("DioxusCharts.initChart('{id}', {json})"));
  });
  let id_drop = div_id.clone();
  use_drop(move || {
    document::eval(&format!("DioxusCharts.disposeChart('{id_drop}')"));
  });
  rsx! {
    div {
      id: "{div_id}",
      class: "w-full h-full min-h-32 bg-[var(--surface-container)]",
    }
  }
}
