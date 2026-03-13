# Error Handling

No exceptions. Errors are values. Every fallible operation returns `Result a e`.

## Result and Maybe

```
Result a e = | Ok a | Err e
Maybe a = | Some a | None
```

Functions that can fail return `Result`:

```
read = (path: Str) -> Str ^ IoErr
```

The `^ IoErr` in the type signature is syntactic sugar for `-> Result Str IoErr`.

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
transform = (path: Str) -> Data ^ IoErr {
  content = read path ^       -- if Err, return Err immediately
  parse content ^             -- same: propagate on error
}
```

`expr ^` unwraps `Ok` or returns `Err` to the enclosing function. The enclosing function's return type must be compatible with the error type.

Multiple `^` in a function chain through naturally:

```
pipeline = (input: Str) -> Output ^ ProcessErr {
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
validate = (age: Int) -> Int ^ Str {
  age < 0 ? Err "age cannot be negative"
  age > 150 ? Err "age unrealistic"
  age
}
```

The last expression in a block is the return value. If it's not wrapped in `Ok`, it's implicitly `Ok` (the compiler wraps it).

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

In unannotated functions, `^` works with any error type — errors propagate dynamically:

```
load = (path) {
  content = fs.read path ^     -- IoErr
  json.parse content ^         -- ParseErr
}
```

Both error types propagate without wrapping. The caller receives whichever error occurred.

In annotated functions, declare a union error type:

```
AppErr = | Io IoErr | Parse ParseErr

load = (path: Str) -> Data ^ AppErr {
  content = fs.read path ^     -- auto-wrapped as Io
  json.parse content ^         -- auto-wrapped as Parse
}
```

The compiler wraps each `^` site's error into the declared union variant when the types don't match directly.

## Implicit Err Early Return

In a function with a `Result` return annotation (`-> T ^ E`), any bare expression statement (not a binding, not the final expression) that evaluates to an `Err` value immediately returns that `Err` from the function. This enables validation-style code without nesting:

```
validate = (age: Int) -> Int ^ Str {
  age < 0 ? Err "negative"         -- if true: Err returned immediately
  age > 150 ? Err "unrealistic"    -- if true: Err returned immediately
  age                               -- last expr: implicitly Ok age
}
```

When `age < 0` is true, the single-arm `?` evaluates to `Err "negative"`. Because this is a bare expression in a Result-annotated function, the function immediately returns `Err "negative"`. When the condition is false, the single-arm `?` evaluates to `()` (unit), which is not an `Err`, so execution continues.

This rule applies **only** to functions with an explicit `-> T ^ E` annotation. In unannotated functions, `Err` values in non-final position are ordinary values with no special behavior.

Bindings are not affected: `x = Err "msg"` stores the Err in `x` without early return.

## Implicit `Ok` Wrapping

The last expression in a function with a `Result` return type is implicitly wrapped in `Ok`:

```
validate = (age: Int) -> Int ^ Str {
  age < 0 ? Err "negative"         -- early return on Err
  age > 150 ? Err "unrealistic"    -- early return on Err
  age                               -- implicitly Ok age
}
```

The last expression is wrapped. Intermediate Err values cause early return (see above).

## Cross-References

- Implementation: [impl-interpreter.md](../impl/impl-interpreter.md) (error propagation, implicit Err early return), [impl-checker.md](../impl/impl-checker.md) (error type compatibility, Err early return validation)
- Design decisions: [design.md](design.md) (Division by zero is a panic, not Err; Implicit Err early return)
- Test suite: [09_errors.lx](../suite/09_errors.lx), [16_edge_cases.lx](../suite/16_edge_cases.lx)
