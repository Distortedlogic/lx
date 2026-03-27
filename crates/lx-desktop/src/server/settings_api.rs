use dioxus::prelude::*;

use crate::pages::settings::state::SettingsData;

use super::STATE;

#[get("/api/settings")]
pub async fn get_settings() -> Result<SettingsData> {
  Ok(STATE.settings.read().await.clone())
}

#[put("/api/settings")]
pub async fn put_settings(new_settings: SettingsData) -> Result<serde_json::Value> {
  let mut settings = STATE.settings.write().await;
  *settings = new_settings.clone();
  drop(settings);
  let _ = tokio::fs::write(&STATE.settings_path, serde_json::to_string_pretty(&new_settings).unwrap_or_default()).await;
  Ok(serde_json::json!({ "status": "saved" }))
}
