use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use super::ServerState;
use crate::contexts::activity_log::ActivityEvent;

#[derive(Deserialize)]
struct ActivityQuery {
  limit: Option<usize>,
}

async fn get_activity(State(state): State<Arc<ServerState>>, Query(query): Query<ActivityQuery>) -> Json<Vec<ActivityEvent>> {
  let events = state.activity.read().await;
  let limit = query.limit.unwrap_or(100).min(500);
  let result: Vec<ActivityEvent> = events.iter().take(limit).cloned().collect();
  Json(result)
}

async fn post_activity(State(state): State<Arc<ServerState>>, Json(event): Json<ActivityEvent>) -> Json<serde_json::Value> {
  let mut events = state.activity.write().await;
  events.push_front(event);
  if events.len() > 500 {
    events.pop_back();
  }
  Json(serde_json::json!({ "status": "ok", "count": events.len() }))
}

pub fn routes() -> Router<Arc<ServerState>> {
  Router::new().route("/api/activity", get(get_activity).post(post_activity))
}
