# Multi-Agent Negotiation

Iterative multi-agent consensus building. Multiple agents propose, critique, and revise positions across rounds until convergence. Distinct from `agent.dialogue` (two-party multi-turn sessions) and `agent.reconcile` (post-hoc merging of completed results).

## Problem

`agent.reconcile` merges results AFTER all agents finish independently — no agent sees another's output. `agent.dialogue` is a two-party conversation. Neither covers the pattern where N agents need to converge on a shared decision before proceeding:

- 3 reviewers need to agree on an architecture before building
- A security analyst and a performance analyst need to align on trade-offs
- Multiple planners need to merge their plans into one coherent plan

The key difference: agents see each other's positions and revise in response.

## `agent.negotiate`

```
use std/agent

consensus = agent.negotiate [architect security perf] {
  proposal: design_doc
  trait: ArchDecision
  max_rounds: 5
  converge: (responses) {
    all_approved = responses | all? (.approved)
    all_approved ? {true -> Ok (merge_feedback responses); false -> :continue}
  }
} ^
```

### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agents` | [Agent] | Yes | Agents participating in negotiation (first arg) |
| `proposal` | Any | Yes | Initial proposal sent to all agents |
| `protocol` | Trait | No | Validates all messages |
| `max_rounds` | Int | No | Maximum negotiation rounds (default: 3) |
| `converge` | Fn | Yes | `(responses) -> Ok result \| :continue` |
| `on_round` | Fn | No | `(round responses) -> ()` callback per round |

### Negotiation Loop

1. Send `{round: 1  proposal  positions: []}` to all agents
2. Collect responses
3. Call `converge(responses)` — if `Ok result`, return result
4. Send `{round: N  proposal  positions: responses}` to all agents (each sees all positions)
5. Repeat until convergence or `max_rounds`
6. If `max_rounds` exceeded, return `Err {reason: "no_consensus"  rounds  final_positions}`

Each round, every agent receives the full set of positions from the previous round. Agents can revise their position based on others' feedback.

### Return Value

```
Ok {
  result: merged_decision
  rounds: 3
  positions: [{agent: "architect"  position: ...} ...]
  unanimous: true
}
```

Or on failure:

```
Err {
  reason: "no_consensus"
  rounds: 5
  positions: [...]
}
```

## Agent Handler Pattern

Agents participating in negotiation handle a standard message shape:

```
Trait NegotiationMsg = {
  round: Int
  proposal: Any
  positions: List
}

handler = (msg) {
  my_review = evaluate msg.proposal
  others = msg.positions
  revised = others | fold my_review (pos acc) {
    incorporate_feedback pos acc
  }
  {approved: revised.score > 0.8  position: revised  feedback: revised.notes}
}
```

## Convergence Functions

Common convergence patterns:

```
-- Unanimous approval
unanimous = (responses) {
  (responses | all? (.approved)) ? {true -> Ok responses; false -> :continue}
}

-- Majority approval
majority = (responses) {
  approved = responses | filter (.approved)
  (len approved) * 2 > (len responses) ? {true -> Ok approved; false -> :continue}
}

-- Score threshold (average score above bar)
score_bar = (threshold) (responses) {
  avg = responses | map (.score) | fold 0 (+) | (s) s / (len responses)
  avg >= threshold ? {true -> Ok {score: avg  positions: responses}; false -> :continue}
}
```

## Relationship to Existing Features

| Feature | Scope | When |
|---------|-------|------|
| `agent.reconcile` | N agents, post-hoc | After all agents finish independently |
| `agent.dialogue` | 2 agents, multi-turn | Sequential conversation |
| `agent.negotiate` | N agents, iterative | Agents see each other's positions, revise |

`negotiate` fills the gap between "merge independently-produced results" and "have a conversation." It's the multi-party consensus mechanism.

## Implementation

Extension to `std/agent` following the agent extension pattern. New file: `stdlib/agent_negotiate.rs`.

The negotiation loop is synchronous (like all current concurrency): agents are queried in sequence per round. Each round collects all responses, then feeds them back. Real concurrent querying requires async (`tokio`).

## Cross-References

- Agent reconciliation (post-hoc merging): [agents-reconcile.md](agents-reconcile.md)
- Agent dialogue (two-party sessions): [agents-dialogue.md](agents-dialogue.md)
- Trait validation: [agents-protocol.md](agents-protocol.md)
- Agent pools (negotiate across pool workers): [agents-pool.md](agents-pool.md)
