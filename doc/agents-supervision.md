# Agent Supervision — Reference

## `agent.supervise`

```
use std/agent

sup = agent.supervise {
  strategy: :one_for_one
  max_restarts: 5
  window: 60
  children: [
    {id: "analyzer"  spawn: () agent.spawn {command: "analyzer"} ^  restart: :permanent}
    {id: "formatter" spawn: () agent.spawn {command: "formatter"} ^ restart: :transient}
    {id: "logger"    spawn: () agent.spawn {command: "logger"} ^    restart: :temporary}
  ]
}
```

Returns a `Supervisor` value (opaque).

## Configuration

| Field | Type | Description |
|-------|------|-------------|
| `strategy` | Symbol | `:one_for_one`, `:one_for_all`, `:rest_for_one` |
| `max_restarts` | Int | Max restarts within `window` before supervisor gives up |
| `window` | Int | Time window in seconds for restart counting |
| `children` | List | Child specifications |

## Strategies

| Strategy | Behavior |
|----------|----------|
| `:one_for_one` | Only the crashed child is restarted. Use for independent children. |
| `:one_for_all` | All children terminated and restarted. Use for interdependent children. |
| `:rest_for_one` | Crashed child + all children started after it restarted. Use for startup-order deps. |

## Child Specification

| Field | Type | Description |
|-------|------|-------------|
| `id` | Str | Unique identifier within this supervisor |
| `spawn` | `() -> Agent` | Zero-arg function that creates the agent |
| `restart` | Symbol | `:permanent`, `:transient`, `:temporary` |

| Restart Policy | Behavior |
|---------------|----------|
| `:permanent` | Always restarted on crash. |
| `:transient` | Restarted only on abnormal exit (nonzero). |
| `:temporary` | Never restarted. |

## `agent.child` — Access Children

```
analyzer = agent.child sup "analyzer"
result = analyzer ~>? {task: "review" path: "src/"} ^
```

Returns the current agent instance for that child ID. Transparently handles restarts.

## `agent.supervise_stop` — Shutdown

```
agent.supervise_stop sup
```

Terminates all children (reverse start order) and the supervisor.

## Gotcha

If `max_restarts` exceeded within `window` seconds, supervisor fails with `Err {type: "supervisor_exhausted" id: child_id restarts: count}`.
