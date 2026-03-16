# Agent Testing Helpers

Mock agents and call-tracking utilities for testing agent interactions. Lightweight helpers in `std/agent`, not a separate module — agent test scenarios are written as regular lx test code using `assert`.

## Problem

RuntimeCtx lets you swap backends (mock AI, mock HTTP). `lx test` runs `.lx` files with `assert`. But testing agent interactions requires mock agents that return scripted responses and track what they received:

```
// Currently: manual mock construction
mock_handler = (msg) {
  msg.task == "review" ? {
    true -> {approved: true score: 0.9}
    false -> {error: "unexpected"}
  }
}
agent = agent.spawn {handler: mock_handler}
// No way to assert what messages were received, in what order
```

## `agent.mock`

Create an agent with scripted responses and call tracking:

```
use std/agent

mock_reviewer = agent.mock [
  {match: {task: "review"} respond: {approved: true score: 0.9}}
  {match: (msg) msg.priority == :critical respond: {approved: false reason: "needs human"}}
  {match: :any respond: {error: "unexpected message"}}
]

result = mock_reviewer ~>? {task: "review" code: diff} ^
assert (result.approved) "should approve"
```

### Rule Format

| Field | Type | Description |
|-------|------|-------------|
| `match` | Record, Fn, or `:any` | Pattern to match incoming message |
| `respond` | Any or Fn | Response value, or `(msg) -> response` for dynamic responses |
| `times` | Int | How many times this rule fires. Default: unlimited |

Rules are evaluated in order (first match wins). Record patterns match if all specified fields match (extra fields ignored).

## Call Tracking

```
calls = agent.mock_calls mock_reviewer ^
// => [{msg: {task: "review" code: "..."} response: {approved: true ...}}]

assert (calls | len == 1) "expected 1 call"

agent.mock_assert_called mock_reviewer {task: "review"} ^
agent.mock_assert_not_called mock_reviewer {task: "delete"} ^
```

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `agent.mock` | `[Rule] -> Agent` | Create mock agent with scripted responses |
| `agent.mock_calls` | `Agent -> [{msg response}]` | Get call history |
| `agent.mock_assert_called` | `Agent Record -> () ^ AssertErr` | Assert a matching call was made |
| `agent.mock_assert_not_called` | `Agent Record -> () ^ AssertErr` | Assert no matching call was made |

## Writing Agent Tests

Agent test scenarios are regular lx test code. No special DSL needed — `each` + `assert` handles interaction sequences:

```
// tests/agents/test_router.lx

use std/agent

mock_security = agent.mock [
  {match: :any respond: {findings: ["sql injection"]}}
]
mock_perf = agent.mock [
  {match: :any respond: {findings: ["slow query"]}}
]

// Test routing
router = create_router {security: mock_security perf: mock_perf}

result = router ~>? {domain: "security" code: "SELECT * FROM users"} ^
assert (result.findings | len > 0) "should route to security"

agent.mock_assert_called mock_security {domain: "security"} ^
agent.mock_assert_not_called mock_perf {domain: "security"} ^
```

### Multi-step scenario

```
steps = [
  {send: {task: "classify" text: "auth bypass"} expect: (r) r.category == "security"}
  {send: {task: "review"} expect: (r) r.findings | len > 0}
  {send: {task: "summarize"} expect: (r) r.type == "summary"}
]

steps | each (step) {
  result = handler step.send
  assert (step.expect result) "step failed: {step.send.task}"
}
```

## Implementation

Helper functions added to `std/agent` in `stdlib/agent.rs`. `agent.mock` returns a handler function that matches messages against rules and records calls in a `Vec<Value>` behind a `Mutex`. No new module needed.

## Cross-References

- Test runner: [toolchain.md](toolchain.md) (`lx test`)
- Agent communication: [agents.md](agents.md) (`~>?` semantics)
- RuntimeCtx: [runtime-backends.md](runtime-backends.md) (backend mocking — complementary)
