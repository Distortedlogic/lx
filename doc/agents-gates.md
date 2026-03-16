# Approval Gates — Reference

## `agent.gate`

```
use std/agent

gate_result = agent.gate "deploy to production" {
  show: {diff: changes tests: test_results risk: "medium"}
  timeout: 300
  on_timeout: :abort
}
```

Blocks until the approver responds. Returns `Result GateResult GateErr`.

### Parameters

| Field | Type | Description |
|-------|------|-------------|
| name | Str | Human-readable gate name (first positional arg) |
| show | Record | Data to present to the approver |
| timeout | Int | Seconds to wait. 0 = wait forever. Default: 0 |
| on_timeout | Symbol | `:abort`, `:approve`, `:reject`, `:escalate`. Default: `:abort` |
| approvers | [Str] | Who can approve. Default: `["human"]` |

### GateResult Protocol

```
Protocol GateResult = {
  approved: Bool
  approver: Str
  reason: Str = ""
  timestamp: Str
}
```

### Timeout Policies

| Policy | Behavior |
|--------|----------|
| `:abort` | Returns `Err "gate_timeout"`. |
| `:approve` | Auto-approves. For low-risk operations. |
| `:reject` | Auto-rejects. For high-risk fail-safe. |
| `:escalate` | Sends second notification, resets timeout. Falls back to `:abort`. |

## Example: Chaining Gates

```
agent.gate "code review" {show: {diff}} ^
agent.gate "security review" {show: {diff security_scan}} ^
agent.gate "deploy approval" {show: {diff env: "production"}} ^
deploy ()
```

Each gate blocks independently. Rejection propagates via `^`.

## Example: Gate with Checkpoint

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
