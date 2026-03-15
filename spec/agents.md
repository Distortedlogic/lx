# Agent Primitives

lx is an agentic workflow language. Agent communication has its own syntax — the parser recognizes `~>` (send) and `~>?` (ask) as distinct operations, just as `$` identifies shell commands.

## Why Language-Level Syntax

Shell commands get `$` — their own lexer mode, AST node, and runtime semantics. Agent communication is THE ENTIRE PURPOSE of lx. It deserves the same treatment. With language-level syntax:

- The runtime can automatically add tracing, timeouts, retries
- Error messages say "agent communication failed" not "function returned Err"
- `Protocol` validates message shapes at send boundaries
- `~>` and `~>?` compose naturally with `^`, `|`, `par`/`sel`

## Agent Values

An agent is a record with a `handler` field. The handler is a function that receives messages and returns responses. Agents created by `agent.spawn` (future) will also be records with handlers backed by subprocesses.

```
echo = {handler: (msg) msg}
adder = {handler: (x) x + 10}
```

Agent factories (closures over config):

```
make_multiplier = (factor) {handler: (x) x * factor}
triple = make_multiplier 3
```

## Communication Syntax

### `~>` — Send (Fire-and-Forget)

Send a message, don't wait for response. Returns `()`.

```
logger ~> {action: "log" data: results}
```

### `~>?` — Ask (Request-Response)

Send a message, wait for the agent's response. Returns the handler's result.

```
result = analyzer ~>? {task: "review" path: "src/"}
```

### Composition with `^`, `|`, `??`

Agent operators compose with existing primitives:

```
-- ask + propagate + pipe
analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)

-- ask + coalesce
fallback_agent ~>? request ?? default_response

-- ask in par block
(security perf) = par {
  sec_agent ~>? {task: "audit"} ^
  perf_agent ~>? {task: "profile"} ^
}
```

### Precedence

`~>` and `~>?` bind at the same level as `++` (tighter than pipe, looser than arithmetic):

```
agent ~>? msg ^ | process
-- parses as: ((agent ~>? msg) ^) | process

data | (d) agent ~>? {data: d} ^
-- lambda captures the ask in its body
```

### Multiline Continuation

`~>` and `~>?` support leading-operator continuation across lines:

```
result = analyzer
  ~>? {task: "review" path: "src/"}
  | (.findings)
  | filter (.critical)
```

## Multi-Agent Orchestration

Compose agents with existing concurrency primitives:

```
(security perf docs) = par {
  sec_agent ~>? {task: "audit" path: "src/"} ^
  perf_agent ~>? {task: "profile" path: "src/"} ^
  docs_agent ~>? {task: "check-coverage" path: "src/"} ^
}
```

Fan-out to dynamic agent pool:

```
tasks = files | pmap (f) {
  a = agent.spawn {name: "reviewer" prompt: "Review {f}"} ^
  a ~>? {file: f action: "review"} ^
}
```

Pipeline agents (output of one feeds the next):

```
raw = fetcher ~>? {url: api_url} ^
parsed = parser ~>? {data: raw.body format: "json"} ^
summary = summarizer ~>? {data: parsed findings: 10} ^
```

## Channels and `sel` (Future)

Channel receive syntax for `sel` blocks is planned but not yet implemented. For now, channel operations use library functions:

```
sel {
  agent.ch_recv ch1 -> handle_response "agent1" it
  agent.ch_recv ch2 -> handle_response "agent2" it
  timeout 30        -> Err "no response"
}
```

## Tool Invocation (MCP)

MCP tools are invoked through `std/mcp`. Supports both stdio (local subprocess) and HTTP streaming (remote server) transports.

```
use std/mcp

-- Local MCP server via stdio
local = mcp.connect {command: "npx" args: ["-y" "mcp-server"]} ^

-- Remote MCP server via HTTP
remote = mcp.connect "https://api.example.com/mcp" ^

tools = mcp.list_tools remote ^
result = mcp.call remote "read_file" {path: "src/main.rs"} ^
mcp.close remote
```

## Context and Memory

Agents persist state across sessions via `std/ctx`. Context is a key-value store backed by files.

```
use std/ctx

memory = ctx.load "memory.json" ^
last_run = ctx.get "last_run" memory ?? "never"
memory = ctx.set "last_run" (time.now () | to_str) memory
ctx.save "memory.json" memory ^
```

## Workflow Patterns

### Retry with Escalation

```
analyze = (path) {
  result = retry 3 () analyzer ~>? {path}
  result ? {
    Ok r  -> r
    Err e -> senior ~>? {path error: e.msg} ^
  }
}
```

### Checkpoint and Resume

```
run_pipeline = (state_path) {
  state = ctx.load state_path ?? %{}
  step = ctx.get "step" state ?? "start"

  step ? {
    "start" -> {
      data = fetch_data ()
      ctx.save state_path (ctx.set "step" "process" (ctx.set "data" data %{})) ^
      run_pipeline state_path
    }
    "process" -> {
      data = ctx.get "data" state ^
      result = process data
      ctx.save state_path (ctx.set "step" "done" (ctx.set "result" result state)) ^
    }
    "done" -> ctx.get "result" state ^
  }
}
```

## Message Contracts (Protocol)

Protocols define the expected shape of agent messages. They validate record structure at the boundary — missing fields, wrong types, and non-record values are caught immediately with clear diagnostics.

### Defining Protocols

```
Protocol ReviewRequest = {task: Str  path: Str  depth: Int = 3}
Protocol CalcRequest = {op: Str  value: Int}
```

Fields have a name and a type. Optional fields have defaults (filled in when missing). Type checking uses runtime type names: `Str`, `Int`, `Float`, `Bool`, `List`, `Record`, `Map`, `Tuple`, `Any`.

### Using Protocols

Apply a Protocol to a record to validate it. Returns the validated record on success (with defaults filled in). Runtime error on failure.

```
msg = ReviewRequest {task: "audit" path: "src/"}
-- msg == {task: "audit" path: "src/" depth: 3}

CalcRequest {op: "double" value: 5}
-- returns {op: "double" value: 5}

CalcRequest {op: "double" value: "five"}
-- RUNTIME ERROR: Protocol CalcRequest: field 'value' expected Int, got Str

CalcRequest {op: "double"}
-- RUNTIME ERROR: Protocol CalcRequest: missing required field 'value'
```

### With Agent Communication

Protocol validation happens before the message reaches the agent:

```
Protocol ReviewRequest = {task: Str  path: Str}
reviewer = {handler: (msg) analyze msg.path msg.task}

reviewer ~>? ReviewRequest {task: "audit" path: "src/"} ^
-- validates, then sends {task: "audit" path: "src/"} to reviewer
```

### Structural Subtyping

Extra fields are allowed — Protocols check that required fields exist with correct types, but don't reject additional fields:

```
Protocol Minimal = {id: Int}
Minimal {id: 1 name: "extra" tags: [1 2]}
-- returns {id: 1 name: "extra" tags: [1 2]}
```

### `Any` Type

Use `Any` for fields that accept any value:

```
Protocol Flexible = {key: Str  value: Any}
Flexible {key: "count" value: 42}       -- ok
Flexible {key: "name" value: "alice"}    -- ok
Flexible {key: "items" value: [1 2 3]}   -- ok
```

### Exports

Protocols can be exported with `+` and imported via `use`:

```
+Protocol ReviewRequest = {task: Str  path: Str}
```

## Implementation Status

- `~>` (send) and `~>?` (ask) — implemented as infix operators
- `Protocol` — implemented as keyword with runtime validation
- `yield` — implemented as keyword with callback-based orchestrator protocol
- `MCP` declarations — implemented as keyword with typed tool contracts and validation
- `with` / field update — implemented with scoped bindings and mutable record field assignment
- Sequential evaluation (like par/sel); real async is future work
- Agents are records with handler functions; subprocess agents via `std/agent` (`__pid` records)
- `agent.spawn` — implemented in `std/agent` (subprocess spawning, JSON-line protocol)
- `std/mcp` — implemented (MCP over stdio via JSON-RPC 2.0 + HTTP streaming via reqwest)
- Channel receive syntax for `sel` — planned

## Cross-References

- Communication patterns build on: [concurrency.md](concurrency.md) (par/sel/pmap)
- Tool invocation: [shell.md](shell.md) ($), stdlib-agents.md (MCP)
- Module details: [stdlib-agents.md](stdlib-agents.md)
- Error handling in agents: [errors.md](errors.md) (^, ??)
- Design rationale: [design.md](design.md) (agent communication syntax)
- Advanced features: [agents-advanced.md](agents-advanced.md) (yield, MCP declarations, with/field update)
- Test suite: [../tests/14_agents.lx](../tests/14_agents.lx)
