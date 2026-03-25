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
