use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use super::control::{ControlChannelState, ControlCommand, ControlResponse, handle_command};

pub async fn run_tcp_control(addr: String, state: Arc<ControlChannelState>) {
  let listener = match TcpListener::bind(&addr).await {
    Ok(l) => l,
    Err(e) => {
      eprintln!("[control] tcp bind failed on {addr}: {e}");
      return;
    },
  };
  eprintln!("[control] listening on tcp://{addr}");

  let (stream, peer) = match listener.accept().await {
    Ok(s) => s,
    Err(e) => {
      eprintln!("[control] tcp accept failed: {e}");
      return;
    },
  };
  eprintln!("[control] connection from {peer}");

  let (reader, mut writer) = stream.into_split();
  let mut lines = BufReader::new(reader).lines();

  while let Ok(Some(line)) = lines.next_line().await {
    let line = line.trim().to_string();
    if line.is_empty() {
      continue;
    }
    let cmd: ControlCommand = match serde_json::from_str(&line) {
      Ok(cmd) => cmd,
      Err(e) => {
        let resp = ControlResponse::err(format!("invalid command: {e}"));
        let mut out = serde_json::to_string(&resp).unwrap_or_default();
        out.push('\n');
        let _ = writer.write_all(out.as_bytes()).await;
        continue;
      },
    };
    let resp = handle_command(cmd, &state);
    let mut out = serde_json::to_string(&resp).unwrap_or_default();
    out.push('\n');
    let _ = writer.write_all(out.as_bytes()).await;

    if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
      break;
    }
  }
}
