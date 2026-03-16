# Deadline Propagation

Propagate time budgets across agent boundaries so sub-agents know how much time remains and can return partial results gracefully.

## Problem

`std/budget` tracks cost (tokens/dollars). `timeout` wraps a single expression. But there's no **time budget** that propagates across agent boundaries.

If an orchestrator has 30 seconds remaining and spawns 3 sub-agents, those sub-agents have no idea they're under time pressure. They might start expensive operations that will be killed mid-way by the orchestrator's timeout. The result: wasted work and ungraceful failures.

What's needed: a deadline that flows from parent to child agent, shrinks as time passes, and lets agents check remaining time and return partial results before the deadline hits.

## Design

### `std/deadline`

```lx
use std/deadline

dl = deadline.create 30000 ^

with deadline.scope dl {
  remaining = deadline.remaining () ^
  expired = deadline.expired () ^

  result = expensive_agent ~>? {task: data} ^

  deadline.remaining () ^ < 5000 ? true -> {
    emit "running low, returning partial"
    break partial_result
  }
}
```

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `deadline.create` | `(ms: Int) -> Result Deadline Str` | Create deadline from now + ms |
| `deadline.create_at` | `(epoch_ms: Int) -> Result Deadline Str` | Create deadline at absolute time |
| `deadline.scope` | `(dl: Deadline body: Fn) -> Result Any Str` | Execute body under deadline |
| `deadline.remaining` | `() -> Result Int Str` | Ms remaining in current scope |
| `deadline.expired` | `() -> Result Bool Str` | Whether deadline has passed |
| `deadline.check` | `() -> Result () Str` | Returns `Err "deadline exceeded"` if expired |
| `deadline.slice` | `(pct: Float) -> Result Deadline Str` | Sub-deadline from remaining time |
| `deadline.extend` | `(dl: Deadline ms: Int) -> Result Deadline Str` | Add time (only from creator scope) |

### Propagation Across Agent Boundaries

When `~>?` is called inside a `deadline.scope`, the remaining time is automatically attached to the message as `_deadline_ms`:

```lx
with deadline.scope dl {
  result = worker ~>? {task: "analyze"} ^
}
```

The sub-agent receives `{task: "analyze" _deadline_ms: 24500}`. If the sub-agent also uses `std/deadline`, it can honor the propagated deadline:

```lx
handler = (msg) {
  dl = msg._deadline_ms
    ? Some ms -> deadline.create ms ^
    : deadline.create 60000 ^

  with deadline.scope dl {
    do_work msg ^
  }
}
```

### `deadline.slice` — Sub-Budgeting

Like `budget.slice`, creates a sub-deadline from a fraction of remaining time:

```lx
with deadline.scope parent_dl {
  phase1_dl = deadline.slice 0.3 ^
  with deadline.scope phase1_dl {
    parse_result = do_parsing data ^
  }

  phase2_dl = deadline.slice 0.7 ^
  with deadline.scope phase2_dl {
    analyze parse_result ^
  }
}
```

### Graceful Degradation Pattern

```lx
with deadline.scope dl {
  full_result := None

  remaining = deadline.remaining () ^
  remaining > 20000 ? true -> {
    full_result <- Some (thorough_analysis data ^)
  }

  remaining = deadline.remaining () ^
  full_result ? {
    Some r -> r
    None -> quick_analysis data ^
  }
}
```

### Integration

- `std/budget` — budget tracks cost, deadline tracks time. Orthogonal. Both can be active in the same scope. A combined check: `budget.remaining b ^ > 0 && !deadline.expired () ^`.
- `timeout` — timeout kills the expression. deadline is cooperative — agents check and decide how to degrade. Use timeout as a hard backstop around deadline-aware code.
- `with context` (planned) — when ambient context ships, deadline propagation can use it instead of `_deadline_ms` field injection. Until then, field injection works.
- `std/pipeline` (planned) — pipeline stages can each get a `deadline.slice` of the overall pipeline deadline.
- `user.check` — can combine: `deadline.expired () ^ || (user.check () ^ | some?)` for "stop if out of time OR user interrupted."

## Implementation

Pure stdlib module. No parser changes. Core state: deadline instant stored in thread-local (or `RuntimeCtx` extension). `remaining` computes delta from `std::time::Instant::now()`. `_deadline_ms` injection in `~>?` path requires a small hook in agent message sending.

Approximately 120 lines of Rust for the module + 15 lines in agent send path for auto-propagation.

## Priority

Tier 2. Time awareness is fundamental for agents operating under real-world constraints. No parser changes. Small implementation. Immediately useful with existing `~>?` and `par`/`pmap`.
