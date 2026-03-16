# Runtime Semantics — Reference

## Numbers

**Integers** — Arbitrary precision (bigint). No overflow.
**Floats** — IEEE 754 `f64`. `3.14` is float, `3` is int. Distinct types.
**Widening** — `3 + 4.0` widens int to float (lossless). Narrowing requires `floor`/`ceil`/`round`/`trunc`.
**Division by zero** — Runtime panic. Use `math.safe_div` for data pipelines.

## Collection Access

`.` (direct) — asserts element exists, runtime error if missing: `xs.0`, `record.name`
`get` (functional) — returns `Maybe a`: `get 0 xs`, `get "name" record`
Negative indices: `xs.-1` is last element. Works with both.

## Strings

UTF-8. `len` = codepoint count. `byte_len` = byte count. `str.3` indexes by codepoint.
`Bytes` is distinct from `Str`. Convert: `bytes | decode "utf-8"`, `str | encode "utf-8"`.

## Equality

Structural. Records are order-independent. Functions are not comparable (runtime error).

## Truthiness

None. Ternary/single-arm `?` require `Bool`. `0 ? ...` and `None ? ...` are type errors. Multi-arm `?` accepts any type.

## Closures and Mutable Capture

Capture lexical scope by reference. Mutable captures (`x :=`) shared across closures.
Capturing mutables inside `par`/`sel`/`pmap` is a compile error.

## `defer`

Cleanup for immediately enclosing block scope, not function. Multiple defers run LIFO. Per-iteration inside loops.

```
handle = fs.open path ^
defer () fs.close handle
```

## Tail Call Optimization

Tail position: function body, `? {}` arm bodies, ternary branches, last expr in block.
NOT tail: function args, left of pipe, inside `^`, inside `par`/`sel`/`pmap`.

## `assert`

Panics (not recoverable). Condition must be `Bool`. In test mode, caught per-test.

```
assert (x > 0) "x must be positive"
```

## Forward References

Top-level: any order (mutual recursion works). Within blocks: sequential only.

## Shadowing

`=` creates new binding hiding previous. Closures that captured the original keep it.

## `to_str`

`"{expr}"` calls `to_str`. Int/Float=decimal, Bool=`"true"`/`"false"`, List=`"[1 2 3]"`, Record=`"{x: 1}"`, Tagged=`"Circle 5.0"`, Unit=`"()"`.

## Tuples

`(expr)` = grouping. `(a b)` = tuple. `()` = unit. No one-element tuple.

## Block Evaluation

Returns last statement value. Intermediate `Err e` in function bodies returns immediately. `break`/`break val` exits `loop`.

## `it` in `sel`

`it` = result of completed expression in `sel` arm handlers. Scoped to handler only.

## Type Coercions

Only automatic: `Int -> Float`. All others explicit: `floor`/`ceil`/`round`/`trunc`, `parse_int`, `parse_float`, `to_str`.
