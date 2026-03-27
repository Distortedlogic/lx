use dioxus::fullstack::{WebSocketOptions, Websocket};
use dioxus::prelude::*;

use super::STATE;

#[get("/ws/events")]
pub async fn ws_events(options: WebSocketOptions) -> Result<Websocket<(), String>> {
  let rx = STATE.event_tx.subscribe();
  Ok(options.on_upgrade(move |mut tx| async move {
    let mut rx = rx;
    while let Ok(event) = rx.recv().await {
      let Ok(json) = serde_json::to_string(&event) else { continue };
      if tx.send(json).await.is_err() {
        break;
      }
    }
  }))
}
