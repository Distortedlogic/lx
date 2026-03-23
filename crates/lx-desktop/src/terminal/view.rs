use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::logger::tracing::error;
use dioxus::prelude::*;
use pane_tree::TabsState;
use pane_tree::{NotificationLevel, PaneNotification};
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc;
use tokio::time::interval;
use uuid::Uuid;
use widget_bridge::use_ts_widget;

use super::use_tabs_state;
use crate::panes::DesktopPane;

pub use super::voice_view::VoiceView;

#[derive(Clone)]
pub struct BrowserNavCtx {
  pub tx: mpsc::UnboundedSender<String>,
  pub rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>,
  pub current_url: Signal<String>,
}

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
pub fn BrowserView(browser_id: String, url: String, devtools: bool) -> Element {
  let (element_id, widget) = use_ts_widget("browser", serde_json::json!({ "url": url, "mode": "cdp" }));

  let eid_rsx = element_id.clone();
  let bid_drop = browser_id.clone();
  let nav_ctx: BrowserNavCtx = use_context();
  use_future(move || {
    let browser_id = browser_id.clone();
    let url = url.clone();
    let mut nav_ctx = nav_ctx.clone();
    async move {
      let session = match browser_cdp::get_or_create_session(&browser_id).await {
        Ok(s) => s,
        Err(e) => {
          error!("browser session create failed: {e:#}");
          widget.send_update(serde_json::json!({"error": format!("{e:#}")}));
          return;
        },
      };

      if !url.is_empty() && url != "about:blank" {
        match session.navigate(&url).await {
          Ok((final_url, _)) => nav_ctx.current_url.set(final_url),
          Err(e) => {
            error!("browser navigate failed: {e}");
            return;
          },
        }
      }

      let Some(mut nav_rx) = nav_ctx.rx.lock().unwrap().take() else {
        return;
      };

      let mut interval = interval(Duration::from_millis(500));

      loop {
        tokio::select! {
            _ = interval.tick() => {
                match session.screenshot().await {
                    Ok(b64) => widget.send_update(b64),
                    Err(e) => { error!("browser screenshot failed: {e}"); break; }
                }
            }
            result = widget.recv::<Value>() => {
                match result {
                    Ok(msg) => match msg["type"].as_str() {
                        Some("click") => {
                            let (Some(x), Some(y)) = (msg["x"].as_f64(), msg["y"].as_f64()) else { continue; };
                            if let Err(e) = session.click(x, y).await { error!("browser click failed: {e}"); break; }
                        }
                        Some("type") => {
                            if let Some(text) = msg["text"].as_str()
                                && let Err(e) = session.type_text(text).await { error!("browser type_text failed: {e}"); break; }
                        }
                        Some("navigate") => {
                            if let Some(nav_url) = msg["url"].as_str() {
                                match session.navigate(nav_url).await {
                                    Ok((final_url, _)) => nav_ctx.current_url.set(final_url),
                                    Err(e) => { error!("browser navigate failed: {e}"); break; }
                                }
                            }
                        }
                        Some("back") => {
                            if let Err(e) = session.go_back().await { error!("browser go_back failed: {e}"); break; }
                            match session.current_url().await { Ok(url) => nav_ctx.current_url.set(url), Err(e) => error!("browser current_url failed: {e}") }
                        }
                        Some("forward") => {
                            if let Err(e) = session.go_forward().await { error!("browser go_forward failed: {e}"); break; }
                            match session.current_url().await { Ok(url) => nav_ctx.current_url.set(url), Err(e) => error!("browser current_url failed: {e}") }
                        }
                        Some("refresh") => {
                            if let Err(e) = session.reload().await { error!("browser reload failed: {e}"); break; }
                            match session.current_url().await { Ok(url) => nav_ctx.current_url.set(url), Err(e) => error!("browser current_url failed: {e}") }
                        }
                        _ => {}
                    },
                    Err(e) => { error!("browser widget recv failed: {e}"); break; }
                }
            }
            cmd = nav_rx.recv() => {
                if let Some(cmd) = cmd {
                    match cmd.as_str() {
                        "back" => {
                            if let Err(e) = session.go_back().await { error!("browser go_back failed: {e}"); }
                            match session.current_url().await { Ok(url) => nav_ctx.current_url.set(url), Err(e) => error!("browser current_url failed: {e}") }
                        }
                        "forward" => {
                            if let Err(e) = session.go_forward().await { error!("browser go_forward failed: {e}"); }
                            match session.current_url().await { Ok(url) => nav_ctx.current_url.set(url), Err(e) => error!("browser current_url failed: {e}") }
                        }
                        "refresh" => {
                            if let Err(e) = session.reload().await { error!("browser reload failed: {e}"); }
                            match session.current_url().await { Ok(url) => nav_ctx.current_url.set(url), Err(e) => error!("browser current_url failed: {e}") }
                        }
                        raw_url => {
                            let url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
                                raw_url.to_string()
                            } else {
                                format!("https://{raw_url}")
                            };
                            match session.navigate(&url).await {
                                Ok((final_url, _)) => nav_ctx.current_url.set(final_url),
                                Err(e) => { error!("browser navigate failed: {e}"); }
                            }
                        }
                    }
                }
            }
        }
      }
    }
  });

  use_drop(move || {
    browser_cdp::remove_session(&bid_drop);
  });

  rsx! {
    div {
      id: "{eid_rsx}",
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
pub fn CanvasView(canvas_id: String, widget_type: String, config: Value) -> Element {
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
  let div_id = use_hook(|| format!("chart-{}", Uuid::new_v4().simple()));
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
