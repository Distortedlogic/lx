# Structured Clarification

A protocol for agents to ask structured questions back to their caller without going through the top-level orchestrator. Inline clarification within a single request-response cycle.

## Problem

`yield` goes to the top-level orchestrator. But when agent B is processing a request from agent A, B might need clarification from A — not from the human. Currently there's no back-channel:

```
handler = (msg) {
  -- msg is ambiguous. Need to ask the sender "which module?"
  -- Can't use yield (goes to top-level orchestrator)
  -- Can't use ~>? (don't know who sent the message)
  -- Only option: return Err "ambiguous: which module?"
  -- Caller must parse the error, resend with more context. Fragile.
}
```

## `caller` — Implicit Binding

Inside an agent handler, `caller` is an implicit binding (like `it` in `sel`) that refers to the agent or context that sent the current message.

```
handler = (msg) {
  ambiguous = msg.path | split "/" | len < 2
  ambiguous ? {
    true -> {
      answer = caller ~>? {
        type: "clarify"
        question: "Which module? I found multiple matches."
        options: ["auth" "auth_middleware" "auth_utils"]
        default: "auth"
      } ^
      process msg answer.choice
    }
    false -> process msg msg.path
  }
}
```

### Semantics

- `caller` is only available inside agent handler functions (the function passed to `{handler: fn}` or registered via `agent.spawn`)
- Outside a handler, referencing `caller` is a runtime error: `"caller is only available inside an agent handler"`
- `caller` is an `Agent` value — supports `~>` and `~>?`
- For subprocess agents, `caller` refers to the parent process
- For in-process agents (record with handler), `caller` refers to the agent value that sent the message

### Caller's Side

The caller must be prepared to receive clarification requests. The simplest approach: use `agent.intercept` to handle clarifications automatically:

```
worker = agent.spawn {command: "worker"} ^

smart_worker = agent.intercept worker (msg next) {
  response = next msg
  response ? {
    Ok {type: "clarify" ..rest} -> {
      answer = resolve_clarification rest
      next (msg | merge {clarification: answer})
    }
    _ -> response
  }
}
```

Or define a `Clarify` trait for structured clarification:

```
Trait Clarify = {
  type: Str = "clarify"
  question: Str
  options: [Str] = []
  default: Str = ""
}

Trait ClarifyResponse = {
  choice: Str
  reasoning: Str = ""
}
```

### Nested Clarification

An agent handling a clarification request has its own `caller` — the agent that asked the clarification. This allows multi-hop clarification chains, though in practice these should be rare and bounded.

### With Dialogue Sessions

Clarification is a single back-and-forth within one request. For sustained multi-turn conversation, use `agent.dialogue`. The distinction:

- **Clarification** — "I need one piece of info to complete your request." Inline, synchronous.
- **Dialogue** — "Let's work through this together over multiple turns." Session-based, accumulates history.

## Implementation Notes

`caller` is set by the interpreter when dispatching a message to a handler. It's stored in the handler's scope as a special binding (like `__ambient_context`). For subprocess agents, the parent's agent handle is the caller. For in-process agents, the sending agent record is the caller.

When `caller` is used with `~>?`, the message is sent back through the same transport (subprocess stdin/stdout or direct function call) as a clarification request. The original `~>?` call blocks until the clarification round-trip completes.

## Cross-References

- Agent handlers: [agents.md](agents.md)
- Message interceptors: [agents-intercept.md](agents-intercept.md)
- Multi-turn dialogue: [agents-dialogue.md](agents-dialogue.md)
- Trait validation: [agents-protocol.md](agents-protocol.md)
- `it` binding precedent: [runtime.md](runtime.md) (`it` in `sel` blocks)
