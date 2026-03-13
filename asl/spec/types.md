# Types

Structural typing — types are shapes, not names. All type annotations are optional. The type system infers everything; annotations serve as documentation and disambiguation.

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
area = (s: Shape) -> Float  s ? {
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

## Generics

Type parameters are lowercase in type definitions:

```
first = (xs: [a]) -> Maybe a  xs ? {
  [x .._] -> Some x
  []      -> None
}

map = (f: a -> b  xs: [a]) -> [b]
```

Generic types in definitions:

```
Pair a b = {fst: a  snd: b}
Tree a = | Leaf a | Node (Tree a) (Tree a)
```

## Type Annotations

Always optional. When present, they follow `:` for parameters and `->` for returns:

```
add = (x y) x + y                     -- inferred
add = (x:Int y:Int) -> Int  x + y     -- annotated

fetch = (url: Str) -> Str ^ HttpErr   -- annotated with error type
```

`^` in type signatures means "this function can fail with this error type":

```
-> Str ^ IoErr        -- returns Str, can fail with IoErr
-> [User] ^ ApiErr    -- returns list of User, can fail with ApiErr
```

## Structural Subtyping

No explicit trait/interface definitions in v1. Instead, any function that expects `{name: Str}` accepts any record that has a `name: Str` field, regardless of what other fields it has.

```
greet = (thing: {name: Str}) "hello {thing.name}"

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

## Function Types

Function types use `->`:

```
Int -> Int                    -- one arg, one return
Int -> Int -> Int             -- two args (right-associative: Int -> (Int -> Int))
(Int -> Bool) -> [Int] -> [Int]  -- higher-order: filter's type
```

In annotations:

```
apply = (f: a -> b  x: a) -> b  f x
map = (f: a -> b  xs: [a]) -> [b]
```

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

## Error Type Flexibility

When a function uses `^` on operations with different error types, the behavior depends on annotations:

**Annotated functions** — the declared error type must be compatible with all `^` sites. Use a tagged union to combine multiple error types:

```
AppErr = | Io IoErr | Parse ParseErr

load = (path: Str) -> Data ^ AppErr {
  content = fs.read path ^       -- IoErr, wrapped as Io
  json.parse content ^           -- ParseErr, wrapped as Parse
}
```

**Unannotated functions** — errors propagate dynamically. The runtime preserves the original error value and its propagation trace. No wrapping needed:

```
load = (path) {
  content = fs.read path ^       -- IoErr propagates as-is
  json.parse content ^           -- ParseErr propagates as-is
}
```

In unannotated code, `^` works with any error type. This is the scripting-friendly default — errors flow without ceremony. Add annotations when you want the type checker to verify error handling.

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

## Type Inference

lx uses bidirectional type checking with local type inference. The inference engine:

1. Propagates known types downward (from annotations and known function signatures)
2. Synthesizes types upward (from literals, operators, and function application)
3. Unifies constraints at each binding site

The inference is local — each function is checked independently. Polymorphic functions are instantiated at each call site. No global constraint solving (Hindley-Milner style), which keeps error messages predictable and localized.

When inference cannot determine a type (ambiguous polymorphism, empty collections), the compiler reports an error at the ambiguous site and suggests adding an annotation. This is rare in practice — most code has enough context from usage.

```
xs = []                    -- ERROR: cannot infer element type
xs = [] : [Int]            -- OK: annotated
xs = [1 2 3]               -- OK: inferred as [Int]
```

Type annotations on function parameters use `:` inline. Standalone type annotations for bindings use `: Type` after the binding name:

```
count : Int = compute_something ()
xs : [Str] = []
```

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

- Implementation: [impl-checker.md](../impl/impl-checker.md) (type inference, structural subtyping, unification)
- Test suite: [12_types.lx](../suite/12_types.lx)
