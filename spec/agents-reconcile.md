# Result Reconciliation

Structured merging of results from parallel agents. When fan-out produces N different answers, `reconcile` applies a merge strategy to produce a single coherent result. This is the single post-collection primitive — it subsumes voting (previously `consensus`), competitive selection (previously `speculate`), and all other result-combination patterns.

## Problem

Fan-out with `par` gives you a list of independent results. Merging them is always manual:

```
results = par {
  | searcher_a ~>? {query: "find vulnerabilities"} ^
  | searcher_b ~>? {query: "find vulnerabilities"} ^
  | searcher_c ~>? {query: "find vulnerabilities"} ^
}
all_findings = results | flat_map (.findings) | unique_by (.file)
```

This gets worse when results genuinely conflict (agent A says "safe," agent B says "vulnerable"). There's no standard way to handle voting, confidence-weighted selection, deduplication, deliberation, or competitive best-of-N.

## `reconcile` Function

```
use std/agent

merged = agent.reconcile results {
  strategy: :union
  key: (.id)
  conflict: (a b) -> a.confidence > b.confidence ? {true -> a false -> b}
}
```

`agent.reconcile` is a function in `std/agent` (not a keyword). It takes a list of results and a strategy configuration.

### Strategies

| Strategy | Description |
|----------|-------------|
| `:union` | Combine all items, deduplicate by `key`. On conflict use `conflict` function. |
| `:intersection` | Keep only items present in all results (by `key`). |
| `:vote` | Each result votes for a value. Most common value wins. Supports quorum. |
| `:highest_confidence` | Pick the result with highest `.confidence` field. |
| `:max_score` | Score all results, return highest. Supports early stop. |
| `:merge_fields` | Merge record fields. Lists are concatenated, scalars use `conflict`. |
| Custom Fn | `(results) -> merged` — full control. |

### Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `strategy` | Symbol or Fn | Yes | Merge strategy. |
| `key` | Fn | For `:union`/`:intersection` | Extract unique key from each item. |
| `conflict` | Fn | No | `(a b) -> winner` when two items have same key. Default: keep first. |
| `flatten` | Str | No | Field name to flatten before reconciling (e.g., `"findings"`). |
| `min_agreement` | Int | No | For `:vote` — minimum votes to include. Default: 1. |
| `quorum` | Symbol or `(n Int)` | No | For `:vote` — `:unanimous`, `:majority`, `:any`, `(n 2)`. |
| `vote_field` | Str | No | For `:vote` — field name to use as vote. Default: `"approved"`. |
| `weight` | Fn | No | For `:vote` — `(agent_id) -> Int` vote weight. |
| `deliberate` | Int | No | For `:vote` — deliberation rounds if quorum not met. Default: 0. |
| `score` | Fn | No | For `:max_score` — `(result) -> Float` scoring function. |
| `early_stop` | Float | No | For `:max_score` — return immediately when a result exceeds this threshold. |
| `on_vote` | Fn | No | Callback `(agent_id vote)` after each vote. |

### Return Value

```
Protocol ReconcileResult = {
  merged: Any
  sources: Int
  conflicts: [{key: Any  values: [Any]  resolved: Any}]
  dropped: [Any]
  rounds: Int
  dissenting: [Str]
}
```

- `merged` — the reconciled result
- `sources` — number of input results
- `conflicts` — items where the conflict function was invoked
- `dropped` — items excluded by strategy (intersection misses, below min_agreement)
- `rounds` — deliberation rounds used (0 if no deliberation)
- `dissenting` — agent IDs that didn't agree (for `:vote` with deliberation)

## Voting (replaces `consensus` keyword)

```
decision = agent.reconcile reviewer_results {
  strategy: :vote
  vote_field: "approved"
  quorum: :majority
  deliberate: 2
  on_vote: (agent_id vote) emit "agent {agent_id} voted {vote}"
}
```

### Deliberation

When `deliberate > 0` and quorum is not met:

1. Each agent receives `{type: "deliberate" votes: all_votes round: N}`
2. Agents see all reasoning and can change their vote
3. Re-evaluate quorum
4. Repeat up to `deliberate` rounds

### Weighted Voting

```
merged = agent.reconcile results {
  strategy: :vote
  quorum: :majority
  weight: (agent_id) -> agent_id == "senior" ? {true -> 2 false -> 1}
}
```

### Custom Vote Types

Not limited to boolean. Agents can vote for any value:

```
merged = agent.reconcile results {
  strategy: :vote
  vote_field: "classification"
  quorum: (n 2)
}
// merged.merged = "security" if 2+ agents voted "security"
```

## Competitive Selection (replaces `speculate` keyword)

```
best = agent.reconcile approach_results {
  strategy: :max_score
  score: evaluate_quality
  early_stop: 95.0
}
```

Run multiple approaches in parallel, score all results, return the highest-scoring one. With `early_stop`, returns immediately when a result exceeds the threshold.

## Usage Patterns

### Union with dedup

```
merged = agent.reconcile search_results {
  strategy: :union
  flatten: "findings"
  key: (f) "{f.file}:{f.line}"
  conflict: (a b) -> a.severity > b.severity ? {true -> a false -> b}
}
```

### Full pipeline: fan-out + reconcile

```
results = par {
  | reviewer_a ~>? task ^
  | reviewer_b ~>? task ^
  | reviewer_c ~>? task ^
}
decision = agent.reconcile results {
  strategy: :vote
  quorum: :majority
  deliberate: 1
}
decision.merged ? { true -> proceed () false -> escalate () }
```

## Implementation

Library function in `stdlib/agent.rs`. Strategies are pattern-matched symbols. The `:vote` strategy includes internal vote-tallying and quorum evaluation logic. The deliberation loop sends follow-up messages to agents via `~>?`. The `:max_score` strategy iterates results, applies the scoring function, and optionally short-circuits on `early_stop`.

## Cross-References

- Parallel execution: [concurrency.md](concurrency.md) (produces the results reconcile merges)
- Knowledge cache: [stdlib-knowledge.md](stdlib-knowledge.md) (`knowledge.merge` is similar but for persistent stores)
- Blackboard: stdlib_roadmap (`std/blackboard` — concurrent writes need reconciliation)
- Refinement: [agents-refine.md](agents-refine.md) (reconcile + refine for multi-reviewer feedback loops)
