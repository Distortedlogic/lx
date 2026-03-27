# Unit 7: Control Channel

Add a `--control` CLI flag with stdin/WebSocket/TCP transports for external control of the interpreter during execution (pause, resume, cancel, inspect, inject).

## Prerequisites

- **Unit 3** (Tool Module) must be complete -- tool connections exist on RuntimeCtx for shutdown on cancel
- **Unit 5** (Stream Module) must be complete -- event stream exists on RuntimeCtx for inspect

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify
- Prefer established crates over custom code

## Current State

### CLI

- `crates/lx-cli/src/main.rs` defines CLI with clap (lines 22-95)
- `Command::Run` has `file: String` and `json: bool` fields (lines 32-36)
- `run_file` function (lines 163-195) creates `RuntimeCtx`, calls `run::run`
- `run::run` in `crates/lx-cli/src/run.rs` (lines 8-32) spawns interpreter inside `tokio_runtime.block_on`

### Interpreter

- `Interpreter::eval` in `crates/lx/src/interpreter/mod.rs` (lines 120-217) is the main eval loop
- The eval method is `#[async_recursion(?Send)]`
- `Interpreter::exec` (lines 91-118) runs program stmts
- `RuntimeCtx` in `crates/lx/src/runtime/mod.rs` (lines 20-39+)

### Dependencies

- `tokio` with features `macros`, `rt-multi-thread`, `sync`, `time` (+ `process`, `io-util` from Unit 1)
- `tokio-tungstenite` already in workspace dependencies (line 70 of root `Cargo.toml`)
- `serde_json` already a dependency

## Files to Create

- `crates/lx/src/runtime/control.rs` -- ControlChannel struct, command types, shared state
- `crates/lx/src/runtime/control_transport.rs` -- stdin, WebSocket, TCP transport implementations

## Files to Modify

- `crates/lx/src/runtime/mod.rs` -- add control module, add control state to RuntimeCtx
- `crates/lx-cli/src/main.rs` -- add `--control` flag to Run command
- `crates/lx-cli/src/run.rs` -- spawn control task, pass to RuntimeCtx
- `crates/lx/src/interpreter/mod.rs` -- add pause check at top of eval

## Step 1: Define control types and shared state

File: `crates/lx/src/runtime/control.rs`

### Command types

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Notify};

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
    pub fn ok() -> Self {
        Self { ok: true, state: None, error: None }
    }

    pub fn with_state(state: serde_json::Value) -> Self {
        Self { ok: true, state: Some(state), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, state: None, error: Some(msg.into()) }
    }
}
```

### Shared control state

```rust
pub struct ControlState {
    pub paused: AtomicBool,
    pub cancelled: AtomicBool,
    pub pause_notify: Notify,
    pub inject_tx: parking_lot::Mutex<Option<mpsc::UnboundedSender<crate::value::LxVal>>>,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            paused: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            pause_notify: Notify::new(),
            inject_tx: parking_lot::Mutex::new(None),
        }
    }
}

impl ControlState {
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        self.pause_notify.notify_waiters();
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.resume();
    }

    pub async fn wait_if_paused(&self) {
        loop {
            let notified = self.pause_notify.notified();
            if !self.paused.load(Ordering::SeqCst) {
                break;
            }
            notified.await;
        }
    }
}
```

### Command processor

```rust
pub async fn process_command(
    cmd: ControlCommand,
    state: &Arc<ControlState>,
) -> ControlResponse {
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
            let info = serde_json::json!({
                "paused": state.is_paused(),
                "cancelled": state.is_cancelled(),
            });
            ControlResponse::with_state(info)
        },
        ControlCommand::Inject { value } => {
            let lx_val = crate::value::LxVal::from(value);
            let tx_guard = state.inject_tx.lock();
            if let Some(ref tx) = *tx_guard {
                match tx.send(lx_val) {
                    Ok(()) => ControlResponse::ok(),
                    Err(_) => ControlResponse::err("no pending yield"),
                }
            } else {
                ControlResponse::err("inject channel not set up")
            }
        },
    }
}
```

## Step 2: Implement transport layer

File: `crates/lx/src/runtime/control_transport.rs`

### Transport kind and parser

```rust
use std::sync::Arc;

use super::control::{ControlCommand, ControlResponse, ControlState, process_command};

pub enum TransportKind {
    Stdin,
    WebSocket(String),
    Tcp(String),
}

impl TransportKind {
    pub fn parse(spec: &str) -> Result<Self, String> {
        match spec {
            "stdin" => Ok(TransportKind::Stdin),
            s if s.starts_with("ws://") => Ok(TransportKind::WebSocket(s.to_string())),
            s if s.starts_with("tcp://") => Ok(TransportKind::Tcp(s.to_string())),
            other => Err(format!("unknown control transport: '{other}' (use stdin, ws://..., or tcp://...)")),
        }
    }
}
```

### Spawn function

```rust
pub fn spawn_control_task(
    kind: TransportKind,
    state: Arc<ControlState>,
) -> tokio::task::JoinHandle<()> {
    match kind {
        TransportKind::Stdin => tokio::spawn(stdin_transport(state)),
        TransportKind::WebSocket(addr) => tokio::spawn(ws_transport(addr, state)),
        TransportKind::Tcp(addr) => tokio::spawn(tcp_transport(addr, state)),
    }
}
```

### Stdin transport

```rust
async fn stdin_transport(state: Arc<ControlState>) {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<ControlCommand>(&line) {
            Ok(cmd) => {
                let resp = process_command(cmd, &state).await;
                if let Ok(json) = serde_json::to_string(&resp) {
                    println!("{json}");
                }
            },
            Err(e) => {
                let resp = ControlResponse::err(format!("invalid command: {e}"));
                if let Ok(json) = serde_json::to_string(&resp) {
                    println!("{json}");
                }
            },
        }
    }
}
```

### TCP transport

```rust
async fn tcp_transport(addr: String, state: Arc<ControlState>) {
    let addr = addr.strip_prefix("tcp://").unwrap_or(&addr);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("control: cannot bind tcp://{addr}: {e}");
            return;
        },
    };

    while let Ok((stream, _)) = listener.accept().await {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let (reader, mut writer) = stream.into_split();
            let reader = BufReader::new(reader);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<ControlCommand>(line.trim()) {
                    Ok(cmd) => {
                        let resp = process_command(cmd, &state).await;
                        if let Ok(json) = serde_json::to_string(&resp) {
                            let _ = writer.write_all(format!("{json}\n").as_bytes()).await;
                        }
                    },
                    Err(e) => {
                        let resp = ControlResponse::err(format!("invalid command: {e}"));
                        if let Ok(json) = serde_json::to_string(&resp) {
                            let _ = writer.write_all(format!("{json}\n").as_bytes()).await;
                        }
                    },
                }
            }
        });
    }
}
```

### WebSocket transport

```rust
async fn ws_transport(addr: String, state: Arc<ControlState>) {
    let addr = addr.strip_prefix("ws://").unwrap_or(&addr);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("control: cannot bind ws://{addr}: {e}");
            return;
        },
    };

    while let Ok((stream, _)) = listener.accept().await {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let ws = match tokio_tungstenite::accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("control: ws handshake failed: {e}");
                    return;
                },
            };
            use futures::stream::StreamExt;
            use futures::sink::SinkExt;
            let (mut write, mut read) = ws.split();
            while let Some(Ok(msg)) = read.next().await {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    match serde_json::from_str::<ControlCommand>(&text) {
                        Ok(cmd) => {
                            let resp = process_command(cmd, &state).await;
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = write.send(
                                    tokio_tungstenite::tungstenite::Message::Text(json.into())
                                ).await;
                            }
                        },
                        Err(e) => {
                            let resp = ControlResponse::err(format!("invalid command: {e}"));
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = write.send(
                                    tokio_tungstenite::tungstenite::Message::Text(json.into())
                                ).await;
                            }
                        },
                    }
                }
            }
        });
    }
}
```

### tokio "net" and "io-std" features

The TCP transport needs `tokio::net` which requires the `net` feature on tokio. The stdin transport uses `tokio::io::stdin()` which requires the `io-std` feature. Add both to the tokio features in the workspace `Cargo.toml`:

```toml
tokio = { version = "1.50.0", features = ["io-std", "io-util", "macros", "net", "process", "rt-multi-thread", "sync", "time"] }
```

`tokio-tungstenite` is already in `crates/lx/Cargo.toml` (line 40: `tokio-tungstenite.workspace = true`). The `futures` crate is already present (line 20: `futures.workspace = true`).

## Step 3: Add control state to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

### 3a: Add module declarations

Add after existing module declarations:
```rust
pub mod control;
pub mod control_transport;
```

### 3b: Add field to RuntimeCtx

Add after existing fields:
```rust
pub control: Option<Arc<control::ControlState>>,
```

SmartDefault defaults `Option<...>` to `None`.

### 3c: Add re-exports

```rust
pub use control::{ControlCommand, ControlResponse, ControlState};
pub use control_transport::{TransportKind, spawn_control_task};
```

## Step 4: Add --control flag to CLI

File: `crates/lx-cli/src/main.rs`

### 4a: Add field to Run command

In the `Command::Run` variant (lines 32-36):

Current:
```rust
Run {
    file: String,
    #[arg(long)]
    json: bool,
},
```

Change to:
```rust
Run {
    file: String,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    control: Option<String>,
},
```

### 4b: Pass control to run_file

In `main()` (line 100-103):

Current:
```rust
Command::Run { file, json } => {
    let resolved = resolve_run_target(&file);
    run_file(&resolved, json)
},
```

Change to:
```rust
Command::Run { file, json, control } => {
    let resolved = resolve_run_target(&file);
    run_file(&resolved, json, control.as_deref())
},
```

### 4c: Update run_file signature and body

Change signature from:
```rust
fn run_file(path: &str, _json: bool) -> ExitCode {
```
to:
```rust
fn run_file(path: &str, _json: bool, control: Option<&str>) -> ExitCode {
```

After the existing `ctx_val` setup (workspace_members, dep_dirs, apply_manifest_backends), add control state setup. Enter the tokio runtime context so `tokio::spawn` works inside `spawn_control_task`:

Insert before `let ctx = Arc::new(ctx_val);`:

```rust
if let Some(spec) = control {
    match lx::runtime::TransportKind::parse(spec) {
        Ok(kind) => {
            let state = Arc::new(lx::runtime::ControlState::default());
            ctx_val.control = Some(Arc::clone(&state));
            let rt = ctx_val.tokio_runtime.clone();
            let _guard = rt.enter();
            let _handle = lx::runtime::spawn_control_task(kind, state);
        },
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        },
    }
}
```

The `rt = ctx_val.tokio_runtime.clone()` clones the `Arc<Runtime>` (cheap) so `_guard` borrows `rt` instead of `ctx_val`, allowing `ctx_val` to be moved into `Arc::new(ctx_val)` afterwards. The `_guard = rt.enter()` enters the tokio runtime context so that `tokio::spawn` works inside `spawn_control_task`.

## Step 5: Add pause check at top of eval

File: `crates/lx/src/interpreter/mod.rs`

In the `eval` method (line 121), add a pause/cancel check at the very top, before matching on the expression.

Current (lines 120-124):
```rust
#[async_recursion(?Send)]
pub(crate) async fn eval(&mut self, eid: ExprId) -> EvalResult<LxVal> {
    let span = self.arena.expr_span(eid);
    let expr = self.arena.expr(eid).clone();
    match expr {
```

Insert between `eval` signature and `let span`:
```rust
if let Some(ref ctrl) = self.ctx.control {
    if ctrl.is_cancelled() {
        return Err(LxError::runtime("cancelled", self.arena.expr_span(eid)).into());
    }
    ctrl.wait_if_paused().await;
}
```

Full result:
```rust
#[async_recursion(?Send)]
pub(crate) async fn eval(&mut self, eid: ExprId) -> EvalResult<LxVal> {
    if let Some(ref ctrl) = self.ctx.control {
        if ctrl.is_cancelled() {
            return Err(LxError::runtime("cancelled", self.arena.expr_span(eid)).into());
        }
        ctrl.wait_if_paused().await;
    }
    let span = self.arena.expr_span(eid);
    let expr = self.arena.expr(eid).clone();
    match expr {
```

This checks before every expression evaluation:
1. If cancelled, immediately return an error
2. If paused, wait until resumed

The `wait_if_paused` is async and blocks the eval loop. Pause takes effect between expression evaluations. If the interpreter is mid-tool-call (in an MCP request), the pause takes effect when the call returns and eval resumes.

## Step 6: Check file lengths

- `control.rs`: ~120 lines (types + shared state + processor) -- under 300
- `control_transport.rs`: ~150 lines (stdin + TCP + WS) -- under 300
- `mod.rs` changes: +5 lines -- still under 300
- `main.rs` changes: +20 lines -- still under 300 (currently 255)
- `interpreter/mod.rs` changes: +5 lines -- still under 300

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. All existing tests pass unchanged (control is opt-in via `--control`)
3. Manual test: Run `lx run main.lx --control stdin` and type `{"cmd":"pause"}` then `{"cmd":"resume"}` to verify pause/resume works
4. Manual test: Run `lx run main.lx --control tcp://localhost:9000` and connect with `nc localhost 9000` to send commands
