# Strategy Memory

`std/strategy` records which approaches work for which problem types and learns across sessions. An agent that tried "bottom-up refactoring" on a large codebase and scored 85 can recall that next time — instead of rediscovering the optimal approach from scratch.

Distinct from `std/memory` (tiered factual knowledge — "this API uses OAuth2"), `std/reputation` (agent quality scores — "this agent is good at security"), and `std/knowledge` (shared discovery cache with provenance). Strategy memory tracks *methods and their outcomes*, not facts or agents.

## Problem

Agents rediscover optimal approaches every session:

```
// Session 1: tried 3 approaches, "incremental" scored best (92)
// Session 2: no memory of this — tries all 3 again
// Session 3: same thing

// The agent has std/memory with facts like "project uses React"
// but no record of "incremental approach works best for React refactoring"
```

`std/introspect` has `strategy_shift` which signals a strategy change within a session, but doesn't persist what worked. `std/trace` records scores per interaction, but doesn't aggregate by approach or problem type. `std/reputation` scores agents, not strategies.

The missing piece: a store keyed by `(problem_type, approach)` that accumulates outcome data and answers "what should I try first?"

## `std/strategy`

### Creating a Store

```
use std/strategy

store = strategy.create "strategies.json" ^
```

File-backed JSON, same pattern as `std/knowledge` and `std/reputation`. Creates the file if it doesn't exist.

### Recording Outcomes

```
strategy.record store {
  problem: "large_refactor"
  approach: "bottom_up"
  score: 92
  context: {file_count: 45 language: "rust" complexity: :high}
} ^

strategy.record store {
  problem: "large_refactor"
  approach: "top_down"
  score: 61
  context: {file_count: 45 language: "rust" complexity: :high}
} ^
```

Each record adds a data point to the `(problem, approach)` pair. `score` is 0-100. `context` is optional metadata for similarity matching.

### Best Approach

```
best = strategy.best_for store "large_refactor" ^
// => {approach: "bottom_up"  avg_score: 88.5  count: 7  trend: :stable}
```

Returns the approach with the highest average score for the given problem type. `trend` is `:improving`, `:stable`, or `:declining` based on recent vs overall scores.

### Ranked Approaches

```
ranked = strategy.rank store "large_refactor" ^
// => [
//   {approach: "bottom_up"     avg_score: 88.5  count: 7  trend: :stable}
//   {approach: "incremental"   avg_score: 76.2  count: 3  trend: :improving}
//   {approach: "top_down"      avg_score: 61.0  count: 2  trend: :declining}
// ]
```

### Context-Aware Suggestion

```
suggested = strategy.suggest store {
  problem: "refactor"
  context: {file_count: 12 language: "rust" complexity: :medium}
} ^
// => {approach: "incremental"  confidence: 0.78  reason: "best match for medium complexity rust"}
```

`strategy.suggest` matches the given problem and context against recorded outcomes. It weighs both score and context similarity. Returns a confidence level reflecting how much data supports the suggestion.

### Approach History

```
history = strategy.history store "large_refactor" "bottom_up" ^
// => [
//   {score: 92  context: {file_count: 45 ...}  timestamp: "2026-03-10T..."}
//   {score: 85  context: {file_count: 30 ...}  timestamp: "2026-03-08T..."}
//   ...
// ]
```

Full history of a specific approach on a specific problem type.

### Adaptive Selection (Explore vs Exploit)

```
choice = strategy.adapt store "code_review" ^
// => {approach: "checklist_based"  mode: :exploit}
// or
// => {approach: "free_form"  mode: :explore}
```

`strategy.adapt` uses epsilon-greedy selection:
- **Exploit** (default 80% of the time): pick the best-scoring approach
- **Explore** (20%): pick a random less-tried approach to gather more data

The explore rate decreases as confidence increases (more data points = more exploitation).

```
choice = strategy.adapt store "code_review" {explore_rate: 0.3} ^
```

### Pruning Old Data

```
strategy.prune store {older_than: 90} ^  // remove entries older than 90 days
strategy.prune store {min_count: 3 below_score: 30} ^  // remove low-performing with few data points
```

### Export and Share

```
data = strategy.export store ^
// => record with all strategy data, serializable

strategy.import store data ^  // merge external strategy data
```

Agents can share strategy data. An experienced agent exports its strategies; a new agent imports them as a starting point.

## Patterns

### With refine

```
best = strategy.best_for store problem_type ^

result = refine draft {
  grade: grade_fn
  revise: (work feedback) {
    ai.prompt "Revise using {best.approach} strategy: {work}\nFeedback: {feedback}" ^
  }
  threshold: 85
  max_rounds: 5
}

strategy.record store {
  problem: problem_type
  approach: best.approach
  score: result.final_score
  context: task_context
} ^
```

### With Planner

```
steps = planner ~>? {task: goal} ^

steps | each (step) {
  choice = strategy.adapt store step.type ^
  result = execute step choice.approach ^
  strategy.record store {
    problem: step.type
    approach: choice.approach
    score: (grade result).score
  } ^
}
```

### Self-Improving Agent

```
agent.on me :shutdown (reason) {
  // persist what worked this session
  session_outcomes | each (outcome) {
    strategy.record store outcome ^
  }
}

agent.on me :startup () {
  store = strategy.create "strategies.json" ^
  preferred = strategy.best_for store current_task_type ^
  log.info "starting with {preferred.approach} (avg: {preferred.avg_score})"
}
```

### With Reputation

```
// reputation tracks WHICH AGENT is good
// strategy tracks WHICH APPROACH is good
// together: pick the right agent AND the right approach

agent_name = reputation.best_for rep task_type ^ | (.agent)
approach = strategy.best_for store task_type ^ | (.approach)

agent = agent.get agent_name ^
result = agent ~>? {task approach} ^
```

## Implementation

`std/strategy` is a new stdlib module. Strategy data is a JSON file:

```json
{
  "entries": {
    "large_refactor::bottom_up": {
      "scores": [92, 85, 88, 91],
      "contexts": [...],
      "timestamps": [...]
    }
  }
}
```

Key is `"{problem}::{approach}"`. Scores and contexts are stored as parallel arrays (bounded to last 100 entries per pair). `best_for` computes mean. `suggest` uses cosine similarity on context fields. `adapt` uses epsilon-greedy with decaying exploration rate.

### Dependencies

- `serde_json` (file persistence)
- `std/time` (timestamps)
- `std/fs` (file I/O)

## Cross-References

- Factual memory: stdlib (`std/memory`) — tiered facts, different concern
- Agent quality: [agents-reputation.md](agents-reputation.md) — scores agents, not approaches
- Discovery cache: [stdlib-knowledge.md](stdlib-knowledge.md) — shared facts with provenance
- Introspect: [stdlib-introspect.md](stdlib-introspect.md) — `strategy_shift` signals changes but doesn't persist
- Trace: stdlib (`std/trace`) — scored spans feed strategy outcomes
- Refinement: [agents-refine.md](agents-refine.md) — strategy selection for revision approaches
