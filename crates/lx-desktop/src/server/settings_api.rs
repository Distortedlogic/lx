use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use super::ServerState;
use crate::pages::settings::state::SettingsData;

async fn get_settings(State(state): State<Arc<ServerState>>) -> Json<SettingsData> {
  let settings = state.settings.read().await.clone();
  Json(settings)
}

async fn put_settings(State(state): State<Arc<ServerState>>, Json(new_settings): Json<SettingsData>) -> Json<serde_json::Value> {
  let mut settings = state.settings.write().await;
  *settings = new_settings.clone();
  drop(settings);

  let result = tokio::fs::write(&state.settings_path, serde_json::to_string_pretty(&new_settings).unwrap_or_default()).await;

  match result {
    Ok(()) => Json(serde_json::json!({ "status": "saved" })),
    Err(e) => Json(serde_json::json!({ "status": "error", "message": format!("{e}") })),
  }
}

pub fn routes() -> Router<Arc<ServerState>> {
  Router::new().route("/api/settings", get(get_settings).put(put_settings))
}
