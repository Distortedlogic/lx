# Built-in Functions Design

How built-in functions (map, filter, fold, etc.) are registered and dispatched.

Implements: [stdlib.md](../spec/stdlib.md), [iteration.md](../spec/iteration.md)

## Registration

Built-in functions are registered in the root `Env` before user code runs. Each built-in is a `BuiltinFunc` value with the signature `fn(&[Value], Span) -> Result<Value, LxError>`.

Built-ins use the same currying mechanism as user functions. `map f` returns a partially-applied `BuiltinFunc` waiting for its second argument.

## Categories

**Transformers** — return a new list:
- `map`, `filter`, `take`, `drop`, `take_while`, `drop_while`, `enumerate`, `step`, `scan`, `flat_map`

**Reducers** — consume a list and return a value:
- `fold`, `sum`, `product`, `sort`, `sort_by`, `rev`, `len`, `uniq`, `uniq_by`, `partition`, `group_by`, `chunks`, `windows`, `flatten`, `intersperse`, `min`, `max`, `min_by`, `max_by`, `join`

**Short-circuit** — stop early:
- `find`, `find_index`, `any?`, `all?`, `none?`, `count`, `first`, `last`

**Side-effect**:
- `each` — applies function to each element, returns `Unit`

All operations are eager — they operate on concrete `Value::List` and return concrete values. No lazy sequences.

## Tuple Auto-Spread in HOFs

When `map`, `filter`, `each`, etc. call their function argument with a tuple value, tuple auto-spread applies. This enables:

```
entries m | map (k v) "{k}: {v}"
enumerate xs | each (i x) $echo "{i}: {x}"
```

The function `(k v) ...` has arity 2. The element is a 2-tuple. The interpreter spreads the tuple into the two parameters.

## Data-Last Convention

All built-ins follow data-last ordering. The collection being operated on is always the last argument:

```
-- map f xs  →  xs | map f  (pipe inserts xs as last arg)
-- fold init f xs  →  xs | fold init f
```

## String Functions

String functions are registered as built-ins (not a stdlib module):

```
trim, trim_start, trim_end, lines, split, join, upper, lower
starts?, ends?, contains?, replace, replace_all, repeat, chars
byte_len, pad_left, pad_right
```

Polymorphic built-ins like `contains?` check the value type at runtime:
- `contains? "sub" "string"` → substring test
- `contains? 3 [1 2 3]` → list membership

## Concurrency Built-ins

`par`, `sel`, `pmap`, `pmap_n`, and `timeout` exist but are currently **sequential implementations**. `par` evaluates stmts in order and returns a tuple. `pmap f xs` maps sequentially. Real async (tokio) is planned but not implemented.

## Debug and Tap

`dbg` prints `[dbg] value` to stderr and returns the value unchanged. `tap f x` evaluates `f(x)` for side effects and returns `x` unchanged.

## Cross-References

- Value types: [impl-interpreter.md](impl-interpreter.md)
- Full built-in list: [stdlib.md](../spec/stdlib.md)
- Stdlib modules (fs, http, etc.): [impl-stdlib.md](impl-stdlib.md)
