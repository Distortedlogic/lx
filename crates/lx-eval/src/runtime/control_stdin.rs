use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};

use super::control::{ControlChannelState, ControlCommand, ControlResponse, handle_command};

fn render_response(resp: &ControlResponse) -> String {
  serde_json::to_string(resp).unwrap_or_else(|e| {
    serde_json::to_string(&ControlResponse::err(format!("response serialization failed: {e}")))
      .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"response serialization failed\"}".to_string())
  })
}

pub async fn run_stdin_control(state: Arc<ControlChannelState>) {
  let stdin = tokio::io::stdin();
  let reader = BufReader::new(stdin);
  let mut lines = reader.lines();

  loop {
    let line = match lines.next_line().await {
      Ok(Some(line)) => line,
      Ok(None) => break,
      Err(e) => {
        eprintln!("[control] stdin read failed: {e}");
        break;
      },
    };
    let line = line.trim().to_string();
    if line.is_empty() {
      continue;
    }
    let cmd: ControlCommand = match serde_json::from_str(&line) {
      Ok(cmd) => cmd,
      Err(e) => {
        let resp = ControlResponse::err(format!("invalid command: {e}"));
        println!("{}", render_response(&resp));
        continue;
      },
    };
    let resp = handle_command(cmd, &state);
    println!("{}", render_response(&resp));

    if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
      break;
    }
  }
}
