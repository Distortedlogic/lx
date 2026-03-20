# Cross-Process Agent Discovery

`std/registry` provides a discovery service for agents running across separate processes. Agents register their capabilities, and orchestrators query the registry to find agents by capability, trait, or domain — without knowing process addresses upfront.

## Problem

`agent.advertise` and `agent.capabilities` work in-process only. Agent A can discover Agent B's capabilities only if they share a runtime. For multi-process agent systems:

- An orchestrator can't say "find me an agent that handles code review" across machines
- Agent startup order matters — if the reviewer starts after the orchestrator, it's invisible
- No health checking — a registered agent might be dead
- No load distribution — all requests go to the first match

Real agent ecosystems span multiple processes. A coding agent spawns on one machine, a review agent on another, a deploy agent on a third. They need to find each other.

## `std/registry`

### Registry Lifecycle

```
use std/registry

reg = registry.start {port: 9100} ^
-- registry is now listening

registry.stop reg
```

The registry is a standalone process. One registry per cluster. Agents connect to it.

### Agent Registration

```
use std/registry

conn = registry.connect {host: "localhost"  port: 9100} ^

registry.register conn {
  name: "code-reviewer"
  traits: ["Reviewer" "SecurityAuditor"]
  protocols: [ReviewRequest AuditRequest]
  domains: ["rust" "python"]
  capacity: 5
  metadata: {version: "1.2.0"  model: "claude-4"}
}

-- on shutdown
registry.deregister conn
```

Registration includes a heartbeat. If the agent stops heartbeating, the registry marks it unhealthy after a configurable timeout.

### Discovery

```
agents = registry.find conn {trait: "Reviewer"} ^
agents = registry.find conn {trait: "ReviewRequest"} ^
agents = registry.find conn {domain: "rust"} ^
agents = registry.find conn {trait: "Reviewer"  domain: "rust"} ^
```

`find` returns a list of `AgentRef` records:

```
AgentRef = {
  name: Str
  address: Str
  traits: [Str]
  protocols: [Str]
  domains: [Str]
  capacity: Int
  load: Int
  healthy: Bool
  metadata: Record
}
```

### Communication via AgentRef

`AgentRef` values work with `~>` and `~>?`. The runtime resolves the `address` field to a network connection:

```
reviewer = registry.find conn {trait: "Reviewer"} ^ | first ^
result = reviewer ~>? {task: "review"  diff: changes} ^
```

### Health & Load

```
registry.health conn "code-reviewer" ^    -- {healthy: Bool  last_seen: Str  uptime_ms: Int}
registry.load conn "code-reviewer" ^      -- {current: Int  capacity: Int  pct: Float}
```

The registry tracks load by counting active `~>?` requests per agent.

### Watching

```
registry.watch conn {trait: "Reviewer"} (event) {
  event ? {
    {kind: "join" agent: a}   -> log.info "{a.name} joined"
    {kind: "leave" agent: a}  -> log.info "{a.name} left"
    {kind: "health" agent: a} -> log.warn "{a.name} unhealthy"
  }
}
```

`watch` receives events when matching agents join, leave, or change health status.

### Load-Balanced Dispatch

```
reviewer = registry.find_one conn {trait: "Reviewer"  strategy: :least_loaded} ^
```

Strategies:
- `:first` — first healthy match (default)
- `:least_loaded` — lowest `load / capacity` ratio
- `:random` — random healthy match
- `:round_robin` — rotate across matches

## Wire Protocol

### Registry Wire Protocol

The registry communicates over TCP using JSON-lines (same pattern as agent subprocess protocol):

```json
{"type": "register", "name": "code-reviewer", "traits": [...], ...}
{"type": "ack", "id": "reg-001"}
{"type": "heartbeat", "name": "code-reviewer"}
{"type": "find", "id": "q-001", "query": {"trait": "Reviewer"}}
{"type": "find_result", "id": "q-001", "agents": [...]}
{"type": "watch", "id": "w-001", "query": {"trait": "Reviewer"}}
{"type": "event", "id": "w-001", "kind": "join", "agent": {...}}
```

### Agent-to-Agent via Registry

When `~>?` is called on an `AgentRef`, the runtime:
1. Opens a TCP connection to `address`
2. Sends the message as JSON-line
3. Reads the response as JSON-line
4. Returns as `Result`

This reuses the existing subprocess JSON-line protocol over TCP instead of stdin/stdout.

## Patterns

### Auto-scaling code review

```
use std/registry

conn = registry.connect {host: "localhost"  port: 9100} ^
files = git.diff {range: "main..HEAD"} ^ | map (.path)

files | pmap_n 3 (file) {
  reviewer = registry.find_one conn {trait: "Reviewer"  strategy: :least_loaded} ^
  reviewer ~>? {task: "review"  file: file} ^
}
```

### Service mesh for agents

```
conn = registry.connect {host: "localhost"  port: 9100} ^

registry.register conn {
  name: "orchestrator"
  traits: ["Orchestrator"]
  protocols: [TaskRequest]
}

registry.watch conn {trait: "Worker"} (event) {
  event.kind == "join" ? {
    worker = event.agent
    emit "New worker: {worker.name} ({worker.domains | join ", "})"
  }
}
```

## Implementation

### Registry server

The registry is a separate binary (`lx registry`) or embedded in `lx agent --registry`. It maintains an in-memory map of registered agents with their metadata, heartbeat timestamps, and load counters.

### `std/registry` module

New stdlib module. Functions use `RuntimeCtx`'s networking (TCP via `std::net` or tokio).

### AgentRef as agent handle

The interpreter's agent communication system (`~>`, `~>?`) checks if the target is an `AgentRef` (has `address` field). If so, it routes over TCP instead of subprocess stdin/stdout.

### Dependencies

- `std::net::TcpStream` / `tokio::net::TcpStream` (TCP connections)
- `std/json` (wire protocol serialization)
- Existing agent communication infrastructure

## Cross-References

- In-process capabilities: `agent.capabilities` / `agent.advertise` in [stdlib-agents.md](stdlib-agents.md)
- Traits: [agents-trait.md](agents-trait.md) — trait-based filtering works with registry queries
- Load balancing: relates to `std/pool` (in-process) — registry is cross-process equivalent
- Streaming: [agents-streaming.md](agents-streaming.md) — `~>>?` should work over registry connections
- Health: `std/circuit` — circuit breakers compose with registry health checks
