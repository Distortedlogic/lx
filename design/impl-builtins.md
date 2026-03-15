# Built-in Functions Design

How built-in functions (map, filter, fold, etc.) are registered and dispatched.

Implements: [stdlib.md](../spec/stdlib.md), [iteration.md](../spec/iteration.md)

## Registration

Built-in functions are registered in the root `Env` before user code runs. Each built-in is a `BuiltinFunc` value:

```rust
fn register_builtins(env: &mut Env) {
    env.bind("map", BuiltinFunc::new("map", 2, builtin_map));
    env.bind("filter", BuiltinFunc::new("filter", 2, builtin_filter));
    env.bind("fold", BuiltinFunc::new("fold", 3, builtin_fold));
    // ...
}
```

Built-ins use the same currying mechanism as user functions. `map f` returns a partially-applied `BuiltinFunc` waiting for its second argument.

## Lazy vs Eager

Built-ins fall into three categories:

**Lazy transformers** — return a new iterator wrapping the input:
- `map`, `filter`, `take`, `drop`, `take_while`, `drop_while`, `enumerate`, `step`, `scan`, `flat_map`

**Eager consumers** — force the input and return a concrete value:
- `fold`, `sum`, `product`, `sort`, `sort_by`, `rev`, `len`, `uniq`, `uniq_by`, `partition`, `group_by`, `chunks`, `windows`, `flatten`, `intersperse`, `min`, `max`, `min_by`, `max_by`, `collect`, `join`

**Short-circuit consumers** — stop pulling from the input early:
- `find`, `find_index`, `any?`, `all?`, `none?`, `count`, `first`, `last`

**Side-effect runners**:
- `each` — applies function to each element, returns `Unit`

## Iterator Detection

When a built-in receives its data argument (last arg), it checks the value type:

1. `Value::List(xs)` → iterate over `xs` directly
2. `Value::Set(xs)` → iterate over `xs` (order not guaranteed)
3. `Value::Map(m)` → iterate over `(key, value)` tuples
4. `Value::Str(s)` → iterate over codepoint strings
5. `Value::Iterator(iter)` → pull from the `LxIterator`
6. `Value::Record(r)` → check for `next` field (iterator protocol)

If the record has a `next` field that is a function, it's treated as a user-defined iterator. The built-in wraps it in an `LxIterator` adapter.

## Lazy Pipeline Mechanics

Lazy built-ins return `Value::Iterator(Box<SomeIterator>)`. Each lazy iterator holds a reference to its upstream:

```rust
struct MapIterator {
    upstream: Box<dyn LxIterator>,
    func: LxFunc,
    interpreter: Arc<Mutex<Interpreter>>,
}

impl LxIterator for MapIterator {
    async fn next(&mut self) -> Option<Value> {
        let val = self.upstream.next().await?;
        let result = self.interpreter.lock().apply(self.func.clone(), val).await;
        Some(result.unwrap())
    }
}
```

A pipeline like `xs | map f | filter g | take 5` builds a chain: `TakeIterator(FilterIterator(MapIterator(ListIterator(xs))))`. Each `next()` call pulls through the chain.

## Tuple Auto-Spread in HOFs

When `map`, `filter`, `each`, etc. call their function argument with a tuple value, tuple auto-spread applies (see [impl-interpreter.md](impl-interpreter.md)). This enables:

```
entries m | map (k v) "{k}: {v}"
enumerate xs | each (i x) $echo "{i}: {x}"
```

The function `(k v) ...` has arity 2. The element is a 2-tuple. The interpreter spreads the tuple into the two parameters.

## Data-Last Convention

All built-ins follow data-last ordering. The "data" argument (the collection being operated on) is always last. This makes currying compose with pipes:

```
-- map f xs  →  xs | map f  (pipe inserts xs as last arg)
-- fold init f xs  →  xs | fold init f  (pipe inserts xs as last arg)
```

## String Functions

String functions are also built-ins (not a separate module). They're registered the same way:

```
trim, trim_start, trim_end, lines, split, join, upper, lower
starts?, ends?, contains?, replace, replace_all, repeat, chars
byte_len, pad_left, pad_right
```

Polymorphic built-ins like `contains?` check the value type at runtime:
- `contains? "sub" "string"` → substring test
- `contains? 3 [1 2 3]` → list membership
- `contains? 3 #{1 2 3}` → set membership

## Concurrency Built-ins

`pmap` and `pmap_n` are built-ins that use tokio's `JoinSet`:

```rust
async fn builtin_pmap(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
    let func = args[0].as_func()?;
    let xs = args[1].as_iterable()?;
    let mut join_set = JoinSet::new();
    for (i, x) in xs.enumerate() {
        let f = func.clone();
        join_set.spawn(async move { (i, apply(f, x).await) });
    }
    let mut results: Vec<(usize, Value)> = Vec::new();
    while let Some(result) = join_set.join_next().await {
        let (i, val) = result??;
        results.push((i, val));
    }
    results.sort_by_key(|(i, _)| *i);
    Ok(Value::List(results.into_iter().map(|(_, v)| v).collect()))
}
```

`pmap_n limit f xs` is identical but spawns at most `limit` tasks at a time using a semaphore.

## Debug and Tap

`dbg` is special: the parser emits `Expr::Dbg(inner)` which captures the source text at compile time. At runtime:

```
eval(Dbg(inner)):
  val = eval(inner)
  eprintln!("[{}:{}] {} = {}", file, line, source_text, val.to_str())
  return val
```

`tap f x` evaluates `f(x)` for side effects and returns `x` unchanged.

## Cross-References

- Value types: [impl-interpreter.md](impl-interpreter.md)
- Lazy sequence spec: [iteration.md](../spec/iteration.md)
- Full built-in list: [stdlib.md](../spec/stdlib.md)
- Stdlib modules (fs, http, etc.): [impl-stdlib.md](impl-stdlib.md)
- Suite tests: [suite/04_functions.lx](../tests/04_functions.lx), [suite/08_iteration.lx](../tests/08_iteration.lx)
