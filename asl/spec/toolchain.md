# Toolchain

The `lx` CLI and its subcommands.

## Commands

### Implemented

```
lx run script.lx              -- interpret and execute
lx run script.lx -- arg1 arg2 -- pass arguments (available as env.args)
lx agent script.lx             -- run as MCP agent (streamable HTTP transport)
```

### Planned (Not Yet Implemented)

```
lx fmt                         -- format all .lx files in project
lx fmt --check                 -- exit nonzero if unformatted (CI mode)
lx test                        -- run test/ scripts, collect assert failures
lx check                       -- type-check without executing
lx build                       -- AOT compile to binary
lx init                        -- create new project
lx repl                        -- interactive session
lx watch script.lx             -- re-run on file changes
```

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

`lx test` runs all `.lx` files in `test/`. Tests use `assert`:

```
-- test/math_test.lx
use ../src/math

assert (add 1 2 == 3)
assert (add 0 0 == 0)
assert (double 5 == 10) "double should multiply by 2"
```

`assert expr` fails the test if `expr` is `false`. `assert expr msg` includes the message in the failure output. All assertions in a file are run (no short-circuit on first failure). Results collected and reported.

## Type Checker

**Status: Not implemented.** Type annotations were removed — there is no type checker.

`lx check` runs type inference and reports errors without executing the script. Useful for catching type mismatches in code that hasn't been run yet.

Checks performed:
- Type inference and compatibility
- Exhaustiveness of pattern matches
- `^` applied to non-Result/non-Maybe types
- Non-Bool expressions in ternary `?` position
- Mutable captures in concurrent contexts (`par`/`sel`/`pmap`)
- Import conflicts and circular imports
- Variant name uniqueness within a module
- Unused bindings (warning, not error)
- Shadowing of built-in functions (warning)

`lx check --strict` treats warnings as errors.

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

**Status: Not implemented.**

`lx init` creates a project skeleton:

```
my-project/
  pkg.lx
  src/
    main.lx
  test/
    main_test.lx
```

`pkg.lx` contains the project name and an empty deps record. `src/main.lx` contains a minimal `+main`. `test/main_test.lx` contains a passing placeholder assertion.

## Test Runner Output

`lx test` output:

```
test/math_test.lx ............. ok (12 assertions)
test/string_test.lx ......F... FAIL
  test/string_test.lx:15: assert (trim "  " == "")
    left:  " "
    right: ""
test/io_test.lx .............. ok (14 assertions)

2 passed, 1 failed (26/27 assertions)
```

`lx test --json` outputs one JSON object per assertion result for programmatic consumption.

## Environment Variables

| Variable | Effect |
|---|---|
| `LX_LOG` | Log level: `debug`, `info`, `warn`, `err`. Default: `info` |
| `LX_THREADS` | Number of worker threads for concurrent operations. Default: CPU cores |
| `LX_NO_COLOR` | Disable colored output (also respects `NO_COLOR` convention) |

## Cross-References

- Implementation: [implementation.md](../impl/implementation.md) (architecture, crate choices), [implementation-phases.md](../impl/implementation-phases.md) (phase 10: toolchain)
- Formatter design: [impl-formatter.md](../impl/impl-formatter.md)
- Diagnostics: [diagnostics.md](diagnostics.md), [impl-error.md](../impl/impl-error.md)
