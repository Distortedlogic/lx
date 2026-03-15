# Runtime Semantics

How values behave at runtime: numeric precision, collection access, string encoding, equality, and interop with the host system.

## Numbers

**Integers** — Arbitrary precision (bigint) by default. No overflow, no surprises. A scripting language should never produce wrong answers because a number got too big. Performance-sensitive code can use `Int32`/`Int64` explicitly.

**Floats** — IEEE 754 `f64`. `3.14` is a float. `3` is an int. Distinct types.

**Widening** — `3 + 4.0` widens the int to float automatically (lossless direction: `Int -> Float`). The result is `7.0` (Float). Narrowing (`Float -> Int`) requires an explicit call: `floor`, `ceil`, `round`, or `trunc`.

**Division by zero** — Runtime panic, not a recoverable error. `10 / 0` prints a diagnostic with source location and aborts (like `assert`). Division by zero is a programmer bug — validate the divisor before dividing. For data pipelines where zero divisors are expected, use `math.safe_div a b -> Result Int Str`. Integer division `//` and modulo `%` also panic on zero. In test mode, panics are caught per-test.

## Collection Access

**Direct access with `.`** — `.0`, `.name`, `.-1`. This is an assertion: "I know this exists." If the index/field is missing, it's a runtime error.

**Functional access with `get`** — `get 0 xs`, `get "name" record`. Returns `Maybe a`. Use when the element might not exist.

```
xs.0              -- 1 (or runtime error if empty)
get 0 xs          -- Some 1 (or None if empty)

record.name       -- "alice" (or runtime error if no `name` field)
get "name" record -- Some "alice" (or None)
```

**Negative indices** — `xs.-1` is the last element, `xs.-2` is second to last. Works with both `.` access and `get`.

## Strings

**Encoding** — All strings are UTF-8. This is not configurable.

**`len`** — Returns codepoint count. `"hello" | len` is `5`. `"cafe\u0301" | len` is `5` (4 chars + combining accent).

**`byte_len`** — Returns byte count. `"hello" | byte_len` is `5`. `"\u00e9" | byte_len` is `2`.

**Indexing** — `str.3` indexes by codepoint. `"hello".1` is `"e"`.

**Binary data** — `Bytes` type, distinct from `Str`. `fs.read_bytes path` returns `Bytes`. `fs.read path` returns `Str` (errors on invalid UTF-8). Conversion: `bytes | decode "utf-8"` and `str | encode "utf-8"`.

## Equality

Structural. `{x: 1 y: 2} == {x: 1 y: 2}` is `true`. Lists, sets, maps, tuples all compare structurally.

Record equality is order-independent: `{x: 1 y: 2} == {y: 2 x: 1}` is also `true`. Records compare by field names and values, not insertion order.

Functions are not comparable. `f == g` is a runtime error regardless of whether they're "the same" function.

## Interop

**Exit codes** — `main` returning `()` exits 0. `main` returning `Err _` exits 1. `main` returning an `Int` exits with that code. `env.exit n` for explicit control anywhere.

**Stdin/stdout/stderr**:
- `$echo` goes to stdout (returns result with `.out`)
- `log` goes to stderr
- Shell commands handle stdin/stdout interaction

**Logging** — Built-in `log` namespace with four level functions:

```
log.info "info message"
log.warn "something odd"
log.err "something wrong"
log.debug "internal detail"
```

`log` is a record with fields `info`, `warn`, `err`, `debug`, each a function taking a string. Output goes to stderr. Controlled by `LX_LOG` env var (`LX_LOG=debug`, `LX_LOG=warn`). Default level: `info`.

**Environment variables**:

```
env.get "PATH"          -- Maybe Str
env.get "PATH" ^        -- Str (propagates Err if not set)
env.get "PATH" | require "PATH must be set" ^  -- with custom message
```

`env.set` is blocked in `--sandbox` mode.

**Shebang** — `#!/usr/bin/env lx` on line 1 is ignored by the lexer.

## Truthiness

There is no truthiness. The `?` ternary and single-arm forms require a `Bool` value:

```
x > 0 ? "positive" : "non-positive"   -- ok: Bool condition
```

Non-booleans in conditional position are type errors:

```
0 ? "yes" : "no"           -- ERROR: Int is not Bool
"" ? "yes" : "no"          -- ERROR: Str is not Bool
None ? "yes" : "no"        -- ERROR: Maybe is not Bool
xs ? "non-empty" : "empty" -- ERROR: use (xs | len > 0) ?
```

The multi-arm `?` (pattern matching) accepts any type — it matches on structure, not truthiness.

## Closures

Functions are closures — they capture their lexical scope by reference. See [syntax.md](syntax.md) for details and examples.

Captured mutable bindings are shared: if a closure captures `x :=`, mutations via `x <-` are visible to all closures sharing that scope. This is intentional for counters and accumulators.

**Concurrency restriction**: capturing a mutable binding inside `par`, `sel`, or `pmap` bodies is a compile error. Concurrent code must not share mutable state implicitly — copy the value into a local immutable binding first:

```
count := 0
-- ERROR: cannot capture mutable `count` in pmap
xs | pmap (x) { count <- count + 1; process x }

-- OK: use fold or collect results
results = xs | pmap process
count = results | len
```

This prevents data races without locks or atomics.

## Bitwise Operations

Bitwise operators are not available — `|`, `&`, and `^` are used for pipes, guards, and error propagation. Bitwise operations are not implemented in v1.

## Tuple Disambiguation

`(expr)` is grouping — it evaluates `expr` and returns the result. `(expr1 expr2)` is a tuple. `()` is unit.

```
(1 + 2)       -- 3 (grouping)
(1 2)         -- tuple of (1 2)
(1 "a" true)  -- 3-tuple
()            -- unit
```

There is no one-element tuple. If you need a single-element container, use `[x]`.

## Block Evaluation and Err Short-Circuit

A block `{ stmt1; stmt2; ...; stmtN }` evaluates each statement in order and returns the value of the last statement. In normal blocks, intermediate expression values are discarded.

In a function body, intermediate expression statements that evaluate to `Err e` cause the function to return `Err e` immediately. See [errors.md](errors.md) for details and examples.

`break` without a value expression returns unit `()` from the enclosing `loop`. `break val` returns `val`.

## `defer` Scoping

`defer` registers cleanup for the **immediately enclosing block scope** (`{}`), not the function. Multiple defers run LIFO when the scope exits (normal completion, error propagation via `^`, or `break`).

```
handle = fs.open path ^
defer () fs.close handle
process handle
-- fs.close runs here when the enclosing block exits
```

Inside a loop, `defer` registers per-iteration:

```
loop {
  h = fs.open (next_path ()) ^
  defer () fs.close h         -- closes h at end of EACH iteration
  process h
}
```

Put `defer` next to the resource acquisition, at the scope level where cleanup should happen. Don't put defers inside loops unless cleanup should happen per-iteration.

## Tail Call Optimization

Tail calls in tail position use constant stack space. Tail position is:

- The body of a function (the last expression)
- Each arm body in a `? { }` match
- The then-branch and else-branch of ternary `? :`
- The last expression in a `{ }` block

NOT tail position:
- Arguments to other functions: `f (g x)` — `g x` is not in tail position
- Left side of `|`: `f x | g` — `f x` is not in tail position
- Inside `^`: `f x ^` — the `^` unwrap means `f x` returns to the unwrap logic, not to the caller
- Inside `par`/`sel`/`pmap` — concurrent blocks don't share the caller's stack

The compiler may warn when a visually-recursive function is not actually tail-recursive.

## `assert` Semantics

`assert` is a hard failure — it panics and stops execution. It is NOT recoverable via `^` or `??`. The condition must be `Bool` — non-boolean values are a type error, consistent with the no-truthiness rule.

```
assert (x > 0)                    -- panics if false
assert (x > 0) "x must be positive"  -- panics with message
```

The panic prints the assertion expression, source location, and optional message to stderr, then exits with code 1. In test mode (`lx test`), the test runner catches panics and continues with other tests, collecting all failures.

`assert` exists for invariant checking and tests, not for error handling. Use `Result`/`^`/`??` for recoverable errors.

## Forward References

Top-level bindings can reference each other regardless of definition order — mutual recursion between top-level functions works:

```
is_even = (n) n == 0 ? true : is_odd (n - 1)
is_odd = (n) n == 0 ? false : is_even (n - 1)
```

Within blocks, bindings are sequential — a binding can only reference bindings defined before it (or itself for direct recursion):

```
process = () {
  helper = (x) x * 2         -- ok
  result = helper 5           -- ok: helper is defined above
  -- bad = undefined_below 5  -- ERROR: not yet defined
}
```

## Shadowing

Shadowing with `=` creates a new immutable binding that hides the previous one in the same scope. The original binding is unaffected in closures that captured it before the shadow:

```
x = 5
f = () x         -- captures x = 5
x = 10           -- shadows x
f ()             -- still 5 (captured the original)
x                -- 10
```

Built-in functions (`map`, `filter`, `len`, etc.) can be shadowed. The compiler warns when shadowing a built-in.

## `it` in `sel` Blocks

`it` is the only implicit binding in lx. In `sel` arm handlers, `it` refers to the result of the completed expression:

```
sel {
  fetch url   -> it.body | process
  timeout 5   -> Err "timed out"
}
```

`it` is scoped to the handler expression — it is not available outside the `sel` block, and it does not shadow any outer binding named `it`.

## Type Coercions

The only automatic coercion is `Int -> Float` widening: `3 + 4.0` produces `7.0`. This is lossless (every integer representable in `Int` that fits in `f64` range converts exactly).

All other conversions require explicit function calls:
- `Float -> Int`: `floor`, `ceil`, `round`, `trunc`
- `Str -> Int`: `parse_int`
- `Str -> Float`: `parse_float`
- `any -> Str`: `to_str` (also used implicitly by string interpolation `"{expr}"`)

There are no implicit conversions between `Bool` and `Int`, `Str` and `Bool`, or any other type pair.

## `to_str` Conversion

String interpolation `"{expr}"` calls `to_str` on the result of `expr`. Every type has a `to_str` representation:

- Int/Float: decimal representation
- Bool: `"true"` / `"false"`
- Str: identity
- List: `"[1 2 3]"`
- Record: `"{x: 1  y: 2}"`
- Tagged union: `"Circle 5.0"`
- Unit: `"()"`
- Maybe/Result: `"Some 42"` / `"Err \"msg\""`

`to_str` is called implicitly only in string interpolation. There is no implicit conversion in any other context.
