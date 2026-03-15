# Error Handling

No exceptions. Errors are values. Every fallible operation returns `Result a e`.

## Result and Maybe

```
Result a e = | Ok a | Err e
Maybe a = | Some a | None
```

Functions that can fail return `Result`.

## Explicit Matching

Handle errors by matching on the result:

```
r = read "file.txt"
r ? {
  Ok content -> process content
  Err e      -> log.err "failed: {e}"
}
```

## Propagation with `^`

`^` propagates errors to the caller — like Rust's `?` operator:

```
transform = (path) {
  content = read path ^
  parse content ^
}
```

`expr ^` unwraps `Ok` or returns `Err` to the enclosing function.

Multiple `^` in a function chain through naturally:

```
pipeline = (input) {
  raw = fetch input ^
  parsed = parse raw ^
  validated = validate parsed ^
  transform validated ^
}
```

Each `^` site is recorded for error traces (see [diagnostics.md](diagnostics.md)).

## Coalescing with `??`

`??` provides a default value when an expression produces `Err` or `None`:

```
content = read "file.txt" ?? "fallback"
user_name = get "name" config ?? "anonymous"
```

Composes with sections in pipelines:

```
data | map fetch | map (?? default_val)
```

## Pipeline Error Patterns

Error handling within pipelines uses existing constructs — no special syntax needed:

```
-- propagate first error out of the pipeline
data | map (x) fetch x ^

-- coalesce each result individually
data | map (x) fetch x ?? default

-- collect results, then separate successes from failures
data | map fetch | partition ok?

-- get list of raw results for manual handling
results = data | map fetch    -- [Result a e]
```

## Error Construction

Create errors directly with `Err`:

```
validate = (age) {
  age < 0 ? Err "age cannot be negative"
  age > 150 ? Err "age unrealistic"
  age
}
```

## Chaining Fallible Operations

Use `^` and pipes together:

```
url | fetch ^ | json.parse ^ | (.data) | validate ^
```

Each `^` short-circuits the pipeline if the preceding step fails.

## `^` on Maybe Values

`^` works on both `Result` and `Maybe`:

```
get 0 xs ^             -- unwraps Some, propagates None as Err
env.get "PATH" ^       -- unwraps Some, propagates None as Err
```

When applied to `None`, `^` produces `Err "None at file:line:col"` with source location. For custom error messages, use `require`:

```
name = get "name" config | require "name field required" ^
```

## Error Type Compatibility

`^` works with any error type — errors propagate dynamically:

```
load = (path) {
  content = fs.read path ^
  json.parse content ^
}
```

Both error types propagate without wrapping. The caller receives whichever error occurred.

## Implicit Err Early Return

Implicit Err early return was tied to `-> T ^ E` annotations, which have been removed. In current lx, `Err` values in non-final position are ordinary values.

## Implicit `Ok` Wrapping

Implicit `Ok` wrapping was tied to `-> T ^ E` annotations, which have been removed. In current lx, functions return values as-is without automatic `Ok` wrapping.

## Cross-References

- Implementation: [impl-interpreter.md](../impl/impl-interpreter.md) (error propagation, implicit Err early return), [impl-checker.md](../impl/impl-checker.md) (error type compatibility, Err early return validation)
- Design decisions: [design.md](design.md) (Division by zero is a panic, not Err; Implicit Err early return)
- Test suite: [09_errors.lx](../suite/09_errors.lx), [16_edge_cases.lx](../suite/16_edge_cases.lx)
