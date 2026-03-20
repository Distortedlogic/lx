# Typed Yield Variants

Extends the `yield` protocol with typed variants so the orchestrator knows what KIND of help the agent needs. Replaces the ad-hoc `{type: "..."}` convention with Trait-validated yield messages.

## Problem

Today all yields look the same to the orchestrator — a JSON blob on stdout. The orchestrator must inspect the payload to figure out what the agent wants:

```
-- Agent yields different things for different reasons
plan = yield {type: "approval" steps: proposed}
guidance = yield {type: "reflection" attempt: result question: "what should I change?"}
data = yield {type: "information" query: "what's the API key for service X?"}
```

The `type` field is a convention, not enforced. The orchestrator can't validate the request shape. No standard set of yield categories exists.

## Yield Traits

Standard Traits for yield communication:

```
Trait YieldApproval = {
  kind: Str = "approval"
  action: Str
  details: Any
  timeout_policy: Str = "block"
}

Trait YieldReflection = {
  kind: Str = "reflection"
  task: Any
  attempt: Any
  question: Str
  context: Any = None
}

Trait YieldInformation = {
  kind: Str = "information"
  query: Str
  context: Any = None
  format: Str = "text"
}

Trait YieldDelegation = {
  kind: Str = "delegation"
  task: Any
  constraints: Any = None
  deadline: Any = None
}

Trait YieldProgress = {
  kind: Str = "progress"
  stage: Str
  pct: Float
  message: Str = ""
}
```

## Typed Yield Syntax

```
plan = yield YieldApproval {
  action: "execute deployment"
  details: {steps: proposed  risk: "medium"}
}

guidance = yield YieldReflection {
  task: original_task
  attempt: failed_result
  question: "My bottom-up approach scored 15/100. What should I change?"
}
```

The Trait validates the yield payload before serialization. The orchestrator receives `{"__yield": {"kind": "approval", "action": "execute deployment", ...}}` and can dispatch on `kind` without inspecting arbitrary fields.

## Orchestrator Trait Extension

The `{"__yield": {...}}` envelope gains a guaranteed `kind` field when a typed yield is used:

```json
{"__yield": {"kind": "approval", "action": "execute deployment", "details": {...}}}
{"__yield": {"kind": "reflection", "task": {...}, "attempt": {...}, "question": "..."}}
{"__yield": {"kind": "information", "query": "API key for service X"}}
```

Untyped `yield expr` still works — produces `{"__yield": <json>}` without a `kind` field. Backwards compatible.

## Response Traits

Each yield kind has an expected response shape:

| Yield Kind | Expected Response |
|------------|-------------------|
| `approval` | `{approved: Bool  reason: Str?}` |
| `reflection` | `{guidance: Str  approach: Str?  constraints: Any?}` |
| `information` | `{answer: Any}` or `{error: Str}` |
| `delegation` | `{result: Any}` or `{error: Str}` |
| `progress` | `{ack: Bool}` (or no response — fire-and-forget) |

Response validation is optional — if the response doesn't match the expected shape, it's returned as-is (the agent handles the mismatch).

## Integration with `agent.gate`

`agent.gate` already uses yield internally. With typed yields, it becomes a thin wrapper:

```
agent.gate action details ^
-- desugars to:
yield YieldApproval {action  details  timeout_policy: gate_policy} ^
```

## Integration with `meta` Block

The `meta` block can yield for reflection when all strategies fail:

```
result = meta task {
  strategies: [...]
  attempt: ...
  evaluate: ...
  on_exhausted: (attempts) {
    guidance = yield YieldReflection {
      task: task
      attempt: attempts | max_by (.quality)
      question: "All {len attempts} strategies failed. What should I try?"
    }
    guidance.approach
  }
}
```

## Implementation

### Parser

No parser change needed — `yield` already takes any expression. `yield YieldApproval {...}` is `yield (YieldApproval {...})` — Trait application followed by yield.

### YieldBackend

The `StdinStdoutYieldBackend` already serializes any Value to JSON. Typed yields produce records with a `kind` field — no backend change needed. The typing happens at the lx level (Trait validation), not the backend level.

### Standard Traits

The Yield Traits are defined in a new module `std/yield`:

```
use std/yield {YieldApproval YieldReflection YieldInformation YieldDelegation YieldProgress}
```

`std/yield` is a Trait-only module — no functions, just Trait definitions.

## Cross-References

- Yield mechanism: [agents-advanced.md](agents-advanced.md)
- Agent gates (uses yield): [agents-gates.md](agents-gates.md)
- Meta block (yields on exhaustion): [agents-meta.md](agents-meta.md)
- Trait validation: [agents-protocol.md](agents-protocol.md)
