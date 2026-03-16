# Content-Addressed Dispatch — Reference

## `agent.dispatch`

Declarative pattern-based message routing. No LLM call — deterministic predicate matching.

```
use std/agent

dispatcher = agent.dispatch [
  {match: {domain: "security"} to: security_agent}
  {match: {priority: :critical} to: fast_agent}
  {match: (msg) msg.files | any (f) f | ends_with ".rs" to: rust_agent}
  {match: :default to: general_agent}
]

result = dispatcher ~>? incoming_message ^
```

### Rule Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `match` | Record, Fn, or `:default` | Yes | Predicate on incoming message |
| `to` | Agent or Fn | Yes | Destination agent or handler |
| `transform` | Fn | No | `(msg) -> msg'` — transform before sending |
| `when` | Fn | No | Additional guard: `(msg) -> Bool` |

### Match Types

- **Record pattern** — `{domain: "security"}` — matches if all specified fields match (extra fields ignored)
- **Function predicate** — `(msg) -> Bool` — arbitrary matching logic
- **`:default`** — always matches (must be last)

Rules evaluate in order — first match wins. No match and no `:default` returns `Err {type: "no_route" message: msg}`.

The dispatcher itself is an agent — use `~>` / `~>?` like any other agent.

## `agent.dispatch_multi`

Fan-out to ALL matching routes (not just first). All matching rules fire in parallel.

```
multi = agent.dispatch_multi [
  {match: {needs_security: true} to: security_agent}
  {match: {needs_perf: true} to: perf_agent}
  {match: :default to: general_agent}
]

results = multi ~>? {needs_security: true needs_perf: true task: "review"} ^
-- => [{agent: "security" result: ...} {agent: "perf" result: ...}]
```

## Dynamic Table Functions

```
agent.dispatch_add dispatcher {match: {domain: "ml"} to: ml_agent} ^
agent.dispatch_remove dispatcher "ml" ^
rules = agent.dispatch_rules dispatcher ^
```

| Function | Description |
|----------|-------------|
| `agent.dispatch_add` | Append rule (inserted before `:default` if one exists) |
| `agent.dispatch_remove` | Remove rules matching a domain name or predicate |
| `agent.dispatch_rules` | Return current routing table as inspectable list |
