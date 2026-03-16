# Agent Pools

First-class identity-less worker groups. A Pool manages N interchangeable agents with the same handler, distributes work, and collects results. Distinct from `agent.supervise` (restart strategies for named agents) and `par` (one-shot concurrency).

## Problem

The most common multi-agent pattern in flows is: spawn N workers, fan out tasks, collect and reconcile results. Today this requires manual ceremony:

```
(a, b, c) = par {
  dispatch.run_one "agents/researcher.lx" msg1
  dispatch.run_one "agents/context.lx" msg2
  dispatch.run_one "agents/search.lx" msg3
}
merged = agent.reconcile [a b c] {strategy: "merge_fields"}
```

Problems: manual lifecycle management, fragile tuple destructuring, no load balancing, no auto-replacement of failed workers.

## `std/pool` Module

### Creating a Pool

```
use std/pool

p = pool.create {
  agent: "agents/worker.lx"
  size: 3
  trait: Reviewer
  overflow: :queue
} ^
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent` | Str | Yes | Path to agent script |
| `size` | Int | Yes | Number of workers |
| `trait` | Trait | No | All workers must implement this Trait |
| `overflow` | Symbol | No | `:queue` (default), `:drop`, `:error` |
| `max_queue` | Int | No | Queue depth limit (default: unlimited) |

Workers are spawned eagerly. If `trait` is specified, each worker is validated at spawn time (see [agents-trait.md](agents-trait.md)).

### Fan-Out / Fan-In

```
tasks = [{task: "review" path: "src/auth"} {task: "review" path: "src/api"}]
results = pool.fan_out p tasks ^
```

`pool.fan_out` distributes tasks across workers round-robin, waits for all results. Returns `[Result]` in task order. Failed tasks return `Err` — partial failures don't abort the batch.

### Map (1:1 task-to-result)

```
results = pool.map p items (item) {task: "analyze" input: item} ^
```

Like `pmap` but uses the pool's workers instead of spawning fresh agents.

### Submit (fire-and-forget)

```
pool.submit p {task: "index" data: batch} ^
```

Enqueues work without waiting for a result. Uses the overflow policy if all workers are busy.

### Drain and Shutdown

```
pool.drain p ^
pool.shutdown p ^
```

`drain` waits for all queued work to complete, then prevents new submissions. `shutdown` kills all workers immediately.

### Status

```
status = pool.status p
-- {size: 3  busy: 2  idle: 1  queued: 5  completed: 42  failed: 1}
```

### Resize

```
pool.resize p 5 ^
```

Adjusts the pool size. New workers are validated against the Trait. Excess workers are drained.

## Integration with Reconcile

```
results = pool.fan_out p tasks ^
merged = agent.reconcile (results | filter_map ok?) {
  strategy: "union"
  key: (r) r.path
}
```

Pool results feed directly into `agent.reconcile` for merging.

## Integration with Supervision

Pools use `agent.supervise` internally for worker restart:

```
p = pool.create {
  agent: "agents/worker.lx"
  size: 3
  restart: :one_for_one
} ^
```

Workers that crash are automatically restarted. The pool maintains the target `size`.

## Implementation

### New Module: `std/pool`

Following the stdlib pattern: `crates/lx/src/stdlib/pool.rs` with `pub fn build() -> IndexMap<String, Value>`.

Functions: `create`, `fan_out`, `map`, `submit`, `drain`, `shutdown`, `status`, `resize`.

### Pool State

`Pool` is an opaque type holding:
- Worker agent handles (Vec of PIDs)
- Task queue (VecDeque)
- Trait constraint (optional)
- Overflow policy
- Completion counters

Uses `LazyLock<DashMap<...>>` for global pool registry (same pattern as agent dialogue sessions).

### Worker Management

Workers are spawned via `agent.spawn` with the pool's config. Each worker gets a unique ID within the pool. The pool routes messages round-robin across idle workers.

## Cross-References

- Agent traits (pool constraint): [agents-trait.md](agents-trait.md)
- Agent supervision (worker restart): [agents-supervision.md](agents-supervision.md)
- Reconciliation (merging pool results): [agents-reconcile.md](agents-reconcile.md)
- Pipeline backpressure: [agents-pipeline.md](agents-pipeline.md)
