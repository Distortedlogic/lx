# Work Item 8: Control Channel

A separate async task that accepts external commands (pause, resume, cancel, inspect, inject) over stdin, WebSocket, or TCP. The interpreter checks a pause flag at expression boundaries. The control channel acts on interpreter state from outside via atomic flags and channels.

## Prerequisites

- Work item 1 (event stream) must be complete -- `EventStream` with `xadd` exists, `RuntimeCtx` has `event_stream` field.
- Work item 5 (agent system refactor) must be complete -- named agent registry with `AgentHandle` containing `pause_flag: Arc<AtomicBool>`, `get_agent_pause_flag`, `agent_names` functions exist.
- Work item 6 (agent messaging) must be complete -- interpreter has `agent_name: Option<String>` field on `Interpreter`.

## Current State

- `crates/lx/src/runtime/mod.rs` -- `RuntimeCtx` struct with `SmartDefault`, holds `tokio_runtime: Arc<tokio::runtime::Runtime>`, `yield_: Arc<dyn YieldBackend>`, `event_stream` field.
- `crates/lx/src/interpreter/mod.rs` -- `Interpreter` struct with `ctx: Arc<RuntimeCtx>`, `env: Arc<Env>`, `arena: Arc<AstArena>`. The `eval` method (line 121) is the core expression evaluation entry point called for every expression.
- `crates/lx/src/interpreter/eval.rs` -- `eval_loop` (line 42) calls `tokio::task::yield_now().await` at the top of each iteration.
- `crates/lx/src/error.rs` -- `EvalSignal { Error(LxError), Break(LxVal) }`.
- `crates/lx/src/runtime/agent_registry.rs` (from work item 5) -- `AgentHandle` with `pause_flag: Arc<AtomicBool>`, `register_agent`, `get_agent_mailbox`, `remove_agent`, `agent_exists`, `agent_names`.
- `crates/lx-cli/src/main.rs` -- `Cli` struct with clap `#[derive(Parser)]`, `Command` enum with `Run { file: String, json: bool }`. `run_file` function (line 166) constructs `RuntimeCtx`, calls `run::run`.
- `crates/lx-cli/src/run.rs` -- `run(source, filename, ctx)` creates `Interpreter`, calls `ctx.tokio_runtime.block_on(async { interp.exec(&program) })`.
- `tokio` is a workspace dependency with `sync`, `net`, `io`, `time`, `macros` features.
- `serde_json` is a workspace dependency.
- `tokio-tungstenite` is NOT currently a workspace dependency (needs adding for WebSocket).

## Files to Create

- `crates/lx/src/runtime/control.rs` -- command/response types, ControlChannelState struct, command handlers, ControlYieldBackend
- `crates/lx/src/runtime/control_stdin.rs` -- stdin transport: read JSON lines from stdin, write responses to stdout
- `crates/lx/src/runtime/control_ws.rs` -- WebSocket transport: listen on a port, accept one connection, read/write WebSocket text messages
- `crates/lx/src/runtime/control_tcp.rs` -- TCP transport: listen on a port, accept one connection, read/write newline-delimited JSON

## Files to Modify

- `crates/lx/src/runtime/mod.rs` -- add `pub mod control; pub mod control_stdin; pub mod control_ws; pub mod control_tcp; pub use control::*;`, add `global_pause`, `cancel_flag`, `inject_tx`, `inject_rx` fields to `RuntimeCtx`
- `crates/lx/src/interpreter/mod.rs` -- add cancel check and global pause check at top of `eval` method, add per-agent pause check
- `crates/lx-cli/src/main.rs` -- add `#[arg(long)] control: Option<String>` to `Command::Run`, pass to `run_file`, pass to `run::run`
- `crates/lx-cli/src/run.rs` -- accept `control: Option<&str>` parameter, parse transport spec, spawn control task inside tokio runtime, swap yield backend when `--control stdin`
- `crates/lx/Cargo.toml` -- add `tokio-tungstenite` dependency
- `Cargo.toml` (workspace root) -- add `tokio-tungstenite` to `[workspace.dependencies]`
- `crates/lx/src/runtime/agent_registry.rs` (from work item 5) -- add `get_agent_pause_flag` function if not already present

## Step 1: Add tokio-tungstenite dependency

File: `Cargo.toml` (workspace root, `[workspace.dependencies]` section)

Add:

```toml
tokio-tungstenite = "0.24"
```

Check crates.io for the latest stable version before adding.

File: `crates/lx/Cargo.toml`

Add under `[dependencies]`:

```toml
tokio-tungstenite = { workspace = true }
```

Also confirm `futures-util` is a dependency (needed for `StreamExt`/`SinkExt` on WebSocket streams). If not present, add:

```toml
futures-util = { workspace = true }
```

## Step 2: Define control types and command handlers

File: `crates/lx/src/runtime/control.rs`

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::error::LxError;
use crate::value::LxVal;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd")]
pub enum ControlCommand {
    #[serde(rename = "pause")]
    Pause { agent: Option<String> },
    #[serde(rename = "resume")]
    Resume { agent: Option<String> },
    #[serde(rename = "cancel")]
    Cancel,
    #[serde(rename = "inspect")]
    Inspect,
    #[serde(rename = "inject")]
    Inject { value: serde_json::Value },
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<InspectState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectState {
    pub paused: bool,
    pub agents: Vec<AgentInspect>,
    pub stream_position: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentInspect {
    pub name: String,
    pub paused: bool,
}

impl ControlResponse {
    pub fn ok() -> Self {
        Self { ok: true, error: None, state: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, error: Some(msg.into()), state: None }
    }

    pub fn with_state(state: InspectState) -> Self {
        Self { ok: true, error: None, state: Some(state) }
    }
}

pub struct ControlChannelState {
    pub global_pause: Arc<AtomicBool>,
    pub cancel_flag: Arc<AtomicBool>,
    pub inject_tx: Option<tokio::sync::mpsc::Sender<LxVal>>,
}

pub fn handle_command(
    cmd: ControlCommand,
    state: &ControlChannelState,
) -> ControlResponse {
    match cmd {
        ControlCommand::Pause { agent } => handle_pause(agent, state),
        ControlCommand::Resume { agent } => handle_resume(agent, state),
        ControlCommand::Cancel => handle_cancel(state),
        ControlCommand::Inspect => handle_inspect(state),
        ControlCommand::Inject { value } => handle_inject(value, state),
    }
}

fn handle_pause(agent: Option<String>, state: &ControlChannelState) -> ControlResponse {
    match agent {
        None => {
            state.global_pause.store(true, Ordering::SeqCst);
            ControlResponse::ok()
        }
        Some(name) => {
            match crate::runtime::agent_registry::get_agent_pause_flag(&name) {
                Some(flag) => {
                    flag.store(true, Ordering::SeqCst);
                    ControlResponse::ok()
                }
                None => ControlResponse::err(format!("agent '{}' not found", name)),
            }
        }
    }
}

fn handle_resume(agent: Option<String>, state: &ControlChannelState) -> ControlResponse {
    match agent {
        None => {
            state.global_pause.store(false, Ordering::SeqCst);
            ControlResponse::ok()
        }
        Some(name) => {
            match crate::runtime::agent_registry::get_agent_pause_flag(&name) {
                Some(flag) => {
                    flag.store(false, Ordering::SeqCst);
                    ControlResponse::ok()
                }
                None => ControlResponse::err(format!("agent '{}' not found", name)),
            }
        }
    }
}

fn handle_cancel(state: &ControlChannelState) -> ControlResponse {
    state.cancel_flag.store(true, Ordering::SeqCst);
    state.global_pause.store(false, Ordering::SeqCst);
    ControlResponse::ok()
}

fn handle_inspect(state: &ControlChannelState) -> ControlResponse {
    let paused = state.global_pause.load(Ordering::SeqCst);
    let agent_names = crate::runtime::agent_registry::agent_names();
    let agents: Vec<AgentInspect> = agent_names
        .into_iter()
        .map(|name| {
            let agent_paused = crate::runtime::agent_registry::get_agent_pause_flag(&name)
                .map(|f| f.load(Ordering::Relaxed))
                .unwrap_or(false);
            AgentInspect { name, paused: agent_paused }
        })
        .collect();
    let stream_position = String::from("0-0");
    ControlResponse::with_state(InspectState { paused, agents, stream_position })
}

fn handle_inject(value: serde_json::Value, state: &ControlChannelState) -> ControlResponse {
    let lx_val = json_to_lxval(value);
    match &state.inject_tx {
        Some(tx) => match tx.try_send(lx_val) {
            Ok(()) => ControlResponse::ok(),
            Err(_) => ControlResponse::err("no pending yield to inject into"),
        },
        None => ControlResponse::err("inject not available (no yield backend configured)"),
    }
}

fn json_to_lxval(value: serde_json::Value) -> LxVal {
    match value {
        serde_json::Value::Null => LxVal::None,
        serde_json::Value::Bool(b) => LxVal::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                LxVal::int(i)
            } else if let Some(f) = n.as_f64() {
                LxVal::float(f)
            } else {
                LxVal::None
            }
        }
        serde_json::Value::String(s) => LxVal::str(s),
        serde_json::Value::Array(a) => {
            LxVal::list(a.into_iter().map(json_to_lxval).collect())
        }
        serde_json::Value::Object(m) => {
            let fields: indexmap::IndexMap<crate::sym::Sym, LxVal> = m
                .into_iter()
                .map(|(k, v)| (crate::sym::intern(&k), json_to_lxval(v)))
                .collect();
            LxVal::record(fields)
        }
    }
}

pub struct ControlYieldBackend {
    pub inject_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<LxVal>>>,
}

impl crate::runtime::YieldBackend for ControlYieldBackend {
    fn yield_value(
        &self,
        value: LxVal,
        span: miette::SourceSpan,
    ) -> Result<LxVal, LxError> {
        println!(
            "{}",
            serde_json::to_string(&value).unwrap_or_else(|_| value.to_string())
        );
        let rx = Arc::clone(&self.inject_rx);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut guard = rx.lock().await;
                guard.recv().await.ok_or_else(|| {
                    LxError::runtime("yield: inject channel closed", span)
                })
            })
        })
    }
}
```

This file is approximately 180 lines. Under 300.

## Step 3: Implement stdin transport

File: `crates/lx/src/runtime/control_stdin.rs`

```rust
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
                let resp = ControlResponse::err(format!("invalid command: {}", e));
                println!("{}", serde_json::to_string(&resp).unwrap_or_default());
                continue;
            }
        };
        let resp = handle_command(cmd, &state);
        println!("{}", serde_json::to_string(&resp).unwrap_or_default());

        if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
    }
}
```

Approximately 30 lines. Under 300.

## Step 4: Implement TCP transport

File: `crates/lx/src/runtime/control_tcp.rs`

```rust
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
        }
    };
    eprintln!("[control] listening on tcp://{addr}");

    let (stream, peer) = match listener.accept().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[control] tcp accept failed: {e}");
            return;
        }
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
                let resp = ControlResponse::err(format!("invalid command: {}", e));
                let mut out = serde_json::to_string(&resp).unwrap_or_default();
                out.push('\n');
                let _ = writer.write_all(out.as_bytes()).await;
                continue;
            }
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
```

Approximately 55 lines. Under 300.

## Step 5: Implement WebSocket transport

File: `crates/lx/src/runtime/control_ws.rs`

```rust
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
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
        }
    };
    eprintln!("[control] listening on ws://{addr}");

    let (stream, peer) = match listener.accept().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[control] ws accept failed: {e}");
            return;
        }
    };
    eprintln!("[control] connection from {peer}");

    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[control] ws handshake failed: {e}");
            return;
        }
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
                let resp = ControlResponse::err(format!("invalid command: {}", e));
                let _ = write
                    .send(Message::Text(
                        serde_json::to_string(&resp).unwrap_or_default().into(),
                    ))
                    .await;
                continue;
            }
        };
        let resp = handle_command(cmd, &state);
        let _ = write
            .send(Message::Text(
                serde_json::to_string(&resp).unwrap_or_default().into(),
            ))
            .await;

        if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
    }
}
```

Approximately 70 lines. Under 300.

## Step 6: Add control channel fields to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

Add module declarations after existing ones:

```rust
pub mod control;
pub mod control_stdin;
pub mod control_tcp;
pub mod control_ws;
pub use control::*;
```

Add four fields to `RuntimeCtx` (after the `event_stream` field or after `tokio_runtime`):

```rust
#[default(Arc::new(std::sync::atomic::AtomicBool::new(false)))]
pub global_pause: Arc<std::sync::atomic::AtomicBool>,
#[default(Arc::new(std::sync::atomic::AtomicBool::new(false)))]
pub cancel_flag: Arc<std::sync::atomic::AtomicBool>,
pub inject_tx: Option<tokio::sync::mpsc::Sender<crate::value::LxVal>>,
pub inject_rx: parking_lot::Mutex<Option<tokio::sync::mpsc::Receiver<crate::value::LxVal>>>,
```

`inject_tx` and `inject_rx` default to `None` via `SmartDefault` (since `Option<T>` defaults to `None`).

## Step 7: Add pause-flag and cancel check to interpreter eval loop

File: `crates/lx/src/interpreter/mod.rs`

At the **top** of the `eval` method (line 121), before `let span = self.arena.expr_span(eid);`, add:

```rust
if self.ctx.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
    return Err(LxError::runtime(
        "program cancelled via control channel",
        self.arena.expr_span(eid),
    ).into());
}

while self.ctx.global_pause.load(std::sync::atomic::Ordering::Relaxed) {
    tokio::task::yield_now().await;
    if self.ctx.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(LxError::runtime(
            "program cancelled via control channel",
            self.arena.expr_span(eid),
        ).into());
    }
}
```

The cancel check is a single atomic load. In the non-paused, non-cancelled case (the overwhelmingly common case), this adds two atomic loads per expression: one for cancel, one for pause. Both are single CPU instructions.

When paused, the interpreter loops with `yield_now()` until the pause flag is cleared. Each iteration re-checks the cancel flag so that `cancel` works while paused.

Also add per-agent pause check (depends on work item 5). After the global pause check, before `let span = ...`:

```rust
if let Some(ref name) = self.agent_name {
    if let Some(flag) = crate::runtime::agent_registry::get_agent_pause_flag(name) {
        while flag.load(std::sync::atomic::Ordering::Relaxed) {
            tokio::task::yield_now().await;
        }
    }
}
```

Full eval method signature after modification:

```rust
#[async_recursion(?Send)]
pub(crate) async fn eval(&mut self, eid: ExprId) -> EvalResult<LxVal> {
    if self.ctx.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(LxError::runtime(
            "program cancelled via control channel",
            self.arena.expr_span(eid),
        ).into());
    }
    while self.ctx.global_pause.load(std::sync::atomic::Ordering::Relaxed) {
        tokio::task::yield_now().await;
        if self.ctx.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(LxError::runtime(
                "program cancelled via control channel",
                self.arena.expr_span(eid),
            ).into());
        }
    }
    if let Some(ref name) = self.agent_name {
        if let Some(flag) = crate::runtime::agent_registry::get_agent_pause_flag(name) {
            while flag.load(std::sync::atomic::Ordering::Relaxed) {
                tokio::task::yield_now().await;
            }
        }
    }

    let span = self.arena.expr_span(eid);
    let expr = self.arena.expr(eid).clone();
    match expr {
        // ... existing match arms unchanged ...
    }
}
```

## Step 8: Add --control CLI flag

File: `crates/lx-cli/src/main.rs`

Add `--control` field to `Command::Run`:

```rust
Run {
    file: String,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    control: Option<String>,
},
```

Update the `match` in `main()`:

```rust
Command::Run { file, json, control } => {
    let resolved = resolve_run_target(&file);
    run_file(&resolved, json, control.as_deref())
},
```

Update `run_file` signature:

```rust
fn run_file(path: &str, _json: bool, control_spec: Option<&str>) -> ExitCode {
```

In `run_file`, after constructing `ctx_val` and before `let ctx = Arc::new(ctx_val);`, handle the control spec:

```rust
if let Some(spec) = control_spec {
    if spec == "stdin" {
        let (inject_tx, inject_rx) = tokio::sync::mpsc::channel::<lx::value::LxVal>(1);
        ctx_val.inject_tx = Some(inject_tx);
        ctx_val.yield_ = Arc::new(lx::runtime::ControlYieldBackend {
            inject_rx: Arc::new(tokio::sync::Mutex::new(inject_rx)),
        });
    }
}
```

This replaces the stdin yield backend with one that waits for inject commands. For TCP and WebSocket transports, the normal yield backend is kept (stdin is still available for yield).

Update the call to `run::run`:

```rust
match run::run(&source, path, &ctx, control_spec) {
```

## Step 9: Wire control channel into run command

File: `crates/lx-cli/src/run.rs`

Update the `run` function signature to accept the control spec:

```rust
pub fn run(
    source: &str,
    filename: &str,
    ctx: &Arc<RuntimeCtx>,
    control_spec: Option<&str>,
) -> Result<(), Vec<LxError>> {
```

Inside the `ctx.tokio_runtime.block_on(async { ... })` block, before `interp.load_default_tools()`, spawn the control task:

```rust
ctx.tokio_runtime.block_on(async {
    if let Some(spec) = control_spec {
        let state = Arc::new(lx::runtime::ControlChannelState {
            global_pause: Arc::clone(&ctx.global_pause),
            cancel_flag: Arc::clone(&ctx.cancel_flag),
            inject_tx: ctx.inject_tx.clone(),
        });

        let state_clone = Arc::clone(&state);
        let spec_owned = spec.to_string();
        tokio::spawn(async move {
            if spec_owned == "stdin" {
                lx::runtime::control_stdin::run_stdin_control(state_clone).await;
            } else if let Some(addr) = spec_owned.strip_prefix("ws://") {
                lx::runtime::control_ws::run_ws_control(
                    addr.to_string(),
                    state_clone,
                ).await;
            } else if let Some(addr) = spec_owned.strip_prefix("tcp://") {
                lx::runtime::control_tcp::run_tcp_control(
                    addr.to_string(),
                    state_clone,
                ).await;
            } else {
                eprintln!("[control] unknown transport: {spec_owned}");
            }
        });
    }

    interp.load_default_tools().await.map_err(|e| vec![e])?;
    match interp.exec(&program).await {
        Ok(val) => {
            if !matches!(val, lx::value::LxVal::Unit) {
                println!("{val}");
            }
            Ok(())
        }
        Err(e) => Err(vec![e]),
    }
})
```

The control task is spawned as a separate tokio task, independent of the interpreter's eval loop. The two communicate only through the shared `ControlChannelState` atomic flags and the `inject_tx` mpsc channel.

## Step 10: Add get_agent_pause_flag to agent registry

File: `crates/lx/src/runtime/agent_registry.rs` (from work item 5)

If this function does not already exist, add it:

```rust
pub fn get_agent_pause_flag(name: &str) -> Option<Arc<std::sync::atomic::AtomicBool>> {
    AGENT_REGISTRY.get(name).map(|e| Arc::clone(&e.pause_flag))
}
```

## Step 11: Handle cancel with tool process cleanup

When cancel is received:
1. `cancel_flag` is set to `true` and `global_pause` is cleared
2. The interpreter's next `eval` call checks `cancel_flag` and returns `Err(LxError::runtime("program cancelled via control channel", span))`
3. The error propagates through `exec` and `eval_stmt`, unwinding the call stack
4. Back in `run.rs`, the error is returned to `run_file` which prints it and exits with code 1
5. If tool processes exist (from work item 3), the interpreter's `exec` shutdown code (added by work item 3) calls `shutdown()` on each tool module, which sends MCP shutdown and kills after 2 seconds

If the interpreter is blocked inside a tool call (MCP request pending), the cancel check does not fire until the MCP call returns. For immediate cancellation during a tool call, the control channel task would need access to the MCP client to send `$/cancelRequest`. This is deferred -- the initial implementation waits for the current expression to finish. The next `eval` call sees the cancel flag.

## Step 12: Verify file lengths

- `control.rs`: ~180 lines (types + handlers + json_to_lxval + ControlYieldBackend). Under 300.
- `control_stdin.rs`: ~30 lines. Under 300.
- `control_tcp.rs`: ~55 lines. Under 300.
- `control_ws.rs`: ~70 lines. Under 300.
- `interpreter/mod.rs`: gains ~20 lines for the pause/cancel check. Current is 218 lines. Under 300.
- `main.rs`: gains ~15 lines for `--control` flag. Current is 258 lines. Under 300.
- `run.rs`: gains ~30 lines for control task spawn. Current is 62 lines. Under 300.

## Error Cases

| Scenario | Response |
|---|---|
| Invalid JSON on wire | `{"ok": false, "error": "invalid command: ..."}` |
| Unknown command string | `{"ok": false, "error": "invalid command: unknown variant ..."}` (serde tagged enum error) |
| Pause non-existent agent | `{"ok": false, "error": "agent 'X' not found"}` |
| Resume non-existent agent | `{"ok": false, "error": "agent 'X' not found"}` |
| Inject with no pending yield | `{"ok": false, "error": "no pending yield to inject into"}` |
| Inject with no yield backend | `{"ok": false, "error": "inject not available (no yield backend configured)"}` |
| TCP bind failure | `[control] tcp bind failed on {addr}: {e}` on stderr, control task exits, program runs without control |
| WebSocket bind failure | `[control] ws bind failed on {addr}: {e}` on stderr, control task exits, program runs without control |
| Connection dropped mid-session | Control task exits, program continues uncontrolled |

## Verification

After all changes:
1. `just diagnose` must pass with no warnings.
2. `just test` must pass all existing tests (no `--control` flag means no control channel; the two atomic loads per expression evaluate to `false` and cost nothing).
3. Manual test -- stdin transport:
   - Write a `loop.lx` file: `i := 0; loop { i <- i + 1; emit i; (i > 1000) ? { break i } }`
   - Run `lx run loop.lx --control stdin`
   - Type `{"cmd": "pause"}` on stdin, press Enter
   - See `{"ok":true}` response on stdout
   - The program stops emitting
   - Type `{"cmd": "inspect"}` on stdin
   - See `{"ok":true,"state":{"paused":true,"agents":[],"stream_position":"0-0"}}`
   - Type `{"cmd": "resume"}` on stdin
   - The program resumes emitting
   - Type `{"cmd": "cancel"}` on stdin
   - The program exits
4. Manual test -- TCP transport:
   - Run `lx run loop.lx --control tcp://127.0.0.1:9000`
   - See `[control] listening on tcp://127.0.0.1:9000` on stderr
   - From another terminal: `nc 127.0.0.1 9000`
   - Type `{"cmd":"pause"}`, see `{"ok":true}`
   - Type `{"cmd":"resume"}`, see `{"ok":true}`
   - Type `{"cmd":"cancel"}`, see `{"ok":true}`, program exits
5. Manual test -- WebSocket transport:
   - Run `lx run loop.lx --control ws://127.0.0.1:8080`
   - See `[control] listening on ws://127.0.0.1:8080` on stderr
   - Connect with `websocat ws://127.0.0.1:8080`
   - Send `{"cmd":"inspect"}`, receive state JSON
6. Manual test -- targeted agent pause:
   - Write agent file with `Agent Worker = { run = () { loop { emit "working" } } }; spawn Worker`
   - Run with `--control stdin`
   - Send `{"cmd":"pause","agent":"Worker"}` -- Worker stops
   - Send `{"cmd":"resume","agent":"Worker"}` -- Worker resumes
