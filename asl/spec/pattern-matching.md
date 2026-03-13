# Pattern Matching

`?` is the match operator — the primary control flow mechanism in lx.

## Multi-Arm Match

Match an expression against patterns. First matching arm wins.

```
x ? {
  0 -> "zero"
  1 -> "one"
  n -> "{n}"
}
```

Arms are `pattern -> expr`. The match expression returns a value (everything is an expression).

## Ternary Form

Two-arm match with `:` separator — replaces if/else:

```
x > 0 ? "positive" : "non-positive"
```

## Single-Arm (Conditional)

One arm, no else. Returns unit if the condition is false:

```
x > 0 ? do_thing x
```

## Destructuring

Records:

```
point ? {
  {x: 0  y} -> "y-axis at {y}"
  {x  y: 0} -> "x-axis at {x}"
  {x  y}    -> "{x},{y}"
}
```

Lists (head/tail):

```
list ? {
  []         -> "empty"
  [x]        -> "singleton: {x}"
  [x ..rest] -> "head={x} rest={rest}"
}
```

Tuples:

```
pair ? {
  (0 _) -> "first is zero"
  (a b) -> "first={a} second={b}"
}
```

Tagged unions:

```
shape ? {
  Circle r     -> 3.14159 * r * r
  Rect w h     -> w * h
  Point {x y}  -> 0.0
}
```

## Binding Destructure

Outside of `?` blocks, destructure directly with `=`:

```
(a b c) = get_triple ()
{name age} = get_person ()
[first ..rest] = get_list ()
```

## Guards

`&` introduces a guard condition on a pattern:

```
n ? {
  0            -> "zero"
  n & (n > 0)  -> "positive"
  _            -> "negative"
}
```

The guard is `& (expr)` where the expr must evaluate to `true` for the arm to match. The pattern variables are in scope within the guard.

## Wildcard

`_` matches anything and discards the value:

```
result ? {
  Ok value -> process value
  Err _    -> log.err "something failed"
}
```

## Nested Patterns

Patterns compose:

```
data ? {
  {users: [first ..rest]  status: "active"} -> process first rest
  {users: []  status: _}                    -> log.info "no users"
  _                                         -> log.warn "unexpected shape"
}
```

## Exhaustiveness

The compiler warns on non-exhaustive matches. Use `_` as a catch-all when needed. Tagged union matches must cover all variants or include `_`.

## Literal Patterns

Integers, floats, strings, and booleans match by value:

```
cmd ? {
  "start" -> run ()
  "stop"  -> halt ()
  "help"  -> usage ()
  _       -> $echo "unknown: {cmd}"
}
```

## `?` Modes

`?` operates in three modes depending on what follows it:

**Multi-arm match** — `expr ? { arms }`. Matches against patterns:

```
x ? {
  0 -> "zero"
  n -> "{n}"
}
```

**Ternary** — `bool_expr ? then_val : else_val`. The expression MUST be `Bool`:

```
x > 0 ? "positive" : "non-positive"
```

**Single-arm** — `bool_expr ? then_val`. Returns `then_val` if true, unit if false. Must be `Bool`:

```
debug? ? log.debug "verbose mode"
```

## `Maybe` and `Result` Matching

`Maybe` and `Result` are tagged unions. Match on them with their constructors:

```
env.get "HOME" ? {
  Some path -> log.info "home is {path}"
  None      -> log.warn "HOME not set"
}

fs.read "config.txt" ? {
  Ok content -> parse content
  Err e      -> log.err "failed: {e}"; default_config
}
```

Combine with destructuring:

```
resp | json.parse ? {
  Ok {."users": users ..} -> users | each process
  Ok _                    -> log.warn "unexpected shape"
  Err e                   -> log.err "parse error: {e}"
}
```

## `?` Disambiguation

`?` followed by `{` **always** starts a multi-arm match. The parser does not try to guess whether `{...}` is a record literal or a match block.

To use a record literal as the then-value of a ternary, wrap it in parens:

```
cond ? ({x: 1  y: 2}) : ({x: 0  y: 0})
```

Without parens, `cond ? {x: 1 ...}` enters match mode and fails when it finds `:` instead of `->`.

## Design Constraints

**No or-patterns** — `1 | 2 -> ...` conflicts with pipe. Use guards instead:

```
n ? {
  n & (n >= 1 && n <= 5) -> "small"
  n & (n >= 6 && n <= 10) -> "medium"
  _ -> "large"
}
```

**No string interpolation patterns** — strings match by exact value only. Use regex or string functions with guards for prefix/suffix/substring matching:

```
url ? {
  u & (starts? "https://" u) -> handle_secure u
  u & (starts? "http://" u)  -> handle_plain u
  _ -> Err "unsupported"
}
```

## Cross-References

- Implementation: [impl-parser.md](../impl/impl-parser.md) (pattern parsing), [impl-checker.md](../impl/impl-checker.md) (exhaustiveness checking), [impl-interpreter.md](../impl/impl-interpreter.md) (pattern evaluation)
- Test suite: [07_patterns.lx](../suite/07_patterns.lx)
