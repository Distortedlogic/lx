# Saga Pattern — Reference

## API

```
use std/saga

saga.run steps                     -- Ok {results} ^ Err {failed_step error compensated compensation_errors?}
saga.run steps options             -- same, with options record
saga.define steps                  -- SagaDef (reusable saga definition)
saga.execute saga_def context      -- same as saga.run
```

## Step Record

| Field | Type | Description |
|-------|------|-------------|
| `id` | Str | Unique step identifier |
| `do` | Fn | Execute step. Receives accumulated results from prior steps. Returns `Result`. |
| `undo` | Fn | Compensating action. Receives this step's result. Called on rollback. |
| `depends` | [Str] | Optional. Steps that must complete first. Without it, steps run sequentially. |

## Execution

1. Steps execute in list order (or dependency order if `depends` is set)
2. Each `do` receives a record of prior step results keyed by `id`
3. If step N fails, `undo` runs in reverse for steps N-1 down to 0
4. Each `undo` receives that step's original result
5. Success: `Ok {results}` — record of all step results keyed by id
6. Failure: `Err {failed_step: id  error: e  compensated: [ids]}`

## Undo Failures

If a compensating action itself fails, the saga records it and continues compensating remaining steps:

```
Err {
  failed_step: "test"
  error: "tests failed"
  compensated: ["implement"]
  compensation_errors: [{step: "analyze" error: "undo failed"}]
}
```

## Saga Options

| Option | Type | Description |
|--------|------|-------------|
| `on_compensate` | Fn | Called before each compensation. For logging/audit. |
| `timeout` | Int | Total saga timeout in seconds. Triggers compensation on exceed. |
| `max_retries` | Int | Retry failed steps before compensating. Default: 0. |

## Example

```
use std/saga

result = saga.run [
  {id: "analyze"    do: () analyzer ~>? {task: "review"} ^           undo: (r) ()}
  {id: "implement"  do: (prev) coder ~>? {task: "fix" ..prev} ^     undo: (r) $git checkout -- .}
  {id: "test"       do: (prev) tester ~>? {task: "test"} ^          undo: (r) ()}
  {id: "deploy"     do: (prev) deployer ~>? {task: "deploy"} ^      undo: (r) deployer ~>? {task: "rollback" id: r.deploy_id}}
]
```
