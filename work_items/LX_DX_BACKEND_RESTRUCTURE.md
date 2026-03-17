# lx-dx Backend Restructure: Generic Library with Output Adapters

## Goal

Restructure the `backends/dx` crate from a standalone Dioxus app into a generic, UI-agnostic library that any Rust application can import to run lx programs and consume runtime events. Provide output adapters (PTY writer, WebSocket stream, HTTP reporter) so consuming apps wire events into their own UI/terminal system. Keep Langfuse integration. Drop the Dioxus components.

## Why

The current lx-dx crate bundles its own Dioxus UI components (pane manager, toolbar, terminal-style panes). This is wrong — the consuming app (mcp-toolbelt desktop) already has a full terminal emulator (xterm.js + portable-pty), pane management (binary tree splits), and workflow APIs (agent registration, task CRUD, event reporting). The lx-dx crate should provide the runtime integration layer, not compete with the host app's UI.

The EventBus + RuntimeEvent + backend implementations are the real value. They need to be decoupled from any specific UI framework so that:
- mcp-toolbelt desktop can route events to its existing PTY terminals
- A CLI tool could route events to stdout with ANSI formatting
- A web service could route events to WebSocket clients
- Tests can subscribe and assert on events in-process

## What Changes

**Drop Dioxus dependency and UI components (Task 1):** Remove `src/components/`, `src/app.rs`, `assets/`. Remove `dioxus` from Cargo.toml. The crate becomes a pure library — no binary target.

**Restructure as lib-only crate (Task 2):** Update Cargo.toml to lib-only. Re-export the public API from lib.rs: `event`, `backends`, `runner`, `langfuse`, `adapters`.

**Add PtyWriter adapter (Task 3):** New `src/adapters/pty.rs` — subscribes to EventBus and writes ANSI-formatted event output to any `impl Write`. AI calls get colored headers/responses, logs get level-colored prefixes, shell commands get monospace formatting, progress gets a bar, errors get red. The consuming app provides the Write handle (e.g., a PTY master fd).

**Add WsStream adapter (Task 4):** New `src/adapters/ws.rs` — subscribes to EventBus and serializes RuntimeEvents as JSON over a WebSocket sink. Uses `tokio-tungstenite`. The consuming app provides the WS connection.

**Add HttpReporter adapter (Task 5):** New `src/adapters/http.rs` — subscribes to EventBus and posts events to a configurable HTTP endpoint. Fire-and-forget, buffered. Maps RuntimeEvent variants to a JSON event report format. The consuming app provides the base URL.

**Add AgentTerminalManager (Task 6):** New `src/adapters/terminal_manager.rs` — orchestrates the per-agent → per-writer mapping. When AgentSpawned fires, calls a user-provided callback to get a Write handle for the new agent. Routes subsequent events for that agent to its writer via PtyWriter. This is the high-level integration point consuming apps use.

**Update ProgramRunner for agent tracking (Task 7):** Extend ProgramRunner to emit AgentSpawned/AgentKilled events when the interpreter calls agent.spawn/agent.kill. Currently the runner only emits ProgramStarted/ProgramFinished.

**Add ANSI formatting module (Task 8):** New `src/adapters/ansi.rs` — ANSI escape code helpers for the PtyWriter. Colors, bold, dim, reset. Formatting functions for each RuntimeEvent type that return styled strings.

## How It Works

The consuming app's integration looks like:

```rust
use lx_dx::event::EventBus;
use lx_dx::runner::ProgramRunner;
use lx_dx::langfuse::LangfuseClient;
use lx_dx::adapters::AgentTerminalManager;

let bus = Arc::new(EventBus::new());
let langfuse = Arc::new(LangfuseClient::from_env());
let runner = ProgramRunner::new(bus.clone(), langfuse);

// App provides a callback that returns a Write handle per agent
let manager = AgentTerminalManager::new(bus.clone(), |agent_id, agent_name| {
    // Desktop app: POST /api/terminal-requests, get PTY handle
    // CLI app: return stdout
    // Test: return Vec<u8> buffer
    Box::new(my_pty_writer_for(agent_id))
});
manager.start(); // spawns background task

runner.run("workflow.lx").await;
```

For lower-level control, apps can subscribe to the EventBus directly and handle events however they want. The adapters are convenience layers.

Tasks are ordered so each leaves the crate compilable. Task 1 removes the UI, Task 2 restructures, Tasks 3-6 add adapters, Tasks 7-8 fill gaps.

## Files Affected

**Deleted:**
- `backends/dx/src/app.rs`
- `backends/dx/src/components/` (entire directory)
- `backends/dx/src/main.rs`
- `backends/dx/assets/` (entire directory)

**Modified:**
- `backends/dx/Cargo.toml` — remove dioxus, add tokio-tungstenite, lib-only
- `backends/dx/src/lib.rs` — re-export public API, remove component modules
- `backends/dx/src/runner.rs` — emit AgentSpawned/AgentKilled events
- `backends/dx/src/event.rs` — add `agent_id()` helper method if missing

**New:**
- `backends/dx/src/adapters/mod.rs`
- `backends/dx/src/adapters/pty.rs` — PtyWriter
- `backends/dx/src/adapters/ws.rs` — WsStream
- `backends/dx/src/adapters/http.rs` — HttpReporter
- `backends/dx/src/adapters/terminal_manager.rs` — AgentTerminalManager
- `backends/dx/src/adapters/ansi.rs` — ANSI formatting helpers

## Task List

### Task 1: Remove Dioxus components and binary target

**Subject:** Strip UI components, app.rs, assets, and main.rs from lx-dx

**Description:** Delete these files and directories:
- `backends/dx/src/app.rs`
- `backends/dx/src/main.rs`
- `backends/dx/src/components/` (all files: mod.rs, pane.rs, pane_manager.rs, toolbar.rs, ai_call.rs, user_prompt.rs)
- `backends/dx/assets/` (style.css)

In `backends/dx/Cargo.toml`:
- Remove the `[[bin]]` section entirely
- Remove `dioxus` from `[dependencies]`
- Keep all other deps (lx, tokio, serde, serde_json, reqwest, chrono, uuid, base64)

**activeForm:** Removing Dioxus UI layer from lx-dx

---

### Task 2: Restructure as lib-only crate

**Subject:** Update lib.rs and Cargo.toml for pure library crate

**Description:** Rewrite `backends/dx/src/lib.rs` to export the public API:

```rust
pub mod event;
pub mod backends;
pub mod runner;
pub mod langfuse;
pub mod adapters;
```

Ensure `backends/dx/Cargo.toml` has only:
```toml
[lib]
path = "src/lib.rs"
```

Verify the existing modules compile without Dioxus imports. The `backends/user.rs` DxUserBackend currently uses a oneshot channel — this is fine, it doesn't depend on Dioxus. Check each backend file for any dioxus imports and remove them.

**activeForm:** Restructuring lx-dx as lib-only crate

---

### Task 3: Add ANSI formatting module

**Subject:** Create adapters/ansi.rs with ANSI escape code helpers for terminal output

**Description:** Create `backends/dx/src/adapters/ansi.rs` with:

- Color constants: `RED`, `GREEN`, `YELLOW`, `BLUE`, `MAGENTA`, `CYAN`, `DIM`, `BOLD`, `RESET`
- `fn format_ai_start(model: &str, prompt: &str) -> String` — blue bold `[AI] {model}` header + dimmed truncated prompt (first 200 chars)
- `fn format_ai_complete(response: &str, model: &str, cost: Option<f64>, duration_ms: u64) -> String` — response text + dim metadata line (model, cost, duration)
- `fn format_ai_error(error: &str) -> String` — red `[AI ERROR] {error}`
- `fn format_log(level: &str, msg: &str) -> String` — level-colored `[{LEVEL}] {msg}` (info=blue, warn=yellow, err=red, debug=dim)
- `fn format_emit(value: &str) -> String` — plain text
- `fn format_shell_exec(cmd: &str) -> String` — dim `$ {cmd}`
- `fn format_shell_result(exit_code: i32, stdout: &str, stderr: &str) -> String` — stdout + red stderr if non-empty + exit code badge
- `fn format_error(error: &str, span_info: Option<&SpanInfo>) -> String` — red bold error with optional location
- `fn format_progress(current: usize, total: usize, message: &str) -> String` — `[=====>    ] 60% {message}` using carriage return for in-place update
- `fn format_agent_spawned(agent_id: &str, name: &str) -> String` — green `[SPAWN] {name} ({agent_id})`
- `fn format_agent_killed(agent_id: &str) -> String` — dim `[EXIT] {agent_id}`
- `fn format_program_started(path: &str) -> String` — bold `[START] {path}`
- `fn format_program_finished(result: &Result<String, String>, duration_ms: u64) -> String` — green OK or red FAIL + duration
- `fn format_event(event: &RuntimeEvent) -> String` — master dispatch that calls the above based on event variant

Create `backends/dx/src/adapters/mod.rs` with `pub mod ansi;`.

**activeForm:** Adding ANSI formatting helpers for terminal output

---

### Task 4: Add PtyWriter adapter

**Subject:** Create adapters/pty.rs that writes ANSI-formatted events to any Write handle

**Description:** Create `backends/dx/src/adapters/pty.rs`:

```rust
pub struct PtyWriter<W: Write + Send + 'static> {
    writer: Mutex<W>,
}

impl<W: Write + Send + 'static> PtyWriter<W> {
    pub fn new(writer: W) -> Self;
    pub fn write_event(&self, event: &RuntimeEvent) -> io::Result<()>;
}
```

`write_event` calls `ansi::format_event(event)` and writes the resulting string + newline to the writer. Flushes after each event. For progress events, writes `\r` prefix instead of newline to update in-place.

Also add:

```rust
pub fn spawn_pty_writer<W: Write + Send + 'static>(
    bus: &EventBus,
    agent_id: String,
    writer: W,
) -> tokio::task::JoinHandle<()>
```

This subscribes to the bus, filters events by agent_id, and writes each matching event to the PtyWriter. Runs until the bus closes or an AgentKilled event for this agent_id arrives.

Add `pub mod pty;` to `adapters/mod.rs`.

**activeForm:** Adding PtyWriter output adapter

---

### Task 5: Add WsStream adapter

**Subject:** Create adapters/ws.rs that streams RuntimeEvents as JSON over WebSocket

**Description:** Create `backends/dx/src/adapters/ws.rs`:

```rust
pub struct WsStream {
    bus: Arc<EventBus>,
}

impl WsStream {
    pub fn new(bus: Arc<EventBus>) -> Self;

    pub async fn stream_to<S: futures::Sink<tokio_tungstenite::tungstenite::Message, Error = E> + Unpin, E: std::fmt::Display>(
        &self,
        sink: &mut S,
        agent_filter: Option<String>,
    );
}
```

Subscribes to the bus, serializes each RuntimeEvent to JSON via serde, sends as a Text WebSocket message. If `agent_filter` is Some, only sends events matching that agent_id. Runs until the bus closes or the sink errors.

Add `tokio-tungstenite = "0.24"` and `futures = "0.3"` to Cargo.toml. Add `pub mod ws;` to `adapters/mod.rs`.

**activeForm:** Adding WebSocket stream adapter

---

### Task 6: Add HttpReporter adapter

**Subject:** Create adapters/http.rs that posts events to an HTTP endpoint

**Description:** Create `backends/dx/src/adapters/http.rs`:

```rust
pub struct HttpReporter {
    bus: Arc<EventBus>,
    client: reqwest::Client,
    base_url: String,
}

impl HttpReporter {
    pub fn new(bus: Arc<EventBus>, base_url: String) -> Self;

    pub fn start(self) -> tokio::task::JoinHandle<()>;
}
```

`start` spawns a tokio task that subscribes to the bus and for each event:
- Serializes to JSON
- POSTs to `{base_url}/api/events` with the JSON body
- Fire-and-forget — log errors but don't block or retry
- Buffers up to 100 events and batch-sends every 500ms to avoid overwhelming the endpoint

Add `pub mod http;` to `adapters/mod.rs`.

**activeForm:** Adding HTTP reporter adapter

---

### Task 7: Add AgentTerminalManager

**Subject:** Create adapters/terminal_manager.rs that maps agents to Write handles

**Description:** Create `backends/dx/src/adapters/terminal_manager.rs`:

```rust
pub type WriterFactory = Box<dyn Fn(&str, &str) -> Box<dyn Write + Send> + Send + Sync>;

pub struct AgentTerminalManager {
    bus: Arc<EventBus>,
    factory: WriterFactory,
}

impl AgentTerminalManager {
    pub fn new(bus: Arc<EventBus>, factory: WriterFactory) -> Self;

    pub fn start(self) -> tokio::task::JoinHandle<()>;
}
```

`start` spawns a tokio task that:
1. Creates a PtyWriter for "main" agent using `factory("main", "main")`
2. Subscribes to the bus
3. On `AgentSpawned { agent_id, name, .. }` — calls `factory(agent_id, name)` to get a new writer, creates a PtyWriter, spawns a `spawn_pty_writer` task for that agent
4. Routes all events to the appropriate PtyWriter based on agent_id
5. On `AgentKilled { agent_id, .. }` — writes the kill event and drops the writer
6. On `ProgramStarted`/`ProgramFinished` — writes to the "main" writer

Tracks active writers in a `HashMap<String, JoinHandle<()>>`.

Add `pub mod terminal_manager;` to `adapters/mod.rs`.

**activeForm:** Adding AgentTerminalManager orchestrator

---

### Task 8: Extend ProgramRunner with agent lifecycle events

**Subject:** Emit AgentSpawned and AgentKilled events from the runner

**Description:** Currently `ProgramRunner::run` only emits `ProgramStarted` and `ProgramFinished`. The interpreter's agent.spawn and agent.kill happen inside the blocking task and don't emit events to the bus.

Two approaches — pick the simpler one:

**Option A (intercepting backend):** The `DxShellBackend` already wraps `ProcessShellBackend`. Since agent.spawn ultimately calls a shell command (`lx run ...`), the `DxShellBackend` can detect spawn commands and emit `AgentSpawned`. But this is fragile.

**Option B (RuntimeCtx callback):** Add an optional `on_agent_spawn: Option<Arc<dyn Fn(String, String) + Send + Sync>>` field to RuntimeCtx. The interpreter calls this when processing agent.spawn. The DxRuntimeCtx sets it to emit `AgentSpawned` to the bus. This requires a small change to the lx crate's RuntimeCtx and interpreter.

Implement Option B. In `crates/lx/src/backends/mod.rs`, add to RuntimeCtx:
```rust
pub on_agent_event: Option<Arc<dyn Fn(AgentEvent) + Send + Sync>>,
```
Where `AgentEvent` is `Spawned { id: String, name: String }` or `Killed { id: String }`.

In `crates/lx/src/interpreter/agents.rs`, call `ctx.on_agent_event` after a successful spawn or kill.

In `backends/dx/src/backends/mod.rs`, set `on_agent_event` in `build_runtime_ctx` to emit `RuntimeEvent::AgentSpawned`/`AgentKilled` to the bus.

**activeForm:** Adding agent lifecycle event emission
