-- Memory: ISA manual (core). lx syntax and semantics — the language primitives.
-- Update when language semantics change. See also AGENTS.md and STDLIB.md.

# lx Core Language

## Basics

```lx
x = 42                       -- immutable binding
x := 0                       -- mutable binding
x <- x + 1                   -- reassign mutable
name = "world"
greeting = "hello {name}"    -- string interpolation
raw = `no {interpolation}`   -- raw string (backtick)
```

Collections:
```lx
[1 2 3]                      -- list (space-separated, no commas)
{x: 1  y: 2}                 -- record (fixed keys, all fields need key: value)
{:}                          -- empty record
(1 "hello" true)             -- tuple
%{"key": "value"}            -- map (arbitrary keys)
```

Access and update:
```lx
xs.0  xs.-1                  -- index / negative index
xs.1..3                      -- slice (start..end exclusive)
record.name                  -- field access (returns None if missing)
{..record  x: 5}             -- record update (new record)
[..xs 4 5]                   -- list spread
```

## Functions

```lx
double = (x) x * 2
add = (x y) x + y
greet = (name  greeting = "hello") "{greeting} {name}"
make_adder = (n) (x) x + n  -- closure
```

Type annotations (validated by `lx check`, ignored by `lx run`):
```lx
add = (x: Int y: Int) -> Int x + y
safe_div = (a: Int b: Int) -> Float ^ Str { ... }
```

## Arithmetic

`/` always returns Float (even for two Ints). `//` is integer division:
```lx
7 / 2          -- 3.5 (Float)
7 // 2         -- 3 (Int)
15 / 3         -- 5.0 (Float)
```

Mixed Int/Float auto-promotes to Float:
```lx
3 * 0.5        -- 1.5 (Int * Float → Float)
10 + 2.0       -- 12.0 (Int + Float → Float)
```

## Pipes — The Core Composition Tool

`|` passes the left value as the **last** argument to the right function:

```lx
xs | map (* 2) | filter (> 0) | sum
url | fetch ^ | (.body) | json.parse ^
names | sort | take 5 | join ", "
```

Sections — partial application syntax:

```lx
(+ 1)      -- (x) x + 1
(* 2)      -- (x) x * 2
(> 0)      -- (x) x > 0
(.name)    -- (x) x.name
(10 -)     -- (x) 10 - x
(?? 0)     -- (x) x ?? 0
```

Prefer `map (.name)` over `map (x) x.name`.

## Error Handling

Errors are values, not exceptions. Two families:

```lx
Ok 42          Err "failed"     -- Result
Some "hi"      None             -- Maybe
```

Two operators:

```lx
value = risky_call ^            -- ^ unwraps Ok/Some, propagates Err/None up
fallback = risky_call ?? "default"  -- ?? provides fallback on Err/None
```

Compose in pipes:
```lx
url | fetch ^ | (.body) | json.parse ^      -- any failure propagates
config | (.timeout) ?? 30                     -- missing field → default
```

Predicates: `ok?`, `err?`, `some?`. `require` converts Maybe to Result.

Structured error tags:
```lx
Err Timeout "took too long"
result ? { Err Timeout msg -> retry; Err e -> fail e; Ok v -> v }
```

## Pattern Matching

```lx
x ? {
  0 -> "zero"
  1 | 2 -> "small"
  n & (n > 100) -> "big {n}"
  _ -> "other"
}

shape ? { Circle r -> r * r  Rect w h -> w * h  Dot -> 0 }
result ? { Ok v -> v  Err e -> handle e }
{name: "alice" ..} -> "found alice"     -- record pattern with rest

x > 0 ? "positive" : "non-positive"     -- ternary
x > 0 ? do_thing x                      -- single-arm (unit if false)
```

Destructuring in bindings:
```lx
(a b c) = (1 2 3)           -- tuple
{name  age} = person         -- record
[first ..rest] = items       -- list with rest
```

## Type Definitions

```lx
Point = {x: Float  y: Float}
Shape = | Circle Float | Rect Float Float | Dot
Tree a = | Leaf a | Node (Tree a) (Tree a)
```

Constructors work as functions: `Circle 5.0`, `Node (Leaf 1) (Leaf 2)`.

## Modules

```lx
use std/json                   -- whole module (json.parse, json.encode)
use std/json : j               -- aliased (j.parse, j.encode)
use std/json {parse encode}    -- selective (parse, encode as bare names)
use ./util                     -- relative import
use brain/protocols            -- workspace member import (member/path)
+exported_fn = (x) x * 2      -- + prefix = exported
```

Resolution order: stdlib → workspace member → relative path.
Workspace: first path segment matches member name → resolve rest from member's root.

## Control Flow

```lx
1..10 | each (n) { process n }         -- iteration
loop { condition ? break value }        -- loop with break
par { fetch url1 ^; fetch url2 ^ }      -- parallel (returns tuple)
sel { fetch url -> it; timeout 5 -> Err "slow" }  -- race
xs | pmap fetch                          -- parallel map
xs | pmap_n 10 fetch                     -- rate-limited parallel map
```

## Shell Integration

```lx
r = $echo "hello {name}"     -- Result {out err code} ShellErr
s = $^pwd | trim              -- $^ extracts stdout string directly
block = ${                    -- multi-line session (commands share state)
  cd /tmp
  pwd
}
```

`|` inside `$` is a shell pipe. To chain to lx: `($^ls src) | lines`.

## Concurrency

```lx
(a b c) = par {
  fetch url1 ^
  fetch url2 ^
  fetch url3 ^
}

winner = sel {
  fetch url   -> it          -- `it` = result of completed arm
  timeout 5   -> Err "slow"
}
```

`par` runs all arms, returns tuple. `sel` races, first wins. `^` in any arm cancels siblings on error.

Mutable bindings (`:=`) cannot be captured in `par`/`sel`/`pmap` bodies.

## Operator Precedence (high to low)

`.` > juxtaposition > unary > `*/%//` > `+-` > `..` > `++ ~> ~>?` > `|` > comparisons > `&&` > `||` > `??` > `^` > `&` > `->` > `?` > `= := <-`
