use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};

use super::control::{ControlChannelState, ControlCommand, ControlResponse, handle_command};

pub async fn run_stdin_control(state: Arc<ControlChannelState>) {
  let stdin = tokio::io::stdin();
  let reader = BufReader::new(stdin);
  let mut lines = reader.lines();

  while let Ok(Some(line)) = lines.next_line().await {
    let line = line.trim().to_string();
    if line.is_empty() {
      continue;
    }
    let cmd: ControlCommand = match serde_json::from_str(&line) {
      Ok(cmd) => cmd,
      Err(e) => {
        let resp = ControlResponse::err(format!("invalid command: {e}"));
        println!("{}", serde_json::to_string(&resp).unwrap_or_default());
        continue;
      },
    };
    let resp = handle_command(cmd, &state);
    println!("{}", serde_json::to_string(&resp).unwrap_or_default());

    if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
      break;
    }
  }
}
