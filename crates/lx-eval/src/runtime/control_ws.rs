use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use super::control::{ControlChannelState, ControlCommand, ControlResponse, handle_command};

pub async fn run_ws_control(addr: String, state: Arc<ControlChannelState>) {
  let listener = match TcpListener::bind(&addr).await {
    Ok(l) => l,
    Err(e) => {
      eprintln!("[control] ws bind failed on {addr}: {e}");
      return;
    },
  };
  eprintln!("[control] listening on ws://{addr}");

  let (stream, peer) = match listener.accept().await {
    Ok(s) => s,
    Err(e) => {
      eprintln!("[control] ws accept failed: {e}");
      return;
    },
  };
  eprintln!("[control] connection from {peer}");

  let ws = match accept_async(stream).await {
    Ok(ws) => ws,
    Err(e) => {
      eprintln!("[control] ws handshake failed: {e}");
      return;
    },
  };
  let (mut write, mut read) = ws.split();

  while let Some(Ok(msg)) = read.next().await {
    let text = match msg {
      Message::Text(t) => t.to_string(),
      Message::Close(_) => break,
      _ => continue,
    };
    let text = text.trim().to_string();
    if text.is_empty() {
      continue;
    }
    let cmd: ControlCommand = match serde_json::from_str(&text) {
      Ok(cmd) => cmd,
      Err(e) => {
        let resp = ControlResponse::err(format!("invalid command: {e}"));
        let _ = write.send(Message::Text(serde_json::to_string(&resp).unwrap_or_default().into())).await;
        continue;
      },
    };
    let resp = handle_command(cmd, &state);
    let _ = write.send(Message::Text(serde_json::to_string(&resp).unwrap_or_default().into())).await;

    if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
      break;
    }
  }
}
