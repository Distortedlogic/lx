# Agent Primitives — Advanced Features

Yield, MCP declarations, `with` scoped bindings, and record field update. These extend the core agent primitives in [agents.md](agents.md).

## Emit (Agent-to-Human Output)

`emit` sends a value to whoever invoked the agent — a human at a terminal, an orchestrator, a parent agent. Fire-and-forget: execution continues immediately. This is the primary mechanism for agent-to-human communication.

### Why Not `$echo`

Flows use `$echo` for user-facing output, but `$echo` is a shell command. It spawns `/bin/sh -c echo ...` for every message, produces unstructured text, and is indistinguishable from any other shell execution. An orchestrator cannot tell the difference between `$echo "step 2 done"` and `$curl https://...` — both are shell executions returning `{out err code}`.

`emit` is to human communication what `~>` is to agent communication — a dedicated primitive with its own AST node, runtime semantics, and interception point.

### Syntax

```
emit "seeding {def.name}..."
emit {type: "progress" step: 3 total: 10 msg: "processing files..."}
emit {type: "result" data: findings summary: "{findings | len} issues found"}
```

`emit expr` evaluates `expr` and sends the value to the emit handler. Returns `()`. Does not block.

### Runtime Behavior

Three modes depending on execution context:

1. **Standalone** (`lx run`) — writes to stdout. Strings print directly. Records/lists are JSON-encoded.
2. **Orchestrated** — calls an `EmitHandler` callback set by the host. The orchestrator decides how to render, route, or store the message.
3. **Subprocess agent** — writes a JSON-line to stdout: `{"type":"emit","value":"seeding code-reviewer..."}`. The parent reads it alongside other protocol messages.

### Backend-Based

Emit routes through `RuntimeCtx.emit` (see [runtime-backends.md](runtime-backends.md)). The embedder provides an `EmitBackend` implementation. Unlike `yield`, emit does not require a custom backend — the default `CliEmitBackend` prints strings directly and JSON-encodes structured values.

### Composition

`emit` is an expression that returns `()`:

```
emit "starting..."
results = data | map process
emit {type: "done" count: results | len}
```

`emit` does not compose with `^` or `|` because its return type is always `()`.

### With Protocol Validation

```
Protocol StatusUpdate = {type: Str  msg: Str  severity: Str = "info"}
emit StatusUpdate {type: "status" msg: "grading complete"}
```

The Protocol validates the record before emitting. Missing fields or type mismatches are caught at the emit boundary.

### `emit` vs Other Output

| Primitive | Direction | Structured? | Blocks? | Use |
|-----------|-----------|-------------|---------|-----|
| `emit` | agent → human/orchestrator | yes | no | Status, progress, results |
| `$echo` | agent → shell stdout | no | no | Shell-level side effects |
| `log.*` | agent → stderr | prefix only | no | Diagnostics, traces |
| `yield` | agent ↔ orchestrator | yes | yes | Interactive plans |
| `~>`/`~>?` | agent → agent | yes (Protocol) | `~>?` yes | Inter-agent messaging |

### Use Case: Progress Reporting

```
+main = () {
  items = load_work_items ()
  emit {type: "start" total: items | len}

  results = items | enumerate | map (i item) {
    emit {type: "progress" step: i + 1 total: items | len item: item.name}
    result = process item
    emit result.passed ? {
      true -> {type: "pass" item: item.name score: result.score}
      false -> {type: "fail" item: item.name reason: result.feedback}
    }
    result
  }

  emit {type: "done" passed: results | filter (.passed) | len total: results | len}
}
```

## Yield (Coroutine Execution)

`yield` pauses execution, sends a value to an orchestrator, and returns the orchestrator's response. This enables executable agent plans — the plan IS an lx program with holes that an LLM or human fills.

### Syntax

```
result = yield {question: "What should I do next?" context: current_state}
```

`yield expr` evaluates `expr`, sends it to the orchestrator callback, and blocks until the orchestrator responds. The response becomes `yield`'s return value.

### Orchestrator Protocol

The orchestrator communicates via JSON lines on stdin/stdout:

```
--> {"type":"yield","value":{"question":"What next?"}}   (lx sends)
<-- {"response":"Continue with step 2"}                    (orchestrator responds)
```

The `lx` process reads one JSON line from stdin as the response. The orchestrator is any process that reads/writes JSON lines — a Python script, another agent, a human with a terminal.

### Backend-Based

Yield routes through `RuntimeCtx.yield_` (see [runtime-backends.md](runtime-backends.md)). The embedder provides a `YieldBackend` implementation. Without a backend, `yield` is a runtime error. The default `CliYieldBackend` uses the JSON-line protocol on stdin/stdout.

### Composition

`yield` composes with existing operators:

```
next_step = yield {question: "What's next?"} ^
data = yield {request: "fetch data"} ^ | json.parse ^
```

### Use Case: Executable Plans

```
plan = () {
  data = yield {action: "gather" sources: ["api" "logs"]}
  analysis = yield {action: "analyze" data: data}
  yield {action: "report" findings: analysis}
}
```

The plan is an lx program. Each `yield` pauses for the orchestrator (LLM/human) to fill in the result. The plan resumes with the response.

## MCP Declarations (Typed Tool Contracts)

`MCP` declarations define typed interfaces for MCP tools. They validate inputs, validate outputs, and generate pre-curried wrapper functions.

### Syntax

```
MCP Calculator = {
  add { a: Int  b: Int } -> Int
  multiply { a: Int  b: Int } -> Int
}
```

A `MCP` declaration is callable — it takes an MCP client and returns a record of typed wrapper functions:

```
use std/mcp
client = mcp.connect {command: "calc-server" args: []} ^
calc = Calculator client

result = calc.add {a: 3 b: 4}
```

### Input Validation

Input fields are validated against declared types before the MCP call. Missing required fields produce `Err`. Wrong types produce `Err`.

### Output Types

Output types can be:
- A type name: `-> Int` (validates output is Int)
- A record shape: `-> {result: Int  confidence: Float}`
- A list: `-> [Str]`
- `Protocol` names are resolved at eval time

### Exports

MCP declarations can be exported:

```
+MCP Calculator = { ... }
```

## `with` Scoped Bindings

`with` creates a scoped binding — the binding exists only within the block body.

### Syntax

```
result = with name = expr {
  body using name
}
```

The binding is lexical (not dynamic). The block returns its last expression.

### Mutable Bindings

Use `:=` for mutable bindings within the `with` scope:

```
with mut counter := 0 {
  counter <- counter + 1
  counter <- counter + 1
  counter
}
```

### Record Field Update

`name.field <- value` updates a field on a mutable record binding:

```
with mut state := {step: "start" data: []} {
  state.step <- "process"
  state.data <- [1 2 3]
  state
}
```

Nested field update: `state.config.timeout <- 30`. Adding new fields is allowed. Requires `:=` binding.

### Use Case: Context Threading

```
with mut state = ctx.load "state.json" ^ {
  state.step <- "process"
  state.data <- data
  ctx.save "state.json" state ^
}
```

## Checkpoint and Rollback

`checkpoint` snapshots mutable state before executing a block. `rollback` restores the snapshot — making agentic trial-and-error safe by default.

### Syntax

```
result = checkpoint "before_refactor" {
  files | each (f) refactor f ^
  test_result = run_tests () ^
  test_result.passed ? {
    true  -> Ok test_result
    false -> rollback "before_refactor"
  }
}
```

`checkpoint "name" { body }` evaluates `body`. If `body` completes normally, the checkpoint is discarded and the result is returned. If `rollback "name"` is called inside `body`, all mutable state (`:=` bindings, context values, file writes via `std/fs`) is restored to the snapshot and the block returns `Err {rolled_back: "name"}`.

### What Gets Snapshotted

- Mutable bindings (`:=`) in the enclosing scope
- Context values (`std/ctx`) that were loaded before the checkpoint
- File system writes made via `std/fs` within the block (tracked and reversed)

Shell commands (`$`) are NOT rolled back — they have external side effects that can't be undone. MCP tool calls are also not rolled back.

### Nested Checkpoints

Checkpoints nest. `rollback` targets a specific checkpoint by name:

```
checkpoint "outer" {
  checkpoint "inner" {
    rollback "inner"
  }
}
```

Rolling back an outer checkpoint also rolls back all inner checkpoints.

### Use Case: Safe Agent Delegation

```
checkpoint "delegation" {
  result = worker ~>? {task: "implement feature"} ^
  audit = auditor ~>? {output: result task: "implement feature"} ^
  audit.passed ? {
    true  -> result
    false -> rollback "delegation"
  }
}
```

## Cross-References

- Core agent primitives: [agents.md](agents.md)
- Concurrency: [concurrency.md](concurrency.md)
- Error handling: [errors.md](errors.md)
- Test suites: [18_yield.lx](../tests/18_yield.lx), [19_mcp_typed.lx](../tests/19_mcp_typed.lx), [22_with.lx](../tests/22_with.lx), emit tests (planned)
