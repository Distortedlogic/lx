# lx-dx: Dioxus Backend for lx — Spec

Konsole-style terminal multiplexer for running and visually monitoring lx agentic workflows. Each agent gets its own pane. LLM calls, messages, traces, and errors stream in real-time. Langfuse provides the observability backbone.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│  lx-dx (Dioxus desktop app)                                 │
│                                                             │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐               │
│  │  main      │  │ researcher│  │  composer  │  ...panes    │
│  │  agent     │  │  agent    │  │  agent     │              │
│  │  pane      │  │  pane     │  │  pane      │              │
│  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘               │
│        │               │               │                    │
│  ┌─────┴───────────────┴───────────────┴──────────────┐     │
│  │            EventBus (tokio broadcast channel)       │     │
│  └─────┬───────────────┬───────────────┬──────────────┘     │
│        │               │               │                    │
│  ┌─────┴─────┐  ┌──────┴──────┐  ┌────┴──────┐             │
│  │ DxRuntime │  │ LangfuseLog │  │ DxUser    │             │
│  │ Ctx       │  │ Backend     │  │ Backend   │             │
│  └───────────┘  └─────────────┘  └───────────┘             │
│                                                             │
└──────────────────────────┬──────────────────────────────────┘
                           │
                    ┌──────┴──────┐
                    │  lx crate   │
                    │ Interpreter │
                    └─────────────┘
```

## Crate Structure

```
backends/dx/
├── spec.md                    (this file)
├── Cargo.toml                 (binary crate: lx-dx)
├── src/
│   ├── main.rs                (Dioxus app entry, window setup)
│   ├── app.rs                 (root component, layout, pane management)
│   ├── event.rs               (RuntimeEvent enum, EventBus)
│   ├── backends/
│   │   ├── mod.rs             (DxRuntimeCtx constructor)
│   │   ├── ai.rs              (DxAiBackend — wraps ClaudeCodeAiBackend, emits events)
│   │   ├── emit.rs            (DxEmitBackend — routes emit to pane)
│   │   ├── log.rs             (LangfuseLogBackend — sends to Langfuse + pane)
│   │   ├── user.rs            (DxUserBackend — renders confirm/choose/ask in pane)
│   │   ├── shell.rs           (DxShellBackend — wraps ProcessShellBackend, emits events)
│   │   └── yield_.rs          (DxYieldBackend — renders yield in pane, captures response)
│   ├── components/
│   │   ├── mod.rs
│   │   ├── pane.rs            (single agent pane — terminal-style output)
│   │   ├── pane_manager.rs    (split/tab layout, add/remove/resize panes)
│   │   ├── toolbar.rs         (load program, run/stop, layout controls)
│   │   ├── ai_call.rs         (LLM call widget — prompt, streaming response, cost, model)
│   │   ├── user_prompt.rs     (confirm/choose/ask modal rendered in pane)
│   │   ├── trace_panel.rs     (optional: Langfuse trace summary sidebar)
│   │   └── program_view.rs    (source code display with execution highlights)
│   └── runner.rs              (spawns interpreter on tokio task, bridges events)
└── assets/
    └── style.css
```

## Core Types

### RuntimeEvent

Every backend implementation emits events to a central bus. Panes subscribe filtered by agent id.

```rust
enum RuntimeEvent {
    AgentSpawned {
        agent_id: String,
        name: String,
        config: serde_json::Value,
        ts: Instant,
    },
    AgentKilled {
        agent_id: String,
        ts: Instant,
    },
    AiCallStart {
        agent_id: String,
        call_id: u64,
        prompt: String,
        model: Option<String>,
        system: Option<String>,
        ts: Instant,
    },
    AiCallComplete {
        agent_id: String,
        call_id: u64,
        response: String,
        cost_usd: Option<f64>,
        duration_ms: u64,
        model: String,
        langfuse_trace_id: Option<String>,
        ts: Instant,
    },
    AiCallError {
        agent_id: String,
        call_id: u64,
        error: String,
        ts: Instant,
    },
    MessageSend {
        from_agent: String,
        to_agent: String,
        msg: serde_json::Value,
        ts: Instant,
    },
    MessageAsk {
        from_agent: String,
        to_agent: String,
        msg: serde_json::Value,
        ts: Instant,
    },
    MessageResponse {
        from_agent: String,
        to_agent: String,
        response: serde_json::Value,
        duration_ms: u64,
        ts: Instant,
    },
    Emit {
        agent_id: String,
        value: String,
        ts: Instant,
    },
    Log {
        agent_id: String,
        level: String,
        msg: String,
        ts: Instant,
    },
    ShellExec {
        agent_id: String,
        cmd: String,
        ts: Instant,
    },
    ShellResult {
        agent_id: String,
        cmd: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
        ts: Instant,
    },
    UserPrompt {
        agent_id: String,
        prompt_id: u64,
        kind: UserPromptKind,
        ts: Instant,
    },
    UserResponse {
        agent_id: String,
        prompt_id: u64,
        response: serde_json::Value,
        ts: Instant,
    },
    TraceSpanRecorded {
        agent_id: String,
        span_id: u64,
        name: String,
        input: String,
        output: String,
        score: Option<f64>,
        ts: Instant,
    },
    Progress {
        agent_id: String,
        current: usize,
        total: usize,
        message: String,
        ts: Instant,
    },
    Error {
        agent_id: String,
        error: String,
        span_info: Option<SpanInfo>,
        ts: Instant,
    },
    ProgramStarted {
        source_path: String,
        ts: Instant,
    },
    ProgramFinished {
        result: Result<String, String>,
        duration_ms: u64,
        ts: Instant,
    },
}

enum UserPromptKind {
    Confirm { message: String },
    Choose { message: String, options: Vec<String> },
    Ask { message: String, default: Option<String> },
}

struct SpanInfo {
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
}
```

### EventBus

```rust
struct EventBus {
    tx: tokio::sync::broadcast::Sender<RuntimeEvent>,
}

impl EventBus {
    fn new() -> Self;
    fn send(&self, event: RuntimeEvent);
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<RuntimeEvent>;
}
```

Shared via `Arc<EventBus>` across all backends and UI components. Dioxus signals read from per-pane filtered subscriptions.

## Backend Implementations

### DxAiBackend

Wraps the real `ClaudeCodeAiBackend` (or any `AiBackend`). Before calling through, emits `AiCallStart`. After return, emits `AiCallComplete` or `AiCallError`. Also creates a Langfuse generation span.

```rust
struct DxAiBackend {
    inner: Box<dyn AiBackend>,
    bus: Arc<EventBus>,
    langfuse: Arc<LangfuseClient>,
    agent_id: String,
}

impl AiBackend for DxAiBackend {
    fn prompt(&self, text: &str, opts: &AiOpts, span: Span) -> Result<Value, LxError> {
        let call_id = next_call_id();
        self.bus.send(AiCallStart { ... });
        let trace = self.langfuse.create_generation(...);
        let start = Instant::now();
        let result = self.inner.prompt(text, opts, span);
        let elapsed = start.elapsed();
        match &result {
            Ok(val) => {
                trace.end_success(...);
                self.bus.send(AiCallComplete { ... });
            }
            Err(e) => {
                trace.end_error(...);
                self.bus.send(AiCallError { ... });
            }
        }
        result
    }
}
```

### LangfuseLogBackend

Replaces `StderrLogBackend`. Sends log entries to both the EventBus (for pane display) and Langfuse (as span events/logs).

```rust
struct LangfuseLogBackend {
    bus: Arc<EventBus>,
    langfuse: Arc<LangfuseClient>,
    agent_id: String,
}

impl LogBackend for LangfuseLogBackend {
    fn log(&self, level: LogLevel, msg: &str) {
        self.bus.send(Log { agent_id, level, msg, ts });
        self.langfuse.log_event(level, msg);
    }
}
```

### DxUserBackend

Renders interactive prompts in the agent's pane instead of stdin/stdout. Uses a oneshot channel per prompt — the backend blocks on the receiver while the UI component sends the user's response.

```rust
struct DxUserBackend {
    bus: Arc<EventBus>,
    agent_id: String,
    response_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<serde_json::Value>>>>,
}

impl UserBackend for DxUserBackend {
    fn confirm(&self, message: &str) -> Result<bool, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let prompt_id = next_prompt_id();
        *self.response_tx.lock() = Some(tx);
        self.bus.send(UserPrompt {
            agent_id, prompt_id,
            kind: Confirm { message },
            ts,
        });
        // Block until UI sends response
        let val = rx.blocking_recv().map_err(|_| "prompt cancelled")?;
        self.bus.send(UserResponse { agent_id, prompt_id, response: val.clone(), ts });
        val.as_bool().ok_or_else(|| "expected bool".into())
    }

    fn choose(&self, message: &str, options: &[String]) -> Result<usize, String> {
        // Same pattern: emit UserPrompt, block on oneshot, emit UserResponse
    }

    fn ask(&self, message: &str, default: Option<&str>) -> Result<String, String> {
        // Same pattern
    }

    fn progress(&self, current: usize, total: usize, message: &str) {
        self.bus.send(Progress { agent_id, current, total, message, ts });
    }

    fn progress_pct(&self, pct: f64, message: &str) {
        let current = (pct * 100.0) as usize;
        self.bus.send(Progress { agent_id, current, total: 100, message, ts });
    }

    fn status(&self, level: &str, message: &str) {
        self.bus.send(Log { agent_id, level, msg: message, ts });
    }

    fn table(&self, headers: &[String], rows: &[Vec<String>]) {
        // Emit as a structured log or a dedicated TableEvent
        self.bus.send(Emit { agent_id, value: format_table(headers, rows), ts });
    }

    fn check_signal(&self) -> Option<Value> {
        // Check for UI-initiated signals (pause, cancel, etc.)
        None
    }
}
```

### DxEmitBackend

Routes `emit` values to the agent's pane.

```rust
struct DxEmitBackend {
    bus: Arc<EventBus>,
    agent_id: String,
}

impl EmitBackend for DxEmitBackend {
    fn emit(&self, value: &Value, _span: Span) -> Result<(), LxError> {
        self.bus.send(Emit { agent_id, value: format!("{value}"), ts: Instant::now() });
        Ok(())
    }
}
```

### DxShellBackend

Wraps `ProcessShellBackend`, emitting `ShellExec` before and `ShellResult` after.

### DxYieldBackend

Renders yield values in the pane and blocks for orchestrator input, same oneshot pattern as `DxUserBackend`.

## Langfuse Integration

### LangfuseClient

Thin wrapper over the Langfuse REST API or the `langfuse` Rust crate (if available; otherwise direct HTTP via reqwest).

```rust
struct LangfuseClient {
    base_url: String,
    public_key: String,
    secret_key: String,
    http: reqwest::Client,
}

impl LangfuseClient {
    fn new_from_env() -> Self;

    fn create_trace(&self, name: &str, metadata: serde_json::Value) -> LangfuseTrace;

    fn create_generation(
        &self,
        trace: &LangfuseTrace,
        name: &str,
        model: &str,
        input: &str,
    ) -> LangfuseGeneration;
}
```

### What Gets Traced

| lx event | Langfuse concept | Data captured |
|----------|-----------------|---------------|
| Program start | Trace | program path, timestamp |
| `ai.prompt` call | Generation (child of trace) | prompt text, system prompt, model, response text, cost, duration, token counts |
| `agent.spawn` | Span (child of trace) | agent name, config |
| `agent ~>?` (ask) | Span (child of agent span) | message, response, duration |
| `agent ~>` (send) | Event on agent span | message payload |
| `log.*` calls | Event on current span | level + message |
| `refine` iterations | Span per round | round number, score, feedback |
| `mcp.call` | Span | tool name, args, result |
| Program end | Trace update | final result, total duration, total cost |

### Trace Hierarchy

```
Trace: "full_pipeline.lx"
├── Span: "main"
│   ├── Span: "run_manual"
│   │   ├── Generation: "router.route" (ai.prompt)
│   │   ├── Span: "agent:researcher" (spawn → kill)
│   │   │   ├── Generation: "research" (ai.prompt inside agent)
│   │   │   └── Generation: "breakdown" (ai.prompt inside agent)
│   │   ├── Span: "agent:composer" (spawn → kill)
│   │   │   ├── Generation: "compose" (ai.prompt inside agent)
│   │   │   └── Span: "refine" (grading loop)
│   │   │       ├── Generation: "grade round 1"
│   │   │       ├── Generation: "revise round 1"
│   │   │       └── Generation: "grade round 2"
│   │   └── Event: "emit: completed 2/3"
```

### Environment Variables

```
LANGFUSE_PUBLIC_KEY    — Langfuse project public key
LANGFUSE_SECRET_KEY    — Langfuse project secret key
LANGFUSE_BASE_URL      — Langfuse instance URL (default: https://cloud.langfuse.com)
```

If not set, the `LangfuseLogBackend` degrades to local-only logging (events still go to EventBus for pane display, just no remote push).

## UI Components

### PaneManager

Konsole-style split/tab layout. Each pane shows one agent's event stream.

- **Horizontal/vertical splits** — drag dividers to resize
- **Tabs within splits** — multiple agents can share a split via tabs
- **Auto-create pane** — when `AgentSpawned` fires, a new pane appears (configurable: auto-split or tab into existing)
- **Close on kill** — when `AgentKilled` fires, pane grays out with exit status (not auto-removed, user dismisses)
- **Main pane** — always present, shows the top-level program's events

```
┌──────────────────────┬──────────────────────┐
│  main                │  researcher          │
│  ─────────────────── │  ─────────────────── │
│  > run_manual        │  [AI] research       │
│  > router.route...   │  prompt: "fix the..."│
│  [AI] routing call   │  response: "Found 3  │
│  response: {type:    │  relevant files..."  │
│  "single"}           │  cost: $0.02         │
│                      │  ─────────────────── │
│                      │  [AI] breakdown      │
│                      │  prompt: "break..."  │
├──────────────────────┴──────────────────────┤
│  composer                                    │
│  ───────────────────────────────────────     │
│  [AI] compose — round 1 of refine           │
│  prompt: "compose solution from tasks..."   │
│  response: "## Implementation\n..."         │
│  cost: $0.04  duration: 8.2s                │
│  ─── refine round 1 ────────────────────    │
│  score: 72/100  feedback: "missing error.." │
│  ─── refine round 2 ────────────────────    │
│  score: 96/100  PASS                        │
│  ─────────────────────────────────────────── │
│  [CONFIRM] Deploy to staging? [Yes] [No]    │
└──────────────────────────────────────────────┘
```

### Pane (Single Agent View)

Terminal-style scrollback buffer. Events render as styled blocks:

| Event type | Rendering |
|-----------|-----------|
| `AiCallStart` | Header: `[AI] {model}` + truncated prompt |
| `AiCallComplete` | Response text + metadata bar (cost, duration, tokens) |
| `Emit` | Plain text line |
| `Log` | Colored by level: info=blue, warn=yellow, err=red, debug=gray |
| `MessageSend` | `→ {target}: {msg_summary}` |
| `MessageAsk` | `⇄ {target}: {msg_summary}` (with response when it arrives) |
| `ShellExec` | `$ {cmd}` monospace |
| `ShellResult` | stdout/stderr with exit code badge |
| `UserPrompt` | Interactive widget (button for confirm, radio for choose, input for ask) |
| `Progress` | Progress bar widget, updates in-place |
| `Error` | Red block with error message and source location |
| `TraceSpanRecorded` | Subtle metadata line with link to Langfuse |

### Toolbar

- **File picker** — select `.lx` file to run
- **Run / Stop** — start execution, cancel via signal
- **Layout** — preset layouts (single, 2-col, 3-col, grid) + manual split
- **Langfuse** — connection status indicator, link to current trace in Langfuse UI
- **Filter** — toggle event types visible in panes (hide/show AI calls, logs, shell, etc.)

### TracePanel (Optional Sidebar)

Pulls from Langfuse API to show:
- Current trace status and total cost
- Generation list with scores
- Latency breakdown
- Link to open full trace in Langfuse web UI

## Runner

The bridge between the Dioxus UI and the lx interpreter.

```rust
struct ProgramRunner {
    bus: Arc<EventBus>,
    langfuse: Arc<LangfuseClient>,
}

impl ProgramRunner {
    async fn run(&self, source_path: &str) -> Result<Value, LxError> {
        let source = tokio::fs::read_to_string(source_path).await?;
        let tokens = lx::lexer::lex(&source)?;
        let program = lx::parser::parse(tokens)?;

        let trace = self.langfuse.create_trace(source_path, json!({}));
        let agent_id = "main".to_string();

        let ctx = Arc::new(RuntimeCtx {
            ai: Arc::new(DxAiBackend {
                inner: Box::new(ClaudeCodeAiBackend),
                bus: self.bus.clone(),
                langfuse: self.langfuse.clone(),
                agent_id: agent_id.clone(),
            }),
            emit: Arc::new(DxEmitBackend {
                bus: self.bus.clone(),
                agent_id: agent_id.clone(),
            }),
            http: Arc::new(ReqwestHttpBackend),
            shell: Arc::new(DxShellBackend {
                inner: ProcessShellBackend,
                bus: self.bus.clone(),
                agent_id: agent_id.clone(),
            }),
            yield_: Arc::new(DxYieldBackend {
                bus: self.bus.clone(),
                agent_id: agent_id.clone(),
            }),
            log: Arc::new(LangfuseLogBackend {
                bus: self.bus.clone(),
                langfuse: self.langfuse.clone(),
                agent_id: agent_id.clone(),
            }),
            user: Arc::new(DxUserBackend {
                bus: self.bus.clone(),
                agent_id: agent_id.clone(),
            }),
        });

        self.bus.send(ProgramStarted { source_path, ts });
        let start = Instant::now();

        let source_dir = Path::new(source_path).parent().map(|p| p.to_path_buf());
        let mut interp = Interpreter::new(&source, source_dir, ctx);

        // Run on blocking task since interpreter is sync
        let result = tokio::task::spawn_blocking(move || interp.exec(&program)).await?;

        self.bus.send(ProgramFinished {
            result: result.as_ref().map(|v| format!("{v}")).map_err(|e| format!("{e}")),
            duration_ms: start.elapsed().as_millis() as u64,
            ts,
        });
        trace.end(result.is_ok());

        result
    }
}
```

### Subprocess Agent Pane Creation

When the interpreter calls `agent.spawn`, the child process is a separate `lx` invocation. To get events from child agents into panes, two approaches:

**Option A: Shared EventBus via IPC** — child processes connect to a Unix socket / named pipe where the parent EventBus listens. Each child gets a `DxRuntimeCtx` pointing at the shared bus. Requires modifying `agent.spawn` to pass connection info via env var.

**Option B: Stderr event protocol** — child processes emit JSON event lines on stderr (alongside normal log output). The parent monitors child stderr and feeds parsed events into the EventBus. No runtime modification needed, just a custom `LogBackend` in the child that writes structured JSON.

Recommended: **Option B** for simplicity. The parent already captures child stdout for IPC (agent ask/send protocol). Adding stderr monitoring is straightforward. The child `lx` process can detect `LX_DX_EVENTS=1` env var and switch to JSON-structured stderr output.

```
Parent process:
  spawn child with LX_DX_EVENTS=1
  → child stderr: {"event":"ai_call_start","agent_id":"researcher","prompt":"..."}
  → parent reads stderr lines, deserializes RuntimeEvent, sends to EventBus
```

## Agent Identity Propagation

Each backend needs to know which agent it belongs to. The `agent_id` field is set when constructing the `RuntimeCtx`:

- `"main"` for the top-level program
- For subprocess agents: derived from spawn config (agent name or PID)
- Propagated to child via `LX_AGENT_ID` env var

When `agent.spawn` is called, the `DxAiBackend` wrapper emits `AgentSpawned` with the new id, and the `PaneManager` auto-creates a pane for it.

## Dependencies

```toml
[dependencies]
dioxus = { version = "0.6", features = ["desktop"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json"] }
lx = { path = "../../crates/lx" }
chrono = "0.4"
uuid = { version = "1", features = ["v4"] }
```

## Langfuse HTTP API Surface Used

No Rust Langfuse SDK exists — use direct HTTP calls via reqwest:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/public/traces` | POST | Create trace for program run |
| `/api/public/traces/{id}` | PATCH | Update trace on completion |
| `/api/public/generations` | POST | Record LLM call (ai.prompt) |
| `/api/public/spans` | POST | Record agent lifecycle, mcp.call, refine rounds |
| `/api/public/events` | POST | Record log entries, emit values, sends |

All calls are fire-and-forget (buffered, async). Network failure does not block the interpreter.

## Open Questions

1. **Streaming AI responses** — current `AiBackend::prompt` is blocking and returns the full response. To show streaming tokens in the pane, we'd need to extend `AiBackend` with a streaming variant or add a callback. For v1, show "thinking..." then the full response on completion.

2. **Multi-program sessions** — should the app support running multiple lx programs simultaneously in separate pane groups? For v1, single program at a time.

3. **Pane persistence** — should pane layout and scroll position persist across app restarts? Nice-to-have, not v1.

4. **Remote Langfuse vs local** — support both cloud Langfuse and self-hosted. The base URL config handles this.

5. **Cost aggregation** — show running total cost in toolbar? Yes, derived from `AiCallComplete` events.

6. **Source highlighting** — when an error occurs, highlight the relevant source line in the program view. Requires the `SpanInfo` from `LxError`. Nice for v2.
