# Iteration — Reference

No `for` or `while`. Iteration uses higher-order functions with pipes. One escape hatch: `loop`/`break`.

## Higher-Order Functions

```
[1 2 3] | map (* 2)              -- [2 4 6]
[1 2 3 4] | filter (> 2)         -- [3 4]
[1 2 3] | fold 0 (+)             -- 6
items | each (item) $echo "{item}"
```

`map` transforms. `filter` keeps matches. `fold` reduces. `each` runs side effects (returns unit).

Additional HOFs:

```
[1 2 3] | scan 0 (+)             -- [0 1 3 6]
[1 2 3] | flat_map (x) [x x]     -- [1 1 2 2 3 3]
[1 2 3] | zip ["a" "b" "c"]      -- [(1 "a") (2 "b") (3 "c")]
[1 2 3 4] | partition (> 2)       -- ([3 4] [1 2])
[3 1 2] | sort                    -- [1 2 3]
[3 1 2] | sort_by (.key)          -- sort by field
[1 2 2 3] | uniq                  -- [1 2 3]
[1 2 3] | rev                     -- [3 2 1]
```

## Ranges

```
1..10                             -- [1 2 3 4 5 6 7 8 9]
1..=10                            -- includes 10
0..5                              -- [0 1 2 3 4]
```

Ranges are ascending only. `10..1` is empty. Use `rev` for descending.

## `step`

```
1..100 | step 2                -- [1 3 5 7 ... 99]
0..50 | step 5                 -- [0 5 10 15 ... 45]
1..=10 | rev | step 2          -- [10 8 6 4 2]
```

## `enumerate`

```
items | enumerate | each (i item) {
  $echo "{i}: {item}"
}
```

Produces `(Int a)` tuples. Tuple auto-spread destructures them in the callback.

## `loop` / `break`

For inherently stateful iteration (user input, retries):

```
loop {
  line = $read
  line == "quit" ? break
  process line
}
```

`break` can carry a value:

```
result = loop {
  input = $read
  input | parse_int ? {
    Ok n -> break n
    Err _ -> $echo "not a number, try again"
  }
}
```

No `continue` — use pattern matching to skip:

```
loop {
  line = $read
  line | trim ? {
    ""  -> ()
    cmd -> process cmd
  }
}
```

Or prefer `filter` pipelines: `lines | filter (!= "") | each process`.
