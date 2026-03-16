# Result Reconciliation — Reference

```
use std/agent
merged = agent.reconcile results {strategy: :union  key: (.id)  conflict: (a b) -> a.confidence > b.confidence ? {true -> a false -> b}}
```

## Strategies

| Strategy | Description |
|----------|-------------|
| `:union` | Combine all, deduplicate by `key`. On conflict use `conflict` fn. |
| `:intersection` | Keep only items present in all results (by `key`). |
| `:vote` | Most common value wins. Supports `quorum`. |
| `:highest_confidence` | Pick result with highest `.confidence` field. |
| `:max_score` | Score all results, return highest. Supports `early_stop`. |
| `:merge_fields` | Merge record fields. Lists concatenated, scalars use `conflict`. |
| Custom Fn | `(results) -> merged` — full control. |

## Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `strategy` | Symbol/Fn | Yes | Merge strategy. |
| `key` | Fn | `:union`/`:intersection` | Extract unique key from each item. |
| `conflict` | Fn | No | `(a b) -> winner` when same key. Default: keep first. |
| `flatten` | Str | No | Field name to flatten before reconciling. |
| `min_agreement` | Int | No | For `:vote` — minimum votes. Default: 1. |
| `quorum` | Symbol/`(n Int)` | No | `:unanimous`, `:majority`, `:any`, `(n 2)`. |
| `vote_field` | Str | No | For `:vote` — field to vote on. Default: `"approved"`. |
| `weight` | Fn | No | For `:vote` — `(agent_id) -> Int`. |
| `deliberate` | Int | No | For `:vote` — rounds if quorum not met. Default: 0. |
| `score` | Fn | No | For `:max_score` — `(result) -> Float`. |
| `early_stop` | Float | No | For `:max_score` — return when result exceeds this. |

## Return Value

```
Protocol ReconcileResult = {
  merged: Any  sources: Int
  conflicts: [{key: Any  values: [Any]  resolved: Any}]
  dropped: [Any]  rounds: Int  dissenting: [Str]
}
```

## Examples

### Vote with deliberation
```
decision = agent.reconcile reviewer_results {strategy: :vote  vote_field: "approved"  quorum: :majority  deliberate: 2}
```

### Union with dedup
```
merged = agent.reconcile search_results {
  strategy: :union  flatten: "findings"  key: (f) "{f.file}:{f.line}"
  conflict: (a b) -> a.severity > b.severity ? {true -> a false -> b}
}
```

### Competitive selection
```
best = agent.reconcile approach_results {strategy: :max_score  score: evaluate_quality  early_stop: 95.0}
```

### Weighted voting
```
merged = agent.reconcile results {
  strategy: :vote  quorum: :majority
  weight: (agent_id) -> agent_id == "senior" ? {true -> 2 false -> 1}
}
```
