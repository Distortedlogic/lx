# Capability-Based Routing

Declarative routing: send a message to the best available agent for a capability, with load-awareness and fallback.

## Problem

`agent.capabilities` and `agent.advertise` exist for runtime capability discovery. `std/agents/router` uses AI to classify messages. `agent.dispatch` routes by pattern matching on message shape. But none of these answer:

**"Send this to whatever agent handles `Trait Reviewer` with the lowest current load."**

Today every flow manually wires agent references: `reviewer ~>? msg`. If you want capability-based routing, you build it yourself — query all agents' capabilities, filter by trait/protocol, check status, pick one, send. This is 10+ lines of boilerplate repeated in every flow that needs dynamic routing.

## Design

### `agent.route` — Single Dispatch

```lx
use std/agent

result = agent.route msg {
  trait: "Reviewer"
} ^
```

Finds an agent implementing `Trait Reviewer` from the current process's known agents, sends `msg` via `~>?`, returns the result. If multiple agents match, selects by load (least busy).

### `agent.route` Options

```lx
result = agent.route msg {
  trait: "Reviewer"             -- match by Trait name
  protocol: "ReviewRequest"     -- OR match by Protocol name
  domain: "security"            -- OR match by advertised domain
  prefer: "least_busy"          -- selection strategy (default)
  fallback: backup_agent        -- explicit fallback if no match
  timeout: 10000                -- per-request timeout
  exclude: [self_agent]         -- agents to skip
} ^
```

Selection strategies:
- `"least_busy"` (default) — agent with fewest in-flight `~>?` requests
- `"round_robin"` — rotate through matching agents
- `"random"` — random selection from matches
- `"highest_score"` — use `std/trace` agent scores if available
- Custom function: `(agents: [Agent]) -> Agent`

### `agent.route_multi` — Fan-Out to All Matching

```lx
results = agent.route_multi msg {
  trait: "Reviewer"
  reconcile: {strategy: "vote" quorum: "majority"}
} ^
```

Sends to ALL matching agents, collects results, applies `agent.reconcile` with the specified strategy. Returns the reconciled result.

### `agent.register` — Make Agent Discoverable

```lx
agent.register reviewer {
  traits: ["Reviewer"]
  protocols: ["ReviewRequest"]
  domains: ["code" "security"]
  max_concurrent: 5
}
```

Registers an agent for capability-based discovery within the current process. Separate from `agent.advertise` (which is metadata only) — `register` adds the agent to the routing table.

### `agent.registered` — Query Routing Table

```lx
agents = agent.registered {trait: "Reviewer"} ^    -- all registered Reviewers
count = agent.registered {domain: "security"} ^ | len
all = agent.registered {} ^                         -- everything
```

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `agent.route` | `(msg: Record opts: Record) -> Result Any Str` | Route to best matching agent |
| `agent.route_multi` | `(msg: Record opts: Record) -> Result Any Str` | Fan-out to all matching agents |
| `agent.register` | `(agent: Agent opts: Record) -> Result () Str` | Register agent for routing |
| `agent.unregister` | `(agent: Agent) -> Result () Str` | Remove from routing table |
| `agent.registered` | `(filter: Record) -> Result [Agent] Str` | Query routing table |

### Integration

- `agent.capabilities` / `agent.advertise` — route reads advertised capabilities; register stores structured routing metadata.
- `agent.implements` / `Trait` — trait matching uses existing `agent.implements` check.
- `agent.dispatch` — dispatch is pattern-based on message shape; route is capability-based on agent traits. Complementary. dispatch can use route as a target: `{match: {domain: "security"} to: (msg) agent.route msg {trait: "SecurityReviewer"} ^}`.
- `agent.reconcile` — `route_multi` uses reconcile internally for multi-agent results.
- `std/registry` (planned) — registry extends this to cross-process. When registry ships, `agent.register` gains an `{scope: "local"}` vs `{scope: "registry"}` option.

## Implementation

Agent extension (sub-module of `std/agent`). No parser changes. In-process routing table as `Arc<RwLock<HashMap<String, AgentEntry>>>`. Load tracking via atomic counters incremented on `~>?` send, decremented on response.

Approximately 150 lines of Rust.

## Priority

Tier 2. Every multi-agent flow that does dynamic agent selection reinvents this. No parser changes. Small implementation surface. Natural stepping stone toward `std/registry` (Tier 3).
