# std/introspect — Reference

## API

### Identity

```
introspect.self ()                 -- {name: Str  role: Str  pid: Int}
introspect.parent ()               -- Maybe {name: Str  pid: Int}
introspect.capabilities ()         -- Capabilities record (tools, fs, network, budget)
```

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

Action types: `"mcp_call"`, `"agent_ask"`, `"fs_read"`, `"shell"`. Target is tool name, agent name, path, or command. Result is truncated.

### Strategy

```
introspect.is_stuck ()             -- Bool (heuristic: last 5 actions same type+target)
introspect.strategy_shift reason   -- () (mark a pivot point, resets stuck detector)
introspect.similar_actions n       -- Int (count of similar actions in last n)
```

## Gotchas

- In standalone mode (not a subprocess), `introspect.self` returns `{name: "main" role: "main" pid: process_pid}`.
- Action log is bounded to last 1000 actions per agent.

## Example — Budget-Aware Processing

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
