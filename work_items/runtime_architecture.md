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

The interpreter forwards any method name to the tool process via MCP `tools/call`. The tool decides what methods it accepts and returns an error for unknown ones. The interpreter does not validate method names against the `tools/list` result — `tools/list` is for discovery and introspection (e.g. UI autocomplete), not for gating calls at runtime. The tool process is the authority on what it accepts.

### How a call works

1. The lx program evaluates `Browser.click "e2"`
2. The interpreter resolves `Browser` — it's a tool module bound by `use tool`
3. The interpreter forwards `.click` as the method name to the tool process
4. The interpreter xadds `{kind: "tool/call", tool: "Browser", method: "click", args: "e2"}` to the event stream
5. The MCP client sends `{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"click","arguments":{"ref":"e2"}}}` on the tool process's stdin
6. The tool process does the work, writes a JSON-RPC response on stdout
7. The MCP client reads the response, deserializes the result to `LxVal`
8. The interpreter xadds `{kind: "tool/result", tool: "Browser", method: "click", result: ...}` to the event stream
9. The `LxVal` result is returned to the lx program

### How tool logs work

The MCP protocol has `notifications/message`. When the tool process sends a log notification through the JSON-RPC protocol on stdout, the interpreter's MCP client receives it and xadds it to the event stream as `{kind: "tool/log", tool: "Browser", level: "info", msg: "..."}`.

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

Another lx program imports it with the same `use tool` syntax:

```lx
use tool "./my_browser.lx" as Browser
```

The interpreter sees the `.lx` extension and loads it as a pure lx module, but routes calls through tool dispatch — same auto-logging, same observability, same module interface. The calling code doesn't know if it's talking to an MCP server or lx code. There is one `use tool` mechanism, not two import styles.

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
- **Process crash:** Returns `Err "tool 'Browser' process exited unexpectedly"`. The connection is dead — subsequent calls return the same error. `use tool` is valid in any scope (not just top-level), so the lx program can re-execute it inside an error handler to spawn a fresh process and rebind the module. The interpreter does not auto-reconnect.
- **Timeouts:** Handled by the MCP protocol. The MCP client manages request timeouts and sends `$/cancelRequest` when they expire. lx does not define its own timeout mechanism.
- **Tool not found:** If the command binary doesn't exist on PATH, `use tool` fails immediately with `Err "command 'agent-browser' not found"`.

### Agent integration

Agents are lx classes defined with the `Agent` keyword. The Agent trait is defined in lx (`std/agent.lx`). An agent defines `handle`, `run`, or both:

- **`handle`** — the reactive interface. The runtime calls it when a message arrives. The return value is delivered back to `ask` callers (or discarded for `tell`).
- **`run`** — the autonomous interface. The runtime calls it once on spawn. The agent drives its own execution — polling, watching, scheduling, whatever it needs.

If only `handle` is defined, the runtime provides a default `run` that loops on incoming messages and dispatches to `handle`. If only `run` is defined, the agent ignores messages. If both are defined, the runtime runs `run` as the agent's main task and calls `handle` when messages arrive — no entanglement, no polling. Execution within the agent is serialized: if `run` is mid-expression-eval when a message arrives, `handle` waits for that expression to finish. An expression is one AST node — a function call, a tool call, an assignment. A tool call that takes 30 seconds is one expression; `handle` waits the full 30 seconds. Between expressions, the runtime can interleave `handle`. No concurrent access to agent state.

`stop` is a keyword available inside agent code (both `handle` and `run`). It terminates the agent — cancels `run`, drains pending messages, unsubscribes from all channels, and writes an `agent/kill` entry to the event stream. After `stop`, the agent is gone. `tell` or `ask` targeting a stopped agent returns `Err "agent 'Name' not running"`.

`spawn` is a keyword that creates a concurrent async task for an Agent class. It instantiates the agent, starts it concurrently, and wires up messaging:

```lx
channel research
channel writing

-- reactive agent: responds to messages
Agent Researcher = {
  subscribes = [research]
  handle = (msg) {
    result = do_research msg.topic
    {topic: msg.topic, data: result}
  }
}

-- autonomous agent: drives its own loop, also responds to messages
Agent Watcher = {
  handle = (msg) {
    (msg.cmd == "stop") ? { stop } : {}
  }
  run = () {
    use std/events
    last = "$"
    loop {
      entry = events.xread last {timeout_ms: 5000}
      entry ? {
        (entry.kind == "runtime/error") ? {
          tell "Coordinator" {alert: entry}
        } : {}
        last = entry.id
      } : {}
    }
  }
}

spawn Researcher
spawn Watcher

-- discover agents on the research channel, send messages directly
peers = research.members
result = ask peers.0 {topic: "quantum computing"}
```

Each agent class is spawned once — agent names are unique, derived from the class name (`"Researcher"`, `"Watcher"`). Spawning the same class twice is an error. If you need multiple agents serving the same role, define distinct classes and subscribe them to the same channel (see Channels below).

Each spawned agent runs in its own concurrent async task with its own module scope. Tool modules are not shared — each agent that needs a tool must `use tool` it in its own code. The event stream IS shared — all agents write to the same stream.

### Messaging: `tell` and `ask`

Agents communicate directly via `tell` (fire-and-forget) and `ask` (request-response). Both dispatch to the target agent's `handle` method:

- `tell agent_name msg` — send a message, don't wait. The agent's `handle` is called; the return value is discarded.
- `ask agent_name msg` — send a message, suspend the calling task until the agent's `handle` returns. The return value of `handle` becomes the result of `ask`.

```lx
tell "Researcher" {topic: "quantum computing"}

result = ask "Researcher" {topic: "quantum computing"}
```

`tell` and `ask` are the only message primitives. They target agents by name. `ask` suspends only the calling task — other agents and the main program continue executing. `run` does not participate in messaging — it's the agent's private autonomous loop. An agent with only `run` and no `handle` cannot receive messages via `tell`/`ask`. The event stream logs all messages (`agent/tell`, `agent/ask`, `agent/reply`).

The main program is not an agent and cannot be targeted by `tell` or `ask` — it has no `handle` method. The main program can send messages to agents (it can call `tell` and `ask`), but agents cannot send messages back to it. Agents that need to surface results to the main program write to the event stream; the main program reads them via `events.xrange` or `events.xread`.

For top-level programs (not spawned agents), `yield` suspends execution and waits for a response. The delivery mechanism is the `YieldBackend` trait on `RuntimeCtx` — the host environment (CLI, desktop, test harness) provides the implementation. When a control channel is active, the CLI's `YieldBackend` implementation writes the yielded value to stdout and waits for an `inject` command on the control channel. When no control channel is active, the host can still implement `YieldBackend` however it wants (e.g. the test harness auto-responds). The control channel is one transport for yield responses, not the only one. Within agents, `yield` is not used — agents communicate via `tell`/`ask`.

### Channels

Channels are the discovery and topology layer. A channel is a named group that agents subscribe to, declaring their interest or capability. The topology is visible at a glance without reading agent internals.

```lx
channel research
channel writing
channel review
```

- `channel.members` — returns the list of agent names currently subscribed to this channel
- `channel.subscribe agent_name` — imperatively register an agent on this channel at runtime

Agents can also subscribe declaratively via `subscribes = [channel1, channel2]` on the Agent class. `subscribes` is a reserved field — the runtime reads it on `spawn` and auto-registers the agent on the listed channels before calling `run` or accepting messages. Declarative subscription is the common case; `channel.subscribe` exists for dynamic registration after spawn.

Channels do not carry messages. They are a registry — you query a channel to discover which agents handle a topic, then send messages directly via `tell`/`ask`. New agents can join existing channels without rewiring anything.

Why channels instead of just `tell "Researcher" msg`? Direct naming works when the caller knows exactly which agent to talk to. Channels matter when it doesn't — dynamic agent pools where the count or names aren't known at write time, broadcasting to all agents with a capability, or letting new agents join a role without updating callers:

```lx
channel workers

Agent FastWorker = { subscribes = [workers], handle = (msg) { ... } }
Agent ThoroughWorker = { subscribes = [workers], handle = (msg) { ... } }
spawn FastWorker
spawn ThoroughWorker

-- fan out to all workers without knowing their names or count
workers.members | each (w) { tell w {task: next_task()} }
```

No ACLs at the runtime level. All agents in the same program can subscribe to any channel and message any agent by name. The programmer controls the topology by declaring channels and deciding which agents subscribe to which.

---

## Event Streams

### What the event stream is

An ordered, append-only log of everything that happened during program execution. Every tool call, every result, every emit, every log, every error, every agent message — with timestamps, source spans, and agent IDs. Three layers, not alternatives:

- **In-memory stream** — always on. The runtime's live event log. lx programs can subscribe via `events.xread`. This is the stream.
- **JSONL persistence** — automatic. The in-memory stream writes to `.lx/stream.jsonl` during execution. Enables post-hoc debug. This is the stream's durability layer.
- **External streaming** — optional. An MCP server (e.g. Valkey) for cross-process subscription and distributed access. Configured in the manifest. This is the stream's distribution layer.

Two purposes from the same data:

- **Observability** — xread on the in-memory stream for real-time watching within the program, or the external backend for cross-process
- **Debug** — xrange on the in-memory stream during execution, or read the JSONL file after the fact

### Configuration

The stream is always on. No `use stream` statement needed. The JSONL file writes to `.lx/stream.jsonl` automatically.

For cross-process streaming, the manifest configures an external backend:

```toml
[stream]
command = "valkey-stream-mcp"
```

When an external backend is configured, the interpreter writes to both the in-memory stream and the external backend. The JSONL file is still written regardless.

The event stream is accessed via `use std/events`. The name `events` is intentional — `stream` is reserved for the existing `std/stream` module (the lazy pull-based data stream with map/filter/collect). Making it an explicit import means lx library code can depend on it — immune cell agents, monitoring tools, debug utilities can all be written as `.lx` modules that `use std/events` and operate on the stream programmatically.

### Events module methods

- `events.xadd {kind: "...", ...}` — append an entry, returns the generated ID string
- `events.xrange start end` — read entries between two IDs. `"-"` means beginning, `"+"` means end. Returns a list of entry records.
- `events.xrange start end {count: N}` — read at most N entries in the range
- `events.xread last_id` — block until a new entry appears after `last_id`. `"$"` means "from now." Returns the entry.
- `events.xread last_id {timeout_ms: N}` — block with timeout. Returns `None` on timeout.
- `events.xlen` — return entry count
- `events.xtrim {maxlen: N}` — remove entries beyond the max length (oldest first)

### Stream entry format

Each entry is a record with:

- `id` — auto-generated, format `{unix_ms}-{seq}`. The millisecond timestamp is wall clock time. The sequence number is a per-millisecond monotonic counter starting at 0. Example: `"1679083200123-0"`, `"1679083200123-1"` for two events in the same millisecond. This matches Valkey stream ID format exactly.
- `kind` — string identifying the event type
- `agent` — string identifying which agent produced it (`"main"` for the top-level program)
- `ts` — unix millisecond timestamp (same as the ms portion of the ID)
- `span` — source location record `{line: N, col: N}` (when available)
- Plus kind-specific fields

### Event kinds

The `kind` field is a stream key for categorizing and filtering entries. These are not lx syntax — they're plain strings.

| Kind | Additional fields | When it's written |
|---|---|---|
| `program/start` | `source_path` | Program begins executing |
| `program/done` | `result`, `duration_ms` | Program finishes |
| `runtime/emit` | `value` | lx code evaluates an `emit` expression |
| `runtime/log` | `level` (info/warn/err/debug), `msg` | lx code calls a `log.*` builtin |
| `runtime/error` | `error`, `span` | A runtime error occurs |
| `tool/call` | `call_id`, `tool`, `method`, `args` | Before a tool module method dispatches to MCP |
| `tool/result` | `call_id`, `tool`, `method`, `result` | After a tool module method returns |
| `tool/error` | `call_id`, `tool`, `method`, `error` | When a tool module method fails |
| `tool/log` | `tool`, `level`, `msg` | Tool sends an MCP `notifications/message` |
| `agent/spawn` | `agent_name`, `class` | lx code evaluates a `spawn` expression |
| `agent/kill` | `agent_name` | An agent is killed |
| `agent/tell` | `from`, `to`, `msg` | An agent sends a fire-and-forget message |
| `agent/ask` | `ask_id`, `from`, `to`, `msg` | An agent sends a request and suspends its task for reply |
| `agent/reply` | `ask_id`, `from`, `to`, `msg` | An agent replies to an ask |
| `channel/subscribe` | `channel`, `agent_name` | An agent subscribes to a channel |
| `yield/out` | `prompt_id`, `value` | Top-level program evaluates a `yield` expression |
| `yield/in` | `prompt_id`, `response` | A yield receives a response via the YieldBackend |

### Auto-logging: where it happens

The interception point is in the interpreter's method dispatch for tool modules. When the interpreter evaluates a field access + call on a module that was created by `use tool`, the dispatch path is:

1. `eval` resolves the field access (`Browser.click`) — recognizes `Browser` as a tool module
2. Before calling the MCP client, it xadds `tool/call` to the stream
3. Calls the MCP client
4. After the call returns, it xadds `tool/result` or `tool/error` to the stream
5. Returns the result to the lx program

This is a single code path in the interpreter that handles ALL tool module calls. Adding a new tool doesn't require any auto-logging code — it's automatic because the dispatch goes through the same path.

The `call_id` is a per-agent monotonic counter incremented per tool call. Each agent maintains its own counter independently — spawned agents initialize theirs on spawn, the top-level program (`"main"`) initializes its counter to 1 at program start. Since multiple agents can each have a `call_id` of 1, the correlation key for pairing `tool/call` with `tool/result` in the stream is `(agent, call_id)` — both fields are present on every tool event entry.

### Auto-logging: what gets logged beyond tool calls

- `emit` expression — the interpreter's eval case for `Expr::Emit` xadds to the stream after calling `ctx.emit`
- `log.*` builtins — each log builtin xadds to the stream after calling `ctx.log`
- `yield` expression — the interpreter's eval case for `Expr::Yield` xadds `yield/out` before yielding and `yield/in` after receiving the response (top-level programs only)
- `tell`/`ask` — the interpreter xadds `agent/tell`, `agent/ask`, `agent/reply` on each message
- `agent.*` builtins — agent.spawn/kill xadd to the stream
- Program start/done — the interpreter xadds `program/start` at the beginning and `program/done` at the end

These are a fixed set of interception points in the interpreter — one per language keyword/builtin that produces side effects. They're not extensible because the set of interpreter operations that produce events is fixed.

### Concurrency and ordering

When multiple agents run in parallel (via `par` or spawned agents), each agent xadds to the same stream. The stream's ID generation (timestamp + sequence counter) provides a total order. The sequence counter is atomic, so concurrent xadds in the same millisecond get distinct sequential IDs. Entries from different agents interleave chronologically.

For causal ordering within a single agent, entries are naturally ordered because a single agent evaluates sequentially — it can't xadd two entries simultaneously.

### Trimming

No auto-trim. The JSONL file and in-memory stream grow for the duration of the program. The lx program can call `events.xtrim {maxlen: 10000}` explicitly if needed. Most lx programs are finite workflows — the stream is bounded by the program's lifetime. If trimming is needed for long-running programs, the lx program does it explicitly.

### Direct use from lx programs

The lx program can write to and read from the stream directly:

```lx
use std/events

events.xadd {kind: "checkpoint", state: some_data}

entries = events.xrange "-" "+"
errors = entries | filter (e) { e.kind == "error" }

-- watch for new events in real time
last = "$"
loop {
  entry = events.xread last {timeout_ms: 5000}
  entry ? {
    emit entry
    last = entry.id
  } : {}
}
```

### Debug

Debug is reading the stream. The stream contains: what happened (kind), when (ts), where in source (span), which agent (agent), what arguments (args), what result (result), what went wrong (error). A debug view is a filtered xrange.

### External streaming backend

For cross-process access, the dev writes an MCP server that handles `xadd`, `xrange`, `xread`, `xlen`, `xtrim` and configures it in the manifest:

```toml
[stream]
command = "valkey-stream-mcp"
```

The interpreter writes to both the in-memory stream (for in-process access) and the external backend (for cross-process access). The JSONL file is always written regardless. The external backend is an additional distribution layer, not a replacement for the in-memory stream or the JSONL persistence.

**Use cases for external streaming:** Admin panels that read the stream to build a real-time debug view of a running program. Immune cell agents — separate lx programs that monitor the event stream of another program, watching for anomalies and reacting (e.g. pausing a misbehaving agent via the control channel when they spot something in the stream). Post-hoc analysis tools that read stream history to reconstruct what happened. The external backend makes the stream available to anything outside the running process.

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
| pause | `{"cmd": "pause"}` | `{"ok": true}` — pauses all agents |
| pause (targeted) | `{"cmd": "pause", "agent": "Researcher"}` | `{"ok": true}` — pauses one agent |
| resume | `{"cmd": "resume"}` | `{"ok": true}` — resumes all paused agents |
| resume (targeted) | `{"cmd": "resume", "agent": "Researcher"}` | `{"ok": true}` — resumes one agent |
| cancel | `{"cmd": "cancel"}` | `{"ok": true}` (then program exits) |
| inspect | `{"cmd": "inspect"}` | `{"ok": true, "state": {"call_stack": [...], "env": {...}, "stream_position": "1679083200123-5"}}` |
| inject | `{"cmd": "inject", "value": ...}` | `{"ok": true}` (value delivered to pending yield) |

All commands are single-line JSON. Responses are single-line JSON. The wire format is the same regardless of transport (stdin, WebSocket, TCP). The transport just carries the bytes.

### How the control channel runs

The control channel is a separate async task, independent of the interpreter's eval loop. The interpreter does not poll or check it. The control channel task acts on the interpreter from outside:

- **Cancel:** The control channel task sends `$/cancelRequest` to the tool process, waits a short grace period, then SIGKILLs the child process. The blocked call returns `Err "cancelled"`. The interpreter sees the error and stops the program. Cancel works mid-call — the interpreter doesn't need to be between steps.
- **Pause:** The control channel task sets a pause flag — either globally (all agents) or on a specific agent's state. The interpreter checks the relevant pause flag before the next eval step and waits until it's cleared. If the interpreter is mid-tool-call, pause takes effect when the call returns. Targeted pause is the key mechanism for immune cell agents: an agent watching the event stream sees something wrong, finds the offending agent's name and call_id in the stream, and sends a targeted pause through the control channel.
- **Resume:** The control channel task clears the pause flag. The interpreter continues.
- **Inspect:** The control channel task reads the interpreter's state directly (env, call stack, stream position) and responds. This can happen while the interpreter is paused or between steps.
- **Inject:** The control channel task delivers a value to a pending yield's response channel.

The interpreter only sees the side effects — a failed MCP call, a pause flag set, a yield response delivered. It doesn't know about the control channel mechanism. The control channel task and the interpreter share state through atomic flags and channels, not through polling.

### Why it's not extensible

The command set is fixed because it maps to interpreter operations: pause the eval loop, snapshot the environment, inject into yield. These are intrinsic to the interpreter. A dev building a new host environment picks a transport flag — they don't extend the command set.

### Relationship to the event stream

- Event stream: interpreter → outside world (what happened)
- Control channel: outside world → interpreter (what to do)

A typical debug workflow: launch with `--control ws://localhost:8080`, connect a WebSocket client, send `{"cmd": "pause"}`, read `.lx/stream.jsonl` to see what happened so far, send `{"cmd": "inspect"}` to see current state, send `{"cmd": "resume"}` to continue.

---

## Relationship to Existing Codebase

### RuntimeCtx backend traits

The current `RuntimeCtx` has `EmitBackend`, `LogBackend`, `LlmBackend`, `HttpBackend`, `YieldBackend` as separate `Arc<dyn Trait>` fields.

In the target architecture, all of these go away. The event stream and control channel replace them:

- `EmitBackend` → `emit` writes a stream entry: `{kind: "runtime/emit", value: ...}`
- `LogBackend` → `log.*` writes a stream entry: `{kind: "runtime/log", level: "...", msg: "..."}`
- `LlmBackend` → LLM is an external tool module: `use tool "claude-mcp" as llm`
- `HttpBackend` → HTTP is an external tool module: `use tool "http-mcp" as http`
- `YieldBackend` → stays. It's the host-provided trait that handles yield delivery — the CLI implementation uses the control channel, the test harness auto-responds, other hosts implement it however they want. Yield also logs to the stream (`yield/out`, `yield/in`). Spawned agents do not use yield — they communicate via `tell`/`ask`.

RuntimeCtx holds the in-memory event stream, the control channel, and the yield backend. Everything else is either a stream entry or a tool module.

### Existing `MCP` keyword

The `MCP` keyword (`MCP Foo = {command: "...", args: [...]}`) desugars to a class with `mcp.connect`/`mcp.call`. `use tool` provides the same capability through the module system. Existing code using `MCP` continues to work — both mechanisms use the same underlying MCP client.

### Existing `CLI` keyword

The `CLI` keyword desugars to a class that shells out via `bash`. `use tool` provides a structured alternative where the tool author writes an MCP server instead of a raw CLI. Existing code using `CLI` continues to work.

### Existing `stream` module

The existing `std/stream` module (`stream.from`, `stream.map`, `stream.filter`, `stream.collect`, etc.) is a lazy pull-based data processing stream. It is unrelated to the event stream. They coexist under different names:

- `use std/stream` — lazy data stream for transforming lists/iterators (`stream.map`, `stream.filter`, etc.)
- `use std/events` — runtime event stream for execution history (`events.xadd`, `events.xrange`, etc.)

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
| Tool module dispatch | Built into interpreter, forwards module method calls to MCP `tools/call`, tool decides what it accepts |
| Event stream auto-logging | Built into interpreter, interception at tool module dispatch + emit/log/yield/agent |
| In-memory event stream | Built into interpreter, always on, JSONL persistence to `.lx/stream.jsonl` automatic |
| Control transports (stdin, WS, TCP) | Built into interpreter, selected via `--control` flag |
