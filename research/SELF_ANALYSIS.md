# lx Primitive Set — Self-Analysis

## The LLVM Analogy

```
┌─────────────────────────────────────────────────────┐
│  Layer 5: lx Packages (pure lx code)                │  ← maximize this
│  saga, reconcile, plan, budget, retry, audit, ...   │
├─────────────────────────────────────────────────────┤
│  Layer 4: Stdlib Bridges (thin Rust → lx wrappers)  │
│  fs, json, re, time, env, math                      │
├─────────────────────────────────────────────────────┤
│  Layer 3: Backend Traits (swappable impls)           │  ← the plugin boundary
│  Ai, Shell, Http, Emit, Yield, Log, User, Pane,    │
│  Embed, Store, Transport                            │
├─────────────────────────────────────────────────────┤
│  Layer 2: Runtime Primitives (execution model)       │
│  par, sel, spawn, send, ask, yield, emit, context   │
├─────────────────────────────────────────────────────┤
│  Layer 1: Language Core (parser + evaluator)         │  ← minimize this
│  values, functions, pipes, patterns, types           │
└─────────────────────────────────────────────────────┘
```

---

## Layer 1: Language Core (Rust, fixed, the grammar)

These are the atoms — what the parser and evaluator understand natively. Nothing here is swappable.

### Data Primitives (8)

| Primitive | What | Why it's a primitive |
|-----------|------|---------------------|
| **Int** | Arbitrary-precision integer | Numeric base type |
| **Float** | f64 | Numeric base type |
| **Bool** | true/false | Logic base type |
| **Str** | Immutable string with interpolation | Text base type, most common value in agentic work |
| **List** | Ordered collection `[a, b, c]` | Universal container |
| **Record** | Named fields `{name: "x", age: 5}` | Structural data — this is the JSON of lx |
| **Tagged** | Sum types `type Result = Ok(T) \| Err(T)` | Discriminated unions — messages, states, outcomes |
| **Fn** | First-class closures `\|x\| x + 1` | Computation unit |

### Computation Primitives (6)

| Primitive | Syntax | Why |
|-----------|--------|-----|
| **Bind** | `x = expr` / `x := expr` | Immutable + mutable bindings |
| **Pipe** | `expr \| fn` | Composition — the defining idiom of lx |
| **Match** | `x ? { Pat -> body }` | Branching + destructuring — how agents decide |
| **Spread** | `{..r, field: val}` / `[..list, item]` | Immutable update — records are the data model |
| **Try/Propagate** | `try fn` / `expr ^` / `expr ??` | Error boundary — `try` catches, `^` propagates, `??` defaults |
| **Pattern** | Literal/Bind/Wildcard/Tuple/List/Record/Tag/Guard | Structural matching on messages |

### Type Primitives (3)

| Primitive | Why it's a primitive |
|-----------|---------------------|
| **Trait** | Interface contracts — agent capabilities, message shapes |
| **Class** | Stateful objects with methods — agents, stores, graders |
| **Type** (sum) | Tagged unions — the message protocol |

### Module Primitives (2)

| Primitive | Why |
|-----------|-----|
| **use** | Import resolution |
| **export** (`+`) | Public API boundary |

**Total: 19 language primitives**

### What to CUT from language core

- **`mcp` declaration syntax** → should be a library pattern, not grammar. MCP is just "call a tool on a server" — expressible as a Trait + backend.
- **`refine` keyword** → it's `loop { grade; break_if_good; revise }`. Library function on top of `loop`.
- **`meta` keyword** → strategy iteration is `strategies | each { try_strategy }`. Library.

---

## Layer 2: Runtime Primitives (Rust, fixed, the execution model)

These are built into the interpreter because they define concurrency semantics that can't be expressed in lx itself.

### Concurrency (4)

| Primitive | Syntax | Semantics |
|-----------|--------|-----------|
| **par** | `par { a; b; c }` | Fork-join — all branches run, return tuple of results |
| **sel** | `sel { a -> h; b -> h }` | Race — first to complete wins, others cancelled |
| **pmap** | `list \| pmap fn` | Parallel map (syntactic sugar for par over list) |
| **timeout** | `timeout ms expr` | Wall-clock deadline |

### Agent Model (4)

| Primitive | Syntax | Semantics |
|-----------|--------|-----------|
| **spawn** | `agent.spawn spec` | Create actor process |
| **send** | `agent ~> msg` | Fire-and-forget message |
| **ask** | `agent ~>? msg` | Request-response message |
| **kill** | `agent.kill ref` | Terminate actor |

### Suspension (3)

| Primitive | Syntax | Semantics |
|-----------|--------|-----------|
| **yield** | `yield value` | Suspend execution, return value to caller, resume with response |
| **emit** | `emit value` | Fire-and-forget output (no suspension) |
| **context** | `context key: val { body }` | Ambient context propagation |

### State (2)

| Primitive | Why it's runtime |
|-----------|-----------------|
| **Store** | Concurrent mutable kv container — needs DashMap/locks, can't be pure lx |
| **Stream** | Channel-based async data flow — needs mpsc primitives |

**Total: 13 runtime primitives**

### What stays as library, not runtime

- **`cron`** → library using `loop` + `time.sleep` + `par`. Not a primitive.
- **`pipeline`** → library using `spawn` + channels. Expressible with `Stream` + `par`.

---

## Layer 3: Backend Traits (Rust traits, swappable — the plugin boundary)

This is the LLVM backend analogy. Each trait defines a capability boundary. Users swap implementations via `lx.toml` or programmatically.

### Current backends (keep all 9, add 2)

| Backend | Trait | Default Impl | Alt Impls to Support |
|---------|-------|-------------|---------------------|
| **Ai** | `prompt(text, opts) → Value` | `ClaudeCodeCli` | `AnthropicApi`, `OpenAiApi`, `OllamaLocal`, `MockAi` |
| **Shell** | `exec(cmd) → Value` | `ProcessShell` | `SshShell`, `ContainerShell`, `WasmShell`, `MockShell` |
| **Http** | `request(method, url, opts) → Value` | `ReqwestHttp` | `CachedHttp`, `MockHttp`, `ProxyHttp` |
| **Emit** | `emit(value)` | `StdoutEmit` | `FileEmit`, `WebSocketEmit`, `NoopEmit` |
| **Yield** | `yield_value(value) → Value` | `StdinStdoutYield` | `ChannelYield`, `UiYield` |
| **Log** | `log(level, msg)` | `StderrLog` | `StructuredLog`, `FileLog`, `TelemetryLog` |
| **User** | `confirm/choose/ask/progress` | `StdinStdoutUser` | `SlackUser`, `DiscordUser`, `UiUser` |
| **Pane** | `open/update/close/list` | `YieldPane` | `TerminalPane`, `DesktopPane` |
| **Embed** | `embed(texts, opts) → Value` | `VoyageEmbed` | `OpenAiEmbed`, `LocalEmbed`, `MockEmbed` |
| **Store** *(new)* | `get/set/delete/query` | `InMemoryStore` | `SqliteStore`, `RedisStore`, `FileStore` |
| **Transport** *(new)* | `connect/send/receive/close` | `StdioTransport` | `HttpSseTransport`, `WebSocketTransport`, `MockTransport` |

### New backends explained

**StoreBackend** — Currently Store is hardcoded as in-memory DashMap with optional JSON file persistence. This should be a backend trait so users can back it with SQLite, Redis, or any kv store. The Store *language primitive* stays the same — `Store()`, `.get`, `.set` — but the persistence layer is swappable.

**TransportBackend** — Currently MCP has its own transport code (stdio, HTTP+SSE). Agent IPC is another transport. Rather than hardcoding these, a generic Transport trait lets you swap how agents and services communicate. This subsumes MCP transport, agent IPC, and future protocols.

### How backends are configured

Currently `lx.toml` only recognizes hardcoded strings. The proposal: **backends are named and registered**, and `lx.toml` references them by name:

```toml
[backends]
ai = "anthropic-api"
shell = "container"
http = "cached-proxy"
store = "sqlite"
transport = "http-sse"

[backends.ai.config]
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[backends.store.config]
path = ".lx/state.db"

[backends.shell.config]
image = "ubuntu:24.04"
```

Backend implementations can be:
1. **Built-in** (compiled into lx binary) — the defaults
2. **Plugin** (loaded from shared library or WASM) — future
3. **lx-defined** (a Class implementing the backend Trait) — this is the self-hosting path

That third option is key: a user could write an AiBackend *in lx itself*:

```lx
class MyAiBackend : [AiBackend] {
  prompt = (text opts) {
    response = http.post "https://api.example.com/v1/chat" {
      body: {messages: [{role: "user", content: text}]}
    } ^
    response.body | json.parse ^
  }
}
```

---

## Layer 4: Stdlib Bridges (Rust, thin wrappers)

These are capabilities that MUST be Rust because they do system I/O, but they're thin — just wrapping the backend trait or a system crate.

| Module | What it wraps | Why Rust |
|--------|---------------|----------|
| **std::fs** | OS file system | System calls |
| **std::json** | serde_json | Performance (parsing speed matters) |
| **std::re** | regex crate | Performance (regex compilation) |
| **std::time** | std::time / chrono | System clock |
| **std::env** | std::env | Environment variables |
| **std::math** | Numeric operations | BigInt performance |

### What moves OUT of Rust stdlib into backends or lx

| Current Module | Where it goes | Why |
|----------------|---------------|-----|
| **std::git** | lx package `pkg/kit/git.lx` wrapping `ShellBackend` | It's just `$git ...` commands — pure string construction + shell calls |
| **std::mcp** | lx package wrapping `TransportBackend` | Protocol logic is pure; transport is the backend |
| **std::ai** | Thin bridge to `AiBackend` trait | Keep as bridge, but shrink — prompt construction moves to lx |
| **std::http** | Thin bridge to `HttpBackend` trait | Already essentially this |
| **std::md** | lx package | Markdown parsing/building is pure string logic |
| **std::diff** | lx package | Pure text diffing |
| **std::cron** | lx package using `time.sleep` + `par` | Pure scheduling logic |
| **std::workspace** | lx package | File discovery + configuration parsing |
| **std::introspect** | lx package | Runtime querying (needs a few builtins, but logic is lx) |

**Target: ~6 Rust stdlib bridges, down from ~40**

---

## Layer 5: lx Packages (pure lx, the standard library)

Everything else. This is where the "Terraform for agentic flows" patterns live — all expressible in lx itself.

### Core patterns (`pkg/core/`)

| Package | What | Primitives it uses |
|---------|------|--------------------|
| **prompt** | Prompt builder (system/section/constraint/render) | Record, Pipe, Spread |
| **plan** | DAG execution with dependency ordering | Store, Loop, Pattern match |
| **saga** | Compensating transactions (do/undo) | Store, Loop, Try, List |
| **reconcile** | Multi-source merging (6 strategies) | Store, Fold, Pattern match |
| **budget** | Cost tracking with parent propagation | Class, Store |
| **retry** | Backoff computation + retry loop | Loop, Time, Pattern match |
| **audit** | Text quality checking | Str operations, Pattern match |
| **score** | Weighted composite scoring | Fold, Record |
| **handoff** | Agent-to-agent context transfer | Record, Trait |

### Agent patterns (`pkg/agents/`)

| Package | What | Primitives it uses |
|---------|------|--------------------|
| **react** | ReAct loop (think/execute/observe) | Spawn, Ask, Loop, Store |
| **dispatch** | Pattern-based message routing | Pattern match, Record |
| **negotiate** | N-party consensus | Ask, Loop, Fold |
| **mock** | Test doubles for agents | Store (call recording) |
| **intercept** | Message middleware chain | Fn composition, List |
| **supervise** | Restart-on-failure | Spawn, Loop, Try |
| **pipeline** | Streaming stage processing | Stream, Spawn, Par |

### AI patterns (`pkg/ai/`)

| Package | What |
|---------|------|
| **quality** | Grade + refine loops |
| **reasoning** | Chain-of-thought construction |
| **perception** | Input classification + intent extraction |
| **reflect** | Post-action learning |

### Data patterns (`pkg/data/`)

| Package | What |
|---------|------|
| **context** | Context window management |
| **memory** | Tiered memory (short/long term) |
| **knowledge** | Knowledge base with retrieval |

---

## Summary: The Complete Primitive Count

| Layer | Count | Nature |
|-------|-------|--------|
| **Language Core** | 19 | Fixed syntax, parser+evaluator |
| **Runtime Primitives** | 13 | Fixed execution semantics |
| **Backend Traits** | 11 | Swappable via `lx.toml` or programmatic |
| **Stdlib Bridges** | 6 | Thin Rust wrappers |
| **lx Packages** | ~40+ | Pure lx, user-readable/modifiable |

**The key principle**: anything that's pure logic (string manipulation, control flow, data transformation) belongs in lx. Rust provides only: (a) the language itself, (b) concurrency primitives, (c) I/O trait boundaries, (d) performance-critical parsing (JSON, regex).

---

## The Swappability Matrix

What a user can customize without touching Rust:

| "I want to..." | Swap this backend | Configure in |
|----------------|-------------------|--------------|
| Use Anthropic API directly | `AiBackend → AnthropicApi` | `lx.toml [backends.ai]` |
| Use OpenAI | `AiBackend → OpenAiApi` | `lx.toml [backends.ai]` |
| Run agents in containers | `ShellBackend → ContainerShell` | `lx.toml [backends.shell]` |
| Persist state to SQLite | `StoreBackend → SqliteStore` | `lx.toml [backends.store]` |
| Send output to Slack | `EmitBackend → SlackEmit` | `lx.toml [backends.emit]` |
| Approval via Slack DM | `UserBackend → SlackUser` | `lx.toml [backends.user]` |
| MCP over HTTP instead of stdio | `TransportBackend → HttpSse` | `lx.toml [backends.transport]` |
| Mock everything for tests | All → `Mock*` variants | `sandbox.scope` in test code |
| Custom grading logic | N/A — it's lx code | Edit `pkg/ai/quality.lx` |
| Custom reconciliation | N/A — it's lx code | Edit `pkg/core/reconcile.lx` |
| Custom agent patterns | N/A — it's lx code | Write new `pkg/agents/*.lx` |

---

## What Changes from Current State

1. **Cut 3 keywords** — `mcp` declaration, `refine`, `meta` become library patterns
2. **Add 2 backends** — `StoreBackend`, `TransportBackend`
3. **Move ~34 Rust stdlib modules to lx** — most of the STDLIB_LX_HOIST work + git, md, diff, cron, workspace, introspect
4. **Make backend registration dynamic** — `lx.toml` references named implementations, not hardcoded strings
5. **Enable lx-defined backends** — a Class implementing a backend Trait can be used as a backend (self-hosting path)

The result: **32 Rust primitives** (19 language + 13 runtime) provide the fixed foundation. **11 backend traits** provide the plugin boundary. Everything else is lx code that users can read, fork, and modify. That's the minimal open covering.

---

## Existing Backend Trait Signatures (Current State)

### AiBackend
```rust
pub trait AiBackend: Send + Sync {
    fn prompt(&self, text: &str, opts: &AiOpts, span: Span) -> Result<Value, LxError>;
}
```
Options: `AiOpts { system, model, max_turns, resume, tools, append_system, disable_tools, json_schema }`

### EmitBackend
```rust
pub trait EmitBackend: Send + Sync {
    fn emit(&self, value: &Value, span: Span) -> Result<(), LxError>;
}
```

### HttpBackend
```rust
pub trait HttpBackend: Send + Sync {
    fn request(&self, method: &str, url: &str, opts: &HttpOpts, span: Span) -> Result<Value, LxError>;
}
```

### ShellBackend
```rust
pub trait ShellBackend: Send + Sync {
    fn exec(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
    fn exec_capture(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
}
```

### YieldBackend
```rust
pub trait YieldBackend: Send + Sync {
    fn yield_value(&self, value: Value, span: Span) -> Result<Value, LxError>;
}
```

### LogBackend
```rust
pub trait LogBackend: Send + Sync {
    fn log(&self, level: LogLevel, msg: &str);
}
```

### UserBackend
```rust
pub trait UserBackend: Send + Sync {
    fn confirm(&self, message: &str) -> Result<bool, String>;
    fn choose(&self, message: &str, options: &[String]) -> Result<usize, String>;
    fn ask(&self, message: &str, default: Option<&str>) -> Result<String, String>;
    fn progress(&self, current: usize, total: usize, message: &str);
    fn progress_pct(&self, pct: f64, message: &str);
    fn status(&self, level: &str, message: &str);
    fn table(&self, headers: &[String], rows: &[Vec<String>]);
    fn check_signal(&self) -> Option<Value>;
}
```

### PaneBackend
```rust
pub trait PaneBackend: Send + Sync {
    fn open(&self, kind: &str, config: &Value, span: Span) -> Result<Value, LxError>;
    fn update(&self, pane_id: &str, content: &Value, span: Span) -> Result<(), LxError>;
    fn close(&self, pane_id: &str, span: Span) -> Result<(), LxError>;
    fn list(&self, span: Span) -> Result<Value, LxError>;
}
```

### EmbedBackend
```rust
pub trait EmbedBackend: Send + Sync {
    fn embed(&self, texts: &[String], opts: &EmbedOpts, span: Span) -> Result<Value, LxError>;
}
```

---

## RuntimeCtx Composition (Current State)

```rust
pub struct RuntimeCtx {
    pub ai: Arc<dyn AiBackend>,
    pub emit: Arc<dyn EmitBackend>,
    pub http: Arc<dyn HttpBackend>,
    pub shell: Arc<dyn ShellBackend>,
    pub yield_: Arc<dyn YieldBackend>,
    pub log: Arc<dyn LogBackend>,
    pub user: Arc<dyn UserBackend>,
    pub pane: Arc<dyn PaneBackend>,
    pub embed: Arc<dyn EmbedBackend>,
    pub on_agent_event: Option<Arc<dyn Fn(AgentEvent) + Send + Sync>>,
    pub source_dir: parking_lot::Mutex<Option<PathBuf>>,
    pub workspace_members: HashMap<String, PathBuf>,
    pub dep_dirs: HashMap<String, PathBuf>,
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,
    pub test_threshold: Option<f64>,
    pub test_runs: Option<u32>,
}
```

Default implementations:
- `ai` → `ClaudeCodeAiBackend` (invokes `claude` CLI)
- `emit` → `StdoutEmitBackend`
- `http` → `ReqwestHttpBackend`
- `shell` → `ProcessShellBackend` (`sh -c`)
- `yield_` → `StdinStdoutYieldBackend` (JSON over stdout/stdin)
- `log` → `StderrLogBackend`
- `user` → `NoopUserBackend` (or `StdinStdoutUserBackend` if TTY)
- `pane` → `YieldPaneBackend`
- `embed` → `VoyageEmbedBackend` (Voyage AI API)

---

## Sandbox/Deny Backends (Current State)

| Deny Backend | Behavior |
|---|---|
| `DenyShellBackend` | Returns error: "shell access denied by sandbox policy" |
| `DenyHttpBackend` | Returns error: "network access denied by sandbox policy" |
| `DenyAiBackend` | Returns error: "AI access denied by sandbox policy" |
| `DenyPaneBackend` | Returns error: "pane access denied by sandbox policy" |
| `DenyEmbedBackend` | Returns error: "embedding access denied by sandbox policy" |

`RestrictedShellBackend` wraps an inner `ShellBackend` with a command allowlist.

Sandbox `scope()` dynamically replaces backends in `RuntimeCtx` based on `Policy`:
```rust
fn build_restricted_ctx(base: &Arc<RuntimeCtx>, policy: &Policy) -> Arc<RuntimeCtx> {
    let ai = if policy.ai { base.ai.clone() } else { Arc::new(DenyAiBackend) };
    let shell = match &policy.shell {
        ShellPolicy::Deny => Arc::new(DenyShellBackend),
        ShellPolicy::AllowList(cmds) => Arc::new(RestrictedShellBackend { ... }),
        ShellPolicy::Allow => base.shell.clone(),
    };
    // ...
}
```

---

## Real Program Patterns Observed

### Pattern 1: Agentic Work Orchestration (workrunner)
- Load config → status tracker → loop work items → (implement → diagnose → fix loop) → grade → record
- Uses: Store, cron, ai.prompt_with, par, pipe chains, pattern matching

### Pattern 2: Multi-Phase Task Orchestration
- Stateful loops with grade gates: implement → diagnose → auto-fix → grade → analyze → fix → re-grade
- Uses: loop/break, mutable bindings (<-), try, pattern match on results

### Pattern 3: AI Agent Wrappers
- Prompt assembly via builder pattern → ai.prompt_with with tool allowlist → persist response
- Uses: pipe, spread, Record, ai backend

### Pattern 4: Parallel Fan-Out Research
- `par { search1; search2; search3 }` → reconcile → synthesize → report
- Uses: par, reconcile, pipe, spread

### Pattern 5: DAG Execution with Hard Gates
- Define steps with deps → plan.run → compile gate (hard fail) → auto-fix → re-check
- Uses: plan, shell ($), loop, pattern match

### Pattern 6: ReAct Loop with Circuit Breaker
- spawn worker → loop { think → execute → observe → verify } → kill
- Uses: spawn, ask (~>?), loop, Store, try

### Pattern 7: Saga-Based Cognitive Pipeline (brain)
- 5 saga steps with real undo compensation → perceive → recall → reason → execute → respond
- Uses: saga, par, sel, context, Store, spawn

### Pattern 8: Prompt Builder
- `create() | system() | section() | constraint() | render()`
- Uses: pipe, spread, Record, fold

### Pattern 9: Reconciliation Strategies
- 6 strategies (union, intersection, vote, highest_confidence, max_score, merge_fields)
- Uses: Store, fold, pattern match, pipe

### Key Language Idioms
- `result | filter ok? | map (.score) | sort_by (.) | rev | first`
- `(a, b, c) = par { x; y; z }`
- `agent ~>? {action: "think" task: t} ^`
- `{..record, field: new_val}`
- `x ? { Ok r -> handle r; Err e -> fail e }`
- `loop { condition ? break value; round <- round + 1 }`
