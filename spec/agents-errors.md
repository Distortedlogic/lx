# Structured Agent Errors

`AgentErr` is a tagged union providing categorized failure types for agent operations. Different failure categories demand different recovery strategies — structured errors enable pattern-matched recovery instead of string parsing.

## Problem

Every agent failure is currently `Err "some string"`. But agents fail in categorically different ways:

| Failure | Current | Correct Recovery |
| --- | --- | --- |
| HTTP 503 | `Err "status 503"` | Retry with backoff |
| Rate limited | `Err "status 429"` | Wait, retry with longer delay |
| Budget exhausted | `Err "budget exceeded"` | Request more budget or degrade |
| Context overflow | `Err "context too large"` | Summarize and retry |
| Agent incompetent | `Err "low quality"` | Route to different agent |
| Upstream down | `Err "connection refused"` | Circuit break |
| Timeout | `Err "timeout"` | Retry with longer deadline |
| Permission denied | `Err "denied"` | Escalate to human |
| Trait violation | `Err "invalid message"` | Log and reject |

With string errors, recovery code is fragile regex matching. With structured errors, recovery is pattern matching:

```
result ?? (e) e ? {
  Timeout info        -> retry_with {base_ms: info.elapsed_ms * 2} f
  RateLimited info    -> time.sleep info.retry_after; retry f
  BudgetExhausted _   -> degrade_quality task
  ContextOverflow info -> summarize_and_retry info.content f
  Incompetent info    -> route_to_next info.task agents
  _                   -> Err e
}
```

## `AgentErr`

```
AgentErr = | Timeout {elapsed_ms: Int  deadline_ms: Int}
           | RateLimited {retry_after_ms: Int  limit: Str}
           | BudgetExhausted {used: Float  limit: Float  resource: Str}
           | ContextOverflow {size: Int  capacity: Int  content: Str}
           | Incompetent {agent: Str  task: Str  score: Float  threshold: Float}
           | Upstream {service: Str  code: Int  message: Str}
           | PermissionDenied {action: Str  resource: Str}
           | TraitViolation {expected: Str  got: Str  message: Str}
           | Unavailable {agent: Str  reason: Str}
           | Cancelled {reason: Str}
           | Internal {message: Str}
```

### Variant semantics

**`Timeout`** — operation exceeded deadline. `elapsed_ms` is actual time. `deadline_ms` is what was set. Recovery: retry with longer deadline or degrade.

**`RateLimited`** — upstream rate limit hit. `retry_after_ms` comes from the service's retry-after header (or estimated). Recovery: sleep and retry.

**`BudgetExhausted`** — cost budget exceeded. `resource` is "tokens", "dollars", "requests", etc. Recovery: request more budget, reduce quality, or stop.

**`ContextOverflow`** — input exceeds context window. `content` is a truncated preview. Recovery: summarize input, chunk and process in parts.

**`Incompetent`** — agent produced output below quality threshold. `score` is what the agent achieved, `threshold` is what was needed. Recovery: route to a different agent.

**`Upstream`** — external service error (HTTP, MCP, etc.). `code` is status code, `service` is the service name. Recovery: retry (transient) or circuit break (persistent).

**`PermissionDenied`** — operation not permitted. From sandbox restrictions or insufficient access. Recovery: escalate to human, request permission.

**`TraitViolation`** — message doesn't match expected Trait shape. Recovery: log, reject, notify sender.

**`Unavailable`** — target agent is not running, not registered, or unhealthy. Recovery: wait and retry, or find alternative agent via registry.

**`Cancelled`** — operation was cancelled by parent, timeout, or user. Recovery: acknowledge and clean up.

**`Internal`** — catch-all for unexpected failures. Recovery: log and propagate.

## Integration Points

### `std/retry`

```
retry_with {
  max: 5
  retry_on: (e) e ? {
    Timeout _      -> true
    RateLimited _  -> true
    Upstream info  -> info.code >= 500
    _              -> false
  }
} f
```

### `std/circuit`

```
circuit = circuit.create {
  trip_on: (e) e ? { Upstream _ -> true  Unavailable _ -> true  _ -> false }
  threshold: 5
  window: 60
}
```

### `agent.reconcile`

```
results | partition ok? | (successes failures) {
  failures.1 | each (e) e ? {
    Incompetent info -> log.warn "agent {info.agent} underperformed: {info.score}"
    _                -> log.err "unexpected: {e}"
  }
  agent.reconcile (successes | map (r) r ^) :vote
}
```

### `refine`

```
refine draft {
  grade: (work) {
    score: evaluate work
    feedback: "..."
  }
  revise: (work feedback) improve work feedback
  on_error: (e round) e ? {
    BudgetExhausted _ -> :stop
    Timeout _         -> :retry
    _                 -> :fail
  }
}
```

## Migration

Existing `Err "string"` values continue to work. `AgentErr` is opt-in: stdlib modules that produce agent-relevant errors return `AgentErr` variants. User code can return either.

The `??` operator and `^` propagation work with both string errors and `AgentErr` variants. Pattern matching on `AgentErr` uses standard tagged union syntax.

### Stdlib adoption

| Module | Current error | Migrates to |
| --- | --- | --- |
| `std/http` | `Err {status code message}` | `Upstream` or `RateLimited` |
| `std/agent` | `Err "string"` | `Timeout`, `Unavailable`, `TraitViolation` |
| `std/mcp` | `Err "string"` | `Upstream`, `Timeout` |
| `std/budget` | `Err "exceeded"` | `BudgetExhausted` |
| `std/context` | `Err "overflow"` | `ContextOverflow` |
| `std/pool` | `Err "string"` | `Unavailable`, `Timeout` |

## Implementation

`AgentErr` is a type definition in `std/agent` (or a new `std/errors` module re-exported by `std/agent`). The tagged union uses existing type definition infrastructure — no parser changes needed.

### Producing errors

Stdlib modules construct `AgentErr` variants using existing tagged value syntax:

```rust
Value::tagged("Timeout", record!{
    "elapsed_ms" => Value::Int(elapsed),
    "deadline_ms" => Value::Int(deadline),
})
```

### Matching errors

Standard pattern matching on tagged unions:

```
result ? {
  Ok v -> process v
  Err (Timeout info) -> retry info
  Err (RateLimited info) -> wait info.retry_after_ms
  Err e -> propagate e
}
```

## Cross-References

- Retry: [stdlib-retry.md](stdlib-retry.md) — retry predicates match on `AgentErr` variants
- Circuit breakers: `std/circuit` — trip conditions match on error variants
- Budget: [agents-budget.md](agents-budget.md) — `BudgetExhausted` variant
- Context: [agents-context-capacity.md](agents-context-capacity.md) — `ContextOverflow` variant
- Registry: [agents-discovery.md](agents-discovery.md) — `Unavailable` when agent not found
- Sandboxing: [toolchain.md](toolchain.md) — `PermissionDenied` from sandbox
