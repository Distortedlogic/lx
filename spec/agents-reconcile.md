# Result Reconciliation

Structured merging of conflicting results from parallel agents. When fan-out produces N different answers, `reconcile` applies a merge strategy to produce a single coherent result.

## Problem

Fan-out with `par` gives you a list of independent results. Merging them is always manual:

```
results = par {
  | searcher_a ~>? {query: "find vulnerabilities"} ^
  | searcher_b ~>? {query: "find vulnerabilities"} ^
  | searcher_c ~>? {query: "find vulnerabilities"} ^
}
// Now what? Manual dedup, confidence comparison, field merging...
all_findings = results | flat_map (.findings) | unique_by (.file)
```

This gets worse when results genuinely conflict (agent A says "safe," agent B says "vulnerable") rather than just being additive. There's no standard way to handle conflict resolution, deduplication, or confidence-weighted merging.

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
| `:vote` | Each result votes for a value. Most common value wins. |
| `:highest_confidence` | Pick the result with highest `.confidence` field. |
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

### Return Value

```
Protocol ReconcileResult = {
  merged: Any
  sources: Int
  conflicts: [{key: Any  values: [Any]  resolved: Any}]
  dropped: [Any]
}
```

- `merged` — the reconciled result
- `sources` — number of input results
- `conflicts` — items where the conflict function was invoked
- `dropped` — items excluded by strategy (intersection misses, below min_agreement)

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

### Voting on classification

```
merged = agent.reconcile classifier_results {
  strategy: :vote
  key: (.classification)
  min_agreement: 2
}
// merged.merged = "security" if 2+ classifiers agreed
```

### Confidence-weighted merge

```
merged = agent.reconcile analysis_results {
  strategy: :highest_confidence
}
// merged.merged = result with highest .confidence
```

### Custom reconciliation

```
merged = agent.reconcile results {
  strategy: (rs) {
    all_findings = rs | flat_map (.findings)
    grouped = all_findings | group_by (.category)
    grouped | map_values (items) {
      items | max_by (.confidence)
    }
  }
}
```

## Implementation

Library function in `stdlib/agent.rs`. No new syntax needed. Strategies are pattern-matched symbols with well-defined behavior. The `conflict` function is called via `call_value` when deduplication encounters collisions. The `:vote` strategy shares vote-tallying logic with `consensus` — both use a shared `tally_votes` utility for counting occurrences and picking winners.

## Cross-References

- Parallel execution: [concurrency.md](concurrency.md) (produces the results reconcile merges)
- Consensus: [agents-consensus.md](agents-consensus.md) (`:vote` strategy shares tally logic with consensus quorum evaluation)
- Knowledge cache: [stdlib-knowledge.md](stdlib-knowledge.md) (`knowledge.merge` is similar but for persistent stores)
- Blackboard: stdlib_roadmap (`std/blackboard` — concurrent writes need reconciliation)
