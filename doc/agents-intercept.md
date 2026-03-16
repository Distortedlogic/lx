# Message Middleware / Interceptors — Reference

## API

```
agent.intercept agent middleware     -- Agent (returns wrapped agent)
                                     --   middleware: (msg next) -> a
```

`next msg` passes the message to the actual agent. Not calling `next` short-circuits (message never sent). Original agent is unchanged (immutable).

## Middleware Chain

Multiple interceptors compose by wrapping. Execution is outside-in; responses flow inside-out:

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

Applies to both `~>` (send, return value discarded) and `~>?` (ask, return value is the result).

## Patterns

### Tracing

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
```

## Gotcha

Protocol validation happens **before** interceptors. Order: Protocol validates -> interceptor chain -> actual send.
