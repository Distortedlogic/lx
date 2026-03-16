# Workflow Status Broadcasting

Passive observability of sibling agent state within parallel workflows. Agents can see what their peers are working on without explicit publish/subscribe setup. Prevents duplicate work and enables opportunistic collaboration.

## Problem

In `par` blocks, sibling agents are invisible to each other:

```
par {
  | agent_a ~>? {task: "search crates.io for auth libraries"} ^
  | agent_b ~>? {task: "search github for auth libraries"} ^
}
```

Agent A discovers that `oauth2-rs` is the clear winner after 10 seconds, but agent B spends 50 more seconds searching GitHub for the same answer. Neither knows what the other found. This causes:

- Duplicate work (both agents discover the same thing)
- Missed opportunities (A's finding could refine B's search)
- Wasted budget (B uses tokens on redundant searches)

`std/blackboard` is opt-in write. `std/events` requires explicit publish. Neither provides *passive* visibility of what peers are doing.

## `workflow.peers` — Sibling Visibility

Extension to `std/agent`. Within a `par` block, agents can query the status of their siblings.

```
par {
  | {
    // Agent A does work, periodically checks peers
    result_a = do_search "crates.io" ^
    workflow.share {finding: result_a status: :found}
    result_a
  }
  | {
    // Agent B checks if anyone already found something
    peers = workflow.peers ()
    already_found = peers | filter (.status == :found)
    already_found | empty? ? {
      true  -> do_search "github" ^
      false -> refine_search (already_found | first | (.finding)) ^
    }
  }
}
```

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `workflow.peers` | `() -> [{id: Str status: Any progress: Float shared: Any}]` | List sibling agents in current `par` block with their shared state. |
| `workflow.share` | `Any -> ()` | Update this agent's shared state, visible to peers. |
| `workflow.on_peer_update` | `(peer_id state) -> () callback -> ()` | Register callback when a peer's shared state changes. |

### PeerState Record

```
Protocol PeerState = {
  id: Str
  status: Any
  progress: Float
  shared: Any
  started_at: Str
  elapsed_ms: Int
}
```

| Field | Description |
|-------|-------------|
| `id` | Auto-assigned branch identifier within `par` block (e.g., `"par_0"`, `"par_1"`). |
| `status` | Last value passed to `workflow.share`, or `:working` by default. |
| `progress` | Float 0.0-1.0 if agent calls `workflow.share {progress: 0.5 ...}`. |
| `shared` | Full shared state record. |
| `started_at` | ISO timestamp when this branch started. |
| `elapsed_ms` | Milliseconds since branch started. |

### Automatic Status

Even without explicit `workflow.share`, peers have basic visibility:

```
peers = workflow.peers ()
// [{id: "par_0" status: :working progress: 0.0 shared: {} started_at: "..." elapsed_ms: 1234}]
```

The runtime automatically tracks `:working` / `:complete` / `:failed` status.

## Usage Patterns

### Early termination on peer discovery

```
par {
  | search_approach_a task ^
  | search_approach_b task ^
  | {
    // Watchdog: if any peer finds a high-confidence result, signal others
    loop {
      peers = workflow.peers ()
      good = peers | filter (p) p.shared.confidence ?? 0.0 > 0.9
      good | empty? ? {
        false -> break (good | first | (.shared.result))
        true  -> $sleep 1
      }
    }
  }
}
```

### Progress aggregation

```
par {
  | { workflow.share {progress: 0.0}; r = step_a ^; workflow.share {progress: 1.0 result: r}; r }
  | { workflow.share {progress: 0.0}; r = step_b ^; workflow.share {progress: 1.0 result: r}; r }
  | {
    loop {
      peers = workflow.peers ()
      total = peers | map (.progress) | avg
      emit "overall progress: {total * 100}%"
      total >= 1.0 ? { true -> break () false -> $sleep 2 }
    }
  }
}
```

## Implementation

Built as a convenience layer on top of `std/blackboard`, not a separate `DashMap`. The `par` block interpreter creates a blackboard scoped to the block. `workflow.share` writes to a key namespaced by the branch ID (e.g., `"par_0"`). `workflow.peers` reads all branch-namespaced keys. `workflow.on_peer_update` delegates to `blackboard.watch` with a branch-key filter. This means `std/blackboard` is the single concurrency-safe shared-state primitive; peer visibility is a typed view over it with automatic status fields.

## Cross-References

- Parallel execution: [concurrency.md](concurrency.md) (`par` blocks where peers exist)
- Blackboard: ROADMAP (underlying primitive — peers is a convenience layer on top)
- Events: ROADMAP (explicit pub/sub — peers is automatic)
- Introspection: [stdlib-introspect.md](stdlib-introspect.md) (self-awareness; peers is sibling-awareness)
