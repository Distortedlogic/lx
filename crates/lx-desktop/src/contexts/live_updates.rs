use std::time::Duration;

use dioxus::prelude::*;
use futures::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::contexts::activity_log::ActivityLog;
use crate::routes::Route;

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

  rsx! {
    Outlet::<Route> {}

  }
}

async fn live_event_loop(activity_log: ActivityLog) {
  let mut backoff = Duration::from_secs(1);
  loop {
    let url = "ws://127.0.0.1:8080/ws/events";
    if let Ok((ws_stream, _)) = connect_async(url).await {
      backoff = Duration::from_secs(1);
      let (_sink, mut stream) = ws_stream.split();
      while let Some(Ok(msg)) = stream.next().await {
        if let Message::Text(text) = msg
          && let Ok(event) = serde_json::from_str::<LiveEvent>(&text)
        {
          handle_live_event(&activity_log, &event);
        }
      }
    }
    tokio::time::sleep(backoff).await;
    backoff = (backoff * 2).min(Duration::from_secs(15));
  }
}

fn handle_live_event(activity_log: &ActivityLog, event: &LiveEvent) {
  let event_type = &event.event_type;
  let payload = event.payload.as_ref();

  match event_type.as_str() {
    "activity.logged" => {
      let message = payload.and_then(|p| p.get("action")).and_then(|a| a.as_str()).unwrap_or("activity event");
      activity_log.push("live", message);
    },
    "agent.status" => {
      let agent_id = payload.and_then(|p| p.get("agentId")).and_then(|a| a.as_str()).unwrap_or("unknown");
      let status = payload.and_then(|p| p.get("status")).and_then(|s| s.as_str()).unwrap_or("unknown");
      activity_log.push("agent_status", &format!("Agent {agent_id}: {status}"));
    },
    "heartbeat.run.status" => {
      let run_id = payload.and_then(|p| p.get("runId")).and_then(|r| r.as_str()).unwrap_or("unknown");
      let status = payload.and_then(|p| p.get("status")).and_then(|s| s.as_str()).unwrap_or("unknown");
      activity_log.push("run_status", &format!("Run {run_id}: {status}"));
    },
    _ => {},
  }
}
