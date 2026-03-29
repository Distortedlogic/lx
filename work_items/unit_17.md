# Unit 17: Live Updates Provider & Data Layer

## Scope

Port the WebSocket-based live updates provider and create Rust API client modules for all Paperclip entity types. The live updates provider connects to a WebSocket endpoint, receives server-sent events, and triggers cache invalidation / toast notifications. The API client modules wrap HTTP calls to the Paperclip backend.

## Paperclip Source Files

| Paperclip file | Purpose |
|---|---|
| `reference/paperclip/ui/src/context/LiveUpdatesProvider.tsx` | WebSocket SSE handler, cache invalidation, toast gating (760 lines) |
| `reference/paperclip/ui/src/api/client.ts` | Base HTTP client with `get/post/put/patch/delete` |
| `reference/paperclip/ui/src/api/index.ts` | Re-exports all 18 API modules |
| `reference/paperclip/ui/src/api/issues.ts` | Issues CRUD + filters |
| `reference/paperclip/ui/src/api/agents.ts` | Agents CRUD + adapter models + env test |
| `reference/paperclip/ui/src/api/projects.ts` | Projects CRUD + workspaces |
| `reference/paperclip/ui/src/api/goals.ts` | Goals CRUD |
| `reference/paperclip/ui/src/api/companies.ts` | Companies CRUD + portability |
| `reference/paperclip/ui/src/api/approvals.ts` | Approvals CRUD + comments |
| `reference/paperclip/ui/src/api/activity.ts` | Activity log + run-issue links |
| `reference/paperclip/ui/src/api/heartbeats.ts` | Heartbeat runs + events + workspace ops |
| `reference/paperclip/ui/src/api/costs.ts` | Cost summaries + breakdowns + finance |
| `reference/paperclip/ui/src/api/dashboard.ts` | Dashboard summary |
| `reference/paperclip/ui/src/api/auth.ts` | Auth session + sign in/up/out |
| `reference/paperclip/ui/src/api/access.ts` | Invites, join requests, CLI auth |
| `reference/paperclip/ui/src/api/secrets.ts` | Company secrets CRUD |
| `reference/paperclip/ui/src/api/health.ts` | Health check |
| `reference/paperclip/ui/src/api/instanceSettings.ts` | General + experimental settings |
| `reference/paperclip/ui/src/api/routines.ts` | Routines CRUD + triggers |
| `reference/paperclip/ui/src/api/sidebarBadges.ts` | Sidebar badge counts |
| `reference/paperclip/ui/src/api/budgets.ts` | Budget overview + policies |
| `reference/paperclip/ui/src/api/companySkills.ts` | Company skills CRUD |
| `reference/paperclip/ui/src/api/plugins.ts` | Plugin management |
| `reference/paperclip/ui/src/api/assets.ts` | Image upload |
| `reference/paperclip/ui/src/api/execution-workspaces.ts` | Execution workspace listing |
| `reference/paperclip/ui/src/lib/queryKeys.ts` | Cache key factory (143 lines) |
| `crates/lx-api/src/lib.rs` | Existing lx-api crate (activity_api, run_api, types, ws_events) |
| `crates/lx-api/src/ws_events.rs` | Existing WebSocket event stream |
| `crates/lx-api/src/types.rs` | Existing types: ActivityEvent, RunStatus, PendingPrompt |

## Preconditions

- `crates/lx-api/` exists with modules: `activity_api`, `run_api`, `types`, `ws_events`
- `crates/lx-desktop/src/contexts/mod.rs` exists with `activity_log`, `status_bar`, and all Unit 3 context modules (theme, toast, dialog, panel, sidebar, breadcrumb, company)
- `crates/lx-desktop/src/contexts/activity_log.rs` exists with `ActivityLog` struct using `Signal<VecDeque<ActivityEvent>>`
- `crates/lx-desktop/src/layout/shell.rs` exists with `Shell` component (modified by Units 3, 4, 5, 16)
- `crates/lx-desktop/src/lib.rs` already contains: `pub mod app;`, `pub mod components;` (Unit 1), `pub mod contexts;`, `pub mod hooks;` (Unit 16), `pub mod layout;`, `pub mod pages;`, `pub mod plugins;` (Unit 15), `pub mod panes;`, `pub mod routes;`, `pub mod styles;`, `pub mod terminal;`, `pub mod voice_backend;`
- Unit 16 is complete (components and hooks modules exist)
- Precondition verified: `reqwest` works in Dioxus 0.7.3 desktop (standard HTTP client, no special requirements). `tokio-tungstenite` works in Dioxus desktop (full tokio runtime available in desktop mode).

## Files Affected

| File | Action |
|---|---|
| `crates/lx-desktop/src/api/mod.rs` | Create: module declarations + base client |
| `crates/lx-desktop/src/api/client.rs` | Create: base HTTP client struct |
| `crates/lx-desktop/src/api/companies.rs` | Create: companies API |
| `crates/lx-desktop/src/api/agents.rs` | Create: agents API |
| `crates/lx-desktop/src/api/issues.rs` | Create: issues API |
| `crates/lx-desktop/src/api/projects.rs` | Create: projects API |
| `crates/lx-desktop/src/api/goals.rs` | Create: goals API |
| `crates/lx-desktop/src/api/activity.rs` | Create: activity API |
| `crates/lx-desktop/src/api/approvals.rs` | Create: approvals API |
| `crates/lx-desktop/src/api/costs.rs` | Create: costs API |
| `crates/lx-desktop/src/api/heartbeats.rs` | Create: heartbeats API |
| `crates/lx-desktop/src/api/auth.rs` | Create: auth API |
| `crates/lx-desktop/src/contexts/live_updates.rs` | Create: live updates provider |
| `crates/lx-desktop/src/contexts/mod.rs` | Modify: add `pub mod live_updates;` |
| `crates/lx-desktop/src/lib.rs` | Modify: add `pub mod api;` |
| `crates/lx-desktop/src/layout/shell.rs` | Modify: wrap children in LiveUpdatesProvider |
| `crates/lx-desktop/Cargo.toml` | Modify: add `reqwest` dependency if not present |

## Tasks

### 1. Check and add `reqwest` dependency

Check `crates/lx-desktop/Cargo.toml` for `reqwest`. If absent, add:

```toml
reqwest = { version = "0.12", features = ["json"] }
```

Also verify `serde_json` and `serde` with `derive` feature are present. If `tokio-tungstenite` or similar WebSocket client is not present, add:

```toml
tokio-tungstenite = "0.24"
futures-util = "0.3"
```

If `url` crate is not present, add it for URL construction.

### 2. Create `crates/lx-desktop/src/api/client.rs`

Base HTTP client wrapping `reqwest::Client`. Port `reference/paperclip/ui/src/api/client.ts`.

```rust
use reqwest::Client;
use serde::de::DeserializeOwned;

pub struct ApiClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String, body: Option<serde_json::Value> },
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> { ... }
    pub async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, ApiError> { ... }
    pub async fn put<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, ApiError> { ... }
    pub async fn patch<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, ApiError> { ... }
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> { ... }
}
```

Each method:
1. Constructs the full URL: `format!("{}{}", self.base_url, path)`
2. Sends the request with `Content-Type: application/json`
3. On non-2xx response: reads body as JSON, extracts `error` field if present, returns `ApiError::Http`
4. On 204: returns `T` by deserializing empty/null (use `serde_json::from_value(serde_json::Value::Null)`)
5. On success: deserializes JSON response body

Add `thiserror` to `Cargo.toml` if not already present.

### 3. Create `crates/lx-desktop/src/api/mod.rs`

```rust
pub mod client;
pub mod activity;
pub mod agents;
pub mod approvals;
pub mod auth;
pub mod companies;
pub mod costs;
pub mod goals;
pub mod heartbeats;
pub mod issues;
pub mod projects;

pub use client::{ApiClient, ApiError};
```

### 4. Create `crates/lx-desktop/src/api/companies.rs`

Port `reference/paperclip/ui/src/api/companies.ts`. Only port the CRUD subset relevant to lx-desktop (skip portability export/import).

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub issue_prefix: Option<String>,
    pub budget_monthly_cents: Option<i64>,
    pub brand_color: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyInput {
    pub name: String,
    pub description: Option<String>,
}

pub async fn list(client: &ApiClient) -> Result<Vec<Company>, ApiError> {
    client.get("/companies").await
}

pub async fn get(client: &ApiClient, id: &str) -> Result<Company, ApiError> {
    client.get(&format!("/companies/{id}")).await
}

pub async fn create(client: &ApiClient, input: &CreateCompanyInput) -> Result<Company, ApiError> {
    client.post("/companies", input).await
}
```

### 5. Create `crates/lx-desktop/src/api/agents.rs` (~45 lines)

Port `reference/paperclip/ui/src/api/agents.ts`. Define `Agent` and `AgentDetail` structs.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub adapter_type: Option<String>,
    pub company_id: Option<String>,
    pub created_at: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str) -> Result<Vec<Agent>, ApiError> {
    client.get(&format!("/companies/{company_id}/agents")).await
}

pub async fn get(client: &ApiClient, id: &str) -> Result<Agent, ApiError> {
    client.get(&format!("/agents/detail/{id}")).await
}

pub async fn create(
    client: &ApiClient,
    company_id: &str,
    input: &serde_json::Value,
) -> Result<Agent, ApiError> {
    client.post(&format!("/companies/{company_id}/agents"), input).await
}
```

### 6. Create `crates/lx-desktop/src/api/issues.rs`

Port `reference/paperclip/ui/src/api/issues.ts`. Define `Issue` struct and list/get/create/search functions.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub identifier: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee_agent_id: Option<String>,
    pub assignee_user_id: Option<String>,
    pub project_id: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueFilters {
    pub status: Option<String>,
    pub project_id: Option<String>,
    pub assignee_agent_id: Option<String>,
    pub q: Option<String>,
}

pub async fn list(
    client: &ApiClient,
    company_id: &str,
    filters: Option<&IssueFilters>,
) -> Result<Vec<Issue>, ApiError> {
    let mut path = format!("/companies/{company_id}/issues");
    if let Some(f) = filters {
        let mut params = Vec::new();
        if let Some(s) = &f.status { params.push(format!("status={s}")); }
        if let Some(p) = &f.project_id { params.push(format!("projectId={p}")); }
        if let Some(a) = &f.assignee_agent_id { params.push(format!("assigneeAgentId={a}")); }
        if let Some(q) = &f.q { params.push(format!("q={q}")); }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }
    }
    client.get(&path).await
}

pub async fn get(client: &ApiClient, id: &str) -> Result<Issue, ApiError> {
    client.get(&format!("/issues/detail/{id}")).await
}

pub async fn create(
    client: &ApiClient,
    company_id: &str,
    input: &serde_json::Value,
) -> Result<Issue, ApiError> {
    client.post(&format!("/companies/{company_id}/issues"), input).await
}
```

### 7. Create `crates/lx-desktop/src/api/projects.rs`

Port `reference/paperclip/ui/src/api/projects.ts`.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub archived_at: Option<String>,
    pub company_id: Option<String>,
    pub created_at: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str) -> Result<Vec<Project>, ApiError> {
    client.get(&format!("/companies/{company_id}/projects")).await
}

pub async fn get(client: &ApiClient, id: &str) -> Result<Project, ApiError> {
    client.get(&format!("/projects/{id}")).await
}

pub async fn create(
    client: &ApiClient,
    company_id: &str,
    input: &serde_json::Value,
) -> Result<Project, ApiError> {
    client.post(&format!("/companies/{company_id}/projects"), input).await
}
```

### 8. Create `crates/lx-desktop/src/api/goals.rs`

Port `reference/paperclip/ui/src/api/goals.ts`.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub level: Option<String>,
    pub status: Option<String>,
    pub company_id: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str) -> Result<Vec<Goal>, ApiError> {
    client.get(&format!("/companies/{company_id}/goals")).await
}

pub async fn create(
    client: &ApiClient,
    company_id: &str,
    input: &serde_json::Value,
) -> Result<Goal, ApiError> {
    client.post(&format!("/companies/{company_id}/goals"), input).await
}
```

### 9. Create `crates/lx-desktop/src/api/activity.rs`

Port `reference/paperclip/ui/src/api/activity.ts`.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEvent {
    pub id: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub action: Option<String>,
    pub actor_type: Option<String>,
    pub actor_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: Option<String>,
}

pub async fn list(
    client: &ApiClient,
    company_id: &str,
    entity_type: Option<&str>,
) -> Result<Vec<ActivityEvent>, ApiError> {
    let mut path = format!("/companies/{company_id}/activity");
    if let Some(et) = entity_type {
        path.push_str(&format!("?entityType={et}"));
    }
    client.get(&path).await
}
```

### 10. Create `crates/lx-desktop/src/api/approvals.rs`

Port `reference/paperclip/ui/src/api/approvals.ts`.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    pub id: String,
    pub status: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub company_id: Option<String>,
    pub created_at: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str) -> Result<Vec<Approval>, ApiError> {
    client.get(&format!("/companies/{company_id}/approvals")).await
}

pub async fn approve(client: &ApiClient, id: &str, note: Option<&str>) -> Result<Approval, ApiError> {
    client.post(&format!("/approvals/{id}/approve"), &serde_json::json!({ "decisionNote": note })).await
}

pub async fn reject(client: &ApiClient, id: &str, note: Option<&str>) -> Result<Approval, ApiError> {
    client.post(&format!("/approvals/{id}/reject"), &serde_json::json!({ "decisionNote": note })).await
}
```

### 11. Create `crates/lx-desktop/src/api/costs.rs`

Port `reference/paperclip/ui/src/api/costs.ts`.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSummary {
    pub total_cents: Option<i64>,
    pub currency: Option<String>,
}

pub async fn summary(
    client: &ApiClient,
    company_id: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<CostSummary, ApiError> {
    let mut path = format!("/companies/{company_id}/costs/summary");
    let mut params = Vec::new();
    if let Some(f) = from { params.push(format!("from={f}")); }
    if let Some(t) = to { params.push(format!("to={t}")); }
    if !params.is_empty() {
        path.push('?');
        path.push_str(&params.join("&"));
    }
    client.get(&path).await
}
```

### 12. Create `crates/lx-desktop/src/api/heartbeats.rs`

Port `reference/paperclip/ui/src/api/heartbeats.ts`.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRun {
    pub id: String,
    pub status: Option<String>,
    pub agent_id: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: Option<String>,
}

pub async fn list(
    client: &ApiClient,
    company_id: &str,
    agent_id: Option<&str>,
) -> Result<Vec<HeartbeatRun>, ApiError> {
    let mut path = format!("/companies/{company_id}/heartbeat-runs");
    if let Some(aid) = agent_id {
        path.push_str(&format!("?agentId={aid}"));
    }
    client.get(&path).await
}

pub async fn get(client: &ApiClient, run_id: &str) -> Result<HeartbeatRun, ApiError> {
    client.get(&format!("/heartbeat-runs/{run_id}")).await
}
```

### 13. Create `crates/lx-desktop/src/api/auth.rs`

Port `reference/paperclip/ui/src/api/auth.ts`.

```rust
use serde::{Deserialize, Serialize};
use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthSession {
    pub session: SessionInfo,
    pub user: UserInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: String,
    pub user_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

pub async fn get_session(client: &ApiClient) -> Result<Option<AuthSession>, ApiError> {
    match client.get::<AuthSession>("/auth/get-session").await {
        Ok(session) => Ok(Some(session)),
        Err(ApiError::Http { status: 401, .. }) => Ok(None),
        Err(e) => Err(e),
    }
}
```

### 14. Create `crates/lx-desktop/src/contexts/live_updates.rs`

Port `reference/paperclip/ui/src/context/LiveUpdatesProvider.tsx`. This is the most complex piece. The Paperclip version is 760 lines. Split the Dioxus version into the provider component and event handling logic.

The Dioxus version will be simpler because:
- No React Query cache to invalidate (lx-desktop uses Dioxus signals, not a query cache)
- Toast suppression logic can be deferred (just log to ActivityLog for now)
- The WebSocket connection uses the existing `lx-api` `ws_events` infrastructure

```rust
use dioxus::prelude::*;
use crate::contexts::activity_log::ActivityLog;

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

    rsx! { Outlet::<crate::routes::Route> {} }
}
```

`live_event_loop` function:

```rust
async fn live_event_loop(activity_log: ActivityLog) {
    loop {
        match connect_ws().await {
            Ok(mut ws) => {
                while let Some(msg) = ws.next().await {
                    match msg {
                        Ok(text) => {
                            if let Ok(event) = serde_json::from_str::<LiveEvent>(&text) {
                                handle_live_event(&activity_log, &event);
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            Err(_) => {}
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
```

`connect_ws` establishes a WebSocket connection to the lx-api server. Use the existing `ws_events` endpoint path `/ws/events`. The connection URL is constructed from a base URL signal/context or defaults to `ws://127.0.0.1:8080/ws/events`.

`handle_live_event` processes each event by type:

```rust
fn handle_live_event(activity_log: &ActivityLog, event: &LiveEvent) {
    let event_type = &event.event_type;
    let payload = event.payload.as_ref();

    match event_type.as_str() {
        "activity.logged" => {
            let message = payload
                .and_then(|p| p.get("action"))
                .and_then(|a| a.as_str())
                .unwrap_or("activity event");
            activity_log.push("live", message);
        }
        "agent.status" => {
            let agent_id = payload
                .and_then(|p| p.get("agentId"))
                .and_then(|a| a.as_str())
                .unwrap_or("unknown");
            let status = payload
                .and_then(|p| p.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            activity_log.push("agent_status", &format!("Agent {agent_id}: {status}"));
        }
        "heartbeat.run.status" => {
            let run_id = payload
                .and_then(|p| p.get("runId"))
                .and_then(|r| r.as_str())
                .unwrap_or("unknown");
            let status = payload
                .and_then(|p| p.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            activity_log.push("run_status", &format!("Run {run_id}: {status}"));
        }
        _ => {}
    }
}
```

Reconnection logic:
- On connection failure or close, wait with exponential backoff: `min(15s, 1s * 2^attempt)` capped at attempt 4
- Reset attempt counter on successful open
- The loop in `live_event_loop` handles this

### 15. Modify `crates/lx-desktop/src/contexts/mod.rs`

Add `pub mod live_updates;` to the existing file:

```rust
pub mod activity_log;
pub mod live_updates;
pub mod status_bar;
```

### 16. Modify `crates/lx-desktop/src/lib.rs`

Edit `lib.rs` -- add `pub mod api;` after `pub mod app;`. All other module declarations (`pub mod components;`, `pub mod hooks;`, `pub mod plugins;`, etc.) already exist from prior units.

### 17. Modify `crates/lx-desktop/src/layout/shell.rs`

Add `LiveUpdatesProvider` into the Shell layout. The provider wraps the main content area so it can provide context to all child routes.

Add import:
```rust
use crate::contexts::live_updates::LiveUpdatesProvider;
```

Replace the `Outlet::<Route> {}` in the main content area with:
```rust
LiveUpdatesProvider {}
```

Since `LiveUpdatesProvider` already renders `Outlet::<Route> {}` as its child, this is a drop-in replacement. The `LiveUpdatesProvider` component renders `Outlet` internally (see task 14).

## Line Count Verification

| File | Estimated lines |
|---|---|
| `api/mod.rs` | 15 |
| `api/client.rs` | ~90 |
| `api/companies.rs` | ~45 |
| `api/agents.rs` | ~45 |
| `api/issues.rs` | ~65 |
| `api/projects.rs` | ~40 |
| `api/goals.rs` | ~30 |
| `api/activity.rs` | ~35 |
| `api/approvals.rs` | ~35 |
| `api/costs.rs` | ~35 |
| `api/heartbeats.rs` | ~35 |
| `api/auth.rs` | ~35 |
| `contexts/live_updates.rs` | ~100 |
| `contexts/mod.rs` (modified) | 4 |
| `lib.rs` (modified) | 15 |
| `layout/shell.rs` (modified) | ~207 |

All under 300 lines.

## Definition of Done

1. `just diagnose` passes with zero warnings
2. `ApiClient` compiles and its `get`/`post`/`patch`/`delete` methods are callable with typed responses
3. All 11 API modules compile with their type definitions and async functions
4. `LiveUpdatesProvider` compiles and attempts WebSocket connection on mount
5. `LiveUpdatesProvider` logs received events to `ActivityLog`
6. `LiveUpdatesProvider` reconnects with exponential backoff on disconnect
7. Shell layout wraps content in `LiveUpdatesProvider`
8. All new files are under 300 lines
9. No code comments or doc strings in new files
