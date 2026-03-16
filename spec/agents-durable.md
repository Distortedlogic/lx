# Durable Execution / Workflow Persistence

Automatic persistence of workflow state at suspension points so long-running agentic workflows survive process death, restart, or deliberate pause — and resume transparently.

## Problem

Everything in lx is in-process. If the process dies, all local bindings, execution position, and agent references are lost. `checkpoint`/`rollback` is in-memory. `ctx.save`/`ctx.load` is manual — the programmer threads persistence everywhere. `yield` is a synchronous stdin/stdout protocol scoped to a single process.

For workflows that run hours or days (multi-day code migrations, long research campaigns, iterative fine-tuning loops), restarting from scratch is unacceptable. This is what Temporal, Inngest, and Azure Durable Functions solve.

```
// Currently: manual persistence threading
with mut state := ctx.load "state.json" ?? {step: 0 results: []} {
  // must manually save after every step
  state.step <- 1
  result = worker ~>? task ^
  state.results <- state.results ++ [result]
  ctx.save "state.json" state ^
  // if process dies between save and next step, partial state
}
```

## `durable` Expression

```
result = durable "migration-2026-03" {
  files = discover_files () ^
  reviewed = files | each (f) {
    reviewer ~>? {task: "review" file: f} ^
  }
  summary = synthesizer ~>? {results: reviewed} ^
  summary
}
```

`durable` wraps a block. The runtime automatically persists execution state at every suspension point. If the process dies and restarts, execution resumes from the last persisted point with all local bindings restored.

### Suspension Points (Auto-Persisted)

| Point | Why |
|-------|-----|
| `yield` | Waiting for orchestrator response |
| `~>?` | Waiting for agent response |
| `refine` rounds | Between grade/revise iterations |
| `plan.run` steps | Between plan step executions |
| `consensus` rounds | Between voting/deliberation rounds |
| `par` joins | After each branch completes |

### Workflow ID

The string after `durable` is the workflow ID. Must be unique. Used for resumption and storage directory naming.

```
durable "unique-workflow-id" { ... }
```

### Return Value

```
Protocol DurableResult = {
  value: Any
  workflow_id: Str
  resumed: Bool
  steps_replayed: Int
}
```

- `resumed: false` — fresh execution.
- `resumed: true` — resumed from persisted state. `steps_replayed` is how many suspension points were fast-forwarded.

## Resumption

### CLI

```
lx resume migration-2026-03
```

Loads persisted state and resumes the workflow from the last suspension point. The original `.lx` file is re-parsed — only the durable block's execution state is restored.

### Programmatic

```
use std/durable

status = durable.status "migration-2026-03" ^
// => {state: :suspended  step: 7  total_steps: 15  last_active: "2026-03-15T10:30:00Z"}

durable.resume "migration-2026-03" ^
durable.cancel "migration-2026-03" ^
durable.list () ^
// => [{id: "migration-2026-03"  state: :suspended} ...]
```

## What Gets Persisted

| Category | Persisted | Notes |
|----------|-----------|-------|
| Local bindings | Yes | Serialized as JSON values |
| Execution position | Yes | Step index within durable block |
| Completed step results | Yes | Cached — replayed on resume, not re-executed |
| Agent references | No | Agents are re-spawned on resume |
| Functions/closures | No | Re-evaluated from source on resume |
| Shell side effects | No | External — not reversible |
| MCP connections | No | Re-established on resume |

Functions and closures cannot be serialized. On resume, the source file is re-parsed and function definitions are re-evaluated. Only data values (strings, numbers, records, lists, maps) are persisted.

## Storage Backend

Pluggable via `RuntimeCtx`. Default: filesystem JSON.

```
.lx-durable/
  migration-2026-03/
    state.json        // current execution position + metadata
    steps/
      0.json          // result of step 0
      1.json          // result of step 1
      ...
```

### DurableBackend Trait

```rust
trait DurableBackend: Send + Sync {
    fn save_state(&self, id: &str, state: &DurableState) -> Result<()>;
    fn load_state(&self, id: &str) -> Result<Option<DurableState>>;
    fn save_step(&self, id: &str, step: usize, value: &Value) -> Result<()>;
    fn load_step(&self, id: &str, step: usize) -> Result<Option<Value>>;
    fn list(&self) -> Result<Vec<DurableInfo>>;
    fn cancel(&self, id: &str) -> Result<()>;
}
```

Embedders can implement database-backed storage, S3, etc.

## Retention and Cleanup

```
durable "workflow-id" {
  retention: (time.hours 24)
  ...body...
}
```

Optional `retention` field. After completion, persisted state is cleaned up after the retention period. Default: 7 days.

## Idempotency

Each suspension point is numbered sequentially. On resume, steps 0..N-1 are replayed from cache (not re-executed). Step N resumes live execution. This means side effects in completed steps are NOT re-run — the cached result is used.

For steps with non-deterministic side effects (API calls, shell commands), the cached result ensures consistency even if the external world has changed. This is the standard durable execution tradeoff.

## Implementation

### Parser

`durable` is a new keyword. Parsed as: `durable <string-expr> { body }`. Optional config fields (`retention`) parsed from the block.

### AST Node

```
Durable {
  id: Expr
  retention: Option<Expr>
  body: Vec<Stmt>
}
```

### Interpreter

The interpreter wraps the body's evaluation. At each suspension point, it checks whether a cached result exists (replay) or needs live execution (persist after completion).

### std/durable Module

New stdlib module. Functions: `status`, `resume`, `cancel`, `list`, `cleanup`.

## Cross-References

- Yield: [agents-advanced.md](agents-advanced.md) (primary suspension point)
- Checkpoint/rollback: [agents-advanced.md](agents-advanced.md) (in-memory snapshots — complementary)
- Plans: [agents-plans.md](agents-plans.md) (plan steps as suspension points)
- Context persistence: stdlib (`std/ctx`) — manual, complementary
- Saga: [agents-saga.md](agents-saga.md) (compensation on failure — orthogonal to persistence)
