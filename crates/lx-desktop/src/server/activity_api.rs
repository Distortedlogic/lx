use dioxus::prelude::*;

use crate::contexts::activity_log::ActivityEvent;

use super::STATE;

#[get("/api/activity?limit")]
pub async fn get_activity(limit: Option<usize>) -> Result<Vec<ActivityEvent>> {
  let events = STATE.activity.read().await;
  let limit = limit.unwrap_or(100).min(500);
  Ok(events.iter().take(limit).cloned().collect())
}

#[post("/api/activity")]
pub async fn post_activity(event: ActivityEvent) -> Result<serde_json::Value> {
  let _ = STATE.event_tx.send(event.clone());
  let mut events = STATE.activity.write().await;
  events.push_front(event);
  if events.len() > 500 {
    events.pop_back();
  }
  Ok(serde_json::json!({ "status": "ok", "count": events.len() }))
}
