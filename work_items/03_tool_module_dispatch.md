# Work Item: MCP Client + Tool Module Dispatch

## Goal

Build the MCP JSON-RPC client, create a tool module representation in `LxVal`, wire `UseKind::Tool` in `eval_use` to spawn an MCP process and bind the tool module, and intercept field access on tool modules to dispatch through MCP `tools/call`. Auto-log `tool/call` and `tool/result` entries to the event stream.

## Preconditions

- **Work item `event_stream_core` is complete:** `EventStream` struct exists at `crates/lx/src/event_stream.rs` with `xadd` method. `RuntimeCtx` has `event_stream: Arc<EventStream>` field.
- **Work item `use_tool_parser` is complete:** `UseKind::Tool { command: Sym, alias: Sym }` variant exists in `crates/lx/src/ast/types.rs`. The stub error in `eval_use` exists and will be replaced.
- `crates/lx/src/value/mod.rs` has the `LxVal` enum with existing variants including `Record`, `Store`, `Stream`.
- `crates/lx/src/interpreter/apply_helpers.rs` has `eval_field_access` handling `FieldKind::Named` for `Record`, `Class`, `Object`, `Store`.
- `crates/lx/src/interpreter/modules.rs` has `eval_use` with the stub `UseKind::Tool` handler.
- `crates/lx/src/interpreter/mod.rs` has the `Interpreter` struct with `ctx: Arc<RuntimeCtx>`.
- `serde_json` is a workspace dependency.
- `tokio` is a workspace dependency (with `process`, `io`, `sync` features).

## Files to Create

### 1. `crates/lx/src/mcp_client.rs`

MCP JSON-RPC client that communicates with a child process over stdin/stdout. Must be under 300 lines.

**Struct: `McpClient`**

```rust
pub struct McpClient {
    child: tokio::process::Child,
    stdin: tokio::io::BufWriter<tokio::process::ChildStdin>,
    stdout: tokio::io::BufReader<tokio::process::ChildStdout>,
    next_id: std::sync::atomic::AtomicU64,
    command: String,
}
```

**Methods:**

- `pub async fn spawn(command: &str) -> Result<Self, String>`
  - Split `command` on whitespace: first token is the program, rest are args
  - Use `tokio::process::Command::new(program).args(rest).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()`
  - If spawn fails (binary not found), return `Err(format!("command '{}' not found", command))`
  - Take stdin and stdout from the child, wrap in `BufWriter`/`BufReader`
  - Call `self.initialize().await?` (the MCP handshake)
  - Return `Ok(Self { ... })`

- `async fn initialize(&mut self) -> Result<(), String>`
  - Send JSON-RPC request: `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"lx","version":"0.1.0"}}}`
  - Read response line from stdout
  - Parse response JSON. Check for `"result"` field. If error, return `Err`.
  - Send notification: `{"jsonrpc":"2.0","method":"notifications/initialized"}`
  - Increment `next_id` to 2

- `pub async fn tools_list(&mut self) -> Result<Vec<ToolInfo>, String>`
  - Send JSON-RPC request: `{"jsonrpc":"2.0","id":N,"method":"tools/list","params":{}}`
  - Read and parse response
  - Deserialize `result.tools` array into `Vec<ToolInfo>`
  - `ToolInfo` is `pub struct ToolInfo { pub name: String, pub description: Option<String> }`

- `pub async fn tools_call(&mut self, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, String>`
  - Send JSON-RPC request: `{"jsonrpc":"2.0","id":N,"method":"tools/call","params":{"name":"<tool_name>","arguments":<arguments>}}`
  - Read response line from stdout
  - Handle JSON-RPC notifications that may arrive before the response (lines with no `id` field): skip them in a loop, continuing to read lines until a response with a matching `id` is found. This is already handled by `send_request` below (line 80: "loop reading lines until we get a response with matching id, skip notifications").
  - Parse the response with an `id` matching our request ID
  - If response has `"error"`, return `Err(error.message)`
  - If response has `"result"`, extract the `content` array, find the first `text` content block, parse its `text` field. Return the parsed JSON value. If content is not text, return the raw result object.

- `pub async fn shutdown(&mut self)`
  - Send JSON-RPC request: `{"jsonrpc":"2.0","id":N,"method":"shutdown","params":{}}`  (best effort, ignore errors)
  - Drop stdin to close the pipe
  - Wait up to 2 seconds for child to exit: `tokio::time::timeout(Duration::from_secs(2), self.child.wait())`
  - If timeout, kill: `self.child.kill()`

- `async fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String>`
  - Get next ID from `self.next_id.fetch_add(1, Ordering::Relaxed)`
  - Build JSON-RPC request object: `{"jsonrpc":"2.0","id":id,"method":method,"params":params}`
  - Serialize to string, append `\n`
  - Write to `self.stdin`, flush
  - Read response line from `self.stdout` using `tokio::io::AsyncBufReadExt::read_line`
  - Parse as JSON
  - Verify `id` matches (loop reading lines until we get a response with matching id, skip notifications)
  - Return the parsed response object

- `pub fn is_alive(&mut self) -> bool`
  - `self.child.try_wait()` returns `Ok(None)` if still running

**Struct: `ToolInfo`**

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
}
```

**Required imports:** `tokio::process`, `tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter}`, `serde_json`, `std::process::Stdio`.

### 2. `crates/lx/src/tool_module.rs`

Tool module state and dispatch. Must be under 300 lines.

**Struct: `ToolModule`**

```rust
pub struct ToolModule {
    pub command: String,
    pub alias: String,
    client: tokio::sync::Mutex<McpClient>,
    call_counter: std::sync::atomic::AtomicU64,
}
```

Using `tokio::sync::Mutex` because MCP calls are async and we need to hold the lock across await points (reading response from stdout).

**Methods:**

- `pub async fn new(command: &str, alias: &str) -> Result<Self, String>`
  - Call `McpClient::spawn(command).await?`
  - Return `Self { command: command.to_string(), alias: alias.to_string(), client: tokio::sync::Mutex::new(client), call_counter: AtomicU64::new(1) }`

- `pub async fn call_tool(&self, method: &str, args: crate::value::LxVal, event_stream: &crate::event_stream::EventStream, agent_name: &str) -> Result<crate::value::LxVal, crate::error::LxError>`
  - Increment `call_counter`, get `call_id`
  - Convert `args` to `serde_json::Value` for the MCP call arguments. If args is a Record, serialize it. If args is a String, wrap as `{"input": args_str}`. If args is Unit, use `{}`.
  - Log `tool/call` to event stream: `event_stream.xadd("tool/call", agent_name, None, fields)` where fields include `call_id`, `tool` (self.alias), `method`, `args`
  - Lock `self.client`, call `client.tools_call(method, arguments).await`
  - On success: convert result `serde_json::Value` to `LxVal` via `LxVal::from(result)` — this conversion exists at `crates/lx/src/value/serde_impl.rs` line 74 (`impl From<serde_json::Value> for LxVal`). Log `tool/result` to event stream with `call_id`, `tool`, `method`, `result`. Return `Ok(result_lxval)`.
  - On error: Log `tool/error` to event stream with `call_id`, `tool`, `method`, `error`. Return `Err(LxError::runtime(format!("tool '{}' method '{}': {}", self.alias, method, error_msg), miette::SourceSpan::new(0.into(), 0)))`.

- `pub async fn shutdown(&self)`
  - Lock `self.client`, call `client.shutdown().await`

## Files to Modify

### 3. `crates/lx/src/value/mod.rs`

**Add a new variant to `LxVal`** for tool modules. Insert after `Stream { id: u64 }` (line 114):

```rust
ToolModule(Arc<crate::tool_module::ToolModule>),
```

This uses `Arc` because tool modules are shared (multiple references in the env) and `ToolModule` contains a `tokio::sync::Mutex<McpClient>` which is not `Clone`.

**Add `#[strum(serialize = "ToolModule")]` attribute** to the variant for display purposes.

**Update the `LxVal` impl:** No new constructor methods needed — callers construct `LxVal::ToolModule(Arc::new(tm))` directly.

### 4. `crates/lx/src/value/serde_impl.rs`

No change needed. The `ToolModule` variant is handled by the existing catch-all `_ =>` arm at line 62: `serializer.serialize_str(&format!("<{}>", self.type_name()))`. This serializes as `"<ToolModule>"`.

### 5. `crates/lx/src/interpreter/apply_helpers.rs`

**Add a match arm in `eval_field_access`** for `LxVal::ToolModule`. In the `FieldKind::Named(name)` match (line 16), add after the `LxVal::Store { .. }` arm (line 32-34), before the `other =>` fallback (line 35):

```rust
LxVal::ToolModule(tm) => {
    let method_name = name.as_str().to_string();
    let tm = Arc::clone(tm);
    Ok(LxVal::BuiltinFunc(crate::value::BuiltinFunc {
        name: "tool.call",
        arity: 3,
        kind: crate::value::BuiltinKind::Async(bi_tool_dispatch),
        applied: vec![
            LxVal::ToolModule(tm),
            LxVal::str(method_name),
        ],
    }))
},
```

This creates a `BuiltinFunc` with arity 3 and 2 pre-applied args (tool module + method name). When the caller writes `Browser.click "e2"`, the interpreter applies `"e2"` as the third arg, making `applied.len() == arity`, which triggers execution with `args = [ToolModule, method_name_str, "e2"]`.

The `BuiltinFunc.name` field is `&'static str` (see `crates/lx/src/value/func.rs` line 42), so use the static `"tool.call"` rather than a dynamically allocated name.

**Define `bi_tool_dispatch` in `apply_helpers.rs`** (or a new file if `apply_helpers.rs` would exceed 300 lines):

```rust
fn bi_tool_dispatch(
    args: Vec<LxVal>,
    span: SourceSpan,
    ctx: Arc<RuntimeCtx>,
) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> {
    Box::pin(async move {
        let LxVal::ToolModule(tm) = &args[0] else {
            return Err(LxError::runtime("tool.call: invalid tool module", span));
        };
        let method = args[1].as_str().ok_or_else(|| LxError::runtime("tool.call: invalid method name", span))?;
        let arg = args[2].clone();
        tm.call_tool(method, arg, &ctx.event_stream, "main").await
    })
}
```

**Required imports in `apply_helpers.rs`:** `std::sync::Arc` is already imported (line 1). Add `use std::future::Future;`, `use std::pin::Pin;`, and `use crate::runtime::RuntimeCtx;`.

### 6. `crates/lx/src/interpreter/modules.rs`

**Replace the stub error** for `UseKind::Tool` (added in work item 2) with actual tool module spawning.

Replace:
```rust
if let UseKind::Tool { command, alias } = &use_stmt.kind {
    return Err(LxError::runtime(
        format!("tool modules not yet implemented (use tool \"{}\" as {})", command, alias),
        span,
    ));
}
```

With:
```rust
if let UseKind::Tool { command, alias } = &use_stmt.kind {
    let cmd_str = command.as_str();
    let alias_str = alias.as_str();
    let tm = crate::tool_module::ToolModule::new(cmd_str, alias_str)
        .await
        .map_err(|e| LxError::runtime(e, span))?;
    let val = LxVal::ToolModule(Arc::new(tm));
    let env = self.env.child();
    env.bind(*alias, val);
    self.env = Arc::new(env);
    return Ok(());
}
```

**Required imports:** Add `use std::sync::Arc;` if not already present (it is, via `use std::sync::Arc;` at line 4).

### 7. `crates/lx/src/lib.rs`

Add module declarations:

```rust
pub mod mcp_client;
pub mod tool_module;
```

### 8. `crates/lx/src/interpreter/mod.rs`

**Add a `tool_modules` field** to track tool modules for shutdown at end of `exec`. Add a field:

```rust
pub(crate) tool_modules: Vec<Arc<crate::tool_module::ToolModule>>,
```

Initialize to `vec![]` in both `new()` and `with_env()` constructors.

In `eval_use` (modules.rs), after creating the tool module, push it to `self.tool_modules`:

```rust
self.tool_modules.push(Arc::clone(&tm_arc));
```

(Where `tm_arc` is the `Arc<ToolModule>` created before binding.)

At the end of `exec()` in `mod.rs`, after the main loop completes, shut down all tool modules:

```rust
for tm in &self.tool_modules {
    tm.shutdown().await;
}
```

Insert this after line 117 (`Ok(result)`) — but before returning, making it:

```rust
let mut result = LxVal::Unit;
let stmts = program.stmts.clone();
for sid in &stmts {
    result = self.eval_stmt(*sid).await.map_err(|e| match e {
        EvalSignal::Error(e) => e,
        EvalSignal::Break(_) => LxError::runtime("break outside loop", self.arena.stmt_span(*sid)),
    })?;
}
for tm in &self.tool_modules {
    tm.shutdown().await;
}
Ok(result)
```

Also need to add the field to `eval_par` and `eval_sel` where new `Interpreter` instances are created inline. In `crates/lx/src/interpreter/eval.rs` lines 109, 135: add `tool_modules: vec![]` to the `Interpreter` struct literals.

## Step-by-Step Instructions

1. Create `crates/lx/src/mcp_client.rs` with `McpClient` and `ToolInfo`.

2. Create `crates/lx/src/tool_module.rs` with `ToolModule`.

3. Add `pub mod mcp_client;` and `pub mod tool_module;` to `crates/lx/src/lib.rs`.

4. Add `ToolModule(Arc<crate::tool_module::ToolModule>)` variant to `LxVal` in `crates/lx/src/value/mod.rs`.

5. Add tool module dispatch in `eval_field_access` in `crates/lx/src/interpreter/apply_helpers.rs`. Define `bi_tool_dispatch` async function.

6. Replace the stub error in `eval_use` in `crates/lx/src/interpreter/modules.rs` with actual spawning logic.

7. Add `tool_modules: Vec<Arc<crate::tool_module::ToolModule>>` field to `Interpreter` in `crates/lx/src/interpreter/mod.rs`. Initialize to `vec![]` in constructors. Add shutdown loop at end of `exec()`.

8. Update `Interpreter` struct literals in `eval_par` and `eval_sel` in `crates/lx/src/interpreter/eval.rs` to include `tool_modules: vec![]`.

9. Handle the new `LxVal::ToolModule` variant in exhaustive matches on `LxVal`:
   - `crates/lx/src/value/display.rs` line 81 (before `LxVal::Stream`): add `LxVal::ToolModule(tm) => write!(f, "<ToolModule:{}>", tm.alias),`
   - `crates/lx/src/value/impls.rs` — `structural_eq` (line 36): no change needed, falls through to `_ => false` at line 74. `hash_value` (line 78): no change needed, falls through to the `Func|MultiFunc|BuiltinFunc|TaggedCtor` arm at line 128 — add `LxVal::ToolModule(_)` to that arm.
   - `crates/lx/src/value/serde_impl.rs` — caught by existing `_ =>` arm at line 62. No change needed.

10. Add `process` and `io-util` features to the workspace `tokio` dependency. The current workspace `tokio` dependency at `Cargo.toml` line 67 has features `["macros", "rt-multi-thread", "sync", "time"]`. Add `"process"` and `"io-util"` to this list. `process` is needed for `tokio::process::Command`. `io-util` is needed for `AsyncBufReadExt` and `AsyncWriteExt`.

## Deliverable

After this work item:
- `use tool "some-mcp-server" as Srv` spawns the MCP process, performs the initialize handshake, and binds `Srv` as a `LxVal::ToolModule` in scope
- `Srv.method_name arg` dispatches through MCP `tools/call`, returns the result as `LxVal`
- Every tool call auto-logs `tool/call` and `tool/result` (or `tool/error`) entries to the event stream
- If the command binary doesn't exist, `use tool` fails with `Err "command 'X' not found"`
- If the MCP process crashes, subsequent calls return `Err "tool 'Srv' process exited"`
- On program exit, all MCP processes receive shutdown and are killed after 2 seconds
