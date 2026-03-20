# Goal

Add `std/ws` — a WebSocket client module using `tokio-tungstenite`. Provides persistent bidirectional connections with send, receive, and callback-based message handling. No backend trait — direct async networking with connection handles stored in a global table (same pattern as Store/Object).

# Why

- WebSocket is the transport layer for Chrome DevTools Protocol (CDP), real-time APIs (Slack, Discord), live dashboards, and SSE alternatives. lx has `std/http` for request-response but nothing for persistent bidirectional connections.
- `pkg/connectors/cdp` (browser automation) requires WebSocket as a foundation. Without `std/ws`, there's no way to implement CDP in lx.
- `tokio-tungstenite` is the standard async WebSocket crate for tokio. lx already depends on tokio.

# What Changes

**Cargo.toml dependency:** Add `tokio-tungstenite = { version = "0.24", features = ["native-tls"] }` to `crates/lx/Cargo.toml`.

**New file `crates/lx/src/stdlib/ws.rs`:** Module entry with `build()`. Global `WS_CONNS: DashMap<u64, WsConn>` connection table. `ws.connect` opens a WebSocket, stores the split stream in the table, returns a handle Record. `ws.send` sends a text message. `ws.recv` receives the next message (blocking). `ws.close` closes the connection and removes from table. `ws.recv_json` receives and parses as JSON.

**Registration in `crates/lx/src/stdlib/mod.rs`:** Add `mod ws;`, register in `get_std_module` and `std_module_exists`.

**Test file `tests/99_ws.lx`:** Basic connection lifecycle test (connect to a public echo server or test with a local server).

# Files Affected

- `crates/lx/Cargo.toml` — add `tokio-tungstenite`
- `crates/lx/src/stdlib/ws.rs` — New file
- `crates/lx/src/stdlib/mod.rs` — Register module
- `tests/99_ws.lx` — New test file

# Task List

### Task 1: Add dependency and create ws.rs with connect and close

**Subject:** Create ws.rs with connection table, connect, and close functions

**Description:** Add `tokio-tungstenite = { version = "0.24", features = ["native-tls"] }` to `crates/lx/Cargo.toml` under `[dependencies]`.

Create `crates/lx/src/stdlib/ws.rs`.

Imports: `std::sync::{Arc, LazyLock, atomic::{AtomicU64, Ordering}}`, `dashmap::DashMap`, `tokio::sync::Mutex as TokioMutex`, `futures::{SinkExt, StreamExt}`, `tokio_tungstenite::{connect_async, tungstenite::Message}`, `indexmap::IndexMap`, `num_bigint::BigInt`, `crate::backends::RuntimeCtx`, `crate::builtins::mk`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`.

Define `type WsSink` as `Arc<TokioMutex<futures::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>>>` and `type WsStream` as `Arc<TokioMutex<futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>>`.

Define `struct WsConn { sink: WsSink, stream: WsStream, url: String }`.

Static: `static WS_CONNS: LazyLock<DashMap<u64, WsConn>> = LazyLock::new(DashMap::new);` and `static NEXT_ID: AtomicU64 = AtomicU64::new(1);`.

Helper `fn conn_id(v: &Value, span: Span) -> Result<u64, LxError>`: extract `__ws_id` from Record, parse as u64, return `LxError::type_err` on failure.

`pub fn build() -> IndexMap<String, Value>`: register:
- `"connect"` → `bi_connect` arity 1
- `"send"` → `bi_send` arity 2
- `"recv"` → `bi_recv` arity 1
- `"recv_json"` → `bi_recv_json` arity 1
- `"close"` → `bi_close` arity 1

`fn bi_connect(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is URL string. Use `tokio::task::block_in_place` + `Handle::current().block_on` (same pattern as `ReqwestHttpBackend`). Inside the async block: call `connect_async(url).await`. On error, return `Ok(Value::Err(...))`. On success, split the stream via `.split()`. Allocate ID via `NEXT_ID.fetch_add`. Store `WsConn { sink: Arc::new(TokioMutex::new(sink)), stream: Arc::new(TokioMutex::new(stream)), url }` in `WS_CONNS`. Return `Ok(Value::Ok(Box::new(record! { "__ws_id" => Value::Int(BigInt::from(id)), "url" => Value::Str(Arc::from(url)) })))`.

`fn bi_close(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is connection handle. Get `conn_id`. Remove from `WS_CONNS`. If found, use `block_in_place` + `block_on` to call `sink.lock().await.close().await`. Return `Ok(Value::Ok(Box::new(Value::Unit)))`. If not found, return `Ok(Value::Err(...))`.

**ActiveForm:** Creating ws.rs with connection table and connect/close

---

### Task 2: Add send, recv, and recv_json functions

**Subject:** Add WebSocket send and receive functions to ws.rs

**Description:** Add to `crates/lx/src/stdlib/ws.rs`:

`fn bi_send(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is connection handle, args[1] is message (Str). Get `conn_id`. Look up connection in `WS_CONNS` via `.get(&id)`. If not found, return `Ok(Value::Err(...))`. Clone the `sink` Arc. Drop the DashMap ref. Use `block_in_place` + `block_on`: `sink.lock().await.send(Message::Text(msg.to_string())).await`. On error, return `Ok(Value::Err(...))`. On success, return `Ok(Value::Ok(Box::new(Value::Unit)))`.

`fn bi_recv(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is connection handle. Get `conn_id`. Look up connection, clone the `stream` Arc. Drop the DashMap ref. Use `block_in_place` + `block_on`: `stream.lock().await.next().await`. Match the result:
- `None` → return `Ok(Value::Err(Box::new(Value::Str(Arc::from("connection closed")))))`
- `Some(Err(e))` → return `Ok(Value::Err(...))`
- `Some(Ok(Message::Text(t)))` → return `Ok(Value::Ok(Box::new(Value::Str(Arc::from(t.as_str())))))`
- `Some(Ok(Message::Binary(b)))` → base64 encode and return as Str
- `Some(Ok(Message::Close(_)))` → remove from `WS_CONNS`, return `Ok(Value::Err(Box::new(Value::Str(Arc::from("connection closed")))))`
- `Some(Ok(Message::Ping(_)))` → send Pong, recurse (call bi_recv again)
- Other → recurse

`fn bi_recv_json(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: Call `bi_recv` first. If it returns `Ok(Str)`, parse the string as JSON via `serde_json::from_str`, convert to lx Value via `json_conv::json_to_lx`. Return `Ok(Value::Ok(Box::new(lx_value)))`. On JSON parse error, return `Ok(Value::Err(...))`. If `bi_recv` returned Err, propagate it.

**ActiveForm:** Adding send and receive functions to ws.rs

---

### Task 3: Register std/ws and write tests

**Subject:** Register ws module in mod.rs and write integration tests

**Description:** Edit `crates/lx/src/stdlib/mod.rs`:

Add `mod ws;` alongside the other module declarations.

In `get_std_module`, add: `"ws" => ws::build(),` in the match arm.

In `std_module_exists`, add `| "ws"` to the matches! pattern.

Create `tests/99_ws.lx`. WebSocket tests need a server. Use a public WebSocket echo service or skip gracefully. Structure:

```
use std/ws

-- Test connect to wss://echo.websocket.org or similar
-- If no network available, the test should handle the error gracefully
result = ws.connect "wss://echo.websocket.events"
result ? {
  Ok conn -> {
    -- Send a message
    ws.send conn "hello from lx" ^

    -- Receive echo
    reply = ws.recv conn ^
    assert (reply == "hello from lx") "echo matches"

    -- Close
    ws.close conn ^
    log.info "99_ws: echo test passed"
  }
  Err e -> {
    log.info "99_ws: skipped (no network: {e})"
  }
}

log.info "99_ws: all passed"
```

If the public echo server is unreliable, the test can be structured to pass gracefully on connection failure. The important thing is that the module compiles and the API is correct.

Run `just diagnose` to verify compilation.

**ActiveForm:** Registering ws module and writing tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/STD_WS.md" })
```

Then call `next_task` to begin.
