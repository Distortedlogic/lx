# Diagnostics

Error messages are my primary debugging interface. I can't set breakpoints, hover over variables, or step through code. Every error must be self-contained and actionable without external tooling.

## Principles

1. **Location first** — every error starts with `file:line:col`
2. **Show the source** — include the failing line, with the specific expression underlined
3. **Explain the mismatch** — "expected X, got Y" with both sides concrete
4. **Suggest a fix** — when the correction is unambiguous, show it
5. **No decoration** — no colors, no ASCII art, no emoji. Plain text only. These are noise tokens when I parse error output.
6. **One error, one action** — each error message should tell me exactly one thing to change. Compound errors split into separate messages.

## Format

Default output (human-readable, but I parse this too):

```
error[type]: src/main.lx:15:23
  |
15|   result = add x "hello"
  |                  ^^^^^^^
  expected: Int
  got:      Str
  fix:      second argument to `add` must be Int
```

Structured output (`lx run --json`):

```
{"level":"error","code":"type","file":"src/main.lx","line":15,"col":23,
 "msg":"type mismatch","expected":"Int","got":"Str",
 "expr":"add x \"hello\"","fix":"second argument to `add` must be Int"}
```

`--json` is critical. When I run lx programmatically (generate script, execute, parse errors, fix, re-execute), structured output eliminates regex parsing of error messages. One JSON object per diagnostic, one per line, to stderr.

## Pipeline Errors

When `data | filter pred | map f | sort` fails because `f` errors on one element, I need three things: which stage, which element, why. Without all three, debugging pipelines is guesswork.

```
error[runtime]: src/main.lx:12
  pipeline stage 3 of 4: map validate
  element #47: {name: "bad"  age: -1}
  |
 8|   age >= 0 ? () : Err "negative age"
  |                   ^^^^^^^^^^^^^^^^^^^
  error: negative age
```

The element index and value are included because I can't reproduce pipeline state — the input may come from a shell command or API call that won't return the same data twice.

## Error Propagation Context

Each `^` site is recorded. When an error propagates through multiple functions, the trace shows every explicit propagation point — not a full call stack (too noisy), just the `^` chain:

```
error[io]: file not found "/tmp/data.csv"
  propagated through:
    src/main.lx:25  content = fs.read path ^
    src/main.lx:18  data = load_data input ^
    src/main.lx:5   result = process args ^
```

This is like Rust's `anyhow` context chain but built into the language. Every `^` is a breadcrumb.

## Parse Errors

Parse errors are the most common error I produce (typos, misremembered syntax). The parser recovers after the first error and reports up to 5 errors per run. Each shows what was expected:

```
error[parse]: src/main.lx:7:12
  |
 7|   data | map * 2
  |              ^
  expected: expression (function or section like `(* 2)`)
  got:      operator `*`
  fix:      wrap in section: `map (* 2)`
```

## Shell Command Errors

When `$cmd` fails, the error includes the command, exit code, AND stderr — because the shell error message is the diagnostic:

```
error[shell]: src/main.lx:10
  |
10|   $^gcc main.c -Wall
  |
  command: gcc main.c -Wall
  exit code: 1
  stderr:
    main.c:5:3: error: implicit declaration of function 'foo'
```

The shell's stderr is indented and included verbatim. No reformatting, no truncation.

## Error Categories

Every error has a category in brackets:

| Category | Meaning |
|---|---|
| `parse` | Syntax error, unexpected token |
| `type` | Type mismatch, arity error |
| `runtime` | Division by zero, index out of bounds, pipeline failure |
| `io` | File not found, permission denied |
| `shell` | Shell command failure |
| `import` | Module not found, circular import |
| `assert` | Assertion failure in tests |
| `defer` | Error during deferred cleanup |

## Truthiness Errors

Since lx has no truthiness, using non-`Bool` values in conditional position is a type error caught at check time:

```
error[type]: src/main.lx:5:1
  |
 5|   count ? "has items" : "empty"
  |   ^^^^^
  expected: Bool
  got:      Int
  fix:      use a comparison: `count > 0 ? ...`
```

## `^` on Wrong Type

Using `^` on a value that is neither `Result` nor `Maybe`:

```
error[type]: src/main.lx:8:20
  |
 8|   name = get_name () ^
  |                      ^
  `^` requires Result or Maybe, got: Str
  fix:      remove `^` — this expression cannot fail
```

## Exhaustiveness Warnings

```
warning[match]: src/main.lx:12
  |
12|   shape ? {
  |   ^^^^^
  non-exhaustive match on Shape
  missing variants: Point
  fix:      add `Point _ -> ...` arm or `_ -> ...` catch-all
```

Exhaustiveness checking applies to tagged unions and `Bool`. Other types (Int, Str, etc.) always require a `_` catch-all.

## `assert` Failures

```
error[assert]: src/test/math_test.lx:8
  |
 8|   assert (add 1 2 == 4) "addition should work"
  |          ^^^^^^^^^^^^^^^
  assertion failed: add 1 2 == 4
  message: addition should work
  values: add 1 2 = 3, 3 == 4 = false
```

The `values` line shows the evaluated sub-expressions, making it clear where the expectation diverged. In test mode, execution continues to the next test. In normal mode, the program exits with code 1.

## Mutable Capture Errors

```
error[concurrency]: src/main.lx:15
  |
15|   xs | pmap (x) { count <- count + 1; process x }
  |                   ^^^^^
  cannot capture mutable binding `count` in concurrent context
  `count` is declared mutable at src/main.lx:12
  fix:      collect results and aggregate sequentially
```

## Import Conflict Errors

```
error[import]: src/main.lx:3
  |
 2|   use ./a {foo}
 3|   use ./b {foo}
  |            ^^^
  `foo` already imported from ./a (line 2)
  fix:      use module alias: `use ./b : b` then `b.foo`
```

## Variant Name Conflicts

```
error[type]: src/types.lx:5
  |
 3|   Color = | Red | Green | Blue
 5|   Light = | Red | Yellow | Green
  |             ^^^
  variant `Red` already defined by `Color` (line 3)
  fix:      rename variant or move to separate module
```

## Cross-References

- Implementation: [impl-error.md](../impl/impl-error.md) (error types, diagnostic generation)
- Error handling spec: [errors.md](errors.md)
- Toolchain JSON output: [toolchain.md](toolchain.md)
- Test suite: suite/15_diagnostics.lx (TODO)
