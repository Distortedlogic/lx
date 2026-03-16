# Diminishing Returns Detection — Reference

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `trace.improvement_rate` | `Int -> ProgressRate` | Compute improvement rate over last N progress spans |
| `trace.should_stop` | `{min_delta: Float window: Int} -> Bool` | True if improvement over window is below min_delta |

Record progress via trace spans with `score` metadata:

```
use std/trace

trace.span "progress" {score: 72 label: "initial draft"}
trace.span "progress" {score: 85 label: "after revision 1"}
trace.span "progress" {score: 87 label: "after revision 2"}
```

## ProgressRate Fields

```
Protocol ProgressRate = {avg_delta: Float  recent_delta: Float  trend: Str  samples: Int}
```

| Field | Description |
|-------|-------------|
| `avg_delta` | Mean score change per checkpoint over the window |
| `recent_delta` | Most recent score change (last two checkpoints) |
| `trend` | `:improving`, `:steady`, `:diminishing`, `:plateau`, or `:regressing` |
| `samples` | Number of checkpoints in the window |

## Trend Classification

```
accelerating = recent_delta > avg_delta * 1.2
steady       = recent_delta >= avg_delta * 0.8
diminishing  = recent_delta < avg_delta * 0.8 && recent_delta > 0
plateau      = recent_delta <= min_delta
regressing   = recent_delta < 0
```

## Adaptive Stopping Example

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
