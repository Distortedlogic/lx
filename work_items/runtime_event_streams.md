# Runtime Event Streams

## Target

Every side effect the interpreter performs — emit, log, LLM call, tool invocation, agent message, yield — becomes an entry in a persistent, ordered stream. The stream is the program's execution history. It serves three purposes from one data structure:

1. **Observability** — subscribe and watch events in real-time (like pub/sub)
2. **Debug** — read back the full history after the fact (like a log)
3. **Resume** — replay the stream to skip completed work on restart (like a checkpoint)

## Design: Valkey Stream Semantics

Valkey streams are append-only logs where each entry has an auto-generated ID (`<ms>-<seq>`) and a set of field-value pairs. They're not pub/sub (messages aren't lost if nobody's listening) and they're not queues (multiple consumers can read the same history). The key operations:

| Operation | What it does |
|-----------|-------------|
| `XADD` | Append an entry, get back its ID |
| `XRANGE` | Read entries between two IDs (history scan) |
| `XREVRANGE` | Read entries in reverse |
| `XREAD` | Block until new entries appear (subscription) |
| `XLEN` | Count entries |
| `XTRIM` | Cap the stream to a max length |
| `XINFO` | Stream metadata |

The lx runtime stream mirrors this. Every backend operation (emit, log, llm.prompt, bash, agent.ask, yield) does an XADD. Consumers can XRANGE over history or XREAD to block for new events.

## How It Replaces the Current Architecture

Today the runtime has 5 separate backend traits that each do their own thing:

```
EmitBackend   → println (fire-and-forget)
LogBackend    → eprintln (fire-and-forget)
LlmBackend    → HTTP call (no record)
HttpBackend   → HTTP call (no record)
YieldBackend  → stdin/stdout or channel (no record)
```

With event streams, every backend writes to the same stream before doing its work:

```
emit "hello"
  → stream.xadd {kind: "emit", value: "hello"}
  → then println

log.info "step 1"
  → stream.xadd {kind: "log", level: "info", msg: "step 1"}
  → then eprintln

llm.prompt "question"
  → stream.xadd {kind: "llm.start", prompt: "question", call_id: 7}
  → HTTP call
  → stream.xadd {kind: "llm.done", call_id: 7, response: "answer", cost: 0.003}

Browser.click "e2"
  → stream.xadd {kind: "tool.start", tool: "browser", action: "click e2", call_id: 8}
  → bash agent-browser
  → stream.xadd {kind: "tool.done", call_id: 8, result: {...}}
```

The stream entry is the canonical record. The backend's actual I/O (println, HTTP, bash) is the side effect.

## Stream Entry Format

Each entry is a Record with a guaranteed `id` and `kind`, plus kind-specific fields:

```lx
{
  id: "1679083200123-0"    -- timestamp-ms + sequence
  kind: "emit"             -- event type
  agent: "main"            -- which agent produced this
  span: {line: 42, col: 5} -- source location
  ts: 1679083200123        -- unix ms
  -- kind-specific fields below
  value: "hello world"
}
```

### Event Kinds

| Kind | Extra Fields | Produced By |
|------|-------------|-------------|
| `program.start` | `source_path` | interpreter startup |
| `program.done` | `result`, `duration_ms` | interpreter shutdown |
| `emit` | `value` | `emit` expression |
| `log` | `level`, `msg` | `log.info/warn/err/debug` |
| `llm.start` | `call_id`, `prompt`, `model`, `tools` | `llm.prompt` / `llm.prompt_with` |
| `llm.done` | `call_id`, `response`, `cost_usd`, `duration_ms` | LLM response |
| `llm.err` | `call_id`, `error` | LLM failure |
| `tool.start` | `call_id`, `tool`, `args` | any `Tool.run()` |
| `tool.done` | `call_id`, `result` | tool completion |
| `tool.err` | `call_id`, `error` | tool failure |
| `shell.exec` | `call_id`, `cmd` | `bash` builtin |
| `shell.done` | `call_id`, `code`, `stdout`, `stderr` | bash completion |
| `agent.spawn` | `agent_id`, `script` | `agent.spawn` |
| `agent.kill` | `agent_id` | `agent.kill` |
| `agent.ask` | `from`, `to`, `msg` | `agent.ask` |
| `agent.tell` | `from`, `to`, `msg` | `agent.tell` |
| `agent.response` | `from`, `to`, `response`, `duration_ms` | agent reply |
| `yield.out` | `prompt_id`, `value` | `yield` expression (outgoing) |
| `yield.in` | `prompt_id`, `response` | `yield` response (incoming) |
| `http.req` | `call_id`, `method`, `url` | `http.get/post/...` |
| `http.res` | `call_id`, `status`, `body` | HTTP response |
| `store.op` | `store_id`, `op`, `key`, `value` | store mutations |
| `error` | `error`, `span_info` | runtime errors |

## Resume: Deterministic Replay

When a program restarts with an existing stream, the interpreter replays it:

1. Load the stream from disk (or Valkey)
2. For each `llm.start` + `llm.done` pair, cache the response keyed by `call_id`
3. For each `tool.start` + `tool.done` pair, cache the result
4. For each `shell.exec` + `shell.done` pair, cache the result
5. Start executing the program normally
6. When the interpreter hits an LLM call / tool call / bash call, check the replay cache first
7. If cached, return the cached result and skip the actual I/O
8. Once the cache is exhausted, switch to live execution (new entries append to the stream)

This is the same pattern as Temporal's deterministic replay — the stream is the event history, and side effects are replayed from history when available.

The `call_id` is a monotonic counter per program run. On replay, the interpreter regenerates the same sequence of call_ids (they're deterministic from the program's control flow), so they match the cached entries.

### What's NOT replayed

- `emit` — re-executed (the user wants to see output again)
- `log` — re-executed
- `yield` — re-executed (the orchestrator may have changed)
- `store.op` — skipped (store persistence handles its own state)

### Resume boundary

The stream stores a `checkpoint` marker entry periodically. On resume, the interpreter fast-forwards to the last checkpoint rather than replaying from the beginning. Checkpoints capture the interpreter's environment snapshot (variable bindings).

## Debug: Stream IS the Debug Log

The stream already contains everything a debugger needs:

- **What happened** — every event in order
- **When** — timestamps on every entry
- **Where** — source spans on every entry
- **Who** — agent_id on every entry
- **What it cost** — LLM cost_usd, duration_ms on completions
- **What went wrong** — error entries with span info

A debug view is just `stream.xrange stream_id "-" "+"` with filtering:

```lx
-- show all errors
stream.xrange s "-" "+" | filter (e) { e.kind == "error" }

-- show all LLM calls and their costs
stream.xrange s "-" "+" | filter (e) { e.kind | starts_with? "llm" }

-- show what agent "worker-3" did
stream.xrange s "-" "+" | filter (e) { e.agent == "worker-3" }
```

## lx-level API

The stream module gets new Valkey-style operations alongside the existing lazy stream ops:

```lx
-- create a runtime event stream (append-only log)
s = stream.log {persist: ".lx/events.jsonl"}

-- append
id = stream.xadd s {kind: "custom", data: "hello"}

-- read history
entries = stream.xrange s "-" "+"           -- all entries
entries = stream.xrange s "1679083200000" "+" -- from timestamp
entries = stream.xrange s "-" "+" {count: 10} -- first 10

-- read new entries (blocking)
entry = stream.xread s "$"                  -- next new entry
entry = stream.xread s "1679083200123-0"   -- after this ID

-- metadata
n = stream.xlen s
info = stream.xinfo s  -- {length, first_id, last_id, ...}

-- trim
stream.xtrim s {maxlen: 10000}
```

## StreamBackend Trait

The runtime gets a new backend trait on RuntimeCtx:

```rust
pub trait StreamBackend: Send + Sync {
    fn xadd(&self, fields: &IndexMap<Sym, LxVal>) -> Result<String, LxError>;
    fn xrange(&self, start: &str, end: &str, count: Option<usize>) -> Result<Vec<StreamEntry>, LxError>;
    fn xread(&self, last_id: &str, block_ms: Option<u64>) -> Result<Option<StreamEntry>, LxError>;
    fn xlen(&self) -> Result<u64, LxError>;
    fn xtrim(&self, maxlen: usize) -> Result<u64, LxError>;
}

pub struct StreamEntry {
    pub id: String,
    pub fields: IndexMap<Sym, LxVal>,
}
```

### Implementations

| Backend | When | Storage |
|---------|------|---------|
| `InMemoryStreamBackend` | Default, CLI, tests | `Vec<StreamEntry>` in memory, optional JSONL flush |
| `ValkeyStreamBackend` | Production, distributed | Valkey XADD/XRANGE via MCP or direct client |
| `NoopStreamBackend` | When streams disabled | Discards everything |

### In-Memory Backend

```rust
pub struct InMemoryStreamBackend {
    entries: RwLock<Vec<StreamEntry>>,
    seq: AtomicU64,
    persist_path: Option<PathBuf>,
}
```

On `xadd`: push to vec, optionally append one JSONL line to disk.
On `xrange`: slice the vec between start/end IDs.
On `xread` with block: use a `tokio::sync::Notify` to wake readers when new entries arrive.

### Valkey Backend

Delegates to Valkey via the MCP server already available in the environment:

```rust
pub struct ValkeyStreamBackend {
    stream_key: String,  // e.g. "lx:run:abc123"
    // uses RuntimeCtx.mcp or direct valkey client
}
```

This means a program's event history can be inspected from any Valkey client, shared across distributed agents, and survives process crashes.

## How Backends Write to the Stream

Each existing backend trait impl gains a reference to the stream and writes before performing its actual work. The stream write is the "before" hook; the actual I/O is the effect.

Example for EmitBackend:

```rust
pub struct StreamingEmitBackend {
    inner: Arc<dyn EmitBackend>,
    stream: Arc<dyn StreamBackend>,
    agent_id: String,
}

impl EmitBackend for StreamingEmitBackend {
    fn emit(&self, value: &LxVal, span: SourceSpan) -> Result<(), LxError> {
        let mut fields = IndexMap::new();
        fields.insert(intern("kind"), LxVal::str("emit"));
        fields.insert(intern("agent"), LxVal::str(&self.agent_id));
        fields.insert(intern("value"), value.clone());
        self.stream.xadd(&fields)?;
        self.inner.emit(value, span)
    }
}
```

This is the decorator pattern — StreamingEmitBackend wraps any EmitBackend. Same for StreamingLogBackend, StreamingLlmBackend, etc. The existing backends don't change. The streaming layer composes on top.

## Relationship to Existing Stream Module

The current `stream.rs` implements **lazy pull-based data streams** (map, filter, take, batch, fold). That's a different concept — it's like Rust iterators, for transforming data.

The runtime event stream is an **append-only persistent log**. Different data structure, different purpose. They share the name "stream" but the API is distinct (xadd/xrange vs map/filter/collect). Both coexist — the lazy stream for data processing, the event stream for runtime history.

## Implementation

### 1. StreamBackend trait and InMemoryStreamBackend

**File:** `crates/lx/src/runtime/mod.rs` — add StreamBackend trait
**File:** `crates/lx/src/runtime/stream_backend.rs` — InMemoryStreamBackend with:
- `Vec<StreamEntry>` behind RwLock
- ID generation: `{unix_ms}-{seq}` format
- Optional JSONL persistence (append-only file writes)
- `Notify` for blocking xread

### 2. Add `stream` field to RuntimeCtx

**File:** `crates/lx/src/runtime/mod.rs`

```rust
pub struct RuntimeCtx {
    // ... existing fields ...
    #[default(Arc::new(NoopStreamBackend))]
    pub event_stream: Arc<dyn StreamBackend>,
}
```

Default is Noop (no overhead if not opted in). CLI and desktop constructors wire up InMemoryStreamBackend.

### 3. Stream builtins: xadd, xrange, xread, xlen, xtrim, xinfo

**File:** `crates/lx/src/stdlib/event_stream.rs` — new module

Register as `stream.xadd`, `stream.xrange`, etc. alongside the existing `stream.from`, `stream.map` etc. Or use a separate namespace like `events.xadd` to avoid confusion. Decision: use `stream.xadd` etc. — the x-prefix already distinguishes them from the lazy stream ops.

### 4. Streaming backend decorators

**File:** `crates/lx/src/runtime/streaming.rs`

Decorator impls that wrap existing backends:
- `StreamingEmitBackend` wraps EmitBackend
- `StreamingLogBackend` wraps LogBackend
- `StreamingLlmBackend` wraps LlmBackend (writes llm.start before, llm.done/llm.err after)
- `StreamingShellBackend` wraps the bash builtin path

Each does `stream.xadd(...)` then delegates to the inner backend.

### 5. Wire decorators in interpreter/CLI startup

**File:** Where RuntimeCtx is constructed (CLI main, lx-desktop, tests)

When event streaming is enabled:
```rust
let stream = Arc::new(InMemoryStreamBackend::new(persist_path));
let emit = Arc::new(StreamingEmitBackend { inner: base_emit, stream: stream.clone(), agent_id: "main".into() });
let log = Arc::new(StreamingLogBackend { inner: base_log, stream: stream.clone(), agent_id: "main".into() });
// etc.
let ctx = RuntimeCtx { emit, log, event_stream: stream, ..defaults };
```

### 6. JSONL persistence

The InMemoryStreamBackend optionally writes each entry as one JSON line to a file:

```jsonl
{"id":"1679083200123-0","kind":"program.start","source_path":"main.lx","ts":1679083200123}
{"id":"1679083200124-0","kind":"emit","agent":"main","value":"hello","ts":1679083200124}
{"id":"1679083200125-0","kind":"llm.start","call_id":1,"prompt":"...","ts":1679083200125}
```

On program start, if the file exists, load it into the in-memory vec (enabling resume).

### 7. Replay cache for resume

**File:** `crates/lx/src/runtime/replay.rs`

```rust
pub struct ReplayCache {
    llm_results: HashMap<u64, LxVal>,     // call_id -> response
    tool_results: HashMap<u64, LxVal>,    // call_id -> result
    shell_results: HashMap<u64, LxVal>,   // call_id -> {code, stdout, stderr}
    next_call_id: AtomicU64,
    exhausted: AtomicBool,
}
```

Built from stream history on startup. The StreamingLlmBackend checks the cache before making a real HTTP call. When the cache misses, `exhausted` flips to true and all subsequent calls are live.

### 8. ValkeyStreamBackend

**File:** `crates/lx/src/runtime/valkey_backend.rs`

Uses the Valkey MCP server (already available) to XADD/XRANGE. Stream key is `lx:run:{run_id}`. This enables:
- Multiple agents writing to the same stream from different processes
- Stream inspection from Valkey CLI or any Valkey client
- Stream survival across process crashes
- Distributed resume

### 9. Tests

- Unit: InMemoryStreamBackend xadd/xrange/xlen/xtrim
- Unit: ID ordering and range queries
- Unit: JSONL round-trip (write + reload)
- Integration: decorator backends actually append entries
- Integration: replay cache skips LLM calls with cached results
- Integration: stream.xadd / stream.xrange from lx code

## Files Changed

- `crates/lx/src/runtime/mod.rs` — StreamBackend trait, event_stream field on RuntimeCtx
- `crates/lx/src/stdlib/mod.rs` — register event_stream module

## Files Created

- `crates/lx/src/runtime/stream_backend.rs` — InMemoryStreamBackend, NoopStreamBackend
- `crates/lx/src/runtime/streaming.rs` — Streaming{Emit,Log,Llm,Shell}Backend decorators
- `crates/lx/src/runtime/replay.rs` — ReplayCache for deterministic resume
- `crates/lx/src/runtime/valkey_backend.rs` — ValkeyStreamBackend
- `crates/lx/src/stdlib/event_stream.rs` — xadd/xrange/xread/xlen/xtrim/xinfo builtins
