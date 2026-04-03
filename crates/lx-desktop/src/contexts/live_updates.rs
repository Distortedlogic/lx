use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;

use dioxus::prelude::*;

use crate::contexts::activity_log::ActivityLog;
use crate::routes::Route;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct StreamEvent {
  #[serde(rename = "type")]
  pub event_type: String,
  #[serde(default)]
  pub name: Option<String>,
  #[serde(default)]
  pub from: Option<String>,
  #[serde(default)]
  pub to: Option<String>,
  #[serde(default)]
  pub body: Option<String>,
  #[serde(default)]
  pub agent: Option<String>,
  #[serde(default)]
  pub tool_name: Option<String>,
  #[serde(default)]
  pub message: Option<String>,
  #[serde(default)]
  pub adapter: Option<String>,
}

#[component]
pub fn LiveUpdatesProvider() -> Element {
  let activity_log = use_context::<ActivityLog>();

  use_future(move || async move {
    jsonl_event_loop(activity_log).await;
  });

  rsx! {
    Outlet::<Route> {}
  }
}

fn event_stream_path() -> PathBuf {
  std::env::var("LX_EVENT_STREAM_PATH").unwrap_or_else(|_| "./events.jsonl".to_string()).into()
}

async fn jsonl_event_loop(activity_log: ActivityLog) {
  let path = event_stream_path();
  loop {
    if path.exists()
      && let Ok(file) = std::fs::File::open(&path)
    {
      let mut reader = BufReader::new(file);
      reader.seek(SeekFrom::End(0)).ok();
      loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
          Ok(0) => {
            tokio::time::sleep(Duration::from_millis(250)).await;
          },
          Ok(_) => {
            let trimmed = line.trim();
            if trimmed.is_empty() {
              continue;
            }
            if let Ok(event) = serde_json::from_str::<StreamEvent>(trimmed) {
              dispatch_event(&activity_log, &event);
            }
          },
          Err(_) => {
            tokio::time::sleep(Duration::from_secs(1)).await;
            break;
          },
        }
      }
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
  }
}

fn s(opt: &Option<String>, fallback: &str) -> String {
  opt.as_deref().unwrap_or(fallback).to_string()
}

fn dispatch_event(activity_log: &ActivityLog, event: &StreamEvent) {
  match event.event_type.as_str() {
    "spawn_agent" => {
      activity_log.push_with_adapter("agent_start", &s(&event.name, "unknown"), event.adapter.clone());
    },
    "stop_agent" => {
      activity_log.push("agent_stop", &s(&event.name, "unknown"));
    },
    "tell" => {
      let msg = format!("{} -> {}: {}", s(&event.from, "?"), s(&event.to, "?"), s(&event.body, ""));
      activity_log.push("tell", &msg);
    },
    "ask" => {
      let msg = format!("{} -> {}: {}", s(&event.from, "?"), s(&event.to, "?"), s(&event.body, ""));
      activity_log.push("ask", &msg);
    },
    "reply" => {
      let msg = format!("{} -> {}: {}", s(&event.from, "?"), s(&event.to, "?"), s(&event.body, ""));
      activity_log.push("reply", &msg);
    },
    "tool_call" => {
      let msg = format!("{}: {}", s(&event.agent, "?"), s(&event.tool_name, "?"));
      activity_log.push("tool_call", &msg);
    },
    "tool_result" => {
      let msg = format!("{}: {} completed", s(&event.agent, "?"), s(&event.tool_name, "?"));
      activity_log.push("tool_result", &msg);
    },
    "error" => {
      activity_log.push("error", &s(&event.message, "unknown error"));
    },
    "log" => {
      activity_log.push("log", &s(&event.message, ""));
    },
    _ => {},
  }
}
