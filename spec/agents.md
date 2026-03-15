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

## Streaming (`~>>?`)

`~>>?` sends a message and returns a lazy sequence of partial results. The agent yields incremental chunks; the caller iterates as they arrive.

```
stream = agent ~>>? {task: "review" files: large_list}
stream | each (chunk) display chunk
```

Composes with pipes and error handling:

```
agent ~>>? {task: "analyze"} | filter (.important) | each (r) log.info r.summary
```

The receiving agent uses `yield` to emit chunks. When the agent returns normally, the stream ends. If the agent errors, the stream produces `Err` on the next read.

### Streaming in par/sel

```
sel {
  agent ~>>? request | take 1 -> it
  timeout 30                   -> Err "timeout"
}
```

## Message Contracts (Protocol)

Protocols define the expected shape of agent messages. They validate record structure at the boundary — missing fields, wrong types, and non-record values are caught immediately with clear diagnostics.

```
Protocol ReviewRequest = {task: Str  path: Str  depth: Int = 3}
msg = ReviewRequest {task: "audit" path: "src/"}
-- msg == {task: "audit" path: "src/" depth: 3}

reviewer ~>? ReviewRequest {task: "audit" path: "src/"} ^
-- validates, then sends {task: "audit" path: "src/"} to reviewer
```

See [agents-protocol.md](agents-protocol.md) for full details: structural subtyping, `Any` type, error messages, exports.

## Multi-Turn Dialogue

Single `~>?` calls are request-response. For sustained multi-turn conversation between agents (debugging together, iterative refinement, negotiation), use `agent.dialogue` sessions. See [agents-dialogue.md](agents-dialogue.md) for full spec.

```
session = agent.dialogue worker {role: "reviewer"} ^
r1 = agent.dialogue_turn session "look at auth module" ^
r2 = agent.dialogue_turn session "check payments too" ^
agent.dialogue_end session
```

## Structured Handoff

When one agent finishes a phase and another takes over, `agent.handoff` transfers structured context — not just results, but what was tried, what assumptions were made, and recommendations. See [agents-handoff.md](agents-handoff.md) for full spec.

## Message Interceptors

`agent.intercept` wraps an agent with middleware for cross-cutting concerns: tracing, rate-limiting, context injection, policy enforcement. See [agents-intercept.md](agents-intercept.md) for full spec.

```
traced = agent.intercept worker (msg next) {
  log.debug "sending: {msg | to_str}"
  next msg
}
traced ~>? {task: "review"} ^
```

## Dynamic Plan Revision

`std/plan` executes plan-as-data with runtime revision — steps can be added, removed, or replaced based on intermediate results. See [agents-plans.md](agents-plans.md) for full spec.

## Implementation Status

- `~>` (send) and `~>?` (ask) — implemented as infix operators
- `~>>?` (stream) — planned (depends on async runtime)
- `Protocol` — implemented as keyword with runtime validation
- `yield` — implemented as keyword with callback-based orchestrator protocol
- `emit` — agent-to-human output primitive, callback-based, fire-and-forget (planned)
- `MCP` declarations — implemented as keyword with typed tool contracts and validation
- `with` / field update — implemented with scoped bindings and mutable record field assignment
- `checkpoint`/`rollback` — planned
- Capability attenuation on `agent.spawn` — planned
- Multi-turn dialogue (`agent.dialogue`) — planned
- Structured handoff (`agent.handoff`) — planned
- Message interceptors (`agent.intercept`) — planned
- Dynamic plan revision (`std/plan`) — planned
- Agent introspection (`std/introspect`) — planned
- Shared knowledge cache (`std/knowledge`) — planned
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
- Protocol details: [agents-protocol.md](agents-protocol.md) (structural subtyping, Any, errors, exports)
- Advanced features: [agents-advanced.md](agents-advanced.md) (emit, yield, MCP declarations, with/field update)
- Multi-turn dialogue: [agents-dialogue.md](agents-dialogue.md)
- Structured handoff: [agents-handoff.md](agents-handoff.md)
- Message interceptors: [agents-intercept.md](agents-intercept.md)
- Dynamic plan revision: [agents-plans.md](agents-plans.md)
- Agent introspection: [stdlib-introspect.md](stdlib-introspect.md)
- Shared knowledge cache: [stdlib-knowledge.md](stdlib-knowledge.md)
- Test suite: [../tests/14_agents.lx](../tests/14_agents.lx)
