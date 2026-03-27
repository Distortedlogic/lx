# Unit 7: Control Channel

## Goal

Implement the control channel: an external async task that sends commands (pause/resume/cancel/inspect/inject) to the interpreter while a program is running. Includes the stdin transport and `--control` CLI flag.

## Preconditions

- Unit 3 complete: Tool module dispatch works (cancel needs to interrupt in-flight tool calls)
- `Interpreter` at `crates/lx/src/interpreter/mod.rs` with `eval` method (line 120)
- `RuntimeCtx` at `crates/lx/src/runtime/mod.rs`
- CLI entry point — check `crates/lx-cli/src/main.rs` for where programs are launched
- `tokio` workspace dependency has `sync` feature (for `watch` channel, `Notify`)
- `tokio-tungstenite` is already a workspace dependency (for WebSocket transport)

## Step 1: Define control channel shared state

File: `crates/lx/src/control/mod.rs` (new module)

Create: `crates/lx/src/control/`

```rust
mod state;
mod commands;
mod transport;

pub use state::ControlState;
pub use commands::{ControlCommand, ControlResponse};
```

Register in `crates/lx/src/lib.rs`: add `pub mod control;`

### Shared state

File: `crates/lx/src/control/state.rs`

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;

pub struct ControlState {
  pause_tx: watch::Sender<bool>,
  pub pause_rx: watch::Receiver<bool>,
  pub cancel: Arc<AtomicBool>,
  pub inject_tx: tokio::sync::mpsc::Sender<crate::value::LxVal>,
  pub inject_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<crate::value::LxVal>>>,
}

impl ControlState {
  pub fn new() -> Self {
    let (pause_tx, pause_rx) = watch::channel(false);
    let (inject_tx, inject_rx) = tokio::sync::mpsc::channel(1);
    Self {
      pause_tx,
      pause_rx,
      cancel: Arc::new(AtomicBool::new(false)),
      inject_tx,
      inject_rx: Arc::new(tokio::sync::Mutex::new(inject_rx)),
    }
  }

  pub fn pause(&self) {
    let _ = self.pause_tx.send(true);
  }

  pub fn resume(&self) {
    let _ = self.pause_tx.send(false);
  }

  pub fn is_paused(&self) -> bool {
    *self.pause_rx.borrow()
  }

  pub fn cancel(&self) {
    self.cancel.store(true, Ordering::Relaxed);
  }

  pub fn is_cancelled(&self) -> bool {
    self.cancel.load(Ordering::Relaxed)
  }
}
```

No new dependencies needed. `AtomicBool` is in `std`.

## Step 2: Command types

File: `crates/lx/src/control/commands.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd")]
pub enum ControlCommand {
  #[serde(rename = "pause")]
  Pause,
  #[serde(rename = "resume")]
  Resume,
  #[serde(rename = "cancel")]
  Cancel,
  #[serde(rename = "inspect")]
  Inspect,
  #[serde(rename = "inject")]
  Inject { value: serde_json::Value },
}

#[derive(Debug, Serialize)]
pub struct ControlResponse {
  pub ok: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

impl ControlResponse {
  pub fn ok() -> Self { Self { ok: true, state: None, error: None } }
  pub fn with_state(state: serde_json::Value) -> Self { Self { ok: true, state: Some(state), error: None } }
  pub fn err(msg: impl Into<String>) -> Self { Self { ok: false, state: None, error: Some(msg.into()) } }
}
```

## Step 3: Add ControlState to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

```rust
pub control: Option<Arc<crate::control::ControlState>>,
```

Default: `None`. Set when `--control` flag is provided.

## Step 4: Check pause flag in eval loop

File: `crates/lx/src/interpreter/mod.rs`

At the top of `eval()` (line 121), before dispatching on the expression, add a pause check:

```rust
#[async_recursion(?Send)]
pub(crate) async fn eval(&mut self, eid: ExprId) -> EvalResult<LxVal> {
  // Check cancel
  if let Some(ref ctrl) = self.ctx.control {
    if ctrl.is_cancelled() {
      let span = self.arena.expr_span(eid);
      return Err(LxError::runtime("program cancelled", span).into());
    }
    // Check pause — wait on the watch channel for state change
    while ctrl.is_paused() {
      let mut rx = ctrl.pause_rx.clone();
      // Wait until the pause flag changes (resume or cancel)
      let _ = rx.changed().await;
    }
  }

  let span = self.arena.expr_span(eid);
  // ... rest of eval ...
```

Note: The interpreter is `#[async_recursion(?Send)]` (line 120). The `yield_now()` is valid here. The pause check is at the top of every eval step, which means it takes effect between expressions (not mid-tool-call). For cancel during tool calls, the MCP client would need cancellation support (deferred).

**Performance consideration:** This adds a branch to every expression evaluation. Since `control` is `Option<Arc<...>>`, the `None` case is a cheap pointer check. The `is_cancelled()` and `is_paused()` calls are atomic loads. Acceptable overhead.

## Step 5: Transport — stdin

File: `crates/lx/src/control/transport.rs`

```rust
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};

use super::commands::{ControlCommand, ControlResponse};
use super::state::ControlState;

pub async fn run_stdin_transport(state: Arc<ControlState>) {
  let stdin = tokio::io::stdin();
  let reader = BufReader::new(stdin);
  let mut lines = reader.lines();

  while let Ok(Some(line)) = lines.next_line().await {
    let response = handle_command(&line, &state);
    let json = serde_json::to_string(&response).unwrap_or_else(|_| r#"{"ok":false}"#.to_string());
    println!("{json}");
  }
}

fn handle_command(line: &str, state: &ControlState) -> ControlResponse {
  let cmd: ControlCommand = match serde_json::from_str(line) {
    Ok(c) => c,
    Err(e) => return ControlResponse::err(format!("parse error: {e}")),
  };

  match cmd {
    ControlCommand::Pause => {
      state.pause();
      ControlResponse::ok()
    },
    ControlCommand::Resume => {
      state.resume();
      ControlResponse::ok()
    },
    ControlCommand::Cancel => {
      state.cancel();
      ControlResponse::ok()
    },
    ControlCommand::Inspect => {
      // Return basic state info
      let state_json = serde_json::json!({
        "paused": state.is_paused(),
        "cancelled": state.is_cancelled(),
      });
      ControlResponse::with_state(state_json)
    },
    ControlCommand::Inject { value } => {
      let lx_val = crate::value::LxVal::from(value);
      match state.inject_tx.try_send(lx_val) {
        Ok(()) => ControlResponse::ok(),
        Err(_) => ControlResponse::err("no pending yield to inject into"),
      }
    },
  }
}
```

## Step 6: Wire --control CLI flag

File: `crates/lx-cli/src/main.rs`

The `Command::Run` variant is at lines 32-36. Add `--control` to the `Run` variant (not the top-level `Cli` struct):

```rust
#[derive(Subcommand)]
enum Command {
  Run {
    file: String,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    control: Option<String>,
  },
  // ... other variants unchanged ...
}
```

In `main.rs`, the `Command::Run { file, json, control }` arm calls `run_file`. Pass `control` through:

```rust
Command::Run { file, json, control } => {
  let resolved = resolve_run_target(&file);
  run_file(&resolved, json, control.as_deref())
},
```

Update `run_file` signature at line 163 to accept `control: Option<&str>`. Inside `run_file`, after creating `ctx_val` (line 173) and before wrapping in `Arc` (line 177):

```rust
if let Some(transport) = control {
  match transport {
    "stdin" => {
      let state = Arc::new(lx::control::ControlState::new());
      let state_clone = Arc::clone(&state);
      ctx_val.tokio_runtime.spawn(async move {
        lx::control::transport::run_stdin_transport(state_clone).await;
      });
      ctx_val.control = Some(state);
    },
    other => {
      eprintln!("unknown control transport: {other} (supported: stdin)");
    },
  }
}
```

WebSocket and TCP transports are out of scope for this unit. Only stdin is implemented. The `--control ws://` and `--control tcp://` paths in Step 6 print an error and continue without a control channel.

## Verification

1. Run `just diagnose`
2. Test: run an lx program with `--control stdin`
3. Send `{"cmd": "inspect"}` on stdin — should get `{"ok": true, "state": {...}}` on stdout
4. Send `{"cmd": "pause"}` — program should pause
5. Send `{"cmd": "resume"}` — program should continue
6. Send `{"cmd": "cancel"}` — program should exit with "cancelled" error
