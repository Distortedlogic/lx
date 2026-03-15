# Message Middleware / Interceptors

When messages flow between agents, cross-cutting concerns arise: tracing, rate-limiting, transformation, policy enforcement. `agent.intercept` provides a middleware layer for `~>` and `~>?` messages without modifying every call site.

## Why Middleware

Without interceptors, adding tracing to agent communication requires wrapping every call:

```
traced_ask = (agent msg) {
  trace.record {from: "self" to: agent msg time: time.now ()}
  agent ~>? msg ^
}
traced_ask worker {task: "review"} ^
traced_ask analyzer {task: "audit"} ^
```

Every call site must use `traced_ask` instead of `~>?`. Adding rate-limiting means another wrapper. Adding context injection means another. The wrappers don't compose.

With interceptors, cross-cutting behavior is registered once and applies to all communication through an agent:

```
use std/agent

traced_worker = agent.intercept worker (msg next) {
  trace.record {from: "self" to: "worker" msg time: time.now ()}
  next msg
}
traced_worker ~>? {task: "review"} ^
```

## API

```
agent.intercept agent middleware     -- Agent (returns wrapped agent)
                                     --   middleware: (msg next) -> a
```

`agent.intercept` takes an agent and a middleware function. The middleware receives the message and a `next` function. Calling `next msg` passes the (potentially modified) message to the actual agent. Not calling `next` short-circuits — the message is never sent.

The returned value is a new agent with the middleware applied. The original agent is unchanged (immutable).

## Middleware Chain

Multiple interceptors compose by wrapping:

```
worker_with_trace = agent.intercept worker (msg next) {
  log.debug "sending: {msg | to_str}"
  result = next msg
  log.debug "received: {result | to_str}"
  result
}

worker_with_rate_limit = agent.intercept worker_with_trace (msg next) {
  circuit.check_rate "sends" 10 (time.sec 1) ^
  next msg
}
```

Interceptors execute outside-in: rate limit checks first, then tracing, then the actual send. Responses flow back inside-out: actual response, then tracing logs it, then rate limiter passes it through.

## Patterns

### Tracing All Communication

```
add_tracing = (agent name) {
  agent.intercept agent (msg next) {
    start = time.now ()
    trace.record {type: "send" agent: name msg time: start}
    result = next msg
    trace.record {type: "recv" agent: name duration: time.since start}
    result
  }
}

traced_reviewer = add_tracing reviewer "reviewer"
traced_analyzer = add_tracing analyzer "analyzer"
```

### Context Injection

```
add_context = (agent ctx_record) {
  agent.intercept agent (msg next) {
    enriched = msg | merge {_context: ctx_record _timestamp: time.now () | to_str}
    next enriched
  }
}

contextualized = add_context worker {project: "lx" session: session_id}
contextualized ~>? {task: "review"} ^
```

### Rate Limiting

```
rate_limited = (agent max_per_sec) {
  agent.intercept agent (msg next) {
    circuit.check_rate "agent_send" max_per_sec (time.sec 1) ^
    next msg
  }
}
```

### Policy Enforcement

```
sandboxed = (agent allowed_actions) {
  agent.intercept agent (msg next) {
    contains? msg.action allowed_actions ? {
      true -> next msg
      false -> Err {denied: "action '{msg.action}' not in allowed set"}
    }
  }
}

safe_worker = sandboxed worker ["read" "analyze" "summarize"]
safe_worker ~>? {action: "delete" target: "prod"} ^
```

### Retry with Backoff

```
with_retry = (agent n) {
  agent.intercept agent (msg next) {
    retry n () next msg
  }
}
```

## Interaction with Protocol

Protocol validation happens before interceptors. If a message fails Protocol validation, the interceptor chain is never entered:

```
Protocol ReviewRequest = {task: Str  path: Str}
intercepted = agent.intercept worker middleware
intercepted ~>? ReviewRequest {task: "review" path: "src/"} ^
```

Order: Protocol validates → interceptor chain → actual send.

## Interaction with `~>` vs `~>?`

Interceptors apply to both `~>` (send) and `~>?` (ask). For `~>`, the middleware's return value is discarded (send is fire-and-forget). For `~>?`, the middleware's return value becomes the ask's result.

## Implementation Status

Planned. `agent.intercept` in `std/agent`. Returns a wrapper agent record whose handler calls the middleware chain.

## Cross-References

- Agent communication: [agents.md](agents.md)
- Protocol validation: [agents.md](agents.md#message-contracts-protocol)
- Circuit breakers (used by interceptors): [stdlib_roadmap.md](../design/stdlib_roadmap.md#stdcircuit)
- Tracing: [stdlib_roadmap.md](../design/stdlib_roadmap.md#stdtrace)
