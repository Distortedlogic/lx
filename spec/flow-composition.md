# Flow Composition

Flows (entire `.lx` programs) as first-class composable values with typed inputs and outputs. Compose flows declaratively with pipes instead of manually spawning subprocesses.

## Problem

You can compose functions (`f | g | h`), compose agents (`agent.reconcile`, `agent.pipeline`), but you can't compose **flows**. Currently, chaining flows requires:

```
worker1 = agent.spawn {command: "lx"  args: ["run" "step1.lx"]} ^
r1 = worker1 ~>? {data: input} ^
worker2 = agent.spawn {command: "lx"  args: ["run" "step2.lx"]} ^
r2 = worker2 ~>? {data: r1} ^
agent.kill worker1
agent.kill worker2
```

This is manual process management — spawn, send, receive, kill, repeat. No declarative way to express "step1 feeds step2 feeds step3." No typed contracts between stages. No automatic cleanup.

Flows should compose like functions:

```
result = input | flow.run "step1.lx" ^ | flow.run "step2.lx" ^ | flow.run "step3.lx" ^
```

## `std/flow`

### Core API

```
use std/flow

f = flow.load "review.lx" ^
result = flow.run f {task: "review"  file: "auth.rs"} ^
```

### `flow.load`

```
flow.load path -> Flow ^ FlowErr
```

Loads a `.lx` file as a `Flow` value. Does not execute it. The flow is parsed and ready to run.

### `flow.run`

```
flow.run flow input -> a ^ FlowErr
```

Executes the flow with the given input record. The flow's exported `+main` function receives the input. Returns the flow's output.

Execution happens in an isolated interpreter scope — separate environment, separate module cache. The flow shares the parent's `RuntimeCtx` (same backends).

### `Flow` type

```
Flow = {
  path: Str
  exports: [Str]
  input_type: Maybe TypeDesc
  output_type: Maybe TypeDesc
}
```

Type information is extracted from `+main`'s annotations if present.

### Pipe Composition

```
pipeline = flow.pipe [
  flow.load "extract.lx" ^
  flow.load "transform.lx" ^
  flow.load "validate.lx" ^
]

result = flow.run pipeline input ^
```

`flow.pipe` creates a composite flow. Output of each stage becomes input of the next. Type annotations are checked at composition time if present.

### Parallel Composition

```
ensemble = flow.par [
  flow.load "reviewer1.lx" ^
  flow.load "reviewer2.lx" ^
  flow.load "reviewer3.lx" ^
]

results = flow.run ensemble input ^
```

`flow.par` runs all flows with the same input, returns a list of results. Combine with `agent.reconcile` for multi-agent patterns:

```
reviews = flow.run ensemble input ^ | agent.reconcile :vote
```

### Conditional Composition

```
pipeline = flow.pipe [
  flow.load "analyze.lx" ^
  flow.branch (result) result.needs_review ? {
    true  -> flow.load "deep_review.lx" ^
    false -> flow.load "quick_check.lx" ^
  }
  flow.load "report.lx" ^
]
```

`flow.branch` takes a routing function that returns a `Flow` based on intermediate results.

### Retry and Error Handling

```
resilient = flow.load "flaky_service.lx" ^
  | flow.with_retry {max: 3  backoff: :exponential}
  | flow.with_timeout 300
  | flow.with_fallback (flow.load "cached_fallback.lx" ^)
```

Flow-level decorators compose error handling around entire flows.

## Patterns

### ETL pipeline

```
use std/flow

pipeline = flow.pipe [
  flow.load "flows/extract.lx" ^
  flow.load "flows/transform.lx" ^
  flow.load "flows/load.lx" ^
]

data_sources | each (source) {
  flow.run pipeline {source: source} ^
}
```

### Multi-reviewer with threshold

```
reviewers = flow.par [
  flow.load "flows/security_review.lx" ^
  flow.load "flows/perf_review.lx" ^
  flow.load "flows/style_review.lx" ^
]

reviews = flow.run reviewers {diff: changes} ^
combined = agent.reconcile reviews :merge_fields
combined.score >= threshold ? emit "Approved" : emit "Needs work"
```

### Flow-of-flows orchestrator

```
use std/flow
use std/tasks

plan = tasks.create "deploy" [
  {name: "test"    flow: flow.load "flows/test.lx" ^}
  {name: "build"   flow: flow.load "flows/build.lx" ^  depends: ["test"]}
  {name: "stage"   flow: flow.load "flows/stage.lx" ^  depends: ["build"]}
  {name: "verify"  flow: flow.load "flows/verify.lx" ^ depends: ["stage"]}
  {name: "deploy"  flow: flow.load "flows/deploy.lx" ^ depends: ["verify"]}
]

tasks.list plan | each (task) {
  result = flow.run task.flow {context: plan} ^
  tasks.complete plan task.name result ^
}
```

## Implementation

### `std/flow` module

New stdlib module with functions: `load`, `run`, `pipe`, `par`, `branch`, `with_retry`, `with_timeout`, `with_fallback`.

### Flow execution

`flow.run` creates a child `Interpreter` with:
- Fresh `Env` (no parent bindings leak)
- Shared `RuntimeCtx` (same AI/HTTP/shell backends)
- Input record injected as `__input` binding (accessed by `+main` parameter)

### Composite flows

`flow.pipe` returns a `Value::Flow` wrapping a `Vec<Flow>`. `flow.run` on a pipe executes sequentially, threading output → input. `flow.par` wraps flows similarly but executes all with the same input.

### Dependencies

- Interpreter (`crates/lx/src/interpreter/`) — for child execution
- Module resolver (`interpreter/modules.rs`) — for loading `.lx` files
- `std/retry` — for `with_retry` decorator
- `std/agent` — for reconcile integration

## Cross-References

- Functions: LANGUAGE.md (Functions section) — flows extend function composition to programs
- Agents: [stdlib-agents.md](stdlib-agents.md) — `agent.spawn` is process-level, flows are interpreter-level
- Pipelines: [agents-pipeline.md](agents-pipeline.md) — agent pipelines with backpressure
- Plans: `std/plan` — plan steps can be flows
- Package manifest: [package-manifest.md](package-manifest.md) — `entry` field identifies the main flow
- Testing: [testing-satisfaction.md](testing-satisfaction.md) — `flow` field in test specs
