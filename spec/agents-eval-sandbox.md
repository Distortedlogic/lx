# Sandboxed Eval — Dynamic Tool Creation

`agent.eval_sandbox` runs dynamically generated lx code in a restricted environment. Enables agents to create new tools, data transformers, and scoring functions at runtime without full reflection or security risks.

## Problem

Agents sometimes need capabilities that don't exist at program-write time:

- `tool_generation.lx` discovers a gap and generates a new tool
- A grader needs a custom scoring function based on task-specific criteria
- A data pipeline needs an ad-hoc transformer for an unexpected format

Today the only option is spawning a new agent subprocess to run generated code. That works but requires process overhead, file I/O, and manual lifecycle management.

Full `eval` (unrestricted code execution from strings) is too dangerous — LLM-generated code could access the filesystem, network, or shell. Agents need a narrow, sandboxed version.

## `agent.eval_sandbox`

```
use std/agent

transformer = agent.eval_sandbox {
  code: "((record) {cleaned: record.name | trim | lower  score: record.value * 0.5})"
  permissions: [:pure]
  timeout: (time.sec 5)
} ^
```

Returns a callable `Value::Fn` that can be used like any other function:

```
result = transformer {name: "  Alice  " value: 10}
```

### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `code` | Str | Yes | lx source code (must evaluate to a function) |
| `permissions` | [Symbol] | No | Capability set (default: `[:pure]`) |
| `timeout` | Duration | No | Max execution time for the eval (default: 5s) |
| `bindings` | Record | No | Values to inject into the sandbox environment |

### Permission Levels

| Permission | Allows |
|------------|--------|
| `:pure` | Arithmetic, string ops, collections, pattern matching. No I/O |
| `:read_fs` | `:pure` + `std/fs` read operations (no write) |
| `:ai` | `:pure` + `std/ai` calls (LLM access) |
| `:network` | `:pure` + `std/http` (outbound HTTP) |
| `:full` | Everything (equivalent to normal lx execution) |

Default is `:pure` — the generated code can transform data but cannot touch the outside world.

### Injecting Context

```
scorer = agent.eval_sandbox {
  code: "((item) item.relevance * weight + (item.tags | len) * tag_bonus)"
  permissions: [:pure]
  bindings: {weight: 0.7  tag_bonus: 0.3}
} ^
```

`bindings` values are available as variables inside the sandbox. The generated code can reference them but not modify the caller's state.

### Error Handling

```
agent.eval_sandbox {code: bad_code} ? {
  Ok fn -> fn input
  Err e -> {
    e.type ? {
      "parse"   -> "generated code has syntax errors: {e.message}"
      "type"    -> "generated code has type errors: {e.message}"
      "timeout" -> "generated code took too long"
      "permission" -> "generated code tried forbidden operation: {e.op}"
      _         -> "eval failed: {e.message}"
    }
  }
}
```

Parse errors, type errors, timeouts, and permission violations all return `Err` — never panic.

## Use Case: Dynamic Tool Generation

```
gap = analyze_workflow current_tools ^
code = ai.prompt "Write an lx function that: {gap.description}.
  Input: {gap.input_schema}. Output: {gap.output_schema}.
  Return ONLY the function, no explanation." ^

new_tool = agent.eval_sandbox {
  code: code
  permissions: [:pure]
  timeout: (time.sec 10)
} ^

result = new_tool gap.test_input ^
```

## Use Case: Custom Grading Functions

```
criteria = yield {type: "grading_criteria" task: task_description}

grader = agent.eval_sandbox {
  code: criteria.grading_function
  permissions: [:pure]
  bindings: {rubric: criteria.rubric  weights: criteria.weights}
} ^

score = grader agent_output
```

## Implementation

### RuntimeCtx Integration

Sandboxed eval creates a new `Interpreter` instance with a restricted `RuntimeCtx`:

```rust
fn eval_sandbox(code: &str, permissions: &[Symbol], bindings: Record, timeout: Duration)
    -> Result<Value, LxError>
{
    let restricted_ctx = ctx.with_permissions(permissions);
    let mut interp = Interpreter::new_with_ctx(restricted_ctx);
    for (k, v) in bindings { interp.env.set(k, v); }
    interp.eval_with_timeout(parse(code)?, timeout)
}
```

Permission enforcement: the restricted `RuntimeCtx` replaces backends with permission-checking wrappers. A `:pure` context has `DenyShellBackend`, `DenyHttpBackend`, etc. that return `Err("permission denied: shell access requires :shell permission")`.

### Security Model

- **No ambient authority**: sandbox starts with empty environment + injected bindings
- **No module imports**: `use` is disabled inside sandbox (can't load std/fs, etc.)
- **No agent operations**: `~>`, `~>?`, `agent.spawn` all denied
- **No shell access**: `$` commands denied
- **Timeout enforcement**: interpreter checks elapsed time between statements
- **Return value only**: sandbox can only return a value, not modify external state

### In `std/agent`

New function in the agent module: `agent.eval_sandbox`. Extension file: `stdlib/agent_eval.rs`.

## Cross-References

- Agent capabilities: [agents-capability.md](agents-capability.md)
- Skill declarations (static version): [agents-skill.md](agents-skill.md)
- Tool generation flow: `flows/tool_generation.lx`
- RuntimeCtx backends: `design/impl-stdlib.md`
