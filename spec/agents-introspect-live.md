# Live Agent Introspection

Runtime observation of all active agents' states, message queues, and dialogues — a structured "what is everyone doing right now?"

## Problem

`std/introspect` provides self-inspection (elapsed time, turn count, action log, stuck detection) for a single agent. `std/trace` records spans after the fact. `std/diag` visualizes static program structure.

None answers: **"What are all my agents doing right now?"**

In a multi-agent flow with 5+ spawned agents, debugging requires manually calling `agent.status` on each, then mentally correlating the results. There's no single call that returns a structured snapshot of the entire agent system: who's running, who's blocked on `~>?`, what messages are in flight, which dialogues are active, what the current load looks like.

## Design

### Extensions to `std/introspect`

```lx
use std/introspect

snapshot = introspect.system () ^
```

Returns:

```lx
{
  agents: [
    {
      name: "reviewer-1"
      status: "busy"
      uptime_ms: 12500
      in_flight: 2
      traits: ["Reviewer"]
      current_task: "reviewing auth.rs"
      dialogues: [{id: "d1" turns: 4 role: "reviewer"}]
    }
    {
      name: "linter-1"
      status: "idle"
      uptime_ms: 12500
      in_flight: 0
      traits: ["Linter"]
      current_task: None
      dialogues: []
    }
  ]
  messages_in_flight: 2
  topics: [{name: "updates" subscribers: 3 published: 15}]
  pools: [{name: "workers" size: 4 busy: 2 queued: 0}]
  supervisors: [{id: "sup1" strategy: "one_for_one" restarts: 1 children: 3}]
}
```

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `introspect.system` | `() -> Result Record Str` | Full system snapshot |
| `introspect.agents` | `() -> Result [Record] Str` | Just agent list with status |
| `introspect.agent` | `(agent: Agent) -> Result Record Str` | Detailed single-agent info |
| `introspect.messages` | `() -> Result [Record] Str` | All in-flight messages |
| `introspect.bottleneck` | `() -> Result Record Str` | Agent with most queued work |
| `introspect.watch` | `(handler: Fn interval_ms: Int) -> Result () Str` | Periodic snapshot callback |

### `introspect.agent` — Deep Single-Agent View

```lx
info = introspect.agent reviewer ^
```

Returns everything `introspect.system` returns per-agent, plus:

```lx
{
  name: "reviewer-1"
  status: "busy"
  uptime_ms: 12500
  in_flight: 2
  pending_messages: [{from: "orchestrator" sent_at: 1710600000 payload_preview: "{task: ...}"}]
  completed: 15
  errors: 1
  avg_response_ms: 340
  traits: ["Reviewer"]
  protocols: ["ReviewRequest"]
  dialogues: [{id: "d1" turns: 4 role: "reviewer" last_turn_ms: 2300}]
  intercepts: 1
  memory_estimate_bytes: 45000
}
```

### `introspect.bottleneck`

Returns the agent with the most queued/in-flight work:

```lx
b = introspect.bottleneck () ^
b ? {
  Some {agent: a in_flight: n} -> log.warn "{a.name} has {n} in-flight"
  None -> log.info "no bottleneck"
}
```

### `introspect.watch` — Periodic Monitoring

```lx
introspect.watch (snapshot) {
  overloaded = snapshot.agents | filter (a) a.in_flight > 10
  overloaded | each (a) log.warn "overloaded: {a.name}"
} 5000
```

Calls handler every N ms with a fresh system snapshot. Returns immediately. Stops when the enclosing scope exits (compatible with `with ... as` cleanup).

### Integration

- `std/introspect` existing — `introspect.self` remains for self-inspection. New functions add system-wide view.
- `std/trace` — trace records historical spans; live introspection shows current state. Complementary.
- `agent.status` — returns simple status string. `introspect.agent` returns rich structured record.
- `std/agents/monitor` — monitor uses `scan_actions` for safety. Live introspection provides the raw data monitors could use.
- `std/diag` — diag shows static structure; introspection shows runtime state. Could compose: `diag.extract_file "flow.lx" ^ | overlay (introspect.system () ^)` for annotated diagrams (future).

## Implementation

Extensions to existing `std/introspect` module in `stdlib/introspect.rs`. System-wide data gathered from agent registry (in-process `HashMap` of spawned agents), pool tracking, topic subscriber lists, supervisor state. All data already exists in the runtime — this module provides structured access.

Approximately 150 lines of additional Rust.

No parser changes. No new keywords.

## Priority

Tier 2. Debugging multi-agent flows without system-wide visibility is painful. All prerequisite data already exists in the runtime. Small surface area. Immediately useful for flow development and monitoring.
