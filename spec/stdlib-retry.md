# Transient Failure Retry

`std/retry` provides retry-with-backoff for transient failures. Distinct from `refine` (quality iteration — "is this good enough?") and `std/circuit` (hard failure limits — "stop trying entirely"). Retry is for "this HTTP call returned 503, wait and try again."

## Problem

Agents calling external services (HTTP APIs, MCP servers, LLMs) hit transient failures constantly: rate limits, timeouts, connection resets, 5xx errors. Current options:

1. `^` — fail immediately, propagate error up. Caller must catch and retry manually.
2. Hand-roll a recursive loop with sleep and counter.

Neither is acceptable for production agentic workflows. Every agent ends up writing the same retry loop with the same bugs (no jitter, no max delay cap, no selective retry).

## `std/retry`

### Core API

```
use std/retry

result = retry f                          -- a ^ RetryErr
result = retry_with opts f                -- a ^ RetryErr
```

`retry` calls `f ()` and retries on `Err`. Returns the first `Ok` result or `Err` after exhausting attempts.

`f` must be `() -> a ^ e` — a zero-arg function returning a Result.

### Options

```
retry_with opts f
```

`opts` record (all optional):
```
{
  max: Int               -- max attempts (default: 3, includes first try)
  backoff: Symbol        -- :constant | :linear | :exponential (default: :exponential)
  base_ms: Int           -- base delay in milliseconds (default: 100)
  max_delay_ms: Int      -- cap on delay (default: 30000)
  jitter: Bool           -- add random jitter (default: true)
  retry_on: Fn           -- (Err -> Bool) predicate, only retry if true (default: all errors)
}
```

### Backoff Strategies

- `:constant` — wait `base_ms` between every attempt
- `:linear` — wait `base_ms * attempt` (100, 200, 300, ...)
- `:exponential` — wait `base_ms * 2^attempt` (100, 200, 400, 800, ...)

All strategies respect `max_delay_ms` cap. When `jitter` is true, actual delay is `delay * random(0.5, 1.5)`.

### Error Type

```
RetryErr = | Exhausted {attempts: Int  last_error: Err  elapsed_ms: Int}
```

`RetryErr` wraps the last error with metadata about the retry sequence.

### Return Value

On success, returns the unwrapped `Ok` value (not wrapped in another `Ok`). On exhaustion, returns `Err (Exhausted {...})`.

## Patterns

### Simple HTTP retry

```
use std/retry
use std/http

body = retry () http.get "https://api.example.com/data" ^ ^
```

Inner `^` propagates HTTP errors (triggering retry). Outer `^` propagates `RetryErr` if all attempts fail.

### Retry with options

```
result = retry_with {max: 5  backoff: :exponential  base_ms: 200} () {
  http.post url payload ^
} ^
```

### Selective retry (only on rate limits)

```
result = retry_with {
  max: 5
  base_ms: 1000
  retry_on: (e) e.status == 429
} () {
  http.get url ^
} ^
```

### Retry MCP tool calls

```
use std/mcp

result = retry_with {max: 3  base_ms: 500} () {
  mcp.call client "search" {query: q} ^
}
```

### Retry with budget awareness

```
use std/retry
use std/budget

result = retry_with {
  max: 5
  retry_on: (e) budget.status b != :exceeded
} () {
  r = ai.prompt_with {prompt: task} ^
  budget.spend b {tokens: r.usage} ^
  r
}
```

### Compose with pipes

```
urls | map (url) retry () http.get url ^ | filter ok?
```

## Implementation

`std/retry` is a new stdlib module with two functions: `retry` and `retry_with`.

The implementation is a loop that:
1. Calls `f ()`
2. On `Ok`, returns the value
3. On `Err`, checks `retry_on` predicate (if provided)
4. If retryable and attempts remain, sleeps for computed delay, increments counter, goto 1
5. If not retryable or attempts exhausted, returns `Err (Exhausted {...})`

Sleep uses `std::thread::sleep`. Jitter uses `rand::thread_rng`.

### Dependencies

- `std::thread::sleep` (delay between attempts)
- `rand` crate (jitter) — or `fastrand` for lighter dependency

## Cross-References

- Quality iteration: `refine` expression — retry is for transient failures, refine is for output quality
- Hard limits: `std/circuit` — circuit breakers prevent retry when a service is down
- Budget: [agents-budget.md](agents-budget.md) — budget-aware retry stops when cost exceeded
- HTTP: [stdlib-modules.md](stdlib-modules.md) (`std/http`) — primary retry target
- MCP: [stdlib-agents.md](stdlib-agents.md) (`std/mcp`) — another primary retry target
