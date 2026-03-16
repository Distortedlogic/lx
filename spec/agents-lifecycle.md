# Agent Lifecycle Hooks

`agent.on` registers internal lifecycle event handlers on agents. Agents react to their own events — idle, shutdown, error, startup — without external supervision and without wrapping every handler in boilerplate.

Distinct from `agent.supervise` (external, manager-side crash recovery). Lifecycle hooks are internal, agent-side self-management. Erlang has `init`/`terminate`/`handle_info` callbacks; lx agents currently have none.

## Problem

Agents need to react to lifecycle transitions:

- **Idle** — persist state, compact memory, release resources when no work arrives
- **Shutdown** — clean up, notify dependents, flush pending writes
- **Error** — log, attempt recovery, notify supervisor without crashing
- **Startup** — load state, register with service discovery, announce readiness

Currently every handler must manually implement these concerns:

```
handler = (msg) {
  // manual error wrapping in every handler
  result = (do_work msg) ? {
    Ok v -> v
    Err e -> {
      log.err "handler error: {e}"
      supervisor ~> {status: "error" error: e}
      {error: e}
    }
  }
  // manual idle detection? impossible without external timer
  // manual shutdown? no signal mechanism
  result
}
```

No way to detect idle (no messages for N seconds), no shutdown signal, no startup hook. Each concern is reimplemented per agent.

## API

### Registering Hooks

```
use std/agent

me = agent.self ()

agent.on me :startup () {
  state = ctx.load "agent_state.json" ?? {tasks: [] history: []}
  log.info "agent started, loaded {state.tasks | len} pending tasks"
}

agent.on me :shutdown (reason) {
  ctx.save "agent_state.json" state ^
  supervisor ~> {status: "shutting_down" agent: me reason}
  log.info "agent shutdown: {reason}"
}

agent.on me :error (err msg) {
  trace.span "handler_error" {error: err message: msg}
  log.err "error processing {msg}: {err}"
  err.recoverable? ? {
    true -> retry 2 () handler msg
    false -> supervisor ~> {status: "failed" error: err}
  }
}

agent.on me :idle 30 () {
  memory.flush! ^
  context.compact win :summarize
  log.debug "idle housekeeping complete"
}
```

### Event Types

| Event | Callback Signature | When |
|-------|-------------------|------|
| `:startup` | `()` | Agent process begins, before first message |
| `:shutdown` | `(reason)` | Agent is being killed or process exits |
| `:error` | `(err msg)` | Unhandled error in handler. `err` is the error, `msg` is the message that caused it |
| `:idle` | `()` | No messages received for N seconds (N specified after `:idle`) |
| `:message` | `(msg)` | Before every incoming message (pre-handler hook) |

### Idle Duration

`:idle` takes a duration in seconds:

```
agent.on me :idle 30 () { ... }   // fire after 30s idle
agent.on me :idle 300 () { ... }  // fire after 5min idle
```

The idle timer resets on every incoming message. If the agent receives a message during idle processing, the idle callback completes and normal handling resumes.

### Pre-Message Hook

`:message` fires before the handler for every incoming message. Useful for logging, validation, or context injection:

```
agent.on me :message (msg) {
  trace.span "incoming" {from: msg._from type: msg.type}
  context.add win {key: "last_msg" content: msg tokens: (context.estimate msg)}
}
```

The hook runs before the handler. If the hook returns `Err`, the message is rejected and the handler does not run.

### Multiple Hooks

Multiple hooks for the same event are allowed. They execute in registration order:

```
agent.on me :shutdown (reason) { flush_state () }
agent.on me :shutdown (reason) { notify_peers () }
// flush_state runs first, then notify_peers
```

### Removing Hooks

```
agent.on_remove me :idle
agent.on_remove me :error
```

Removes all hooks for the specified event.

## Patterns

### Persistent Agent

```
me = agent.self ()

agent.on me :startup () {
  state := ctx.load "agent_state.json" ?? {history: [] config: defaults}
}

agent.on me :shutdown (reason) {
  ctx.save "agent_state.json" state ^
}

agent.on me :idle 60 () {
  ctx.save "agent_state.json" state ^
  log.debug "auto-saved state"
}
```

### Self-Healing Agent

```
agent.on me :error (err msg) {
  err_count := (err_count ?? 0) + 1
  err_count > 5 ? {
    true -> {
      log.err "too many errors, requesting restart"
      supervisor ~> {action: "restart" agent: me reason: "error_threshold"}
    }
    false -> {
      log.warn "error {err_count}/5: {err}"
      retry 1 () handler msg
    }
  }
}
```

### With Introspect

```
agent.on me :idle 30 () {
  introspect.is_stuck me ? {
    true -> {
      introspect.strategy_shift me "idle_detected"
      log.warn "stuck and idle — shifting strategy"
    }
    false -> log.debug "idle but not stuck"
  }
}
```

### With Supervision

Lifecycle hooks and supervision compose — hooks handle the agent's internal response, supervision handles the external recovery:

```
// internal (agent side)
agent.on me :error (err msg) {
  log.err "error: {err}"
  state.errors = state.errors ++ [err]
}

// external (supervisor side)
agent.supervise worker {
  strategy: :one_for_one
  max_restarts: 3
  window: 60
}
```

The error hook fires first (agent tries internal recovery), then if the agent crashes, the supervisor restarts it.

## Implementation

Extension to `std/agent`. Each agent holds a `LifecycleHooks` struct:

```
struct LifecycleHooks {
    startup: Vec<Value>,     // closures
    shutdown: Vec<Value>,
    error: Vec<Value>,
    idle: Vec<(u64, Value)>, // (seconds, closure)
    message: Vec<Value>,
}
```

The agent runtime checks hooks at lifecycle transitions:
- `startup` hooks run after the agent process starts, before entering the message loop
- `shutdown` hooks run when `agent.kill` is called or the process exits
- `error` hooks wrap the handler invocation in a catch
- `idle` hooks are driven by a timer that resets on each message
- `message` hooks run before each handler invocation

### Dependencies

- `std/time` (idle timer)
- Existing agent runtime infrastructure

## Cross-References

- Supervision: [agents-supervision.md](agents-supervision.md) — external crash recovery (complementary)
- Introspect: [stdlib-introspect.md](stdlib-introspect.md) — query agent state (hooks are event-driven)
- Trace: stdlib (`std/trace`) — lifecycle events can emit spans
- Context capacity: [agents-context-capacity.md](agents-context-capacity.md) — idle hook for context compaction
