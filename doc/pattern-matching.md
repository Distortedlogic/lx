# Pattern Matching — Reference

`?` is the match operator — three modes based on what follows it.

## Multi-Arm Match

```
x ? {
  0 -> "zero"
  1 -> "one"
  n -> "{n}"
}
```

First matching arm wins. Returns a value.

## Ternary Form

`x > 0 ? "positive" : "non-positive"` — condition must be `Bool`.

## Single-Arm (Conditional)

`x > 0 ? do_thing x` — returns unit if false. Condition must be `Bool`.

## Destructuring

Records:
```
point ? { {x: 0  y} -> "y-axis" | {x  y: 0} -> "x-axis" | {x  y} -> "{x},{y}" }
```

Lists: `[] -> "empty"` | `[x] -> "one"` | `[x ..rest] -> "many"`

Tuples: `(0 _) -> "first zero"` | `(a b) -> "pair"`

Tagged unions: `Circle r -> pi * r * r` | `Rect w h -> w * h`

## Binding Destructure

Outside `?` blocks, destructure with `=`:

```
(a b c) = get_triple ()
{name age} = get_person ()
[first ..rest] = get_list ()
```

## Guards

`&` introduces a guard: `n & (n > 0) -> "positive"`. Pattern variables are in scope within the guard.

## Wildcard

`_` matches anything, discards the value.

## Nested Patterns

```
data ? {
  {users: [first ..rest]  status: "active"} -> process first rest
  {users: []  status: _}                    -> log.info "no users"
  _                                         -> log.warn "unexpected"
}
```

## Maybe and Result

```
env.get "HOME" ? { Some path -> use path | None -> log.warn "not set" }
fs.read "f.txt" ? { Ok content -> parse content | Err e -> default }
```

## Literal Patterns

Integers, floats, strings, booleans match by value.

## Exhaustiveness

Compiler warns on non-exhaustive matches. Use `_` as catch-all.

## Gotchas

- `?` followed by `{` always starts multi-arm match. For record literal as ternary value, wrap in parens: `cond ? ({x: 1}) : ({x: 0})`
- No or-patterns (`1 | 2 -> ...`) — conflicts with pipe. Use guards.
- No string interpolation patterns — exact value only. Use guards with string functions.
