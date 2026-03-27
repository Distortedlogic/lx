# lx Runtime Architecture: Backends, Event Streams, Control Channel

## Overview

lx has three runtime boundaries: backends (interpreter calls out to external tools), event streams (persistent log of everything that happened), and control channel (outside world commands the interpreter). All three are configured at runtime. None require writing Rust or recompiling the interpreter.

A dev working with lx writes two things: lx programs (`.lx` files) and MCP servers (external tools in any language). Everything else ships with the interpreter.

---

## Backends

### What a backend is

A backend is an MCP server — a process that speaks JSON-RPC on stdin/stdout. The lx interpreter connects to it as an MCP client. The lx program registers it as a module through the standard module system:

```lx
use tool "agent-browser" as Browser
```

From that point on, `Browser` is a module in scope like any other. Method calls dispatch through the MCP protocol.

### Syntax: `use tool`

`use tool` is a new variant of the `use` statement. The parser distinguishes it by the `tool` keyword followed by a string literal (the command name), not a module path. Existing `use` forms are unaffected:

- `use std/tool` — imports the Tool trait from stdlib (unchanged)
- `use tool "agent-browser" as Browser` — connects to an MCP server and binds it as a module
- `use ./my_module` — imports a local `.lx` file (unchanged)

The `as` clause is required for `use tool` — the module needs a name in scope.

### Relationship to existing `MCP` and `CLI` keywords

The existing `MCP` keyword (`MCP Foo = {command: "...", args: [...]}`) desugars to a class with `mcp.connect` and `mcp.call`. The existing `CLI` keyword desugars to a class that shells out via `bash`.

`use tool` replaces both. It's the single mechanism for connecting to external tools. The `MCP` and `CLI` keywords become unnecessary — `use tool` does what they do through the module system instead of keyword desugaring. They can be deprecated and eventually removed.

### MCP process lifecycle

**Spawn:** The interpreter spawns the tool process on the `use tool` statement — not lazily on first call. This makes connection errors visible at the point of registration, not buried in a later method call.

**Connection:** The interpreter's MCP client sends `initialize` on connect, receives the server's capabilities, and calls `tools/list` to discover available methods. The connection persists for the lifetime of the module binding (typically the program's lifetime).

**Shutdown:** On program exit (normal or error), the interpreter sends MCP `shutdown` to all connected tool processes and waits briefly for them to exit. If they don't exit, they are killed.

**Crash recovery:** If a tool process crashes mid-program, the next method call on that module returns `Err "tool process exited"`. The lx program can handle this with normal error handling (`^` propagation or match). The interpreter does not auto-restart — the lx program decides whether to retry.

### Tool discovery

The interpreter forwards any method name to the tool process via MCP `tools/call`. The tool decides what methods it accepts and returns an error for unknown ones. The interpreter does not gate method names — it passes them through and returns whatever the tool returns.

### How a call works

1. The lx program evaluates `Browser.click "e2"`
2. The interpreter resolves `Browser` — it's a tool module bound by `use tool`
3. The interpreter resolves `.click` — it's a method discovered via `tools/list`
4. If an event stream is active, the interpreter xadds `{kind: "tool.call", tool: "Browser", method: "click", args: "e2"}`
5. If resume replay cache has a matching entry (see Resume section), return the cached result and skip steps 6-8
6. The MCP client sends `{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"click","arguments":{"ref":"e2"}}}` on the tool process's stdin
7. The tool process does the work, writes a JSON-RPC response on stdout
8. The MCP client reads the response, deserializes the result to `LxVal`
9. If an event stream is active, the interpreter xadds `{kind: "tool.result", tool: "Browser", method: "click", result: ...}`
10. The `LxVal` result is returned to the lx program

### How tool logs work

The MCP protocol has `notifications/message`. When the tool process sends a log notification through the JSON-RPC protocol on stdout, the interpreter's MCP client receives it and xadds it to the event stream as `{kind: "tool.log", tool: "Browser", level: "info", msg: "..."}`.

stderr on the tool process is unstructured noise that lx ignores. stdout is exclusively the JSON-RPC protocol channel.

### How to write a backend

Write an MCP server in any language. It speaks JSON-RPC on stdin/stdout. It implements `initialize`, `tools/list`, and `tools/call`. It can send `notifications/message` for structured logging. It doesn't import anything from lx, doesn't link against anything, doesn't know lx exists. It just handles the MCP protocol.

An lx program registers it with `use tool "command-name" as ModuleName`.

### Pure lx backends

A backend can also be a `.lx` file that composes builtins and other tools:

```lx
use tool "agent-browser" as ab

+open = (url) { ab.navigate {url: url} }
+click = (ref) { ab.interact {action: "click", ref: ref} }
```

Another lx program imports it with `use ./my_browser as Browser`. Same module interface. The calling code doesn't know if it's talking to an MCP server or lx code.

### Swapping backends

The lx program decides which backend to use by what it imports:

```lx
use tool "agent-browser" as Browser
```

```lx
use tool "playwright-mcp" as Browser
```

Same module name, same methods, different external tool. The lx code using `Browser` is identical in both cases.

### Error handling

- **MCP error response:** The MCP protocol includes error codes and messages in JSON-RPC responses. The interpreter wraps these as `LxVal::Err` with the error message. The lx program handles them normally.
- **Process crash:** Returns `Err "tool 'Browser' process exited unexpectedly"`. The connection is dead — subsequent calls return the same error. The lx program can re-execute `use tool` to spawn a fresh process and rebind the module, or handle the error and move on. The interpreter does not auto-reconnect.
- **Timeouts:** Handled by the MCP protocol. The MCP client manages request timeouts and sends `$/cancelRequest` when they expire. lx does not define its own timeout mechanism.
- **Tool not found:** If the command binary doesn't exist on PATH, `use tool` fails immediately with `Err "command 'agent-browser' not found"`.

### Agent integration

When an agent is spawned via `agent.spawn`, it gets its own interpreter with its own module scope. Tool modules are NOT shared — each agent that needs a tool must `use tool` it in its own script. This means each agent gets its own MCP connection and its own tool process (or a separate session on the same process, depending on the tool).

The event stream IS shared — all agents write to the same stream. The `agent` field on each stream entry identifies which agent produced it.

### Agent communication

Agents are lx programs. The Agent trait is defined in lx (`std/agent.lx`). The event stream is the shared communication channel between all agents:

- `agent.ask(agent_id, msg)` — send a message to any agent, wait for a response
- `agent.tell(agent_id, msg)` — send a message to any agent, do not wait
- `yield value` — send a value out, wait for the next message

All agent communication flows through the event stream. Any agent can message any other agent by ID. The stream provides broadcast, pub/sub, and direct messaging through the same substrate. The event stream logs all exchanges (`agent.ask`, `agent.tell`, `agent.response`, `yield.out`, `yield.in`).

For top-level programs (not spawned agents), `yield` delivers through the control channel — the orchestrator (CLI, desktop) receives the yielded value and responds via the inject command.

---

## Event Streams

### What the event stream is

An ordered, append-only log of everything that happened during program execution. Every tool call, every result, every emit, every log, every error, every agent message — with timestamps, source spans, and agent IDs. Three layers, not alternatives:

- **In-memory stream** — always on. The runtime's live event log. Agents within the same program subscribe via xread. This is the stream.
- **JSONL persistence** — automatic. The in-memory stream writes to `.lx/stream.jsonl` during execution. Enables resume across restarts and post-hoc debug. This is the stream's durability layer.
- **External streaming** — optional. An MCP server (e.g. Valkey) for cross-process subscription and distributed access. Configured in the manifest. This is the stream's distribution layer.

Three purposes from the same data:

- **Observability** — xread on the in-memory stream for real-time watching within the program, or the external backend for cross-process
- **Debug** — xrange on the in-memory stream during execution, or read the JSONL file after the fact
- **Resume** — load the JSONL file on restart, replay cached tool results

### Configuration

The stream is always on. No `use stream` statement needed. The JSONL file writes to `.lx/stream.jsonl` automatically.

For cross-process streaming, the manifest configures an external backend:

```toml
[stream]
command = "valkey-stream-mcp"
```

When an external backend is configured, the interpreter writes to both the in-memory stream and the external backend. The JSONL file is still written for resume.

The `stream` module is always available in lx programs — no import needed.

This does NOT conflict with the existing `std/stream` module (the lazy pull-based data stream with map/filter/collect). That module is imported via `use std/stream`. Different module, different purpose. The lazy data stream is for transforming data. The event stream is for runtime history. They coexist.

### Stream module methods

- `stream.xadd {kind: "...", ...}` — append an entry, returns the generated ID string
- `stream.xrange start end` — read entries between two IDs. `"-"` means beginning, `"+"` means end. Returns a list of entry records.
- `stream.xrange start end {count: N}` — read at most N entries in the range
- `stream.xread last_id` — block until a new entry appears after `last_id`. `"$"` means "from now." Returns the entry.
- `stream.xread last_id {timeout_ms: N}` — block with timeout. Returns `None` on timeout.
- `stream.xlen` — return entry count
- `stream.xtrim {maxlen: N}` — remove entries beyond the max length (oldest first)

### Stream entry format

Each entry is a record with:

- `id` — auto-generated, format `{unix_ms}-{seq}`. The millisecond timestamp is wall clock time. The sequence number is a per-millisecond monotonic counter starting at 0. Example: `"1679083200123-0"`, `"1679083200123-1"` for two events in the same millisecond. This matches Valkey stream ID format exactly.
- `kind` — string identifying the event type
- `agent` — string identifying which agent produced it (`"main"` for the top-level program)
- `ts` — unix millisecond timestamp (same as the ms portion of the ID)
- `span` — source location record `{line: N, col: N}` (when available)
- Plus kind-specific fields

### Event kinds

| Kind | Additional fields | Produced by |
|---|---|---|
| `program.start` | `source_path` | Interpreter startup |
| `program.done` | `result`, `duration_ms` | Interpreter shutdown |
| `emit` | `value` | `emit` expression |
| `log` | `level` (info/warn/err/debug), `msg` | `log.info/warn/err/debug` builtins |
| `tool.call` | `call_id`, `tool`, `method`, `args` | Auto-logged before any tool module call |
| `tool.result` | `call_id`, `tool`, `method`, `result` | Auto-logged after any tool module call |
| `tool.error` | `call_id`, `tool`, `method`, `error` | Auto-logged when a tool module call fails |
| `tool.log` | `tool`, `level`, `msg` | MCP `notifications/message` from a tool |
| `agent.spawn` | `agent_id`, `script` | `agent.spawn` builtin |
| `agent.kill` | `agent_id` | `agent.kill` builtin |
| `agent.ask` | `from`, `to`, `msg` | `agent.ask` builtin |
| `agent.tell` | `from`, `to`, `msg` | `agent.tell` builtin |
| `agent.response` | `from`, `to`, `response`, `duration_ms` | Agent reply |
| `yield.out` | `prompt_id`, `value` | `yield` expression (outgoing) |
| `yield.in` | `prompt_id`, `response` | `yield` response (incoming) |
| `error` | `error`, `span` | Runtime errors |

### Auto-logging: where it happens

The interception point is in the interpreter's method dispatch for tool modules. When the interpreter evaluates a field access + call on a module that was created by `use tool`, the dispatch path is:

1. `eval` resolves the field access (`Browser.click`) — recognizes `Browser` as a tool module
2. Before calling the MCP client, it xadds `tool.call` to the stream (if stream is active)
3. Calls the MCP client
4. After the call returns, it xadds `tool.result` or `tool.error` to the stream
5. Returns the result to the lx program

This is a single code path in the interpreter that handles ALL tool module calls. Adding a new tool doesn't require any auto-logging code — it's automatic because the dispatch goes through the same path.

The `call_id` is a monotonic counter incremented per tool call during the program run. It pairs the `tool.call` and `tool.result` entries.

### Auto-logging: what gets logged beyond tool calls

- `emit` expression — the interpreter's eval case for `Expr::Emit` xadds to the stream after calling `ctx.emit`
- `log.*` builtins — each log builtin xadds to the stream after calling `ctx.log`
- `yield` expression — the interpreter's eval case for `Expr::Yield` xadds `yield.out` before yielding and `yield.in` after receiving the response
- `agent.*` builtins — agent.spawn/kill/ask/tell xadd to the stream
- Program start/done — the interpreter xadds `program.start` at the beginning and `program.done` at the end

These are a fixed set of interception points in the interpreter — one per language keyword/builtin that produces side effects. They're not extensible because the set of interpreter operations that produce events is fixed.

### Concurrency and ordering

When multiple agents run in parallel (via `par` or spawned agents), each agent xadds to the same stream. The stream's ID generation (timestamp + sequence counter) provides a total order. The sequence counter is atomic, so concurrent xadds in the same millisecond get distinct sequential IDs. Entries from different agents interleave chronologically.

For causal ordering within a single agent, entries are naturally ordered because a single agent evaluates sequentially — it can't xadd two entries simultaneously.

### Trimming

No auto-trim. The JSONL file and in-memory stream grow for the duration of the program. The lx program can call `stream.xtrim {maxlen: 10000}` explicitly if needed. Most lx programs are finite workflows — the stream is bounded by the program's lifetime. If trimming is needed for long-running programs, the lx program does it explicitly.

### Direct use from lx programs

The lx program can write to and read from the stream directly:

```lx
stream.xadd {kind: "checkpoint", state: some_data}

entries = stream.xrange "-" "+"
errors = entries | filter (e) { e.kind == "error" }

-- watch for new events in real time
last = "$"
loop {
  entry = stream.xread last {timeout_ms: 5000}
  entry ? {
    emit entry
    last = entry.id
  } : {}
}
```

### Debug

Debug is reading the stream. The stream contains: what happened (kind), when (ts), where in source (span), which agent (agent), what arguments (args), what result (result), what went wrong (error). A debug view is a filtered xrange.

### Resume

**How matching works:** Each tool call gets a monotonic `call_id` (1, 2, 3, ...) assigned by the interpreter in the order calls are encountered during execution. The call_id is deterministic — the same program with the same inputs produces the same sequence of call_ids because the control flow is the same.

On restart with an existing JSONL file:

1. The interpreter loads the JSONL file into the in-memory stream
2. It scans for `tool.call` + `tool.result` pairs, keyed by `call_id`
3. It builds a replay cache: `HashMap<u64, LxVal>` mapping call_id to cached result
4. It resets the call_id counter to 1
5. It begins executing the program normally
6. When a tool call is reached, the interpreter checks: is there a cached result for the current call_id?
7. If yes: return the cached result, skip the actual MCP call, increment call_id
8. If no: the cache is exhausted. Execute the call live, xadd the result, increment call_id. All subsequent calls are live.

**What gets replayed:** Only tool calls (external MCP calls). These are the expensive, non-deterministic operations.

**What gets re-executed:** `emit`, `log`, `yield`, store operations, pure lx computation. These are either cheap (emit/log), stateful (store), or interactive (yield — the orchestrator may have changed).

**When replay breaks:** If the program's control flow changes between runs (different code, different conditional branches due to external state), the call_id sequence diverges from the cached entries. At that point the cache misses and execution continues live. This is safe — it just means less replay benefit, not incorrect behavior.

### External streaming backend

For cross-process subscription, the dev writes an MCP server that handles `xadd`, `xrange`, `xread`, `xlen`, `xtrim` and configures it in the manifest:

```toml
[stream]
command = "valkey-stream-mcp"
```

The interpreter writes to both the in-memory stream (for in-process subscription) and the external backend (for cross-process subscription). The JSONL file is always written regardless. The external backend is an additional distribution layer, not a replacement for the in-memory stream or the JSONL persistence.

---

## Control Channel

### What the control channel is

The control channel is how the outside world sends commands to the interpreter while a program is running. It is not something the lx program uses or configures — it's between the host environment and the interpreter.

### How it works

The interpreter ships with built-in control transports: stdin, WebSocket, TCP. The user picks one at launch:

```bash
lx run main.lx --control stdin
lx run main.lx --control ws://localhost:8080
lx run main.lx --control tcp://localhost:9000
lx run main.lx                                  # no control channel (default)
```

Default is no control channel — the program runs uncontrolled. Adding `--control` enables it.

### Command set

| Command | Request | Response |
|---|---|---|
| pause | `{"cmd": "pause"}` | `{"ok": true}` |
| resume | `{"cmd": "resume"}` | `{"ok": true}` |
| cancel | `{"cmd": "cancel"}` | `{"ok": true}` (then program exits) |
| inspect | `{"cmd": "inspect"}` | `{"ok": true, "state": {"call_stack": [...], "env": {...}, "stream_position": "1679083200123-5"}}` |
| inject | `{"cmd": "inject", "value": ...}` | `{"ok": true}` (value delivered to pending yield) |

All commands are single-line JSON. Responses are single-line JSON. The wire format is the same regardless of transport (stdin, WebSocket, TCP). The transport just carries the bytes.

### How the control channel runs

The control channel is a separate async task, independent of the interpreter's eval loop. The interpreter does not poll or check it. The control channel task acts on the interpreter from outside:

- **Cancel:** The control channel task tells the MCP client to drop the in-flight connection. The blocked call returns `Err "cancelled"`. The interpreter sees the error and stops the program. Cancel works mid-call — the interpreter doesn't need to be between steps.
- **Pause:** The control channel task sets a pause flag on the interpreter's shared state. The interpreter checks this flag before the next eval step and waits until it's cleared. If the interpreter is mid-tool-call, pause takes effect when the call returns.
- **Resume:** The control channel task clears the pause flag. The interpreter continues.
- **Inspect:** The control channel task reads the interpreter's state directly (env, call stack, stream position) and responds. This can happen while the interpreter is paused or between steps.
- **Inject:** The control channel task delivers a value to a pending yield's response channel.

The interpreter only sees the side effects — a failed MCP call, a pause flag set, a yield response delivered. It doesn't know about the control channel mechanism. The control channel task and the interpreter share state through atomic flags and channels, not through polling.

### Why it's not extensible

The command set is fixed because it maps to interpreter operations: pause the eval loop, snapshot the environment, inject into yield. These are intrinsic to the interpreter. A dev building a new host environment picks a transport flag — they don't extend the command set.

### Relationship to the event stream

- Event stream: interpreter → outside world (what happened)
- Control channel: outside world → interpreter (what to do)

A typical debug workflow: launch with `--control ws://localhost:8080`, connect a WebSocket client, send `{"cmd": "pause"}`, call `stream.xrange "-" "+"` to see what happened so far, send `{"cmd": "inspect"}` to see current state, send `{"cmd": "resume"}` to continue.

---

## Relationship to Existing Codebase

### RuntimeCtx backend traits

The current `RuntimeCtx` has `EmitBackend`, `LogBackend`, `LlmBackend`, `HttpBackend`, `YieldBackend` as separate `Arc<dyn Trait>` fields.

In the target architecture, all of these go away. The event stream and control channel replace them:

- `EmitBackend` → `emit` writes a stream entry: `{kind: "emit", value: ...}`
- `LogBackend` → `log.*` writes a stream entry: `{kind: "log", level: "...", msg: "..."}`
- `LlmBackend` → LLM is an external tool module: `use tool "claude-mcp" as llm`
- `HttpBackend` → HTTP is an external tool module: `use tool "http-mcp" as http`
- `YieldBackend` → stays. Yield logs to the stream (`yield.out`, `yield.in`) but the actual value delivery depends on context: the control channel for top-level programs, the agent communication mechanism for spawned agents. The YieldBackend trait abstracts over which is active.

RuntimeCtx holds the in-memory event stream, the control channel, and the yield backend. Everything else is either a stream entry or a tool module.

### Existing `MCP` keyword

The `MCP` keyword (`MCP Foo = {command: "...", args: [...]}`) desugars to a class with `mcp.connect`/`mcp.call`. `use tool` provides the same capability through the module system. Existing code using `MCP` continues to work — both mechanisms use the same underlying MCP client.

### Existing `CLI` keyword

The `CLI` keyword desugars to a class that shells out via `bash`. `use tool` provides a structured alternative where the tool author writes an MCP server instead of a raw CLI. Existing code using `CLI` continues to work.

### Existing `stream` module

The existing `std/stream` module (`stream.from`, `stream.map`, `stream.filter`, `stream.collect`, etc.) is a lazy pull-based data processing stream. It is unrelated to the event stream. They coexist:

- `use std/stream` — lazy data stream for transforming lists/iterators
- `stream` (always available, no import) — runtime event stream for execution history

Different syntax, different module, different purpose. No collision.

---

## What a dev writes

| What they're doing | What they write | Language |
|---|---|---|
| lx program | `.lx` files | lx |
| External tool backend | MCP server | Any |
| Pure lx backend | `.lx` module composing other tools | lx |
| External stream backend | MCP server handling xadd/xrange/xread (for cross-process subscription) | Any |
| New host environment | Nothing — pick a `--control` transport flag | N/A |

No Rust. No recompilation. No plugin system. MCP servers and lx code.

## What ships with the interpreter

| Component | Ships as |
|---|---|
| MCP client | Built into interpreter, connects to tool processes via `use tool` |
| Tool module dispatch | Built into interpreter, bridges module method calls to MCP `tools/call` |
| Tool dispatch | Built into interpreter, forwards method names to tool process, tool decides what it accepts |
| Event stream auto-logging | Built into interpreter, interception at tool module dispatch + emit/log/yield/agent |
| In-memory event stream | Built into interpreter, always on, JSONL persistence to `.lx/stream.jsonl` automatic |
| Resume/replay cache | Built into interpreter, activated when stream history exists on startup |
| Control transports (stdin, WS, TCP) | Built into interpreter, selected via `--control` flag |
