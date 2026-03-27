# lx-api: Shared Server Functions & Types

## Goal

Extract server fn definitions and shared types into `crates/lx-api` so lx-mobile can call them via `use_action`/`use_websocket` instead of raw reqwest/tokio-tungstenite.

## Architecture

```
lx-api
├── src/types.rs        — ActivityEvent, RunStatus, PendingPrompt
├── src/activity_api.rs — server fns + own LazyLock state (ACTIVITY, EVENT_TX)
├── src/run_api.rs      — server fns + own LazyLock state (RUN_STATUS, PROMPTS); imports ACTIVITY from activity_api for health endpoint
├── src/ws_events.rs    — server fn; imports EVENT_TX from activity_api
└── src/lib.rs

lx-desktop { lx-api = features=["server"] }
  → ActivityLog context wrapper stays in lx-desktop, references ActivityEvent from lx-api
  → SettingsData/EnvEntry/SettingsState stay entirely in lx-desktop (desktop-local UI, nothing consumes them)
  → Server fn bodies execute here (per-module statics initialized on first access)
  → main.rs: dioxus::launch(App)

lx-mobile { lx-api = no server feature }
  → dioxus::fullstack::set_server_url("http://127.0.0.1:8080") then dioxus::launch(App)
  → use_action(get_run_status), use_action(get_prompts), etc.
  → use_websocket(|| ws_events(WebSocketOptions::new()))
  → No reqwest, no tokio-tungstenite, no api_client.rs, no ws_client.rs
```

No monolithic ServerState struct. Each server fn module owns its own LazyLock statics for the state it needs. Cross-module dependencies are explicit imports (ws_events and run_api::health import from activity_api).

## Verified constraints

**Macro cfg-safety**: The dioxus `#[get]`/`#[post]` macro places server-only extractor types inside `#[cfg(feature = "server")]` blocks in the generated code (verified in `reference/dioxus/packages/fullstack-macro/src/lib.rs` lines 526-571). File-level imports of server-only items (like per-module statics) must be `#[cfg(feature = "server")]` gated since they're outside the macro's scope.

**Feature compatibility**: `dioxus` features `["mobile", "router", "fullstack"]` coexist without conflict. `fullstack` enables `dioxus-fullstack` (client stubs + HTTP via reqwest). `mobile` enables `dioxus-desktop` (native rendering). They're orthogonal. Verified in `reference/dioxus/packages/dioxus/Cargo.toml`.

**Workspace dioxus**: Root `Cargo.toml` already declares `dioxus = { version = "0.7.3", features = ["fullstack", "router"] }` as a workspace dep. lx-mobile currently overrides to `["mobile", "router"]` — will change to `["mobile", "router", "fullstack"]`.

**reqwest/tokio-tungstenite**: Both are workspace deps also used by `crates/lx`. They stay in workspace `[dependencies]` even after removing from lx-mobile.

**Server URL**: `dioxus::fullstack::set_server_url(url)` called before `dioxus::launch()`. Takes `&'static str`. Stored in `OnceLock`. Fallback auto-discovery reads `DIOXUS_DEVSERVER_IP` and `DIOXUS_DEVSERVER_PORT` env vars (defaults `127.0.0.1:8080`). In lx-mobile, call unconditionally — mobile never compiles with server feature. Ref: `reference/dioxus/examples/07-fullstack/desktop/src/main.rs`.

**ActivityEvent field mismatch**: lx-mobile `pages/events.rs` accesses `event.get("type")` and `event.get("agent_id")`. The actual `ActivityEvent` struct has fields `timestamp`, `kind`, `message` — no `type` field, no `agent_id` field. The mobile code is wrong today (shows "unknown"/"system" for everything). Typed websocket events will surface this as compile errors. Fix: use `event.kind` and remove `agent_id` references. `ActivityLog::push()` only sets `timestamp`, `kind`, `message`. There is no agent_id.

**Typed websocket transport**: Change `ws_events` from `Websocket<(), String>` to `Websocket<(), ActivityEvent>`. Eliminates manual `serde_json::to_string` on server and `serde_json::from_str` on client. Dioxus `JsonEncoding` (default) handles serde automatically.

**Settings are dead code**: The `GET/PUT /api/settings` server endpoints have no callers. lx-mobile's `api_client.rs` has no settings methods. The desktop UI uses `SettingsState`/`dioxus_storage` directly. `SettingsData`, `EnvEntry`, `SettingsState` are desktop-local UI types — they don't move to lx-api.

**PromptResponse bug**: The current `post_respond` only takes a `prompt_id` — it doesn't pass the user's chosen response value (yes/no, choice index, text input). The `PromptResponse` struct needs a `response: serde_json::Value` field to carry the actual answer.

## Task list

### Phase 1: Create lx-api crate

**1.1** Create `crates/lx-api/Cargo.toml`. Deps: `dioxus` (workspace), `serde` (workspace), `serde_json` (workspace), `tokio` (workspace, optional). Server feature enables `dep:tokio` and `dioxus/server`. Add `"crates/lx-api"` to workspace members in root `Cargo.toml`.

**1.2** Create `src/types.rs` with `ActivityEvent`, `RunStatus`, `PendingPrompt`, `PromptResponse` (with added `response: serde_json::Value` field). All `Serialize + Deserialize`. Source definitions: `ActivityEvent` from `lx-desktop/src/contexts/activity_log.rs:4-9`, `RunStatus` from `lx-desktop/src/server/mod.rs:14-21`, `PendingPrompt` from `lx-desktop/src/server/mod.rs:23-29`, `PromptResponse` from `lx-desktop/src/server/run_api.rs:22-25`.

**1.3** Create `src/activity_api.rs`. Owns two `#[cfg(feature = "server")]` statics: `ACTIVITY: LazyLock<RwLock<VecDeque<ActivityEvent>>>` and `pub EVENT_TX: LazyLock<broadcast::Sender<ActivityEvent>>`. Two server fns: `get_activity` (`#[get("/api/activity?limit")]`) reads ACTIVITY, `post_activity` (`#[post("/api/activity")]`) writes ACTIVITY and sends on EVENT_TX. EVENT_TX is `pub` because ws_events.rs imports it.

**1.4** Create `src/run_api.rs`. Owns two `#[cfg(feature = "server")]` statics: `RUN_STATUS: LazyLock<RwLock<RunStatus>>` and `PROMPTS: LazyLock<RwLock<Vec<PendingPrompt>>>`. Imports `ACTIVITY` from `activity_api` (cfg-gated) for the health endpoint. Four server fns: `health` (`#[get("/api/health")]`), `get_run_status` (`#[get("/api/run/status")]`), `get_prompts` (`#[get("/api/run/prompts")]`), `post_respond` (`#[post("/api/run/respond")]`).

**1.5** Create `src/ws_events.rs`. Imports `EVENT_TX` from `activity_api` (cfg-gated). One server fn: `ws_events` (`#[get("/ws/events")]`) returns `Websocket<(), ActivityEvent>` (typed, not String). Server body subscribes to EVENT_TX and forwards events.

**1.6** Create `src/lib.rs`. Exports `types`, `activity_api`, `run_api`, `ws_events`.

### Phase 2: Update lx-desktop

**2.1** Add `lx-api = { path = "../lx-api", features = ["server"] }` to Cargo.toml.

**2.2** Delete `src/server/settings_api.rs` (dead code), `src/server/activity_api.rs`, `src/server/run_api.rs`, `src/server/ws_events.rs`.

**2.3** Delete `src/server/mod.rs` and remove `pub mod server` from `src/lib.rs`. Nothing outside the server module imports from it.

**2.4** Update `src/contexts/activity_log.rs` — remove `ActivityEvent` struct definition, import from `lx_api::types::ActivityEvent`. Keep `ActivityLog` context wrapper and its `provide()`/`push()` methods unchanged.

**2.5** Settings (`src/pages/settings/`) — no changes. Desktop-local types stay put.

**2.6** Verify remaining imports: `src/pages/activity.rs` uses ActivityEvent via `crate::contexts::activity_log` (no change), `src/layout/shell.rs` uses ActivityLog (no change).

### Phase 3: Update lx-mobile

**3.1** Update Cargo.toml: add `lx-api = { path = "../lx-api" }`, change dioxus features to `["mobile", "router", "fullstack"]`, remove `reqwest` and `tokio-tungstenite`.

**3.2** Delete `src/api_client.rs` and `src/ws_client.rs`.

**3.3** Update `src/main.rs`: remove `mod api_client` and `mod ws_client`, add `dioxus::fullstack::set_server_url("http://127.0.0.1:8080")` before `dioxus::launch()`. No `#[cfg]` guard — mobile never compiles with server feature.

**3.4** Update `src/app.rs` — remove `LxClient` context provider.

**3.5** Rewrite `src/pages/status.rs`: replace `LxClient` + `use_future` polling with `use_action(get_run_status)`. Call on interval, read result from `.value()`. Import `get_run_status` from `lx_api::run_api`. `use_action` API: verify against `reference/dioxus/packages/hooks/src/action.rs`.

**3.6** Rewrite `src/pages/events.rs`: replace `EventWsClient` + `tokio::spawn` with `use_websocket(|| ws_events(WebSocketOptions::new()))` and `socket.recv()` in `use_future`. Change `Vec<serde_json::Value>` to `Vec<ActivityEvent>`. Replace `event.get("type")` with `event.kind`. Remove `event.get("agent_id")` references (field doesn't exist). Keep filter logic and `render_mobile_event` — adapt to use struct fields directly.

**3.7** Rewrite `src/pages/approvals.rs`: replace `LxClient::fetch_pending_prompts()` with `use_action(get_prompts)`. Replace `LxClient::post_user_response()` with direct `post_respond()` call. Delete local `PendingPrompt`/`PromptKind` types and `parse_pending_prompt()` — use `lx_api::types::PendingPrompt` directly, match on `prompt.kind.as_str()`. Delete `send_response()` helper.

### Phase 4: Cleanup

**4.1** Remove `reqwest` and `tokio-tungstenite` from lx-mobile's Cargo.toml. They stay in workspace deps (used by `crates/lx`).

**4.2** Verify no dead imports remain in either crate.
