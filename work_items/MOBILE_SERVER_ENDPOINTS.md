# Goal

Add server endpoints that lx-mobile needs and refactor all existing server endpoints from manual axum routes to Dioxus `#[get]`/`#[post]`/`#[put]` macros. Eliminate the axum Router entirely. Server state is a module-level `LazyLock<ServerState>`.

# Why

Dioxus provides `#[get("/path")]`, `#[post("/path")]`, `#[put("/path")]` macros that auto-register HTTP endpoints with full control over method, path, params, and return type. The existing server module manually builds an axum `Router` with `.route()` calls and `State` extractors. This is unnecessary — the Dioxus macros do the same thing with less code and automatic registration.

# Dioxus server endpoint patterns (from reference/dioxus/packages/fullstack/tests/compile-test.rs)

```rust
#[get("/api/items/{id}?amount&offset")]
async fn get_item(id: i32, amount: Option<i32>, offset: Option<i32>) -> Result<Json<Item>> {
    Ok(Json(Item { id, amount, offset }))
}

#[post("/api/items")]
async fn create_item(data: Json<Item>) -> Result<()> {
    Ok(())
}

#[get("/ws/stream")]
async fn ws_stream(options: WebSocketOptions) -> Result<Websocket<String, String>> {
    Ok(options.on_upgrade(|mut tx| async move { ... }))
}
```

- `#[get]`, `#[post]`, `#[put]`, `#[patch]`, `#[delete]` — full HTTP method control
- Path params: `/items/{id}`, query params: `?amount&offset`
- Takes `Json<T>`, axum extractors, `WebSocketOptions`, `FileStream`
- Returns `Result<T>` where T is `Json<T>`, `String`, `Bytes`, `Websocket`, or anything `IntoResponse`
- Routes register automatically — no manual Router construction
- Imported from `dioxus_fullstack` or `dioxus::fullstack`

# Prerequisites

WU-1 (Mobile Crate Setup).

# Files Affected

| File | Change |
|------|--------|
| `src/server/mod.rs` | LazyLock state, new types, remove Router |
| `src/server/run_api.rs` | New — run status/prompts/respond |
| `src/server/ws_events.rs` | New — WebSocket event stream |
| `src/server/settings_api.rs` | Rewrite with #[get]/#[put] |
| `src/server/activity_api.rs` | Rewrite with #[get]/#[post], add broadcast |
| `src/main.rs` | Simplify server launch — no router merging |

# Task List

### Task 1: Rewrite mod.rs — LazyLock state, new types, remove Router

**Subject:** Replace Arc/Router with LazyLock, add run/prompt types

**Description:** Rewrite `crates/lx-desktop/src/server/mod.rs`:

```rust
mod activity_api;
mod run_api;
mod settings_api;
mod ws_events;

use std::collections::VecDeque;
use std::sync::LazyLock;

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
```

Removed entirely: `router()` function, `health` handler, `Default` impl, `ServerState::new()`, all axum imports (`Router`, `routing::get`, `extract::State`, `Json`, `Arc`). The module is now just types and a `LazyLock` static.

**ActiveForm:** Rewriting mod.rs with LazyLock state

---

### Task 2: Simplify main.rs — remove router merging

**Subject:** Server launch no longer needs manual router construction

**Description:** Edit `crates/lx-desktop/src/main.rs`. The current server main:

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

Replace with:

```rust
#[cfg(feature = "server")]
fn main() {
  dioxus::serve(|| async {
    let cfg = dioxus::server::ServeConfig::builder();
    Ok(axum::Router::new().serve_dioxus_application(cfg, lx_desktop::app::App))
  });
}
```

The `#[get]`/`#[post]`/`#[put]` endpoints register automatically through Dioxus's distributed registration system. `serve_dioxus_application` sets up SSR and all registered server endpoints. No manual merging.

If `serve_dioxus_application` requires an import (it's an extension trait on `Router`), check `reference/dioxus/packages/playwright-tests/fullstack/src/main.rs` line 18 — it uses `axum::Router::new().serve_dioxus_application(cfg, app)` directly. The import comes from `dioxus::server` or `dioxus_fullstack`.

**ActiveForm:** Simplifying server launch

---

### Task 3: Create run API endpoints

**Subject:** Add run status, prompts, respond, and health endpoints

**Description:** Create `crates/lx-desktop/src/server/run_api.rs`:

```rust
use axum::Json;
use dioxus::prelude::*;

use super::{PendingPrompt, RunStatus, STATE};

#[get("/api/health")]
pub async fn health() -> Result<Json<serde_json::Value>> {
  let event_count = STATE.activity.read().await.len();
  Ok(Json(serde_json::json!({ "status": "ok", "events": event_count })))
}

#[get("/api/run/status")]
pub async fn get_run_status() -> Result<Json<RunStatus>> {
  Ok(Json(STATE.run_status.read().await.clone()))
}

#[get("/api/run/prompts")]
pub async fn get_prompts() -> Result<Json<Vec<PendingPrompt>>> {
  Ok(Json(STATE.prompts.read().await.clone()))
}

#[derive(serde::Deserialize)]
pub struct PromptResponse {
  pub prompt_id: u64,
}

#[post("/api/run/respond")]
pub async fn post_respond(data: Json<PromptResponse>) -> Result<Json<serde_json::Value>> {
  STATE.prompts.write().await.retain(|p| p.prompt_id != data.prompt_id);
  Ok(Json(serde_json::json!({ "status": "ok" })))
}
```

Paths match exactly what `api_client.rs` in lx-mobile calls: `GET /api/run/status`, `GET /api/run/prompts`, `POST /api/run/respond`. The `#[get]`/`#[post]` macros handle registration. `STATE` is accessed directly.

**ActiveForm:** Creating run API endpoints

---

### Task 4: Rewrite settings and activity endpoints

**Subject:** Replace axum handlers with Dioxus endpoint macros

**Description:** Rewrite `crates/lx-desktop/src/server/settings_api.rs`:

```rust
use axum::Json;
use dioxus::prelude::*;

use crate::pages::settings::state::SettingsData;

use super::STATE;

#[get("/api/settings")]
pub async fn get_settings() -> Result<Json<SettingsData>> {
  Ok(Json(STATE.settings.read().await.clone()))
}

#[put("/api/settings")]
pub async fn put_settings(new_settings: Json<SettingsData>) -> Result<Json<serde_json::Value>> {
  let mut settings = STATE.settings.write().await;
  *settings = new_settings.0.clone();
  drop(settings);
  let _ = tokio::fs::write(&STATE.settings_path, serde_json::to_string_pretty(&new_settings.0).unwrap_or_default()).await;
  Ok(Json(serde_json::json!({ "status": "saved" })))
}
```

Rewrite `crates/lx-desktop/src/server/activity_api.rs`:

```rust
use axum::Json;
use dioxus::prelude::*;

use crate::contexts::activity_log::ActivityEvent;

use super::STATE;

#[get("/api/activity?limit")]
pub async fn get_activity(limit: Option<usize>) -> Result<Json<Vec<ActivityEvent>>> {
  let events = STATE.activity.read().await;
  let limit = limit.unwrap_or(100).min(500);
  Ok(Json(events.iter().take(limit).cloned().collect()))
}

#[post("/api/activity")]
pub async fn post_activity(event: Json<ActivityEvent>) -> Result<Json<serde_json::Value>> {
  let _ = STATE.event_tx.send(event.0.clone());
  let mut events = STATE.activity.write().await;
  events.push_front(event.0);
  if events.len() > 500 {
    events.pop_back();
  }
  Ok(Json(serde_json::json!({ "status": "ok", "count": events.len() })))
}
```

`get_activity` uses `?limit` in the path macro to extract the query parameter. `post_activity` broadcasts via `event_tx` before adding to the deque.

**ActiveForm:** Rewriting settings and activity endpoints

---

### Task 5: Create WebSocket events endpoint

**Subject:** Add WebSocket event stream

**Description:** Create `crates/lx-desktop/src/server/ws_events.rs`:

```rust
use dioxus::fullstack::{Websocket, WebSocketOptions};
use dioxus::prelude::*;

use super::STATE;

#[get("/ws/events")]
pub async fn ws_events(options: WebSocketOptions) -> Result<Websocket<String, ()>> {
  let rx = STATE.event_tx.subscribe();
  Ok(options.on_upgrade(move |mut tx| async move {
    let mut rx = rx;
    while let Ok(event) = rx.recv().await {
      let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => continue,
      };
      if tx.send(json).await.is_err() {
        break;
      }
    }
  }))
}
```

`Websocket<String, ()>` — sends `String` (JSON events), receives nothing (the mobile client only reads from this socket). Each connection subscribes to the broadcast channel. Events are serialized and sent as text. Client disconnect breaks the loop.

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
