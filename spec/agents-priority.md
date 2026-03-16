# Message Priority

Binary priority on agent messages: critical or normal. Without priority, a cancel signal sits behind 50 status updates in an agent's message queue.

## Problem

All `~>` and `~>?` messages are processed FIFO. In multi-agent systems:

- A supervisor sends a `:cancel` to a busy worker — the worker processes 50 pending tasks before seeing the cancel
- An urgent security alert from a monitor agent waits behind routine status checks

## Two Levels

| Level | Symbol | Use Case |
|-------|--------|----------|
| Critical | `:critical` | Cancel, kill, emergency stop. Checked before normal processing. |
| Normal | (default) | Everything else. |

Four-level priority (`:high`, `:low`) was considered and rejected — the implementation complexity (per-agent priority queues, level configuration) doesn't justify the benefit. The only case that genuinely needs priority is "stop what you're doing."

## Syntax

Priority is a field on the message record, recognized by the runtime:

```
agent ~> {type: "cancel" _priority: :critical}
agent ~>? {task: "review"} ^
```

The `_priority` field is:
- Prefixed with `_` to signal it's metadata, not application data
- Stripped from the message before delivery to the handler
- Defaults to normal if not specified
- Only `:critical` is a valid non-default value

## Processing

`:critical` messages do not preempt mid-handler execution. Instead, long-running handlers check for critical messages:

```
handler = (msg) {
  items | each (item) {
    agent.check_critical () ? {
      Some cancel_msg -> break (Err "cancelled")
      None -> process item
    }
  }
}
```

`agent.check_critical` returns `Some msg` if a critical message is pending, `None` otherwise. Non-blocking.

## With Ambient Context

Priority integrates with ambient context deadline propagation:

```
with context deadline: 10 {
  agent ~>? {task: "review"} ^
  // If 95%+ of deadline elapsed, message sent as :critical
}
```

## With Supervision

Supervisors send restart/stop commands as `:critical`:

```
child ~> {type: "stop" reason: "supervisor_shutdown" _priority: :critical}
```

## Implementation Notes

For subprocess agents, the priority field is included in the JSON-line protocol:

```
{"type":"message","priority":"critical","value":{...}}
```

For in-process agents, `agent.check_critical` polls a single-slot buffer. No priority queue needed — just a flag + one pending critical message.

## Cross-References

- Agent communication: [agents.md](agents.md)
- Ambient context: [agents-ambient.md](agents-ambient.md)
- Supervision: [agents-supervision.md](agents-supervision.md)
