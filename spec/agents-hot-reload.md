# Agent Hot Reload

Allow running agents to update their handler functions and capabilities at runtime without restart.

## Problem

If an agent learns (via `refine`, `std/profile`, or orchestrator feedback) that a different approach works better, it can store that knowledge — but it can't change its own behavior. The handler function is fixed at spawn time. To change behavior, you kill the agent and spawn a new one, losing all in-process state (dialogue sessions, in-flight messages, interceptors).

`meta` block (planned, Tier 3) addresses strategy-level iteration — trying fundamentally different approaches within a single expression. But `meta` doesn't help a long-lived agent that needs to evolve its handler over its lifetime.

What's needed: a running agent can swap its handler, add/remove capabilities, and update its trait conformance — while preserving its identity, dialogues, and interceptors.

## Design

### `agent.reload` — Replace Handler

```lx
use std/agent

worker = agent.spawn {command: "lx" args: ["run" "worker.lx"]} ^

agent.reload worker {
  handler: new_handler_fn
} ^
```

For in-process agents (record agents, not subprocess agents):

```lx
reviewer = {
  name: "reviewer"
  __traits: ["Reviewer"]
  handler: (msg) basic_review msg
}

agent.reload reviewer {
  handler: (msg) thorough_review msg
}
```

### `agent.evolve` — Conditional Self-Update

For agents that want to update themselves based on accumulated experience:

```lx
use std/agent
use std/profile

handler = (msg) {
  result = current_approach msg ^

  profile.record "review" {input: msg output: result}
  stats = profile.stats "review" ^

  stats.success_rate < 0.7 ? true -> {
    new_approach = select_better_approach stats ^
    agent.evolve {handler: new_approach}
  }

  result
}
```

`agent.evolve` updates the calling agent's own handler. Only callable from within an agent handler. Takes effect on the NEXT message — the current message completes with the old handler.

### `agent.update_traits` — Modify Capabilities

```lx
agent.update_traits worker {
  add: ["SecurityReviewer"]
  remove: ["BasicReviewer"]
} ^
```

Updates the agent's `__traits` list and re-validates trait conformance. If the new handler doesn't satisfy a newly-added trait, returns `Err`.

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `agent.reload` | `(agent: Agent opts: Record) -> Result () Str` | Replace agent handler externally |
| `agent.evolve` | `(opts: Record) -> Result () Str` | Self-update handler (from within handler) |
| `agent.update_traits` | `(agent: Agent changes: Record) -> Result () Str` | Add/remove trait conformance |

### Reload Options

```lx
agent.reload worker {
  handler: new_fn           -- new handler function (required)
  preserve_dialogues: true  -- keep active dialogue sessions (default true)
  preserve_intercepts: true -- keep interceptor chain (default true)
  on_reload: (old new) {    -- transition callback
    log.info "reloaded {agent.name worker}"
  }
}
```

### Constraints

- **Subprocess agents**: `agent.reload` on a subprocess agent returns `Err "cannot reload subprocess agent"`. Subprocess agents must be killed and respawned. This is a fundamental limitation — the subprocess runs its own code.
- **In-flight messages**: Messages currently being processed complete with the old handler. Only the next message uses the new handler.
- **Interceptors**: Preserved by default. The interceptor chain wraps whatever the current handler is.
- **Trait validation**: If `preserve_intercepts` is true and the new handler changes the response shape, existing interceptors that depend on the old shape may fail. The reload itself succeeds but the next message may error.

### Integration

- `std/profile` — profile stores accumulated strategy outcomes. `agent.evolve` uses profile data to decide when and how to update.
- `refine` — refine iterates within a single call. `evolve` changes behavior across calls. Complementary.
- `Trait` declarations — `update_traits` re-validates against existing Trait definitions.
- `Agent` declarations (planned) — declared agents could include an `evolve:` policy (e.g., `evolve: {on: "low_score" strategy: profile_based}`).
- `agent.supervise` — supervision restarts preserve the latest handler (not the original).

## Implementation

Agent extension (sub-module of `std/agent`). For in-process record agents: handler field is stored behind `Arc<RwLock<>>`, reload swaps the inner value. `evolve` sets a thread-local flag that the agent dispatch loop checks after handler return.

Approximately 80 lines of Rust.

No parser changes. No new keywords.

## Priority

Tier 3. Enables adaptive long-lived agents but not critical path — most flows use short-lived agents. Benefits multiply once `std/profile` (Tier 1) ships, giving agents data to drive evolution decisions. No parser changes.
