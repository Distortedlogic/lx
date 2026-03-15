# Iteration

No `for` or `while`. Iteration uses higher-order functions with pipes. One imperative escape hatch: `loop`/`break`.

## Higher-Order Functions

The core iteration primitives, all pipe-friendly (data-last):

```
[1 2 3] | map (* 2)              -- [2 4 6]
[1 2 3 4] | filter (> 2)         -- [3 4]
[1 2 3] | fold 0 (+)             -- 6
items | each (item) $echo "{item}"
```

`map` transforms each element. `filter` keeps matching elements. `fold` reduces to a single value. `each` runs a side effect per element (returns unit).

Additional HOFs:

```
[1 2 3] | scan 0 (+)             -- [0 1 3 6] (fold returning all intermediate values, initial value included)
[1 2 3] | flat_map (x) [x x]     -- [1 1 2 2 3 3]
[1 2 3] | zip ["a" "b" "c"]      -- [(1 "a") (2 "b") (3 "c")]
[1 2 3 4] | partition (> 2)       -- ([3 4] [1 2])
[3 1 2] | sort                    -- [1 2 3]
[3 1 2] | sort_by (.key)          -- sort by field
[1 2 2 3] | uniq                  -- [1 2 3]
[1 2 3] | rev                     -- [3 2 1]
```

## Ranges

Ranges produce lists:

```
1..10                             -- [1 2 3 4 5 6 7 8 9]
1..=10                            -- includes 10
0..5                              -- [0 1 2 3 4]
1..10 | step 2                    -- [1 3 5 7 9]
```

## Loop/Break

The imperative escape hatch for inherently stateful iteration (reading lines, interactive input, retry loops):

```
loop {
  line = $read
  line == "quit" ? break
  process line
}
```

`loop` and `break` are the only loop constructs. `break` can carry a value:

```
result = loop {
  input = $read
  input | parse_int ? {
    Ok n -> break n
    Err _ -> $echo "not a number, try again"
  }
}
```

Prefer `map`/`filter`/`fold` over `loop`. Use `loop` only when the iteration depends on mutable external state (user input, network responses, retries).

## Pattern Matching with std/re

For regex operations, use `std/re` with string patterns:

```
use std/re
re.is_match "\\d+" input
re.match "(\\w+)-(\\d+)" text
```

See [stdlib-modules.md](stdlib-modules.md) for the full `std/re` API.

## `step` for Ranges

`step` takes every nth element from a list:

```
1..100 | step 2                -- [1 3 5 7 ... 99]
0..50 | step 5                 -- [0 5 10 15 ... 45]
```

Ranges are ascending only. `10..1` is empty. Use `rev` for descending:

```
1..=10 | rev                   -- [10 9 8 ... 1]
1..=10 | rev | step 2          -- [10 8 6 4 2]
```

## Design Notes

**No `for` or `while`** — `each`, `map`, and `filter` with pipes cover every iteration pattern with fewer tokens. `items | each (i) $echo "{i}"` is shorter than `for i in items { $echo "{i}" }`.

**No `continue`** — use pattern matching inside `loop` to skip:

```
loop {
  line = $read
  line | trim ? {
    ""  -> ()
    cmd -> process cmd
  }
}
```

Or prefer `filter` pipelines over a list: `lines | map trim | filter (!= "") | each process`.

## `enumerate` for Index Access

When you need the index alongside the element, use `enumerate`:

```
items | enumerate | each (i item) {
  $echo "{i}: {item}"
}
```

`enumerate` produces `(Int a)` tuples. The destructuring `(i item)` splits them. This replaces the `for (i, x) in xs.iter().enumerate()` pattern from other languages.

**No comprehensions** — `map`/`filter` with sections cover the same ground: `1..=10 | filter even? | map (* 2)` instead of `[x * 2 for x in 1..10 if x % 2 == 0]`. The pipeline form reads left-to-right and composes without new syntax.

## Cross-References

- Implementation: [impl-builtins.md](../design/impl-builtins.md), [impl-interpreter.md](../design/impl-interpreter.md)
- Test suite: [08_iteration.lx](../tests/08_iteration.lx)
