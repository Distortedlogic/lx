# Resource / Cost Budget Accounting

`std/budget` tracks cumulative resource spend, projects whether remaining budget covers remaining work, and enables adaptive strategy selection. Circuit breakers are walls; budgets are gradients.

## Problem

`std/circuit` fires when a hard limit is reached — turn count, timeout, action repetition. But agents need to *adapt* before hitting the wall:

- "I've used 40% of my token budget on step 1 of 5 — switch to a cheaper approach"
- "At current rate, I'll exceed my API call limit by step 8 — consolidate remaining steps"
- "This approach costs 3x more per step than expected — abandon and try the cheaper path"

`ai.prompt_with` returns cost info per call. `with context` (planned) will propagate budget limits. But there's no accounting layer that tracks cumulative spend and projects costs.

## `std/budget`

### Core API

```
use std/budget

b = budget.create {tokens: 50000  api_calls: 20  wall_time: 300}
```

Creates a budget with named resource dimensions. All dimensions are optional — track only what matters.

### Spending

```
result = ai.prompt_with {prompt: task} ^
budget.spend b {tokens: result.usage  api_calls: 1} ^
```

`budget.spend` deducts from the budget. Returns `Err "budget_exceeded"` if any dimension goes negative (but does not prevent the call — the work is already done).

### Querying

```
remaining = budget.remaining b
// => {tokens: 38000  api_calls: 19  wall_time: 245}

used = budget.used b
// => {tokens: 12000  api_calls: 1  wall_time: 55}

pct = budget.used_pct b
// => {tokens: 24.0  api_calls: 5.0  wall_time: 18.3}
```

### Projection

```
budget.project b {remaining_steps: 4} ^
// => {
//   projected_total: {tokens: 60000  api_calls: 5}
//   will_exceed: ["tokens"]
//   headroom: {tokens: -10000  api_calls: 15}
// }
```

`budget.project` extrapolates from average spend-per-step. Returns which dimensions will exceed and by how much.

### Adaptive Strategy

```
strategy = budget.status b ? {
  :comfortable -> :detailed
  :tight       -> :summary
  :critical    -> :minimal
  :exceeded    -> :stop
}
```

`budget.status` returns a symbol based on used percentage:
- `:comfortable` — < 50% used
- `:tight` — 50-80% used
- `:critical` — > 80% used
- `:exceeded` — any dimension negative

Thresholds are configurable:

```
b = budget.create {tokens: 50000} {tight_at: 60  critical_at: 90}
```

### Sub-budgets

```
sub = budget.slice b {tokens: 10000  api_calls: 5}
```

`budget.slice` creates a sub-budget that draws from the parent. When the sub-budget spends, the parent's remaining decreases too. Useful for allocating budget to sub-tasks:

```
steps | each (step) {
  step_budget = budget.slice b {tokens: budget.remaining b | (.tokens) / remaining_steps}
  execute_step step step_budget ^
}
```

### With speculate

```
best = speculate approaches task {
  score: evaluate
  on_result: (approach result score) {
    budget.spend b {tokens: result.usage  api_calls: 1} ^
  }
}
```

### With refine

```
result = refine draft {
  grade: (work) {
    r = ai.prompt_structured Grade "evaluate: {work}" ^
    budget.spend b {tokens: r.usage  api_calls: 1} ^
    r
  }
  revise: (work feedback) {
    budget.status b == :critical ? {
      true -> work
      false -> ai.prompt_structured Revision "revise: {work}" ^
    }
  }
  threshold: 85
  max_rounds: 5
}
```

## Implementation

`std/budget` is a new stdlib module. Budget state is a mutable record with initial, used, and remaining values per dimension. `project` divides used-so-far by steps-so-far and multiplies by total steps. `slice` creates a child budget that shares a reference to the parent's counters.

### Dependencies

- `std/time` (wall_time tracking)
- `parking_lot::Mutex` (thread-safe spend tracking for `par` blocks)

## Cross-References

- Circuit breakers: ROADMAP (`std/circuit`) — hard limits vs gradient budgets
- Ambient context: [agents-ambient.md](agents-ambient.md) (budget propagation)
- AI module: ROADMAP (`std/ai`) — `prompt_with` returns usage info
- Reconcile: [agents-reconcile.md](agents-reconcile.md) (cost tracking across `:max_score` branches)
- Refinement: [agents-refine.md](agents-refine.md) (budget-aware revision)
