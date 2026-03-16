# Agent Reputation / Learning Router

`std/reputation` tracks cross-interaction quality scores for agents. Audit outcomes feed back into routing decisions — agents that consistently fail at certain task types get deprioritized. Closes the loop between `std/agents/reviewer` (post-hoc analysis) and `std/agents/router` (task routing).

## Problem

Router classifies tasks via LLM, but doesn't learn from outcomes. Reviewer analyzes transcripts, but findings don't affect future routing. Every workflow starts fresh — no memory of which agent configurations work best for which task types.

```
agent = router ~>? {task: "security audit" prompt: desc} ^
result = agent ~>? task ^
grade = auditor ~>? {output: result task: desc} ^
// grade.passed is false — this agent is bad at security audits
// ...next time, router picks the same agent again
```

## `std/reputation`

### Core API

```
use std/reputation

rep = reputation.load "reputation.json" ^
```

Loads or creates a reputation store. File-backed JSON, same pattern as `std/knowledge`.

### Recording Outcomes

```
reputation.record rep {
  agent: "code-reviewer-v2"
  task_type: "security_audit"
  passed: grade.passed
  score: grade.score
  timestamp: time.now ()
} ^
```

Each record updates a running score for the (agent, task_type) pair. Score is an exponentially-weighted moving average — recent outcomes matter more than old ones.

### Querying Scores

```
score = reputation.score rep "code-reviewer-v2" "security_audit" ^
// => {score: 72.5  total: 15  recent: 5  trend: :declining}
```

- `score` — EWMA score (0-100)
- `total` — total interactions recorded
- `recent` — interactions in the last N (configurable, default 10)
- `trend` — `:improving`, `:stable`, `:declining` based on recent vs overall

### Best Agent for Task Type

```
best = reputation.best_for rep "security_audit" ^
// => {agent: "security-specialist" score: 94.2}

ranked = reputation.rank rep "security_audit" ^
// => [{agent: "security-specialist" score: 94.2} {agent: "code-reviewer-v2" score: 72.5} ...]
```

### Integration with Router

```
use std/reputation
use std/agents/router

rep = reputation.load "reputation.json" ^

ranked = reputation.rank rep task_type ^
ranked | first | (.score) > 80 ? {
  true -> ranked | first | (.agent)
  false -> router ~>? {task: task_type prompt: desc} ^
}
```

Use reputation when there's enough history. Fall back to LLM routing when there isn't.

### Decay

Scores decay over time. An agent that was good 100 interactions ago but hasn't been used recently has an uncertain reputation:

```
rep = reputation.load "reputation.json" {decay_half_life: 50} ^
```

`decay_half_life` is the number of interactions after which old scores contribute 50% weight. Default: 100.

### Minimum History

```
score = reputation.score rep agent task_type ^
score.total < 5 ? {
  true -> :insufficient_data
  false -> score.score > 80 ? {true -> :trusted  false -> :untrusted}
}
```

Don't trust scores with too few data points. `reputation.best_for` accepts a `min_history` parameter:

```
best = reputation.best_for rep "security_audit" {min_history: 5} ^
```

### With Consensus

```
agents = reputation.rank rep "code_review" {min_history: 3} ^
  | take 3
  | map (.agent)
  | map (name) agent.get name ^

decision = consensus agents {
  prompt: {task: "review" code: diff}
  quorum: :majority
  timeout: 60
}
```

Pick the top 3 agents by reputation for consensus voting.

## Implementation

`std/reputation` is a new stdlib module. Reputation data is a JSON file with structure:

```json
{
  "scores": {
    "agent_id::task_type": {
      "ewma": 85.2,
      "total": 23,
      "history": [{"score": 90, "ts": 1710000000}, ...]
    }
  }
}
```

EWMA with configurable alpha (derived from `decay_half_life`). History is bounded (last 100 entries per pair).

### Dependencies

- `serde_json` (file persistence)
- `std/time` (timestamps)
- `std/fs` (file I/O)

## Cross-References

- Router agent: ROADMAP (`std/agents/router`) — fallback when reputation insufficient
- Reviewer agent: ROADMAP (`std/agents/reviewer`) — outcomes feed reputation
- Auditor agent: ROADMAP (`std/agents/auditor`) — grade results feed reputation
- Knowledge base: [stdlib-knowledge.md](stdlib-knowledge.md) — same file-backed pattern
- Reconcile: [agents-reconcile.md](agents-reconcile.md) — reputation-informed agent selection for voting
