use dioxus::prelude::*;

use crate::types::{PendingPrompt, PromptResponse, RunStatus};

#[cfg(feature = "server")]
use std::sync::LazyLock;
#[cfg(feature = "server")]
use tokio::sync::RwLock;

#[cfg(feature = "server")]
use crate::activity_api::ACTIVITY;

#[cfg(feature = "server")]
static RUN_STATUS: LazyLock<RwLock<RunStatus>> = LazyLock::new(|| RwLock::new(RunStatus::default()));

#[cfg(feature = "server")]
static PROMPTS: LazyLock<RwLock<Vec<PendingPrompt>>> = LazyLock::new(|| RwLock::new(Vec::new()));

#[get("/api/health")]
pub async fn health() -> Result<serde_json::Value> {
  let guard = ACTIVITY.read().await;
  let event_count = (*guard).len();
  Ok(serde_json::json!({ "status": "ok", "events": event_count }))
}

#[get("/api/run/status")]
pub async fn get_run_status() -> Result<RunStatus> {
  Ok(RUN_STATUS.read().await.clone())
}

#[get("/api/run/prompts")]
pub async fn get_prompts() -> Result<Vec<PendingPrompt>> {
  Ok(PROMPTS.read().await.clone())
}

#[post("/api/run/respond")]
pub async fn post_respond(data: PromptResponse) -> Result<serde_json::Value> {
  PROMPTS.write().await.retain(|p| p.prompt_id != data.prompt_id);
  Ok(serde_json::json!({ "status": "ok" }))
}
