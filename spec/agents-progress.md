# Diminishing Returns Detection

Gradient-based progress tracking for agents to know when effort is no longer paying off. Distinct from circuit breakers (hard limits) and stuck detection (binary). This tracks quality-over-time and computes improvement rate to enable adaptive stopping.

## Problem

`circuit.check` is a wall — you hit 25 turns and stop. `introspect.is_stuck()` is binary — you're repeating yourself or you're not. Neither answers: "I've been improving, but my last 3 turns only improved quality by 0.3% each — should I wrap up?"

Real agent work has phases:
1. Rapid improvement (first draft → good draft)
2. Diminishing returns (good draft → slightly better draft)
3. Plateau (changes don't measurably improve quality)

Agents need to detect phase transitions and adapt: stop refining, switch strategy, or ask for help.

## `introspect.progress` — Progress Tracker

Extension to `std/introspect`. Tracks scored checkpoints over time.

```
use std/introspect

introspect.record_progress {score: 72 label: "initial draft"}

// ... do some work ...

introspect.record_progress {score: 85 label: "after revision 1"}
introspect.record_progress {score: 87 label: "after revision 2"}
introspect.record_progress {score: 87.5 label: "after revision 3"}

rate = introspect.improvement_rate 3
// rate = {avg_delta: 0.83  trend: :diminishing  recent_delta: 0.5  samples: 3}
```

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `introspect.record_progress` | `{score: Float label: Str} -> ()` | Record a scored checkpoint. |
| `introspect.improvement_rate` | `Int -> ProgressRate` | Compute improvement rate over last N checkpoints. |
| `introspect.progress_history` | `() -> [{score label timestamp}]` | Full history of scored checkpoints. |
| `introspect.should_stop` | `{min_delta: Float window: Int} -> Bool` | True if improvement over window is below min_delta. |

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
| `trend` | `:improving` (accelerating), `:steady` (consistent gains), `:diminishing` (decelerating), `:plateau` (< 0.5 delta), `:regressing` (negative delta). |
| `samples` | Number of checkpoints in the window. |

### Trend Classification

```
accelerating = recent_delta > avg_delta * 1.2
steady       = recent_delta >= avg_delta * 0.8
diminishing  = recent_delta < avg_delta * 0.8 && recent_delta > 0
plateau      = recent_delta <= min_delta
regressing   = recent_delta < 0
```

Thresholds are configurable via `introspect.configure_progress {min_delta: 0.5}`.

## Usage Patterns

### Adaptive stopping

```
loop {
  result = improve current_work ^
  grade = evaluate result ^
  introspect.record_progress {score: grade.score label: "round {round}"}

  introspect.should_stop {min_delta: 1.0 window: 3} ? {
    true  -> break result
    false -> current_work = result
  }
}
```

### Strategy shift on plateau

```
rate = introspect.improvement_rate 3
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
    introspect.record_progress {score: g.score label: "refine"}
    g
  }
  revise: improve
  threshold: 90
  max_rounds: 10
  on_round: (round work score) {
    introspect.should_stop {min_delta: 1.0 window: 3} ? {
      true  -> emit "stopping early: diminishing returns"
      false -> ()
    }
  }
}
```

## Implementation

Extension to `std/introspect` in `stdlib/introspect.rs`. Progress checkpoints stored as a `Vec<ProgressEntry>` in the interpreter's introspection state alongside the existing action log. Bounded to last 1000 entries like actions. Progress data is exposed via `pub(crate)` accessors so `std/circuit` can incorporate progress trend into trip decisions — e.g., trip when score has plateaued AND turn limit is near. This is part of the broader architecture where introspect is the single source of truth for agent state and circuit reads from it.

## Cross-References

- Stuck detection: [stdlib-introspect.md](stdlib-introspect.md) (`is_stuck` is binary; this is gradient)
- Circuit breakers: stdlib_roadmap (`std/circuit` — hard limits; reads from introspect for action/progress data)
- Refinement loops: [agents-refine.md](agents-refine.md) (`on_round` callback)
- Strategy shift: [stdlib-introspect.md](stdlib-introspect.md) (`strategy_shift`)
