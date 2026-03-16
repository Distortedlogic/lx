# Meta Block — Strategy-Level Iteration

`meta` is a block expression that iterates across *approaches*, not across revisions of the same output. Where `refine` says "try again, do it better," `meta` says "that approach isn't viable — try a fundamentally different one."

## Problem

`refine` iterates on output quality within a fixed approach (grade/revise loop). But agents often face a higher-level decision: which approach to use in the first place? Today this requires manual code:

```
strategy := "bottom_up"
result := attempt strategy task
result.score < 30 ? {
  true -> {
    strategy <- "top_down"
    introspect.strategy_shift "bottom_up unviable"
    result <- attempt strategy task
  }
  false -> ()
}
```

This is imperative, doesn't generalize, and doesn't compose with `std/strategy` for cross-session learning.

## Syntax

```
result = meta task {
  strategies: ["bottom_up" "top_down" "decompose"]
  attempt: (strategy task) execute_with strategy task
  evaluate: (result strategy) {
    viable: result.score > 30
    quality: result.score
    reason: result.feedback
  }
  select: :sequential
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `strategies` | [Any] | Yes | Ordered list of approaches to try |
| `attempt` | Fn | Yes | `(strategy task) -> result` — execute task with given strategy |
| `evaluate` | Fn | Yes | `(result strategy) -> {viable quality reason}` |
| `select` | Symbol / Fn | No | Strategy selection: `:sequential` (default), `:random`, or custom `(strategies history) -> strategy` |
| `on_switch` | Fn | No | `(from_strategy to_strategy reason) -> ()` callback |

### Selection Modes

- `:sequential` — try strategies in list order, stop at first viable result
- `:random` — random selection from untried strategies
- `:best_first` — if `std/strategy` store provided, try historically best strategy first
- Custom function — `(strategies history) -> strategy` where history is `[{strategy quality viable}]`

## Return Value

On success (at least one viable attempt):

```
Ok {
  result: the_output
  strategy: "decompose"
  attempts: [
    {strategy: "bottom_up"  quality: 15  viable: false  reason: "too many files"}
    {strategy: "top_down"   quality: 28  viable: false  reason: "missing context"}
    {strategy: "decompose"  quality: 85  viable: true   reason: "clean splits"}
  ]
}
```

On failure (all strategies exhausted):

```
Err {
  reason: "all_exhausted"
  attempts: [...]
  best: {strategy: "top_down"  quality: 28}
}
```

## Integration with `std/strategy`

```
use std/strategy

store = strategy.create "strategies.json"

result = meta task {
  strategies: ["bottom_up" "top_down" "decompose"]
  attempt: (s t) execute_with s t
  evaluate: (r s) {viable: r.score > 30  quality: r.score  reason: r.feedback}
  select: :best_first
  store: store
}
```

When `store` is provided:
- Before attempting, query `strategy.best_for store task.type` to reorder strategies
- After each attempt, record `strategy.record store {problem approach score}`
- Cross-session learning: next time, the best historical strategy is tried first

## Integration with `refine`

`meta` and `refine` compose — `meta` selects the approach, `refine` optimizes within it:

```
result = meta task {
  strategies: ["bottom_up" "top_down"]
  attempt: (strategy task) {
    refine (execute_with strategy task) {
      grade: (work) evaluate work
      revise: (work feedback) improve work feedback
      threshold: 80
      max_rounds: 3
    }
  }
  evaluate: (result strategy) {viable: result.score > 40  quality: result.score}
}
```

## Implementation

### Parser

`meta` is a new keyword, parsed as `meta <expr> { <fields> }`. Similar to `refine` — the parser validates required fields.

### AST

```
Expr::Meta {
    task: Box<SExpr>,
    strategies: Box<SExpr>,
    attempt: Box<SExpr>,
    evaluate: Box<SExpr>,
    select: Option<Box<SExpr>>,
    on_switch: Option<Box<SExpr>>,
    store: Option<Box<SExpr>>,
}
```

### Interpreter

1. Evaluate `strategies` list and `task`
2. Select next strategy (via `select` mode)
3. Call `attempt(strategy, task)` to get result
4. Call `evaluate(result, strategy)` to get `{viable quality reason}`
5. If viable, return `Ok {...}`
6. If not viable, record attempt, call `on_switch` if present, select next
7. If all exhausted, return `Err {...}` with best attempt

## Cross-References

- Output refinement: [agents-refine.md](agents-refine.md)
- Strategy memory: [agents-strategy.md](agents-strategy.md)
- Introspection (stuck detection): [stdlib-introspect.md](stdlib-introspect.md)
- Progress tracking: [agents-progress.md](agents-progress.md)
