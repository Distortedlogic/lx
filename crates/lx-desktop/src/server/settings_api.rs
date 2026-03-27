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
