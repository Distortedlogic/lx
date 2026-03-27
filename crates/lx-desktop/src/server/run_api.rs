use dioxus::prelude::*;

use super::{PendingPrompt, RunStatus, STATE};

#[get("/api/health")]
pub async fn health() -> Result<serde_json::Value> {
  let event_count = STATE.activity.read().await.len();
  Ok(serde_json::json!({ "status": "ok", "events": event_count }))
}

#[get("/api/run/status")]
pub async fn get_run_status() -> Result<RunStatus> {
  Ok(STATE.run_status.read().await.clone())
}

#[get("/api/run/prompts")]
pub async fn get_prompts() -> Result<Vec<PendingPrompt>> {
  Ok(STATE.prompts.read().await.clone())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PromptResponse {
  pub prompt_id: u64,
}

#[post("/api/run/respond")]
pub async fn post_respond(data: PromptResponse) -> Result<serde_json::Value> {
  STATE.prompts.write().await.retain(|p| p.prompt_id != data.prompt_id);
  Ok(serde_json::json!({ "status": "ok" }))
}
