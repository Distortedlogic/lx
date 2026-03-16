# Content-Addressed Message Routing

Declarative, pattern-based message dispatch to agents based on message content. No LLM call, no runtime classification — just predicate matching on message fields. Deterministic, composable, cheap.

## Problem

`std/agents/router` uses LLM classification to route tasks. This works for ambiguous natural-language prompts but is overkill (and expensive) when routing rules are known:

```
// Currently: LLM call for every routing decision
route = router ~>? {task: msg.task prompt: msg.description} ^
// Costs tokens, adds latency, non-deterministic
```

Many routing decisions are deterministic: security messages go to the security agent, high-priority messages go to the fast agent, messages with certain fields go to certain handlers. Pattern-based dispatch handles these cases without an LLM.

## `agent.dispatch`

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

### Parameters

`agent.dispatch` takes a list of routing rules and returns an agent (handler function). Each rule:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `match` | Record, Fn, or `:default` | Yes | Predicate on incoming message |
| `to` | Agent or Fn | Yes | Destination agent or handler |
| `transform` | Fn | No | `(msg) -> msg'` — transform message before sending |
| `when` | Fn | No | Additional guard: `(msg) -> Bool` |

### Match Types

- **Record pattern** — `{domain: "security"}` — matches if all specified fields match (extra fields ignored)
- **Function predicate** — `(msg) -> Bool` — arbitrary matching logic
- **`:default`** — always matches (must be last)

Rules evaluate in order. First match wins.

### Return Value

The dispatcher itself is an agent — it implements the standard handler interface. Callers interact with it via `~>` and `~>?` like any other agent.

The dispatched message returns whatever the target agent returns.

### No Match

If no rule matches and there's no `:default`, returns:

```
Err {type: "no_route" message: msg}
```

## `agent.dispatch_multi`

Fan-out to ALL matching routes (not just first):

```
multi = agent.dispatch_multi [
  {match: {needs_security: true} to: security_agent}
  {match: {needs_perf: true} to: perf_agent}
  {match: :default to: general_agent}
]

results = multi ~>? {needs_security: true needs_perf: true task: "review"} ^
// => [{agent: "security" result: ...} {agent: "perf" result: ...}]
```

All matching rules fire in parallel. Returns a list of `{agent result}` records.

## Dynamic Tables

Routing tables can be modified at runtime:

```
agent.dispatch_add dispatcher {match: {domain: "ml"} to: ml_agent} ^
agent.dispatch_remove dispatcher "ml" ^
rules = agent.dispatch_rules dispatcher ^
```

### `agent.dispatch_add`

Appends a rule. Inserted before `:default` if one exists.

### `agent.dispatch_remove`

Removes rules matching a domain name or predicate.

### `agent.dispatch_rules`

Returns the current routing table as a list of rule records (without handler functions — safe for logging/inspection).

## Composition with LLM Router

Pattern dispatch and LLM routing compose naturally:

```
dispatcher = agent.dispatch [
  {match: {domain: "security"} to: security_agent}
  {match: {domain: "performance"} to: perf_agent}
  {match: :default to: (msg) {
    route = router ~>? {task: msg.task} ^
    route.agent ~>? msg ^
  }}
]
```

Deterministic rules handle known patterns; the LLM router is the fallback for ambiguous cases. Saves tokens on the common paths.

## Composition with Intercept

Interceptors wrap individual agents. Dispatch wraps routing. They compose:

```
logged_security = agent.intercept security_agent logging_middleware
dispatcher = agent.dispatch [
  {match: {domain: "security"} to: logged_security}
  ...
]
traced_dispatcher = agent.intercept dispatcher tracing_middleware
```

## Implementation

Library function in `std/agent`. `agent.dispatch` returns a handler function that iterates rules, evaluates match predicates, and delegates to the first matching agent. `agent.dispatch_multi` uses `par`-style fan-out for all matches.

Routing table is a `Vec<DispatchRule>` behind a `Mutex` for dynamic modification.

## Cross-References

- Router agent: stdlib (`std/agents/router`) — LLM-based, complementary
- Intercept: [agents-intercept.md](agents-intercept.md) — per-agent middleware (composable)
- Agent communication: [agents.md](agents.md) — `~>?` semantics
- Priority: [agents-priority.md](agents-priority.md) — `_priority` field as a match criterion
- Capability: [agents-capability.md](agents-capability.md) — capabilities as routing criteria
