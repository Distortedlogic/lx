use dioxus::fullstack::{WebSocketOptions, Websocket};
use dioxus::prelude::*;

use crate::types::ActivityEvent;

#[cfg(feature = "server")]
use crate::activity_api::EVENT_TX;

#[get("/ws/events")]
pub async fn ws_events(options: WebSocketOptions) -> Result<Websocket<(), ActivityEvent>> {
  let rx = EVENT_TX.subscribe();
  Ok(options.on_upgrade(move |mut tx| async move {
    let mut rx = rx;
    while let Ok(event) = rx.recv().await {
      if tx.send(event).await.is_err() {
        break;
      }
    }
  }))
}
