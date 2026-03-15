# Types

Type definitions define shapes for records and tagged unions. Type annotations on function parameters, return types, and bindings are optional. The type checker (`lx check`) validates annotations via bidirectional inference; `lx run` skips checking and relies on runtime semantics.

## Record Types

A record type defines a shape (set of fields and their types):

```
Point = {x: Float  y: Float}
```

Any record with matching fields satisfies the type. Structural subtyping: a record with extra fields is a valid `Point` if it has `x: Float` and `y: Float`.

```
p = {x: 3.0  y: 4.0  z: 5.0}   -- satisfies Point (extra field ok)
```

## Tagged Unions (Sum Types)

Variants prefixed with `|`:

```
Shape =
  | Circle Float
  | Rect Float Float
  | Point {x: Float  y: Float}
```

Constructing values:

```
c = Circle 5.0
r = Rect 3.0 4.0
```

Eliminating with pattern matching:

```
area = (s) s ? {
  Circle r     -> 3.14159 * r * r
  Rect w h     -> w * h
  Point {x y}  -> 0.0
}
```

## Built-in Unions

```
Maybe a = | Some a | None
Result a e = | Ok a | Err e
```

`Maybe` for optional values. `Result` for fallible operations. These are the only union types used by the stdlib — no `Option`/`Either`/`Optional` aliases.

`Bytes` is a distinct primitive type for binary data. Created by `fs.read_bytes`, consumed by `decode`. See [runtime.md](runtime.md) for details.

## Structural Subtyping

Any record with matching fields satisfies a structural expectation. No explicit trait/interface definitions needed.

```
greet = (thing) "hello {thing.name}"

greet {name: "alice"}                    -- works
greet {name: "bob"  age: 30}             -- works (extra field ignored)
greet {name: "carol"  email: "c@x.com"}  -- works
```

This gives duck-typing ergonomics with structural guarantees.

## Type Alias Semantics

Record type definitions are structural aliases:

```
Point = {x: Float  y: Float}
Velocity = {x: Float  y: Float}
```

`Point` and `Velocity` are interchangeable — they describe the same shape. A type alias is a name for a structure, not a distinct type.

Tagged union definitions are nominal — the variant tags distinguish them:

```
Shape = | Circle Float | Rect Float Float
Color = | Red | Green | Blue
```

`Shape` and `Color` are distinct. The tags (`Circle`, `Red`, etc.) are the distinguishing identifiers.

## Recursive Types

Type definitions can reference themselves:

```
Tree a = | Leaf a | Node (Tree a) (Tree a)
Json =
  | JNull
  | JBool Bool
  | JNum Float
  | JStr Str
  | JArr [Json]
  | JObj %{Str: Json}
```

## `^` on Maybe Values

`^` works on both `Result` and `Maybe`:

```
x = some_result ^    -- unwraps Ok, propagates Err
y = some_maybe ^     -- unwraps Some, propagates None-as-Err
```

When `^` is applied to `None`, it produces `Err "None at file:line:col"` and propagates. For descriptive error messages, use `require`:

```
path = env.get "PATH" | require "PATH not set" ^
```

`require` converts `Maybe a` to `Result a Str`: `Some v` becomes `Ok v`, `None` becomes `Err msg`.

## Nominal vs Structural

Record types are structural — two types with the same fields are interchangeable:

```
Point = {x: Float  y: Float}
Velocity = {x: Float  y: Float}
-- Point and Velocity are the same type
```

Tagged unions are nominal — the variant tags distinguish them:

```
Shape = | Circle Float | Rect Float Float
Color = | Red | Green | Blue
-- Shape and Color are distinct, even if they had the same variant names
```

Variant constructors (`Circle`, `Red`, etc.) are globally unique within a module. Two tagged unions in the same module cannot share a variant name. Imported variant names follow the same rule — if two imports define the same variant, use module-qualified access.

## Cross-References

- Test suite: [12_types.lx](../tests/12_types.lx)
