# Agent Primitives — Advanced Features

Yield, MCP declarations, `with` scoped bindings, and record field update. These extend the core agent primitives in [agents.md](agents.md).

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

### Callback-Based

Yield is callback-based, not coroutine-based. The host sets a `YieldHandler` callback before execution. Without a handler, `yield` is a runtime error.

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

## Cross-References

- Core agent primitives: [agents.md](agents.md)
- Concurrency: [concurrency.md](concurrency.md)
- Error handling: [errors.md](errors.md)
- Test suites: [18_yield.lx](../suite/18_yield.lx), [19_mcp_typed.lx](../suite/19_mcp_typed.lx), [22_with.lx](../suite/22_with.lx)
