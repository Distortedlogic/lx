# Diminishing Returns Detection

Gradient-based progress tracking for agents to know when effort is no longer paying off. Distinct from circuit breakers (hard limits) and stuck detection (binary). This tracks quality-over-time and computes improvement rate to enable adaptive stopping.

## Problem

`circuit.check` is a wall — you hit 25 turns and stop. `introspect.is_stuck()` is binary — you're repeating yourself or you're not. Neither answers: "I've been improving, but my last 3 turns only improved quality by 0.3% each — should I wrap up?"

Real agent work has phases:
1. Rapid improvement (first draft → good draft)
2. Diminishing returns (good draft → slightly better draft)
3. Plateau (changes don't measurably improve quality)

## Trace-Based Progress Tracking

Progress checkpoints are recorded as trace spans with score metadata. This integrates with `std/trace` rather than requiring a separate storage mechanism.

```
use std/trace

trace.span "progress" {score: 72 label: "initial draft"}

// ... do some work ...

trace.span "progress" {score: 85 label: "after revision 1"}
trace.span "progress" {score: 87 label: "after revision 2"}
trace.span "progress" {score: 87.5 label: "after revision 3"}
```

### Query Functions

Utility functions in `std/trace` that operate on progress spans:

| Function | Signature | Description |
|----------|-----------|-------------|
| `trace.improvement_rate` | `Int -> ProgressRate` | Compute improvement rate over last N progress spans. |
| `trace.should_stop` | `{min_delta: Float window: Int} -> Bool` | True if improvement over window is below min_delta. |

```
rate = trace.improvement_rate 3
// rate = {avg_delta: 0.83  trend: :diminishing  recent_delta: 0.5  samples: 3}

trace.should_stop {min_delta: 1.0 window: 3}
// => true (last 3 deltas are all below 1.0)
```

### ProgressRate Record

```
Protocol ProgressRate = {
  avg_delta: Float
  recent_delta: Float
  trend: Str
  samples: Int
}
```

| Field | Description |
|-------|-------------|
| `avg_delta` | Mean score change per checkpoint over the window. |
| `recent_delta` | Most recent score change (last two checkpoints). |
| `trend` | `:improving` (accelerating), `:steady` (consistent gains), `:diminishing` (decelerating), `:plateau` (< min_delta), `:regressing` (negative delta). |
| `samples` | Number of checkpoints in the window. |

### Trend Classification

```
accelerating = recent_delta > avg_delta * 1.2
steady       = recent_delta >= avg_delta * 0.8
diminishing  = recent_delta < avg_delta * 0.8 && recent_delta > 0
plateau      = recent_delta <= min_delta
regressing   = recent_delta < 0
```

## Usage Patterns

### Adaptive stopping

```
loop {
  result = improve current_work ^
  grade = evaluate result ^
  trace.span "progress" {score: grade.score label: "round {round}"}

  trace.should_stop {min_delta: 1.0 window: 3} ? {
    true  -> break result
    false -> current_work = result
  }
}
```

### Strategy shift on plateau

```
rate = trace.improvement_rate 3
rate.trend ? {
  :diminishing -> switch_to_different_approach ()
  :plateau     -> finalize_and_return ()
  :regressing  -> rollback_to_best ()
  _            -> continue_current_approach ()
}
```

### Integration with refine

```
result = refine draft {
  grade: (work) {
    g = evaluate work
    trace.span "progress" {score: g.score label: "refine"}
    g
  }
  revise: improve
  threshold: 90
  max_rounds: 10
  on_round: (round work score) {
    trace.should_stop {min_delta: 1.0 window: 3} ? {
      true  -> emit "stopping early: diminishing returns"
      false -> ()
    }
  }
}
```

## Implementation

Extension to `std/trace` in `stdlib/trace.rs`. `trace.improvement_rate` and `trace.should_stop` query the existing trace span storage, filtering for spans named "progress" that have a `score` field. No separate storage — progress data lives alongside all other trace spans.

## Cross-References

- Trace spans: stdlib (`std/trace` — progress checkpoints are trace spans)
- Stuck detection: [stdlib-introspect.md](stdlib-introspect.md) (`is_stuck` is binary; this is gradient)
- Circuit breakers: stdlib_roadmap (`std/circuit` / `std/budget` — hard limits)
- Refinement loops: [agents-refine.md](agents-refine.md) (`on_round` callback)
- Strategy shift: [stdlib-introspect.md](stdlib-introspect.md) (`strategy_shift`)
