# std/introspect — Agent Introspection

An agent should be able to ask: "what have I tried?" "am I repeating myself?" "how much budget have I consumed?" `std/introspect` exposes runtime metadata about the current agent's identity, capabilities, resource consumption, and action history.

## Why Not `std/circuit`

`std/circuit` (planned) is a mechanism — it fires when limits are breached. It doesn't let the agent reason about its own state. An agent using `std/circuit` gets stopped; an agent using `std/introspect` can proactively change strategy before hitting a limit.

```
use std/introspect

budget = introspect.budget ()
budget.remaining < budget.total * 0.2 ? {
  true -> switch_to_lightweight_approach ()
  false -> continue_thorough_analysis ()
}
```

## API

### Identity

```
introspect.self ()                 -- {name: Str  role: Str  pid: Int}
introspect.parent ()               -- Maybe {name: Str  pid: Int}
introspect.capabilities ()         -- Capabilities record (tools, fs, network, budget)
```

`introspect.self` returns the current agent's identity as set by `agent.spawn` config. In standalone mode (not a subprocess), returns `{name: "main" role: "main" pid: process_pid}`.

### Resource Consumption

```
introspect.budget ()               -- {total: Int  spent: Int  remaining: Int}
introspect.elapsed ()              -- Duration (wall-clock since agent start)
introspect.turn_count ()           -- Int (number of yield/dialogue turns)
```

Budget tracks token spend if `capabilities.budget.tokens` was set. Without a budget cap, `total` and `remaining` are -1 (unlimited).

### Action History

```
introspect.actions ()              -- [{type: Str  target: Str  time: Str  result: Str}]
introspect.actions_since marker    -- [{...}] (actions after a named marker)
introspect.mark name               -- () (place a named marker in history)
```

Actions are tool calls, agent messages, file reads/writes, and shell commands. Each entry records the type (`"mcp_call"`, `"agent_ask"`, `"fs_read"`, `"shell"`), target (tool name, agent name, path, command), timestamp, and result summary (truncated).

### Strategy

```
introspect.is_stuck ()             -- Bool (heuristic: last N actions are repetitive)
introspect.strategy_shift reason   -- () (mark a pivot point in action history)
introspect.similar_actions n       -- Int (count of similar actions in last n)
```

`is_stuck` uses a simple heuristic: if the last 5 actions have the same type and target, the agent is likely stuck. `strategy_shift` inserts a marker that resets the stuck detector — the agent is explicitly trying something new.

## Usage Patterns

### Adaptive Strategy

```
use std/introspect

review = (path) {
  introspect.mark "start_review"

  result = analyze path ^
  result.issues | empty? ? {
    true -> {
      introspect.is_stuck () ? {
        true -> {
          introspect.strategy_shift "switching to broader search"
          analyze (parent_dir path) ^
        }
        false -> result
      }
    }
    false -> result
  }
}
```

### Budget-Aware Processing

```
use std/introspect

process_items = (items) {
  items | map (item) {
    budget = introspect.budget ()
    budget.remaining < 500 ? {
      true -> {summary: "skipped — budget low" item: item.name}
      false -> full_analysis item ^
    }
  }
}
```

### Auto-Populating Handoff

Introspection data can feed into structured handoff:

```
use std/introspect
use std/agent

build_handoff = (result) {
  actions = introspect.actions ()
  Handoff {
    result
    tried: actions | filter (.type == "mcp_call") | map (.target) | uniq
    files_read: actions | filter (.type == "fs_read") | map (.target) | uniq
    tools_used: actions | filter (.type == "mcp_call") | map (.target) | uniq
    duration_ms: introspect.elapsed () | time.to_ms
  }
}
```

## Runtime Model

Introspection data is collected by the interpreter as a side effect of evaluation. Each tool call, agent message, file operation, and shell command appends to a per-agent action log. The log is bounded (last 1000 actions by default) to prevent unbounded memory growth.

In standalone mode, all introspection functions work — the "agent" is the main process.

## Implementation Status

Planned. Requires interpreter-level action logging.

## Cross-References

- Circuit breakers (complementary): [stdlib_roadmap.md](../design/stdlib_roadmap.md#stdcircuit)
- Structured handoff: [agents-handoff.md](agents-handoff.md)
- Agent capabilities: [stdlib-agents.md](stdlib-agents.md#capability-attenuation)
- Trace collection (external observation): [stdlib_roadmap.md](../design/stdlib_roadmap.md#stdtrace)
