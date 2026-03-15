# Agent Supervision

Erlang-style supervision trees for automatic agent restart on failure. Without this, a crashed subprocess agent returns `AgentErr::Disconnected` and the caller must manually detect and restart — fragile boilerplate that every multi-agent system needs.

## Problem

Agent subprocesses crash. Network drops. OOM kills. In any serious multi-agent system, the question isn't whether agents will fail but how the system recovers. Currently, every caller must write:

```
loop {
  worker = agent.spawn {command: "worker"} ^
  result = worker ~>? task
  result ? {
    Err e -> log.warn "worker died: {e}, restarting..."
    Ok v  -> break v
  }
}
```

This is repeated everywhere, error-prone, and doesn't handle cascading failures.

## `agent.supervise` — Supervision Trees

```
use std/agent

sup = agent.supervise {
  strategy: :one_for_one
  max_restarts: 5
  window: 60
  children: [
    {id: "analyzer"  spawn: () agent.spawn {command: "analyzer"} ^  restart: :permanent}
    {id: "formatter" spawn: () agent.spawn {command: "formatter"} ^ restart: :transient}
    {id: "logger"    spawn: () agent.spawn {command: "logger"} ^    restart: :temporary}
  ]
}
```

`agent.supervise` is a library function in `std/agent`. It returns a `Supervisor` value (opaque, like `Agent`).

### Configuration

| Field | Type | Description |
|-------|------|-------------|
| `strategy` | Symbol | `:one_for_one`, `:one_for_all`, `:rest_for_one` |
| `max_restarts` | Int | Max restarts within `window` before supervisor gives up |
| `window` | Int | Time window in seconds for restart counting |
| `children` | List | Child specifications |

### Strategies

**`:one_for_one`** — Only the crashed child is restarted. Other children are unaffected. Use when children are independent.

**`:one_for_all`** — All children are terminated and restarted when any one crashes. Use when children are interdependent and a partial restart leaves the system in an inconsistent state.

**`:rest_for_one`** — The crashed child and all children started after it are terminated and restarted. Children started before the crashed one are unaffected. Use when children have a startup-order dependency.

### Child Specification

| Field | Type | Description |
|-------|------|-------------|
| `id` | Str | Unique identifier within this supervisor |
| `spawn` | `() -> Agent` | Zero-arg function that creates the agent |
| `restart` | Symbol | `:permanent`, `:transient`, `:temporary` |

**`:permanent`** — Always restarted on crash. For long-lived worker agents.

**`:transient`** — Restarted only if it crashes abnormally (nonzero exit). Normal exit is not restarted. For task agents that are expected to complete.

**`:temporary`** — Never restarted. For one-shot agents. If it crashes, the supervisor logs the failure but does not restart.

### Accessing Children

```
analyzer = agent.child sup "analyzer"
result = analyzer ~>? {task: "review" path: "src/"} ^
```

`agent.child supervisor id` returns the current agent instance for that child ID. If the child was restarted, this returns the new instance. The caller does not need to track restarts.

### Supervisor Lifecycle

```
agent.supervise_stop sup
```

Terminates all children and the supervisor. Children are terminated in reverse start order.

### Max Restart Intensity

If `max_restarts` is exceeded within `window` seconds, the supervisor itself fails with `Err {type: "supervisor_exhausted" id: child_id restarts: count}`. The caller must handle this — usually by logging and propagating the error.

### Nested Supervisors

Supervisors are agents. A supervisor can be a child of another supervisor, forming a supervision tree:

```
top = agent.supervise {
  strategy: :one_for_one
  children: [
    {id: "worker_pool" spawn: () agent.supervise {
      strategy: :one_for_one
      children: workers
    } restart: :permanent}
    {id: "coordinator" spawn: make_coordinator restart: :permanent}
  ]
}
```

### Events

Supervisors emit events on child lifecycle changes:

```
agent.on_supervise_event sup (evt) {
  evt.type ? {
    "child_started"  -> log.info "{evt.id} started"
    "child_crashed"  -> log.warn "{evt.id} crashed: {evt.reason}"
    "child_restarted" -> log.info "{evt.id} restarted (attempt {evt.count})"
    "max_restarts"   -> log.err "{evt.id} exceeded restart limit"
    _ -> ()
  }
}
```

## Cross-References

- Agent spawning: [agents.md](agents.md)
- Agent lifecycle: [stdlib-agents.md](stdlib-agents.md)
- Circuit breakers (complementary): stdlib_roadmap (`std/circuit`)
- Introspection: [stdlib-introspect.md](stdlib-introspect.md)
