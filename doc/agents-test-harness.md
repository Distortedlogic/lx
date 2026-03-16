# Agent Test Harness — Reference

## `agent.mock`

Create a mock agent with scripted responses and call tracking:

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
| `respond` | Any or Fn | Response value, or `(msg) -> response` for dynamic |
| `times` | Int | How many times this rule fires. Default: unlimited |

Rules evaluate in order — first match wins. Record patterns match if all specified fields match (extra fields ignored).

## Call Tracking

| Function | Signature |
|----------|-----------|
| `agent.mock` | `[Rule] -> Agent` |
| `agent.mock_calls` | `Agent -> [{msg response}]` |
| `agent.mock_assert_called` | `Agent Record -> () ^ AssertErr` |
| `agent.mock_assert_not_called` | `Agent Record -> () ^ AssertErr` |

```
calls = agent.mock_calls mock_reviewer ^
assert (calls | len == 1) "expected 1 call"

agent.mock_assert_called mock_reviewer {task: "review"} ^
agent.mock_assert_not_called mock_reviewer {task: "delete"} ^
```

## Test Example

```
use std/agent

mock_security = agent.mock [
  {match: :any respond: {findings: ["sql injection"]}}
]
mock_perf = agent.mock [
  {match: :any respond: {findings: ["slow query"]}}
]

router = create_router {security: mock_security perf: mock_perf}
result = router ~>? {domain: "security" code: "SELECT * FROM users"} ^
assert (result.findings | len > 0) "should route to security"

agent.mock_assert_called mock_security {domain: "security"} ^
agent.mock_assert_not_called mock_perf {domain: "security"} ^
```
