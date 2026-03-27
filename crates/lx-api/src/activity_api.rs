use dioxus::prelude::*;

use crate::types::ActivityEvent;

#[cfg(feature = "server")]
use std::collections::VecDeque;
#[cfg(feature = "server")]
use std::sync::LazyLock;
#[cfg(feature = "server")]
use tokio::sync::{RwLock, broadcast};

#[cfg(feature = "server")]
pub(crate) static ACTIVITY: LazyLock<RwLock<VecDeque<ActivityEvent>>> = LazyLock::new(|| RwLock::new(VecDeque::new()));

#[cfg(feature = "server")]
pub static EVENT_TX: LazyLock<broadcast::Sender<ActivityEvent>> = LazyLock::new(|| broadcast::channel(1024).0);

#[get("/api/activity?limit")]
pub async fn get_activity(limit: Option<usize>) -> Result<Vec<ActivityEvent>> {
  let events = ACTIVITY.read().await;
  let limit = limit.unwrap_or(100).min(500);
  Ok(events.iter().take(limit).cloned().collect())
}

#[post("/api/activity")]
pub async fn post_activity(event: ActivityEvent) -> Result<serde_json::Value> {
  let _ = EVENT_TX.send(event.clone());
  let mut events = ACTIVITY.write().await;
  events.push_front(event);
  if events.len() > 500 {
    events.pop_back();
  }
  Ok(serde_json::json!({ "status": "ok", "count": events.len() }))
}
