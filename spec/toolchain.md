# Toolchain

The `lx` CLI and its subcommands.

## Commands

### Implemented

```
lx run script.lx              -- interpret and execute
lx run script.lx -- arg1 arg2 -- pass arguments (available as env.args)
lx check script.lx            -- type-check without executing
lx agent script.lx             -- run as MCP agent (streamable HTTP transport)
```

### Planned (Not Yet Implemented)

```
lx fmt                         -- format all .lx files in project
lx fmt --check                 -- exit nonzero if unformatted (CI mode)
lx test                        -- run tests (assert + satisfaction-based)
lx build                       -- AOT compile to binary
lx init [name]                 -- create new project with lx.toml
lx install                     -- resolve and lock dependencies
lx update                      -- update deps within semver constraints
lx repl                        -- interactive session
lx watch script.lx             -- re-run on file changes
lx registry                    -- start cross-process agent registry
lx signal PID JSON             -- send interrupt signal to running agent
```

### `lx signal` — User-Initiated Interruption

Sends a signal to a running lx process. The signal is written to `.lx/signals/{pid}.json` and picked up by the runtime at the next `user.check` call or `:signal` lifecycle hook check.

```bash
lx signal 12345 '{"action": "redirect", "task": "fix the auth bug instead"}'
lx signal 12345 '{"action": "stop"}'
lx signal 12345 '{"action": "pause", "reason": "waiting for review"}'
```

The signal file is single-slot: latest signal wins if multiple arrive between checks. The runtime polls every 100ms at natural yield points (loop iterations, between pipeline stages).

## Agent Mode

`lx agent script.lx` runs a script as an MCP server with streamable HTTP transport. The script's exported MCP declarations become available as tools.

The agent listens for MCP requests over HTTP, executing tool calls against the lx script's declared tools.

## Execution Model

Default: interpreted. `lx run` parses and executes directly. Fast startup matters — agents generate-then-execute; compilation latency is waste.

`lx build` produces a native binary via AOT compilation. For deploying scripts as standalone tools.

Scripts can be executed directly with a shebang:

```
#!/usr/bin/env lx
+main = () $echo "hello"
```

## Formatter

**Status: Not implemented.**

`lx fmt` — one canonical format. Zero options. Zero configuration.

This matters because I generate code across separate invocations and need consistency without remembering prior formatting choices. If the format is configurable, different invocations might produce different styles. One format eliminates this.

Rules:
- 2-space indent (fewer tokens than 4-space)
- Pipes: one stage per line when chain exceeds 2 stages
- Records: inline when 3 or fewer fields, one field per line otherwise
- Stable-sorted `use` imports (stdlib first, then relative)
- No trailing whitespace, always trailing newline
- Single blank line between top-level bindings
- No blank lines inside blocks

`lx fmt --check` exits nonzero if any file would change. For CI gates.

## Test Runner

**Status: Not implemented.** Tests are currently run via `just test` which uses `cargo run -p lx-cli` to execute suite files.

`lx test` supports two testing modes:

### Assert-based (deterministic)

For testing language features and deterministic logic. Files with `assert` statements:

```
-- test/math_test.lx
use ../src/math

assert (add 1 2 == 3)
assert (add 0 0 == 0)
assert (double 5 == 10) "double should multiply by 2"
```

`assert expr` fails the test if `expr` is `false`. All assertions in a file are run (no short-circuit). Results collected and reported.

### Satisfaction-based (agentic)

For testing non-deterministic agentic flows. Full spec: [testing-satisfaction.md](testing-satisfaction.md).

Files using `std/test` with `test.spec` / `test.scenario` calls run satisfaction scoring. Each scenario is executed multiple times, scored by a grader function across weighted dimensions, and compared against a threshold:

```
-- test/review_test.lx
use std/test

spec = test.spec "code review" {
  flow: "./src/review.lx"
  grader: (output scenario) {
    relevance: audit.references_task output scenario.task
    quality: audit.rubric output scenario.rubric
  }
  threshold: 0.75
}

test.scenario spec "bug fix" {
  input: {task: "fix null check"}
  rubric: ["identifies the null" "proposes a fix"]
  runs: 3
}
```

Configuration defaults come from `lx.toml`'s `[test]` section.

## Type Checker

**Status: Implemented (Session 29).** Bidirectional inference with unification and structural subtyping.

`lx check` runs type inference and reports errors without executing the script. `lx run` skips checking entirely — annotations are documentation at runtime.

Currently checks:
- Return type matches body type when `-> Type` is annotated
- Binding type matches value type when `name: Type = expr` is annotated
- Arithmetic/logic type consistency (e.g., `Int + Str` is a mismatch)
- `^` applied to non-Result/non-Maybe types
- Ternary condition must be Bool

Planned but not yet implemented:
- Exhaustiveness of pattern matches
- Mutable captures in concurrent contexts (`par`/`sel`/`pmap`)
- Trait field types ↔ function param annotations
- Import conflict detection
- `lx check --strict` for warnings-as-errors

## Sandboxing

**Status: Not implemented.**

Default is full access — restricting shell/fs/net defeats the purpose of a scripting language. But for running generated code from untrusted contexts:

**`--sandbox`** prompts before each new capability category:

```
script wants to: read filesystem
  allow once / allow always / deny? >
```

**Selective denial:**

```
lx run --deny-net script.lx       -- no network access
lx run --deny-shell script.lx     -- no shell commands
lx run --deny-fs script.lx        -- no filesystem access
lx run --allow-read --deny-write script.lx  -- read-only filesystem
```

Capability denial is enforced at the runtime level. A denied operation returns `Err PermissionDenied`.

## REPL

**Status: Not implemented.**

`lx repl` starts an interactive session:

```
$ lx repl
lx> x = 5
lx> x * 2
10
lx> [1 2 3] | map (* x) | sum
30
```

Bindings persist across lines. The result of each expression is printed (unless it's unit). Errors show inline without crashing the session.

`lx repl --json` outputs structured results for programmatic use.

## Notebook Mode

**Status: Not implemented.**

`lx notebook` starts an incremental execution session where blocks separated by `---` execute in a shared environment:

```
$ lx notebook
use std/fs
data = fs.read "input.csv" ^
---
rows = data | lines | drop 1 | map (split ",")
---
rows | len
=> 42
```

Each block executes with all previous bindings in scope. Results are printed after each block. This matches the generate-observe-generate workflow: produce code, observe output, produce more code.

## Watch Mode

**Status: Not implemented.**

`lx watch script.lx` re-runs the script on file changes. Useful during development: edit the script, save, see output immediately.

`lx watch --test` re-runs tests on changes to `src/` or `test/` files.

## Project Initialization

**Status: Not implemented.** Full spec: [package-manifest.md](package-manifest.md).

`lx init` creates a project with an `lx.toml` manifest:

```
$ lx init my-flow
Created my-flow/
  lx.toml
  src/main.lx
  test/main_test.lx
```

`lx.toml` declares package identity, dependencies, backend configuration, and test settings:

```toml
[package]
name = "my-flow"
version = "0.1.0"
entry = "src/main.lx"

[deps]

[test]
threshold = 0.75
runs = 1
```

`lx init --flow` creates a flow-oriented project with `src/agents/` and `test/scenarios/` directories.

All `lx` subcommands (`run`, `test`, `check`) walk up from cwd to find `lx.toml` as the project root.

## Test Runner Output

`lx test` output shows both modes:

```
-- assert-based --
test/math_test.lx ............. ok (12 assertions)
test/string_test.lx ......F... FAIL
  test/string_test.lx:15: assert (trim "  " == "")
    left:  " "
    right: ""

-- satisfaction-based --
test/review_test.lx
  code review
    bug fix .................. 0.82 PASS (3 runs, mean 0.82, min 0.71, max 0.91)
    refactor ................. 0.69 FAIL (3 runs, mean 0.69, min 0.55, max 0.80)

Assert: 1 passed, 1 failed (26/27 assertions)
Satisfaction: 1/2 scenarios passed (threshold: 0.75)
```

CLI flags for satisfaction tests:

```
lx test --tag smoke              -- run scenarios tagged "smoke"
lx test --scenario "bug fix"     -- run specific scenario
lx test --threshold 0.90         -- override threshold
lx test --runs 10                -- override run count
```

`lx test --json` outputs structured results for programmatic consumption.

## Environment Variables

| Variable | Effect |
|---|---|
| `LX_LOG` | Log level: `debug`, `info`, `warn`, `err`. Default: `info` |
| `LX_THREADS` | Number of worker threads for concurrent operations. Default: CPU cores |
| `LX_NO_COLOR` | Disable colored output (also respects `NO_COLOR` convention) |

## Cross-References

- Formatter spec: [formatter.md](formatter.md)
- Diagnostics: [diagnostics.md](diagnostics.md)
- Package manifest: [package-manifest.md](package-manifest.md)
- Satisfaction testing: [testing-satisfaction.md](testing-satisfaction.md)
- Flow composition: [flow-composition.md](flow-composition.md)
- Agent discovery: [agents-discovery.md](agents-discovery.md)
