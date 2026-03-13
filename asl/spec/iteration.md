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

Ranges produce lazy sequences:

```
1..10                             -- [1 2 3 4 5 6 7 8 9]
1..=10                            -- includes 10
0..5                              -- [0 1 2 3 4]
1..10 | step 2                    -- [1 3 5 7 9]
```

Ranges are lazy — `1..1000000` doesn't allocate a million-element list. Elements are produced on demand as the pipeline pulls them.

## Lazy Sequences

Ranges and generators produce lazy sequences. Pipeline stages (`map`, `filter`, `take`) propagate laziness — they transform elements on demand without materializing intermediate lists.

```
nat | filter prime? | take 10     -- first 10 primes
```

`nat` is an infinite lazy sequence of natural numbers. `filter prime?` tests each lazily. `take 10` pulls exactly 10 through and stops. No infinite computation occurs.

Collecting operations force evaluation: `sort`, `rev`, `len`, `collect`, `uniq`, `partition`, and assignment to a list binding.

```
xs = 1..100 | filter even?        -- forces: xs is now [2 4 6 ... 100]
1..100 | filter even? | take 5    -- lazy: only computes [2 4 6 8 10]
```

## Loop/Break

The imperative escape hatch for inherently stateful iteration (reading lines, interactive input, retry loops):

```
loop {
  line = io.read_line ^
  line == "quit" ? break
  process line
}
```

`loop` and `break` are the only loop constructs. `break` can carry a value:

```
result = loop {
  input = io.read_line ^
  input | parse_int ? {
    Ok n -> break n
    Err _ -> $echo "not a number, try again"
  }
}
```

Prefer `map`/`filter`/`fold` over `loop`. Use `loop` only when the iteration depends on mutable external state (user input, network responses, retries).

## Regex

`r/pattern/flags` — first-class regex literals.

```
input | match r/(\d+)-(\w+)/     -- match groups
input | replace r/old/g "new"    -- global replace
input | split r/\s+/             -- split on whitespace
input | test r/^\d+$/            -- Bool: does it match?
```

Flags: `i` (case-insensitive), `g` (global), `m` (multiline), `s` (dotall), `x` (extended/comments).

Match results:

```
"abc-123" | match r/([a-z]+)-(\d+)/ ? {
  Some groups -> "letters={groups.1} digits={groups.2}"
  None        -> "no match"
}
```

## Iterator Protocol

Any record with a `next` field of type `() -> Maybe a` is iterable by pipelines. No special syntax or keyword needed — structural typing handles it.

```
counter = (start end) {
  n := start
  {next: () n >= end ? None : { val = n; n <- n + 1; Some val }}
}

counter 1 5 | map (* 2) | collect    -- [2 4 6 8]
```

Pipeline functions (`map`, `filter`, `take`, etc.) check for the `next` field and consume lazily. This is how ranges, `io.stdin`, and `fs.walk` work internally.

To create a generator from a function:

```
fib = () {
  a := 0; b := 1
  {next: () { val = a; tmp = a + b; a <- b; b <- tmp; Some val }}
}

fib () | take 10 | collect    -- [0 1 1 2 3 5 8 13 21 34]
```

## `step` for Ranges

`step` takes every nth element from a lazy sequence:

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
  line = io.read_line ^
  line | trim ? {
    ""  -> ()
    cmd -> process cmd
  }
}
```

Or prefer `filter` pipelines: `io.stdin | map trim | filter (!= "") | each process`.

## `enumerate` for Index Access

When you need the index alongside the element, use `enumerate`:

```
items | enumerate | each (i item) {
  $echo "{i}: {item}"
}
```

`enumerate` produces `(Int a)` tuples. The destructuring `(i item)` splits them. This replaces the `for (i, x) in xs.iter().enumerate()` pattern from other languages.

## Infinite Sequences

`nat` is a built-in infinite lazy sequence of natural numbers starting from 0:

```
nat                           -- 0 1 2 3 4 ...
nat | drop 1                  -- 1 2 3 4 5 ...
nat | filter prime? | take 10 -- first 10 primes
```

Create infinite sequences with the iterator protocol:

```
forever = (val) {next: () Some val}
forever "hello" | take 3 | collect    -- ["hello" "hello" "hello"]

cycle = (xs) {
  i := 0
  {next: () { val = xs.(i % (xs | len)); i <- i + 1; Some val }}
}
cycle [1 2 3] | take 7 | collect     -- [1 2 3 1 2 3 1]
```

Infinite sequences MUST be consumed with `take`, `take_while`, or similar — piping to `collect`, `sort`, `len`, or other forcing operations on an infinite sequence will not terminate.

**No comprehensions** — `map`/`filter` with sections cover the same ground: `1..=10 | filter even? | map (* 2)` instead of `[x * 2 for x in 1..10 if x % 2 == 0]`. The pipeline form reads left-to-right and composes without new syntax.

## Cross-References

- Implementation: [impl-builtins.md](../impl/impl-builtins.md) (lazy vs eager, iterator detection), [impl-interpreter.md](../impl/impl-interpreter.md) (lazy sequences)
- Test suite: [08_iteration.lx](../suite/08_iteration.lx)
