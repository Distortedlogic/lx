# Deadlock Detection

Runtime detection of circular `~>?` waits in multi-agent systems. When agent A waits for B and B waits for A, the system detects the cycle and breaks it with a diagnostic error instead of hanging silently.

## Problem

With `~>?` (synchronous ask), deadlocks are possible:

```
// Agent A's handler:
handler_a = (msg) -> agent_b ~>? {need: "data from B"} ^

// Agent B's handler:
handler_b = (msg) -> agent_a ~>? {need: "data from A"} ^

// Deadlock: A waits for B, B waits for A
agent_a ~>? {start: true}  // hangs forever
```

This gets worse with:
- `caller` (inline clarification creates bidirectional message flows)
- Chains: A→B→C→A (indirect cycles)
- Conditional cycles (only deadlock on certain code paths)

Currently, the only protection is `timeout` on `~>?`, which gives a generic timeout error with no diagnostic information about the cycle.

## Wait-For Graph

The runtime maintains a lightweight wait-for graph:

```
// Internal (not user-visible):
// When agent A sends ~>? to agent B:
//   wait_graph.add_edge(A, B)
// When B responds:
//   wait_graph.remove_edge(A, B)
// Before adding an edge, check for cycle:
//   if wait_graph.has_cycle_through(A, B) -> DeadlockErr
```

### Detection

Before every `~>?` send, the runtime checks if adding this wait would create a cycle. If yes, the send fails immediately with a `DeadlockErr`:

```
result = agent_b ~>? msg
// If this would create a cycle:
// Err (DeadlockErr {
//   cycle: ["agent_a" "agent_b" "agent_a"]
//   initiator: "agent_a"
//   message: "deadlock detected: agent_a -> agent_b -> agent_a"
// })
```

### Error Type

```
Trait DeadlockErr = {
  type: Str = "deadlock"
  cycle: [Str]
  initiator: Str
  message: Str
}
```

The `cycle` field shows the full chain: `["A" "B" "C" "A"]` for a 3-agent cycle.

## Handling Deadlocks

Since `DeadlockErr` is a normal error, it composes with `^` and `??`:

```
result = agent_b ~>? msg ?? {
  type: "deadlock" -> {
    emit "deadlock with {it.cycle | join " -> "}, using cached result"
    cached_result
  }
  _ -> Err it
}
```

### Retry with different agent

```
result = agent_b ~>? msg ?? {
  type: "deadlock" -> agent_c ~>? msg ^
}
```

### Break cycle by yielding

```
result = agent_b ~>? msg ?? {
  type: "deadlock" -> {
    partial = yield {type: "deadlock_break" cycle: it.cycle need: msg}
    partial
  }
}
```

## Scope

Detection covers:
- Direct `~>?` cycles (A↔B)
- Indirect cycles (A→B→C→A)
- `caller ~>?` cycles (clarification creating back-channel deadlocks)

Detection does NOT cover:
- `~>` (fire-and-forget) — can't deadlock, no wait
- External system waits (HTTP, shell) — out of scope
- Resource contention (file locks) — different problem

## Configuration

Deadlock detection is on by default. Can be disabled for performance in trusted workflows:

```
with deadlock_detection: false {
  // No cycle checking on ~>? in this scope
  agent_b ~>? msg ^
}
```

## Implementation

The wait-for graph is a `HashMap<AgentId, AgentId>` in the interpreter, tracking which agent is currently waiting for which. Cycle detection is a simple DFS from the target agent back through the graph — O(N) where N is the number of active waits (typically small). The graph is updated on every `~>?` entry and exit.

Since `par` is currently sequential, true concurrent deadlocks can't occur yet. But the detection infrastructure should be built now so it works correctly when real async lands. In sequential mode, a deadlock manifests as: A calls B, B's handler calls A, A is already blocked → cycle detected immediately.

## Cross-References

- Agent communication: [agents.md](agents.md) (`~>?` is the operation that can deadlock)
- Structured clarification: [agents-clarify.md](agents-clarify.md) (`caller` creates bidirectional flows)
- Supervision: [agents-supervision.md](agents-supervision.md) (supervisor can restart deadlocked agents)
- Ambient context: [agents-ambient.md](agents-ambient.md) (deadline timeout is the fallback if detection misses)
- Circuit breakers: ROADMAP (`std/circuit` — timeout is a coarser deadlock escape)
