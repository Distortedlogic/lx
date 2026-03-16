# Agent Pipeline with Backpressure

`agent.pipeline` connects agents into a processing pipeline with consumer-driven flow control. When a downstream agent is slower than upstream, the pipeline applies backpressure — the producer blocks or drops messages instead of unbounded buffering.

Distinct from `|>>` streaming pipe (data-level reactive dataflow through functions) and `par` (uncoordinated parallel execution). This is agent-level orchestration where each stage is a subprocess agent with its own processing rate.

## Problem

Agent pipelines are common: parser -> analyzer -> reporter. Currently, there are two options:

```
// Option 1: sequential — no parallelism between stages
parsed = parser ~>? raw ^
analyzed = analyzer ~>? parsed ^
reported = reporter ~>? analyzed ^

// Option 2: par — no coordination, unbounded buffering
par {
  parsed = parser ~>? raw ^
  analyzed = analyzer ~>? parsed ^
}
```

Option 1 wastes time (stage N+1 idles while stage N works). Option 2 has no flow control — if parser produces 100 items/sec and analyzer handles 10 items/sec, 90 items/sec accumulate in memory.

Real pipelines need:
- Stages running concurrently
- Bounded buffers between stages
- Backpressure when buffers fill
- Overflow policies (block, drop oldest, sample)

## API

### Creating a Pipeline

```
use std/agent

pipe = agent.pipeline [parser analyzer reporter]
```

Creates a pipeline connecting agents in order. Each agent's output feeds as input to the next agent via `~>?`. Default buffer size between stages: 10 messages.

### Options

```
pipe = agent.pipeline [parser analyzer reporter] {
  buffer: 5
  overflow: :block
}
```

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `buffer` | Int | 10 | Max messages queued between stages |
| `overflow` | `:block`, `:drop_oldest`, `:drop_newest`, `:sample` | `:block` | What to do when buffer is full |
| `timeout` | Int (seconds) | None | Max time a message can spend in the pipeline |

Overflow policies:
- `:block` — producer waits until consumer catches up
- `:drop_oldest` — discard oldest buffered message to make room
- `:drop_newest` — discard the incoming message
- `:sample` — accept every Nth message (rate determined by buffer fill level)

### Sending to Pipeline

```
agent.pipeline_send pipe raw_data ^
```

Sends a message to the pipeline head (first agent). If the first stage's input buffer is full, behavior follows the overflow policy.

### Collecting from Pipeline

```
result = agent.pipeline_collect pipe ^
```

Collects the next completed result from the pipeline tail (last agent's output).

### Batch Processing

```
results = items | map (item) {
  agent.pipeline_send pipe item ^
} | collect

outputs = results | map (_) {
  agent.pipeline_collect pipe ^
}
```

Or with the convenience function:

```
outputs = agent.pipeline_batch pipe items ^
// sends all items and collects all results, respecting backpressure
```

### Pipeline Stats

```
stats = agent.pipeline_stats pipe
// => {
//   stages: [
//     {name: "parser"    queued: 3  processed: 47  avg_ms: 120}
//     {name: "analyzer"  queued: 8  processed: 44  avg_ms: 340}
//     {name: "reporter"  queued: 1  processed: 42  avg_ms: 50}
//   ]
//   total_processed: 42
//   total_dropped: 0
//   bottleneck: "analyzer"
//   throughput: 2.9  // items/sec at tail
// }
```

`bottleneck` identifies the slowest stage. `throughput` is the end-to-end rate.

### Pressure Monitoring

```
agent.pipeline_on_pressure pipe :high (stats) {
  log.warn "pipeline pressure: bottleneck at {stats.bottleneck}"
}
```

Pressure levels mirror `std/context`: `:low` (buffer < 50% full), `:moderate` (50-75%), `:high` (75-90%), `:critical` (> 90%).

### Pipeline Control

```
agent.pipeline_pause pipe
agent.pipeline_resume pipe
agent.pipeline_drain pipe ^   // wait for all buffered items to complete
agent.pipeline_close pipe ^   // drain then stop
```

## Patterns

### Adaptive Pipeline

```
pipe = agent.pipeline [fetcher processor writer] {buffer: 20 overflow: :block}

agent.pipeline_on_pressure pipe :high (stats) {
  stats.bottleneck == "processor" ? {
    true -> {
      extra = agent.spawn "processor" processor_handler ^
      agent.pipeline_add_worker pipe "processor" extra ^
    }
    false -> ()
  }
}
```

Scale out the bottleneck stage by adding workers. `pipeline_add_worker` adds a parallel worker to a stage — messages are distributed round-robin.

### With Budget

```
use std/budget

b = budget.create {api_calls: 100}
pipe = agent.pipeline [classifier enricher writer]

agent.pipeline_on_pressure pipe :critical (stats) {
  budget.status b == :tight ? {
    true -> agent.pipeline_pause pipe
    false -> ()
  }
}
```

### With Reconcile

```
pipe = agent.pipeline [
  fetcher
  (items) par { analyzer_a ~>? items ^  analyzer_b ~>? items ^ }
    | agent.reconcile :union
  writer
]
```

A pipeline stage can itself fan out to multiple agents and reconcile results.

## Implementation

Extension to `std/agent`. A pipeline is a `Vec<PipelineStage>` where each stage wraps an agent with an input buffer (bounded channel). A coordinator thread reads from each stage's output and feeds the next stage's input.

Backpressure is implemented via bounded channels — `send` on a full channel blocks (for `:block` policy) or applies the overflow policy.

### Dependencies

- `std::sync::mpsc` or `crossbeam::channel` (bounded channels for backpressure)
- `parking_lot::Mutex` (stats tracking)

### Scaling

`pipeline_add_worker` adds a second agent to a stage. The stage's input buffer distributes to workers round-robin. Worker outputs merge into the stage's output buffer.

## Cross-References

- Streaming pipe: [concurrency-reactive.md](concurrency-reactive.md) — `|>>` is data-level, this is agent-level
- Reconcile: [agents-reconcile.md](agents-reconcile.md) — fan-out/merge within pipeline stages
- Budget: [agents-budget.md](agents-budget.md) — pipeline cost tracking
- Intercept: [agents-intercept.md](agents-intercept.md) — middleware per stage
- Supervision: [agents-supervision.md](agents-supervision.md) — restart crashed pipeline stages
