use std::sync::{Arc, Mutex};

use dioxus::logger::tracing::error;
use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct BrowserNavCtx {
  pub tx: mpsc::UnboundedSender<String>,
  pub rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>,
  pub current_url: Signal<String>,
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
      let session = match common_cdp::get_or_create_session(&browser_id).await {
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

      let mut nav_rx = {
        let Ok(mut guard) = nav_ctx.rx.lock() else {
          error!("browser nav_rx mutex poisoned");
          return;
        };
        let Some(rx) = guard.take() else {
          return;
        };
        rx
      };

      let mut frames = match session.start_screencast().await {
        Ok(s) => s,
        Err(e) => {
          error!("browser start_screencast failed: {e}");
          return;
        },
      };

      loop {
        tokio::select! {
            frame = frames.next() => {
                let Some(frame) = frame else { break; };
                let b64 = String::from(frame.data.clone());
                widget.send_update(b64);
                if let Err(e) = session.ack_frame(frame.session_id).await { error!("browser ack_frame failed: {e}"); break; }
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
                        Some("key") => {
                            let key = msg["key"].as_str().unwrap_or("");
                            let code = msg["code"].as_str().unwrap_or("");
                            let mods = &msg["modifiers"];
                            let mut modifier_flags: i64 = 0;
                            if mods["alt"].as_bool().unwrap_or(false) { modifier_flags |= 1; }
                            if mods["ctrl"].as_bool().unwrap_or(false) { modifier_flags |= 2; }
                            if mods["meta"].as_bool().unwrap_or(false) { modifier_flags |= 4; }
                            if mods["shift"].as_bool().unwrap_or(false) { modifier_flags |= 8; }
                            if let Err(e) = session.dispatch_key(key, code, modifier_flags).await { error!("browser dispatch_key failed: {e}"); break; }
                        }
                        Some("scroll") => {
                            let x = msg["x"].as_f64().unwrap_or(0.0);
                            let y = msg["y"].as_f64().unwrap_or(0.0);
                            let delta_x = msg["deltaX"].as_f64().unwrap_or(0.0);
                            let delta_y = msg["deltaY"].as_f64().unwrap_or(0.0);
                            if let Err(e) = session.scroll(x, y, delta_x, delta_y).await { error!("browser scroll failed: {e}"); break; }
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
    common_cdp::remove_session(&bid_drop);
  });

  rsx! {
    div {
      id: "{eid_rsx}",
      class: "w-full h-full bg-[var(--surface-container)]",
    }
  }
}
