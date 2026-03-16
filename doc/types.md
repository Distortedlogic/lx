# Types — Reference

## Record Types

```
Point = {x: Float  y: Float}
```

Structural subtyping: any record with matching fields satisfies the type. Extra fields allowed.

## Tagged Unions (Sum Types)

```
Shape = | Circle Float | Rect Float Float | Point {x: Float  y: Float}
```

Construction: `Circle 5.0`, `Rect 3.0 4.0`

Elimination via pattern matching:

```
area = (s) s ? {
  Circle r -> 3.14159 * r * r | Rect w h -> w * h | Point {x y} -> 0.0
}
```

## Maybe and Result

```
Maybe a = | Some a | None
Result a e = | Ok a | Err e
```

`^` works on both: unwraps `Ok`/`Some`, propagates `Err`/`None`-as-Err.
`None` via `^` produces `Err "None at file:line:col"`. Use `require` for descriptive errors:

```
path = env.get "PATH" | require "PATH not set" ^
```

## Structural Subtyping

Any record with matching fields works. No explicit interface definitions needed:

```
greet = (thing) "hello {thing.name}"
greet {name: "alice"}            -- works
greet {name: "bob"  age: 30}     -- works (extra field ignored)
```

## Type Alias Semantics

Record types are structural aliases — same fields = same type (`Point` and `Velocity` with `{x: Float y: Float}` are interchangeable).

Tagged unions are nominal — variant tags distinguish them.

Variant constructors are globally unique within a module.

## Recursive Types

```
Tree a = | Leaf a | Node (Tree a) (Tree a)
Json = | JNull | JBool Bool | JNum Float | JStr Str | JArr [Json] | JObj %{Str: Json}
```
