# Message Priority

Priority levels on agent messages for urgency-aware processing. Without priority, a critical cancel signal sits behind 50 status updates in an agent's message queue.

## Problem

All `~>` and `~>?` messages are processed FIFO. In high-throughput multi-agent systems, this causes problems:

- A supervisor sends a `:cancel` to a busy worker — the worker processes 50 pending tasks before seeing the cancel
- An urgent security alert from a monitor agent waits behind routine status checks
- A deadline-approaching task gets no priority over tasks with plenty of time

## Priority Levels

Four priority levels, from highest to lowest:

| Level | Symbol | Use Case |
|-------|--------|----------|
| Critical | `:critical` | Cancel, kill, emergency stop. Preempts current processing. |
| High | `:high` | Time-sensitive work, deadline-approaching tasks |
| Normal | `:normal` | Default. Regular agent communication. |
| Low | `:low` | Background tasks, optional work, status updates |

## Syntax

Priority is a field on the message record, recognized by the runtime:

```
agent ~> {type: "cancel" _priority: :critical}
agent ~>? {task: "review" _priority: :high} ^
agent ~> {type: "status" data: stats _priority: :low}
```

The `_priority` field is:
- Prefixed with `_` to signal it's metadata, not application data
- Stripped from the message before delivery to the handler (the handler sees the message without `_priority`)
- Defaults to `:normal` if not specified
- Validated at send time — invalid priority values produce `Err "invalid priority"`

## Processing Order

Agents process messages in priority order within each level, FIFO within the same level:

1. All `:critical` messages (FIFO among critical)
2. All `:high` messages (FIFO among high)
3. All `:normal` messages (FIFO among normal)
4. All `:low` messages (FIFO among low)

### Critical Preemption

`:critical` messages can preempt in-progress work. When a critical message arrives while an agent is processing a normal message:

- For subprocess agents: the critical message is delivered as a separate JSON-line. The subprocess must check for incoming messages between processing steps.
- For in-process agents: the critical message is queued and processed next (no mid-handler interruption).

Full preemption (interrupting a handler mid-execution) is not supported — it would require cooperative multitasking that complicates the runtime significantly. Instead, long-running handlers should periodically check for critical messages:

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
  -- As deadline approaches, runtime auto-escalates priority
  agent ~>? {task: "review"} ^
  -- If 80%+ of deadline elapsed, message sent as :high
  -- If 95%+ of deadline elapsed, message sent as :critical
}
```

Auto-escalation thresholds are configurable:

```
with context deadline: 30 priority_escalation: {high: 0.7 critical: 0.9} {
  agent ~>? msg ^
}
```

## With Supervision

Supervisors send restart/stop commands as `:critical`:

```
-- Internal to agent.supervise implementation:
child ~> {type: "stop" reason: "supervisor_shutdown" _priority: :critical}
```

## Implementation Notes

For subprocess agents, the priority field is included in the JSON-line protocol:

```
{"type":"message","priority":"critical","value":{...}}
```

The subprocess runtime maintains a priority queue (4 deques, one per level). The main loop checks `:critical` first on each iteration.

For in-process agents (record with handler), messages are dispatched immediately — priority only matters when messages are buffered (e.g., in a channel or during `par` execution).

## Cross-References

- Agent communication: [agents.md](agents.md)
- Ambient context: [agents-ambient.md](agents-ambient.md)
- Supervision: [agents-supervision.md](agents-supervision.md)
- Agent introspection: [stdlib-introspect.md](stdlib-introspect.md)
