# Goal

Add the four server endpoints that lx-mobile needs: run status, pending prompts, prompt response, and WebSocket event stream. All in lx-desktop's server module behind the `server` feature.

# Why

lx-mobile connects to lx-desktop's server. It needs:
- `GET /api/run/status` — execution state polling
- `GET /api/run/prompts` — pending human-in-the-loop prompts
- `POST /api/run/respond` — respond to a prompt
- `WS /ws/events` — real-time event stream

None of these exist. The server currently has `/api/health`, `/api/settings`, and `/api/activity`.

# Prerequisites

WU-1 (Mobile Crate Setup) — lx-mobile must compile so we can verify the endpoints match what the client expects.

# Architecture

All new state goes into `ServerState` in `server/mod.rs`:
- `run_status: RwLock<RunStatus>` — current execution state
- `prompts: RwLock<Vec<PendingPrompt>>` — pending prompts queue
- `event_tx: broadcast::Sender<ActivityEvent>` — broadcast channel for WebSocket streaming

The `ActivityLog::push` in `contexts/activity_log.rs` currently writes to a `Signal<VecDeque>`. For server mode, the server's `POST /api/activity` endpoint writes to the server's `VecDeque`. The WebSocket endpoint subscribes to the broadcast channel.

The `RunStatus` and `PendingPrompt` types match what `api_client.rs` in lx-mobile expects:
- `RunStatus`: `{ status: String, source_path: Option<String>, elapsed_ms: Option<u64>, cost: Option<f64>, error: Option<String> }`
- `PendingPrompt`: `{ prompt_id: u64, kind: String, message: String, options: Option<Vec<String>> }`

# Files Affected

| File | Change |
|------|--------|
| `src/server/mod.rs` | Add RunStatus, PendingPrompt to ServerState, broadcast channel |
| `src/server/run_api.rs` | New — GET /api/run/status, GET /api/run/prompts, POST /api/run/respond |
| `src/server/ws_events.rs` | New — WS /ws/events upgrade endpoint |

# Task List

### Task 1: Add run state and prompt types to ServerState

**Subject:** Extend ServerState with execution tracking and prompt queue

**Description:** Edit `crates/lx-desktop/src/server/mod.rs`.

Add imports:
```rust
use tokio::sync::broadcast;
```

Add these types before `ServerState`:

```rust
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
```

Add three fields to `ServerState`:
```rust
pub run_status: RwLock<RunStatus>,
pub prompts: RwLock<Vec<PendingPrompt>>,
pub event_tx: broadcast::Sender<ActivityEvent>,
```

Update `ServerState::new()` to initialize them:
```rust
let (event_tx, _) = broadcast::channel(1024);
```

Add `run_status: RwLock::new(RunStatus::default())`, `prompts: RwLock::new(Vec::new())`, and `event_tx` to the struct construction.

Update the `Default` impl if needed (it delegates to `new()`).

Add `mod run_api;` and `mod ws_events;` to the module declarations.

Merge the new routes in the `router()` function:
```rust
Router::new()
  .route("/api/health", get(health))
  .merge(settings_api::routes())
  .merge(activity_api::routes())
  .merge(run_api::routes())
  .merge(ws_events::routes())
  .with_state(state)
```

**ActiveForm:** Adding run state and prompt types to ServerState

---

### Task 2: Create run API endpoints

**Subject:** Add status, prompts, and respond endpoints

**Description:** Create `crates/lx-desktop/src/server/run_api.rs`:

```rust
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use super::{PendingPrompt, RunStatus, ServerState};

async fn get_run_status(State(state): State<Arc<ServerState>>) -> Json<RunStatus> {
  Json(state.run_status.read().await.clone())
}

async fn get_prompts(State(state): State<Arc<ServerState>>) -> Json<Vec<PendingPrompt>> {
  Json(state.prompts.read().await.clone())
}

#[derive(serde::Deserialize)]
struct PromptResponse {
  prompt_id: u64,
  response: serde_json::Value,
}

async fn post_respond(
  State(state): State<Arc<ServerState>>,
  Json(body): Json<PromptResponse>,
) -> Json<serde_json::Value> {
  let mut prompts = state.prompts.write().await;
  prompts.retain(|p| p.prompt_id != body.prompt_id);
  Json(serde_json::json!({ "status": "ok" }))
}

pub fn routes() -> Router<Arc<ServerState>> {
  Router::new()
    .route("/api/run/status", get(get_run_status))
    .route("/api/run/prompts", get(get_prompts))
    .route("/api/run/respond", post(post_respond))
}
```

The `post_respond` handler removes the prompt from the queue. The actual response forwarding to the lx runtime will be added when the runtime has a prompt/response channel — for now, removing the prompt acknowledges it.

**ActiveForm:** Creating run API endpoints

---

### Task 3: Create WebSocket events endpoint

**Subject:** Add WebSocket upgrade endpoint that streams ActivityEvents

**Description:** Create `crates/lx-desktop/src/server/ws_events.rs`:

```rust
use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use super::ServerState;

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<ServerState>>) -> impl IntoResponse {
  let rx = state.event_tx.subscribe();
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

pub fn routes() -> Router<Arc<ServerState>> {
  Router::new().route("/ws/events", get(ws_handler))
}
```

Each WebSocket connection subscribes to the broadcast channel. When `event_tx.send(event)` is called (by the `POST /api/activity` endpoint or future runtime event emitter), all connected WebSocket clients receive the event as a JSON text message. If the client disconnects, the loop breaks.

The `axum::extract::ws` module is available in axum 0.8 without extra features.

**ActiveForm:** Creating WebSocket events endpoint

---

### Task 4: Wire activity POST to broadcast

**Subject:** Make POST /api/activity also broadcast events to WebSocket clients

**Description:** Edit `crates/lx-desktop/src/server/activity_api.rs`. In the `post_activity` handler, after pushing the event to the deque, also send it to the broadcast channel.

Read the current file first. Find the `post_activity` function. After the `events.push_front(event)` line, add:

```rust
let _ = state.event_tx.send(event.clone());
```

This requires the `event` to be cloned before pushing to the deque (since push_front takes ownership). Reorder: clone first, push the clone to the deque, send the original to broadcast. Or clone before both operations.

The `push_front` currently takes the owned `event`. Change to:

```rust
let _ = state.event_tx.send(event.clone());
events.push_front(event);
```

The broadcast send is before the deque push so WebSocket clients get the event immediately. The `let _ =` ignores the send result (returns Err if no receivers are subscribed, which is harmless).

**ActiveForm:** Wiring activity POST to broadcast channel

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
