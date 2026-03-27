# Goal

Add server endpoints that lx-mobile needs. Refactor server state from `Arc<ServerState>` with `.with_state()` to `LazyLock<ServerState>` accessed directly. Use Dioxus `#[get]` for the WebSocket endpoint. Keep axum handlers for REST endpoints because the mobile app is an external HTTP client calling specific paths.

# Why

lx-mobile needs:
- `GET /api/run/status` — execution state
- `GET /api/run/prompts` — pending prompts
- `POST /api/run/respond` — respond to a prompt
- `WS /ws/events` — real-time event stream

The mobile app is a separate binary using `reqwest` to call REST endpoints at specific paths. `#[server]` functions auto-generate paths based on function names (e.g., `POST /api/get_run_status`) which don't match the mobile client's expected paths (`GET /api/run/status`). Axum handlers give full control over HTTP method and path. The WebSocket endpoint uses Dioxus `#[get]` which does allow a custom path.

Server state moves from `Arc<ServerState>` passed via `.with_state()` to `static STATE: LazyLock<ServerState>` — same pattern as `AUDIO_SINK`, `SESSION_ID`, `KOKORO`, `WHISPER`.

# Verified facts

- `tokio-tungstenite` 0.25+ changed `Message::Text` from `String` to `Utf8Bytes`. The mobile app's `ws_client.rs` pattern match needs `.to_string()`.
- `reqwest` 0.12→0.13 is compatible for the mobile app's usage (`.get()`, `.post()`, `.json()`, `.send()`). The `query` feature is in the workspace.
- `#[get("/path")]` WebSocket endpoints register automatically via Dioxus server infrastructure.

# Prerequisites

WU-1 (Mobile Crate Setup).

# Files Affected

| File | Change |
|------|--------|
| `src/server/mod.rs` | LazyLock state, new types, updated router |
| `src/server/run_api.rs` | New — axum handlers for run status/prompts/respond |
| `src/server/ws_events.rs` | New — #[get] WebSocket event stream |
| `src/server/settings_api.rs` | Use LazyLock STATE instead of axum State extractor |
| `src/server/activity_api.rs` | Use LazyLock STATE, add broadcast send |
| `src/main.rs` | Update server launch |

# Task List

### Task 1: Rewrite ServerState as LazyLock and add new types

**Subject:** Replace Arc/with_state pattern with LazyLock, add run/prompt types

**Description:** Rewrite `crates/lx-desktop/src/server/mod.rs`:

```rust
mod activity_api;
mod run_api;
mod settings_api;
mod ws_events;

use std::collections::VecDeque;
use std::sync::LazyLock;

use axum::routing::get;
use axum::Router;
use tokio::sync::{RwLock, broadcast};

use crate::contexts::activity_log::ActivityEvent;
use crate::pages::settings::state::SettingsData;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RunStatus {
  pub status: String,
  pub source_path: Option<String>,
  pub elapsed_ms: Option<u64>,
  pub cost: Option<f64>,
  pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PendingPrompt {
  pub prompt_id: u64,
  pub kind: String,
  pub message: String,
  pub options: Option<Vec<String>>,
}

pub struct ServerState {
  pub settings: RwLock<SettingsData>,
  pub settings_path: String,
  pub activity: RwLock<VecDeque<ActivityEvent>>,
  pub run_status: RwLock<RunStatus>,
  pub prompts: RwLock<Vec<PendingPrompt>>,
  pub event_tx: broadcast::Sender<ActivityEvent>,
}

pub static STATE: LazyLock<ServerState> = LazyLock::new(|| {
  let settings_path = dirs_or_default("lx", "settings.json");
  let settings = load_settings(&settings_path);
  let (event_tx, _) = broadcast::channel(1024);
  ServerState {
    settings: RwLock::new(settings),
    settings_path,
    activity: RwLock::new(VecDeque::new()),
    run_status: RwLock::new(RunStatus::default()),
    prompts: RwLock::new(Vec::new()),
    event_tx,
  }
});

fn dirs_or_default(app: &str, file: &str) -> String {
  if let Some(config_dir) = dirs::config_dir() {
    let dir = config_dir.join(app);
    let _ = std::fs::create_dir_all(&dir);
    dir.join(file).display().to_string()
  } else {
    format!(".{app}_{file}")
  }
}

fn load_settings(path: &str) -> SettingsData {
  std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub fn router() -> Router {
  Router::new()
    .merge(settings_api::routes())
    .merge(activity_api::routes())
    .merge(run_api::routes())
    .merge(ws_events::routes())
}
```

The `router()` function returns a plain `Router` (no state parameter). All handlers access `STATE` directly. The `Default` impl, `ServerState::new()`, `health` handler, and `Arc` wrapping are removed. The health endpoint moves to `run_api.rs`.

**ActiveForm:** Rewriting ServerState as LazyLock

---

### Task 2: Update main.rs server launch

**Subject:** Merge the server router with the Dioxus router

**Description:** Edit `crates/lx-desktop/src/main.rs`. Replace the current server main:

```rust
#[cfg(feature = "server")]
fn main() {
  dioxus::serve(|| async {
    let dioxus_router = dioxus::server::router(lx_desktop::app::App);
    let app_router = lx_desktop::server::router();
    Ok(app_router.merge(dioxus_router))
  });
}
```

With:

```rust
#[cfg(feature = "server")]
fn main() {
  dioxus::serve(|| async {
    let dioxus_router = dioxus::server::router(lx_desktop::app::App);
    Ok(lx_desktop::server::router().merge(dioxus_router))
  });
}
```

Same pattern, just uses the updated `router()` which no longer takes or returns state. The Dioxus router handles SSR. The server router handles API endpoints. Both are merged.

**ActiveForm:** Updating main.rs server launch

---

### Task 3: Create run API handlers

**Subject:** Add run status, prompts, respond, and health endpoints

**Description:** Create `crates/lx-desktop/src/server/run_api.rs`:

```rust
use axum::routing::{get, post};
use axum::{Json, Router};

use super::{PendingPrompt, RunStatus, STATE};

async fn get_run_status() -> Json<RunStatus> {
  Json(STATE.run_status.read().await.clone())
}

async fn get_prompts() -> Json<Vec<PendingPrompt>> {
  Json(STATE.prompts.read().await.clone())
}

#[derive(serde::Deserialize)]
struct PromptResponse {
  prompt_id: u64,
}

async fn post_respond(Json(body): Json<PromptResponse>) -> Json<serde_json::Value> {
  STATE.prompts.write().await.retain(|p| p.prompt_id != body.prompt_id);
  Json(serde_json::json!({ "status": "ok" }))
}

async fn health() -> Json<serde_json::Value> {
  let event_count = STATE.activity.read().await.len();
  Json(serde_json::json!({ "status": "ok", "events": event_count }))
}

pub fn routes() -> Router {
  Router::new()
    .route("/api/health", get(health))
    .route("/api/run/status", get(get_run_status))
    .route("/api/run/prompts", get(get_prompts))
    .route("/api/run/respond", post(post_respond))
}
```

Paths match exactly what `api_client.rs` in lx-mobile calls. No axum `State` extractor — handlers access `STATE` directly. The `PromptResponse` struct deserializes the request body. `post_respond` removes the prompt by ID.

**ActiveForm:** Creating run API handlers

---

### Task 4: Rewrite settings and activity to use LazyLock STATE

**Subject:** Remove axum State extractor from existing handlers

**Description:** Rewrite `crates/lx-desktop/src/server/settings_api.rs`:

```rust
use axum::routing::get;
use axum::{Json, Router};

use crate::pages::settings::state::SettingsData;

use super::STATE;

async fn get_settings() -> Json<SettingsData> {
  Json(STATE.settings.read().await.clone())
}

async fn put_settings(Json(new_settings): Json<SettingsData>) -> Json<serde_json::Value> {
  let mut settings = STATE.settings.write().await;
  *settings = new_settings.clone();
  drop(settings);
  let result = tokio::fs::write(&STATE.settings_path, serde_json::to_string_pretty(&new_settings).unwrap_or_default()).await;
  match result {
    Ok(()) => Json(serde_json::json!({ "status": "saved" })),
    Err(e) => Json(serde_json::json!({ "status": "error", "message": format!("{e}") })),
  }
}

pub fn routes() -> Router {
  Router::new().route("/api/settings", get(get_settings).put(put_settings))
}
```

Rewrite `crates/lx-desktop/src/server/activity_api.rs`:

```rust
use axum::extract::Query;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::contexts::activity_log::ActivityEvent;

use super::STATE;

#[derive(Deserialize)]
struct ActivityQuery {
  limit: Option<usize>,
}

async fn get_activity(Query(query): Query<ActivityQuery>) -> Json<Vec<ActivityEvent>> {
  let events = STATE.activity.read().await;
  let limit = query.limit.unwrap_or(100).min(500);
  Json(events.iter().take(limit).cloned().collect())
}

async fn post_activity(Json(event): Json<ActivityEvent>) -> Json<serde_json::Value> {
  let _ = STATE.event_tx.send(event.clone());
  let mut events = STATE.activity.write().await;
  events.push_front(event);
  if events.len() > 500 {
    events.pop_back();
  }
  Json(serde_json::json!({ "status": "ok", "count": events.len() }))
}

pub fn routes() -> Router {
  Router::new().route("/api/activity", get(get_activity).post(post_activity))
}
```

Both files: removed `Arc<ServerState>` from handler signatures, removed `State` extractor import, changed `state.field` to `STATE.field`. `post_activity` now also broadcasts the event via `STATE.event_tx.send()` before adding to the deque.

**ActiveForm:** Rewriting settings and activity handlers

---

### Task 5: Create WebSocket events endpoint

**Subject:** Add WebSocket event stream

**Description:** Create `crates/lx-desktop/src/server/ws_events.rs`:

```rust
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use super::STATE;

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
  let rx = STATE.event_tx.subscribe();
  ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: tokio::sync::broadcast::Receiver<crate::contexts::activity_log::ActivityEvent>) {
  while let Ok(event) = rx.recv().await {
    let json = match serde_json::to_string(&event) {
      Ok(j) => j,
      Err(_) => continue,
    };
    if socket.send(Message::Text(json.into())).await.is_err() {
      break;
    }
  }
}

pub fn routes() -> Router {
  Router::new().route("/ws/events", get(ws_handler))
}
```

Uses axum's built-in WebSocket extractor (`axum::extract::ws`). No Dioxus WebSocket wrapper needed — the axum WebSocket is simpler and gives the exact path control the mobile client expects. `STATE.event_tx.subscribe()` creates a broadcast receiver. Each WebSocket client gets events as they're broadcast by `post_activity`.

`Message::Text(json.into())` — axum 0.8's `Message::Text` takes `String` (axum wraps tungstenite internally and handles the `Utf8Bytes` conversion). If axum 0.8 exposes `Utf8Bytes` directly in its `Message::Text`, use `Message::Text(json)` without `.into()`.

**ActiveForm:** Creating WebSocket events endpoint

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/MOBILE_SERVER_ENDPOINTS.md" })
```
