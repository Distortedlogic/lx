# Goal

Add API routes to the Axum server router so the `server` feature provides a functional backend for SSR mode: health check, settings persistence, and activity event streaming.

# Why

The server module at `src/server/mod.rs` returns `Router::new()` with zero routes. When lx-desktop runs in server mode (`--features server`), the Dioxus fullstack renderer serves HTML but there are no API endpoints for data that the frontend pages need. The Settings page needs a persistence endpoint (since `window.localStorage` is not available server-side). The Activity page needs an event stream. A health check is standard for any server.

# Prerequisites

All other WUs should be complete first. The server API surface is defined by what the frontend pages require. Specifically:
- WU-3 (Settings) defines `SettingsData` — the server needs CRUD for this struct
- WU-1 (Shell State) defines `ActivityEvent` — the server needs to serve these

# Architecture

All routes live under `/api/`. The server uses Axum 0.8 (already a dependency). State is managed via `axum::extract::State` wrapping an `Arc<ServerState>` struct. Settings are persisted to a JSON file on disk (`~/.config/lx/settings.json`). Activity events are held in-memory in a `RwLock<VecDeque>`.

For SSR mode, the frontend pages will need to be adapted to fetch from these endpoints instead of using context signals — but that adaptation is outside this unit's scope. This unit only builds the API surface.

# Files Affected

| File | Change |
|------|--------|
| `src/server/mod.rs` | Rewrite with API routes and shared state |
| `src/server/settings_api.rs` | New file — settings CRUD handlers |
| `src/server/activity_api.rs` | New file — activity stream handler |

# Task List

### Task 1: Create server shared state and health endpoint

**Subject:** Set up Axum shared state and /api/health route

**Description:** Rewrite `crates/lx-desktop/src/server/mod.rs`:

```rust
mod activity_api;
mod settings_api;

use std::collections::VecDeque;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use tokio::sync::RwLock;

use crate::contexts::activity_log::ActivityEvent;
use crate::pages::settings::state::SettingsData;

pub struct ServerState {
    pub settings: RwLock<SettingsData>,
    pub activity: RwLock<VecDeque<ActivityEvent>>,
    pub settings_path: String,
}

impl ServerState {
    pub fn new() -> Self {
        let settings_path = dirs_or_default("lx", "settings.json");
        let settings = load_settings(&settings_path);
        Self {
            settings: RwLock::new(settings),
            activity: RwLock::new(VecDeque::new()),
            settings_path,
        }
    }
}

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
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

async fn health(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let event_count = state.activity.read().await.len();
    Json(serde_json::json!({
        "status": "ok",
        "events": event_count,
    }))
}

pub fn router() -> Router {
    let state = Arc::new(ServerState::new());
    Router::new()
        .route("/api/health", get(health))
        .merge(settings_api::routes())
        .merge(activity_api::routes())
        .with_state(state)
}
```

This requires adding `dirs` as a dependency for config directory resolution. Add to `Cargo.toml`:

```toml
dirs = { version = "6", optional = true }
```

And update the server feature:

```toml
server = ["dioxus/server", "dep:axum", "dep:dirs"]
```

If adding `dirs` is undesirable, replace `dirs::config_dir()` with a hardcoded path like `$HOME/.config/lx/settings.json` using `std::env::var("HOME")`.

Also, `ActivityEvent` must be accessible from the server module. Currently it's defined in `contexts/activity_log.rs`. Ensure the struct is `pub` and its fields are `pub` (they should be from WU-1). Similarly, `SettingsData` must be re-exported or accessible — it's in `pages/settings/state.rs` which should already be `pub`.

**ActiveForm:** Creating server shared state and health endpoint

---

### Task 2: Create settings CRUD API handlers

**Subject:** Add GET/PUT endpoints for settings persistence

**Description:** Create `crates/lx-desktop/src/server/settings_api.rs`:

```rust
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, put};
use axum::{Json, Router};

use super::ServerState;
use crate::pages::settings::state::SettingsData;

async fn get_settings(State(state): State<Arc<ServerState>>) -> Json<SettingsData> {
    let settings = state.settings.read().await.clone();
    Json(settings)
}

async fn put_settings(
    State(state): State<Arc<ServerState>>,
    Json(new_settings): Json<SettingsData>,
) -> Json<serde_json::Value> {
    let mut settings = state.settings.write().await;
    *settings = new_settings.clone();
    drop(settings);

    let result = tokio::fs::write(
        &state.settings_path,
        serde_json::to_string_pretty(&new_settings).unwrap_or_default(),
    )
    .await;

    match result {
        Ok(()) => Json(serde_json::json!({ "status": "saved" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": format!("{e}") })),
    }
}

pub fn routes() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/api/settings", get(get_settings).put(put_settings))
}
```

`GET /api/settings` returns the current settings as JSON. `PUT /api/settings` accepts a `SettingsData` body, updates the in-memory state, and writes to disk as prettified JSON. The response indicates success or failure.

**ActiveForm:** Creating settings CRUD API handlers

---

### Task 3: Create activity stream API handler

**Subject:** Add GET endpoint for activity events

**Description:** Create `crates/lx-desktop/src/server/activity_api.rs`:

```rust
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use super::ServerState;
use crate::contexts::activity_log::ActivityEvent;

#[derive(Deserialize)]
struct ActivityQuery {
    limit: Option<usize>,
}

async fn get_activity(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ActivityQuery>,
) -> Json<Vec<ActivityEvent>> {
    let events = state.activity.read().await;
    let limit = query.limit.unwrap_or(100).min(500);
    let result: Vec<ActivityEvent> = events.iter().take(limit).cloned().collect();
    Json(result)
}

async fn post_activity(
    State(state): State<Arc<ServerState>>,
    Json(event): Json<ActivityEvent>,
) -> Json<serde_json::Value> {
    let mut events = state.activity.write().await;
    events.push_front(event);
    if events.len() > 500 {
        events.pop_back();
    }
    Json(serde_json::json!({ "status": "ok", "count": events.len() }))
}

pub fn routes() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/api/activity", get(get_activity).post(post_activity))
}
```

`GET /api/activity?limit=50` returns the most recent N events (default 100, max 500). `POST /api/activity` accepts an `ActivityEvent` body and pushes it to the front of the deque. The deque is capped at 500 entries.

For `ActivityEvent` to work with JSON serialization, it must derive `Serialize` and `Deserialize`. If WU-1 did not add these derives, add them now:

In `src/contexts/activity_log.rs`, change `#[derive(Clone, PartialEq)]` on `ActivityEvent` to `#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]`.

**ActiveForm:** Creating activity stream API handler

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_SERVER_API.md" })
```
