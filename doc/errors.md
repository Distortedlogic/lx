# Error Handling — Reference

No exceptions. Errors are values.

## Result and Maybe

```
Result a e = | Ok a | Err e
Maybe a = | Some a | None
```

## Matching

```
r = read "file.txt"
r ? {
  Ok content -> process content
  Err e      -> log.err "failed: {e}"
}
```

## Propagation with `^`

`^` unwraps `Ok`/`Some` or returns `Err` to the caller (like Rust's `?`):

```
pipeline = (input) {
  raw = fetch input ^
  parsed = parse raw ^
  validated = validate parsed ^
  transform validated ^
}
```

On `None`, `^` produces `Err "None at file:line:col"`. For custom messages, use `require`:

```
name = get "name" config | require "name field required" ^
```

## Coalescing with `??`

`??` provides a default for `Err` or `None`:

```
content = read "file.txt" ?? "fallback"
user_name = get "name" config ?? "anonymous"
```

## Error Construction

```
validate = (age) {
  age < 0 ? Err "age cannot be negative"
  age > 150 ? Err "age unrealistic"
  age
}
```

## Pipeline Patterns

```
data | map (x) fetch x ^           -- propagate first error
data | map (x) fetch x ?? default  -- coalesce each result
data | map fetch | partition ok?   -- separate successes/failures
results = data | map fetch         -- [Result a e] for manual handling
```

Chaining fallible ops:

```
url | fetch ^ | json.parse ^ | (.data) | validate ^
```

## `^` on Maybe

```
get 0 xs ^             -- unwraps Some, propagates None as Err
env.get "PATH" ^       -- unwraps Some, propagates None as Err
```
