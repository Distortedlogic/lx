# UI Alignment Unit 03: Replace WebSocket Consumer with JSONL EventStream

## Goal

Remove the Paperclip WebSocket consumer in `live_updates.rs` and replace it with a JSONL file-watching event stream consumer. Remove the `tokio-tungstenite` dependency.

---

## Current Implementation

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/contexts/live_updates.rs`

- Connects to `ws://127.0.0.1:8080/ws/events` via `tokio-tungstenite`
- Deserializes incoming `Message::Text` into `LiveEvent { event_type, company_id, payload }`
- Uses camelCase serde rename
- Handles three Paperclip event types: `activity.logged`, `agent.status`, `heartbeat.run.status`
- Pushes to `ActivityLog` via `activity_log.push(kind, message)`
- Reconnects with exponential backoff (1s to 15s)

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/contexts/activity_log.rs`

- `ActivityLog` has `push(&self, kind: &str, message: &str)` method
- Stores events in `Signal<VecDeque<ActivityEvent>>` capped at 500 entries
- `ActivityEvent` has fields: `timestamp`, `kind`, `message`

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/Cargo.toml`

- `tokio-tungstenite` on line 44, `futures` on line 40

---

## New Implementation

Watch a JSONL file for appended lines. Each line is a JSON object representing an lx event. The file path is read from environment variable `LX_EVENT_STREAM_PATH`, defaulting to `"./events.jsonl"`.

### JSONL Event Types and Mappings

Each line in the JSONL file is a JSON object with at minimum a `type` field and a `timestamp` field.

| JSONL `type` value | `ActivityLog::push` kind | message format |
|---|---|---|
| `spawn_agent` | `"agent_start"` | `"{name}"` (from `.name` field) |
| `stop_agent` | `"agent_stop"` | `"{name}"` (from `.name` field) |
| `tell` | `"tell"` | `"{from} -> {to}: {body}"` (from `.from`, `.to`, `.body`) |
| `ask` | `"ask"` | `"{from} -> {to}: {body}"` |
| `reply` | `"reply"` | `"{from} -> {to}: {body}"` |
| `tool_call` | `"tool_call"` | `"{agent}: {tool_name}"` (from `.agent`, `.tool_name`) |
| `tool_result` | `"tool_result"` | `"{agent}: {tool_name} completed"` |
| `error` | `"error"` | `"{message}"` (from `.message`) |
| `log` | `"log"` | `"{message}"` (from `.message`) |

---

## Changes

### Step 1: Remove `tokio-tungstenite` from Cargo.toml

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/Cargo.toml`

Delete the following line entirely (no replacement):

**old_string:**
```
tokio-tungstenite = { workspace = true }
```

**new_string:**
(empty — line is deleted)

### Step 2: Rewrite live_updates.rs

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/contexts/live_updates.rs`

Replace the entire file content with the following:

**old_string:**
```
use std::time::Duration;

use dioxus::prelude::*;
use futures::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::contexts::activity_log::ActivityLog;
use crate::routes::Route;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveEvent {
  #[serde(rename = "type")]
  pub event_type: String,
  pub company_id: Option<String>,
  pub payload: Option<serde_json::Value>,
}

#[component]
pub fn LiveUpdatesProvider() -> Element {
  let activity_log = use_context::<ActivityLog>();

  use_future(move || async move {
    live_event_loop(activity_log).await;
  });

  rsx! {
    Outlet::<Route> {}

  }
}

async fn live_event_loop(activity_log: ActivityLog) {
  let mut backoff = Duration::from_secs(1);
  loop {
    let url = "ws://127.0.0.1:8080/ws/events";
    if let Ok((ws_stream, _)) = connect_async(url).await {
      backoff = Duration::from_secs(1);
      let (_sink, mut stream) = ws_stream.split();
      while let Some(Ok(msg)) = stream.next().await {
        if let Message::Text(text) = msg
          && let Ok(event) = serde_json::from_str::<LiveEvent>(&text)
        {
          handle_live_event(&activity_log, &event);
        }
      }
    }
    tokio::time::sleep(backoff).await;
    backoff = (backoff * 2).min(Duration::from_secs(15));
  }
}

fn handle_live_event(activity_log: &ActivityLog, event: &LiveEvent) {
  let event_type = &event.event_type;
  let payload = event.payload.as_ref();

  match event_type.as_str() {
    "activity.logged" => {
      let message = payload.and_then(|p| p.get("action")).and_then(|a| a.as_str()).unwrap_or("activity event");
      activity_log.push("live", message);
    },
    "agent.status" => {
      let agent_id = payload.and_then(|p| p.get("agentId")).and_then(|a| a.as_str()).unwrap_or("unknown");
      let status = payload.and_then(|p| p.get("status")).and_then(|s| s.as_str()).unwrap_or("unknown");
      activity_log.push("agent_status", &format!("Agent {agent_id}: {status}"));
    },
    "heartbeat.run.status" => {
      let run_id = payload.and_then(|p| p.get("runId")).and_then(|r| r.as_str()).unwrap_or("unknown");
      let status = payload.and_then(|p| p.get("status")).and_then(|s| s.as_str()).unwrap_or("unknown");
      activity_log.push("run_status", &format!("Run {run_id}: {status}"));
    },
    _ => {},
  }
}
```

**new_string:**
```
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
  std::env::var("LX_EVENT_STREAM_PATH")
    .unwrap_or_else(|_| "./events.jsonl".to_string())
    .into()
}

async fn jsonl_event_loop(activity_log: ActivityLog) {
  let path = event_stream_path();
  loop {
    if path.exists() {
      if let Ok(file) = std::fs::File::open(&path) {
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(0)).ok();
        loop {
          let mut line = String::new();
          match reader.read_line(&mut line) {
            Ok(0) => {
              tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Ok(_) => {
              let trimmed = line.trim();
              if trimmed.is_empty() {
                continue;
              }
              if let Ok(event) = serde_json::from_str::<StreamEvent>(trimmed) {
                dispatch_event(&activity_log, &event);
              }
            }
            Err(_) => {
              tokio::time::sleep(Duration::from_secs(1)).await;
              break;
            }
          }
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
      activity_log.push("agent_start", &s(&event.name, "unknown"));
    }
    "stop_agent" => {
      activity_log.push("agent_stop", &s(&event.name, "unknown"));
    }
    "tell" => {
      let msg = format!("{} -> {}: {}", s(&event.from, "?"), s(&event.to, "?"), s(&event.body, ""));
      activity_log.push("tell", &msg);
    }
    "ask" => {
      let msg = format!("{} -> {}: {}", s(&event.from, "?"), s(&event.to, "?"), s(&event.body, ""));
      activity_log.push("ask", &msg);
    }
    "reply" => {
      let msg = format!("{} -> {}: {}", s(&event.from, "?"), s(&event.to, "?"), s(&event.body, ""));
      activity_log.push("reply", &msg);
    }
    "tool_call" => {
      let msg = format!("{}: {}", s(&event.agent, "?"), s(&event.tool_name, "?"));
      activity_log.push("tool_call", &msg);
    }
    "tool_result" => {
      let msg = format!("{}: {} completed", s(&event.agent, "?"), s(&event.tool_name, "?"));
      activity_log.push("tool_result", &msg);
    }
    "error" => {
      activity_log.push("error", &s(&event.message, "unknown error"));
    }
    "log" => {
      activity_log.push("log", &s(&event.message, ""));
    }
    _ => {}
  }
}
```

### Step 3: Leave `futures` in Cargo.toml

Leave the `futures` dependency in Cargo.toml — it is used by `terminal/browser_view.rs`.

---

## Verification

After applying these changes:
1. The desktop app no longer attempts to connect to `ws://127.0.0.1:8080/ws/events`
2. It instead tail-reads `./events.jsonl` (or path from `LX_EVENT_STREAM_PATH` env var)
3. Each JSONL line with a recognized `type` field dispatches to `ActivityLog::push` with the mapped kind and formatted message
4. The `tokio-tungstenite` dependency is removed from `Cargo.toml`
