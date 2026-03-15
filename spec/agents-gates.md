# Approval Gates

Structured human-in-the-loop checkpoints that pause execution, present state, and wait for approval. More structured than `yield` (which is generic orchestrator communication) — gates are specifically "here's what I'm about to do, approve or reject."

## Problem

`yield` is generic: send any value, get any response. This generality makes it hard to build standard tooling around approval workflows. An orchestrator reading `yield {action: "deploy"}` doesn't know whether it's a question, a status update, or an approval request without parsing the value.

Automated pipelines need explicit gates where humans inject judgment. These gates need: structured presentation, timeout policies, escalation paths, and audit trails.

## `gate` — Approval Function

```
use std/agent

gate_result = agent.gate "deploy to production" {
  show: {diff: changes tests: test_results risk: "medium"}
  timeout: 300
  on_timeout: :abort
}
```

`agent.gate` is a library function in `std/agent`. It blocks until the approver responds.

### Parameters

| Field | Type | Description |
|-------|------|-------------|
| name | Str | Human-readable gate name (first positional arg) |
| show | Record | Data to present to the approver |
| timeout | Int | Seconds to wait. 0 = wait forever. Default: 0 |
| on_timeout | Symbol | `:abort`, `:approve`, `:reject`, `:escalate`. Default: `:abort` |
| approvers | [Str] | Who can approve. Default: `["human"]` |

### Return Value

```
Protocol GateResult = {
  approved: Bool
  approver: Str
  reason: Str = ""
  timestamp: Str
}
```

Returns `Result GateResult GateErr`. Compose with `^`:

```
agent.gate "delete user data" {
  show: {users: user_ids count: user_ids | len}
  timeout: 600
  on_timeout: :abort
} ^
```

### Timeout Policies

**`:abort`** — Gate returns `Err "gate_timeout"`. The caller handles the timeout.

**`:approve`** — Gate auto-approves on timeout. Use for low-risk operations where human review is preferred but not required.

**`:reject`** — Gate auto-rejects on timeout. Use for high-risk operations that should fail-safe.

**`:escalate`** — Gate sends a second notification (e.g., to a different approver or channel) and resets the timeout. If the escalation also times out, falls back to `:abort`.

### Runtime Behavior

Three modes (matching `yield` and `emit` patterns):

1. **Standalone** (`lx run`) — Prints gate info to stdout, reads approval from stdin (`y`/`n`).
2. **Orchestrated** — Calls `GateHandler` callback. The host decides how to present and collect approval (Slack message, web UI, etc.).
3. **Subprocess** — Sends JSON-line `{"type":"gate","name":"...","show":{...}}` to stdout. Reads `{"approved":true/false,"reason":"..."}` from stdin.

### Chaining Gates

```
agent.gate "code review" {show: {diff}} ^
agent.gate "security review" {show: {diff security_scan}} ^
agent.gate "deploy approval" {show: {diff env: "production"}} ^
deploy ()
```

Each gate blocks independently. If any gate is rejected, `^` propagates the error.

### With Checkpoint

```
checkpoint "deployment" {
  agent.gate "deploy" {show: deploy_plan} ^
  deploy () ^
  healthy = health_check () ^
  healthy ? {
    false -> rollback "deployment"
    true  -> Ok "deployed"
  }
}
```

Gate + checkpoint = approved operations that can be rolled back.

### Audit Trail

Every gate records its result in the introspection action log (`std/introspect`):

```
{type: "gate" name: "deploy" approved: true approver: "human" timestamp: "..."}
```

This integrates with `std/trace` for compliance and post-hoc review.

## Cross-References

- Yield (generic orchestrator communication): [agents-advanced.md](agents-advanced.md)
- Checkpoint/rollback: [agents-advanced.md](agents-advanced.md)
- Introspection action log: [stdlib-introspect.md](stdlib-introspect.md)
- Trace collection: stdlib_roadmap (`std/trace`)
