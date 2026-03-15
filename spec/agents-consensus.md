# Consensus / Quorum Protocols

Multi-agent voting and agreement with configurable quorum policies. Distinct from dialogue (two-party), saga (transactions), and fan-out + collect (structural). Consensus is about *semantic agreement* among N agents on a decision.

## Problem

Real agentic workflows need multi-reviewer agreement:
- Code review: 3 reviewers, majority must approve
- Risk assessment: all security agents must agree "safe"
- Research: multiple searchers vote on best finding

Currently this is hand-rolled:

```
results = par {
  | reviewer_a ~>? task ^
  | reviewer_b ~>? task ^
  | reviewer_c ~>? task ^
}
approvals = results | filter (.approved)
passed = (len approvals) >= 2
```

This misses: deliberation (agents seeing each other's reasoning), configurable quorum types, structured disagreement, and the common pattern of "discuss then re-vote."

## `consensus` Expression

```
decision = consensus [reviewer_a reviewer_b reviewer_c] {
  prompt: {task: "review" code: diff}
  quorum: :majority
  timeout: 60
}
```

`consensus` is a new keyword. It fans out a prompt to all agents, collects responses, and evaluates them against a quorum policy.

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `prompt` | Any | Yes | Message sent to all agents. |
| `quorum` | Symbol or `(n Int)` | Yes | `:unanimous`, `:majority`, `:any`, or `(n 2)` for specific count. |
| `timeout` | Int | Yes | Seconds to wait for all responses. |
| `vote_field` | Str | No | Field name to use as vote. Default: `"approved"` (Bool). |
| `deliberate` | Int | No | Number of deliberation rounds if quorum not met initially. Default: 0. |
| `on_vote` | Fn | No | Callback `(agent_id vote)` after each vote. |

### Return Value

```
Protocol ConsensusResult = {
  decided: Bool
  outcome: Any
  votes: [{agent: Str  vote: Any  reasoning: Str}]
  rounds: Int
  dissenting: [Str]
}
```

- `decided: true` — quorum was met. `outcome` is the majority/winning value.
- `decided: false` — timeout or deliberation exhausted. `outcome` is `()`. `dissenting` lists agents that didn't agree.

### Execution

1. Send `prompt` to all agents via `~>?` (parallel)
2. Collect responses. Each must have the `vote_field` (default `approved`) and optionally `reasoning`.
3. Evaluate against quorum policy
4. If met → return `ConsensusResult {decided: true ...}`
5. If not met and `deliberate > 0`:
   a. Send each agent a `{type: "deliberate" votes: all_votes round: N}` message
   b. Agents see all reasoning and can change their vote
   c. Re-evaluate quorum
   d. Repeat up to `deliberate` rounds
6. If still not met → return `ConsensusResult {decided: false ...}`

### Quorum Policies

| Policy | Meaning |
|--------|---------|
| `:unanimous` | All agents must agree |
| `:majority` | More than half must agree |
| `:any` | At least one agrees |
| `(n K)` | At least K agents must agree |

### Weighted Voting

```
decision = consensus agents {
  prompt: task
  quorum: :majority
  timeout: 60
  weight: (agent_id) -> agent_id == "senior" ? {true -> 2 false -> 1}
}
```

### Custom Vote Types

Not limited to boolean approve/reject. Agents can vote for any value:

```
decision = consensus agents {
  prompt: {task: "classify" text}
  quorum: (n 2)
  timeout: 30
  vote_field: "classification"
}
// decision.outcome = "security" if 2+ agents voted "security"
```

## Implementation

`consensus` desugars to `par` fan-out + quorum evaluation + optional deliberation loop. Library-level logic, keyword-level syntax.

### AST Node

```
Consensus {
  agents: Expr
  config: Record
}
```

## Shared Vote Tallying

The quorum evaluation logic (counting votes, checking majority/unanimous/threshold) must be shared with `agent.reconcile`'s `:vote` strategy. Both count occurrences of values and pick winners. Extract a shared `tally_votes` + `meets_quorum` utility so consensus and reconcile use the same counting code.

## Cross-References

- Parallel execution: [concurrency.md](concurrency.md)
- Agent communication: [agents.md](agents.md)
- Dialogue (two-party): [agents-dialogue.md](agents-dialogue.md)
- Capability discovery: [agents-capability.md](agents-capability.md) (filter agents before consensus)
- Approval gates: [agents-gates.md](agents-gates.md) (human approval vs agent consensus)
- Result reconciliation: [agents-reconcile.md](agents-reconcile.md) (`:vote` strategy shares tally logic)
