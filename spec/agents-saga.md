# Saga Pattern

Transactional multi-agent operations with compensating actions. When a multi-step workflow involving multiple agents fails partway through, compensating actions undo the completed steps in reverse order.

## Problem

`checkpoint`/`rollback` is single-agent — it snapshots and restores mutable state within one process. But multi-agent workflows have distributed state:

```
checkpoint "pipeline" {
  analysis = analyzer ~>? {task: "review"} ^
  implementation = coder ~>? {task: "fix" findings: analysis} ^
  test_result = tester ~>? {task: "test"} ^
  test_result.passed ? {
    false -> rollback "pipeline"
    true  -> Ok "done"
  }
}
```

`rollback` restores local state, but the coder already wrote files, the analyzer already consumed budget, and external systems may have been modified. `checkpoint` can't undo those side effects.

## `std/saga` Module

```
use std/saga

result = saga.run [
  {id: "analyze"    do: () analyzer ~>? {task: "review"} ^           undo: (r) ()}
  {id: "implement"  do: (prev) coder ~>? {task: "fix" ..prev} ^     undo: (r) $git checkout -- .}
  {id: "test"       do: (prev) tester ~>? {task: "test"} ^          undo: (r) ()}
  {id: "deploy"     do: (prev) deployer ~>? {task: "deploy"} ^      undo: (r) deployer ~>? {task: "rollback" id: r.deploy_id}}
]
```

`saga.run` is a library function in `std/saga`. It executes steps in order. If any step fails, compensating actions (`undo`) run in reverse order for all completed steps.

### Step Record

| Field | Type | Description |
|-------|------|-------------|
| `id` | Str | Unique step identifier |
| `do` | Fn | Execute step. Receives accumulated results from prior steps. Returns `Result`. |
| `undo` | Fn | Compensating action. Receives this step's result. Called on rollback. |

### Execution

1. Steps execute in list order (respecting any declared dependencies)
2. Each step's `do` function receives a record of all prior step results, keyed by `id`
3. If step N succeeds, its result is added to the accumulated record
4. If step N fails:
   a. Steps N-1, N-2, ..., 0 have their `undo` functions called in reverse order
   b. Each `undo` receives that step's original result
   c. `saga.run` returns `Err {failed_step: id error: e compensated: [ids]}`
5. If all steps succeed, returns `Ok {results}` — record of all step results keyed by id

### Undo Failures

If a compensating action itself fails, the saga records it and continues compensating remaining steps:

```
Err {
  failed_step: "test"
  error: "tests failed"
  compensated: ["implement"]
  compensation_errors: [{step: "analyze" error: "undo failed"}]
}
```

The caller receives full information about what was and wasn't cleaned up.

### With Dependencies

```
result = saga.run [
  {id: "a" do: () step_a ()     undo: (r) undo_a r}
  {id: "b" do: (prev) step_b prev.a  undo: (r) undo_b r  depends: ["a"]}
  {id: "c" do: (prev) step_c prev.a  undo: (r) undo_c r  depends: ["a"]}
  {id: "d" do: (prev) step_d prev    undo: (r) undo_d r  depends: ["b" "c"]}
]
```

Steps with satisfied dependencies can execute concurrently (like `std/plan`). The `depends` field is optional — without it, steps execute sequentially.

### Saga Options

```
saga.run steps {
  on_compensate: (step_id result) log.warn "compensating {step_id}"
  timeout: 300
  max_retries: 0
}
```

| Option | Type | Description |
|--------|------|-------------|
| `on_compensate` | Fn | Called before each compensation. For logging/audit. |
| `timeout` | Int | Total saga timeout in seconds. Triggers compensation on exceed. |
| `max_retries` | Int | Retry failed steps before compensating. Default: 0 (no retry). |

### Named Sagas

For reuse and clarity:

```
deploy_saga = saga.define [
  {id: "build"  do: build_step   undo: cleanup_build}
  {id: "test"   do: test_step    undo: ()}
  {id: "deploy" do: deploy_step  undo: rollback_deploy}
]

result = saga.execute deploy_saga {env: "production"}
```

`saga.define` creates a reusable saga definition. `saga.execute` runs it with initial context.

## Cross-References

- Single-agent checkpoint/rollback: [agents-advanced.md](agents-advanced.md)
- Plan execution with dependencies: [agents-plans.md](agents-plans.md)
- Agent error types: [agents.md](agents.md)
- Structured concurrency: [concurrency.md](concurrency.md)
