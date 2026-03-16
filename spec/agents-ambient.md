# Ambient Context

Scoped ambient state that flows through call chains without explicit parameter threading. Like Go's `context.Context` — deadline, budget, request ID, and cancellation propagate automatically to all agent operations within a scope.

## Problem

Every agent operation needs context: deadlines, budgets, trace IDs, credentials. Currently this must be threaded manually through every function:

```
review = (ctx file) {
  content = fs.read file ^
  result = ctx.agent ~>? {task: "review" content deadline: ctx.deadline} ^
  log_with_trace ctx.trace_id "reviewed {file}"
  result
}
```

This is verbose and error-prone. Forgetting to pass `ctx` is silent and causes missing deadlines, untracked requests, and budget overruns.

## `with context` — Ambient Scope

```
with context deadline: (time.now + 30) request_id: "abc-123" budget: 500 {
  analyzer = agent.spawn {command: "analyzer"} ^
  result = analyzer ~>? {task: "review"} ^
  mcp.call client "read_file" {path: "src/"} ^
}
```

All agent and tool operations inside the block inherit the ambient context. `agent.spawn` passes the deadline to the subprocess. `~>?` propagates the request ID. `mcp.call` respects the budget.

### Syntax

```
with context key1: val1  key2: val2 {
  body
}
```

`with context` is an extension of the existing `with` keyword. The keyword `context` after `with` signals ambient binding rather than lexical binding. The fields form a record that becomes the ambient context for the block.

### Nesting and Override

```
with context deadline: 60 budget: 1000 {
  with context budget: 200 {
    -- deadline: 60 (inherited), budget: 200 (overridden)
    agent ~>? msg ^
  }
  -- deadline: 60, budget: 1000 (restored)
}
```

Inner `with context` merges with outer. Inner values override outer values for the same key. On scope exit, the outer context is restored.

### Reading Ambient Context

```
with context deadline: 60 request_id: "abc" {
  dl = context.deadline         -- 60
  rid = context.request_id      -- "abc"
  all = context.current ()      -- {deadline: 60 request_id: "abc"}
}
```

`context` is a built-in module-like binding (similar to `log`) available inside any `with context` block. Outside a `with context` block, `context.current ()` returns `{}` (empty record).

## Standard Context Fields

These fields have runtime-level support — the interpreter recognizes them and enforces their semantics:

| Field | Type | Behavior |
|-------|------|----------|
| `deadline` | Int (seconds from now) | Agent operations fail with `Err "deadline_exceeded"` when elapsed time exceeds deadline. Propagated to subagents as remaining time. |
| `budget` | Int (abstract units) | Decremented by agent ops (ask=10, tool call=5, spawn=20). Operations fail with `Err "budget_exceeded"` when budget reaches 0. |
| `request_id` | Str | Automatically attached to all `~>` / `~>?` messages as `_request_id` field. Propagated to subagents. |
| `trace_id` | Str | Attached to log output. Propagated to subagents. |

### Deadline Propagation

```
with context deadline: 30 {
  -- 10 seconds pass during analysis...
  sub = agent.spawn {command: "worker"} ^
  -- subprocess receives deadline: 20 (remaining time)
  sub ~>? {task: "work"} ^
  -- if deadline exceeded, returns Err "deadline_exceeded"
}
```

The runtime calculates remaining time at each propagation point. Subagents get the remaining budget, not the original. This prevents cascading timeouts where subagents think they have the full time.

### Budget Propagation

```
with context budget: 100 {
  a ~>? msg ^          -- costs 10, remaining: 90
  b ~>? msg ^          -- costs 10, remaining: 80
  mcp.call c "tool" {} ^  -- costs 5, remaining: 75
}
```

Budget costs are configurable per operation type. Default costs are intentionally abstract — the exact values will be tuned based on real usage patterns.

## Custom Context Fields

Any field can be added to the ambient context. Fields not in the standard set are passed through as-is — no runtime enforcement, but available via `context.current ()`:

```
with context team: "backend" env: "staging" {
  ctx = context.current ()
  -- ctx == {team: "backend" env: "staging"}
}
```

Custom fields propagate to subagents as part of the ambient context record.

## Implementation Notes

Ambient context is stored in the interpreter's `Env` as a special binding (`__ambient_context`). `with context` creates a child scope with the merged context. Agent operations (`~>`, `~>?`, `agent.spawn`, `mcp.call`) read from the ambient context in the current scope.

For subprocess agents, the ambient context is serialized as a JSON-line `{"type":"context",...}` message on spawn. The subprocess interpreter initializes its ambient context from this message.

## Cross-References

- Scoped bindings: [agents-advanced.md](agents-advanced.md) (`with`)
- Agent spawning: [agents.md](agents.md)
- Introspection budget: [stdlib-introspect.md](stdlib-introspect.md) (`introspect.budget`)
- Circuit breakers: ROADMAP (`std/circuit`)
