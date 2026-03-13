# Interpreter Design

Tree-walking async interpreter. Evaluates AST nodes to produce runtime values.

Implements: [runtime.md](../spec/runtime.md), [concurrency.md](../spec/concurrency.md), [shell.md](../spec/shell.md)

## Architecture

The interpreter is an `async fn eval(&mut self, expr: &Expr) -> Result<Value, LxError>`. Every expression evaluation is async from the start — synchronous operations just resolve immediately. This is required because any expression might contain a shell command, HTTP call, or `par`/`sel` block.

```rust
struct Interpreter {
    env: Env,
    source: Arc<str>,
    diagnostics: Vec<Diagnostic>,
}
```

## Value Representation

```rust
enum Value {
    Int(BigInt),
    Float(f64),
    Bool(bool),
    Str(Arc<str>),
    Unit,
    Bytes(Arc<[u8]>),

    List(Arc<Vec<Value>>),
    Record(Arc<IndexMap<String, Value>>),
    Map(Arc<IndexMap<Value, Value>>),
    Set(Arc<IndexSet<Value>>),
    Tuple(Arc<Vec<Value>>),

    Func(LxFunc),
    BuiltinFunc(BuiltinFunc),

    Ok(Box<Value>),
    Err(Box<Value>),
    Some(Box<Value>),
    None,

    Iterator(Box<dyn LxIterator>),
    ShellResult { out: Arc<str>, err: Arc<str>, code: i32 },
    Opaque(Arc<dyn Any + Send + Sync>),
}
```

### Why Arc

Collections use `Arc` for cheap cloning. lx is immutable-by-default — `{..p x: 5.0}` creates a new record by cloning the inner map and inserting. With `Arc`, the clone is O(1) until mutation is needed (clone-on-write semantics).

Mutable bindings (`x :=`) hold a `Value` directly. Reassignment (`x <- new_val`) replaces the value in the environment slot.

### LxFunc

```rust
struct LxFunc {
    params: Vec<Param>,
    body: Arc<SExpr>,
    closure: Env,
    arity: usize,
    applied: Vec<Value>,
}
```

`applied` tracks partial application for auto-currying. When `applied.len() < arity`, the function is partially applied. When `applied.len() == arity`, execution begins.

### BuiltinFunc

```rust
struct BuiltinFunc {
    name: &'static str,
    arity: usize,
    func: fn(&mut Interpreter, Vec<Value>) -> BoxFuture<Result<Value>>,
    applied: Vec<Value>,
}
```

Built-in functions use the same currying mechanism as user functions.

## Environment

```rust
struct Env {
    bindings: HashMap<String, Slot>,
    parent: Option<Arc<Env>>,
}

enum Slot {
    Immutable(Value),
    Mutable(Arc<Mutex<Value>>),
}
```

Variable lookup walks the parent chain. Mutable bindings use `Arc<Mutex<Value>>` so closures that share a mutable binding see the same slot. The mutex is never contended in single-threaded code (zero overhead on modern implementations).

## Expression Evaluation

### Pipe

`eval(Pipe { left, right })`:
1. Evaluate `left` → `val`
2. Evaluate `right` → `func`
3. Apply `func` with `val` as the last argument

If `func` is partially applied with N args and needs N+1, `val` fills the last slot. If `func` is a 1-arg function, `val` is the sole argument.

### Tuple Auto-Spread

When applying a function with arity N to a single tuple of size N, the tuple is spread into the parameters. This enables `enumerate | each (i item) body` and `entries | map (k v) body` to work naturally.

```
apply(func, [tuple_val]):
  if func.arity > 1 && tuple_val is Tuple(elems) && elems.len() == func.arity:
    apply(func, elems)  -- spread
  else:
    normal application
```

### Pattern Matching

`eval_match(scrutinee, arms)`:
1. Evaluate scrutinee
2. For each arm: try to match the pattern against the value
3. If matched, bind pattern variables and evaluate the arm body
4. If guard exists, evaluate guard — if false, continue to next arm
5. If no arm matches, runtime panic (non-exhaustive)

Pattern matching is a separate function `try_match(pattern, value) -> Option<Vec<(String, Value)>>` that returns bindings on success.

### Shell Execution

`eval_shell(ShellExpr)`:
1. For each `ShellPart`, evaluate interpolation holes and concatenate into a command string
2. Spawn via `tokio::process::Command::new("sh").arg("-c").arg(&cmd_str)`
3. Capture stdout, stderr, exit code
4. For `$cmd`: return `Ok(ShellResult { out, err, code })` or `Err(ShellErr)` on spawn failure
5. For `$^cmd`: if exit code == 0, return `Ok(out)`; else return `Err(ShellErr { cmd, msg: err })`
6. For `$$cmd`: same as `$cmd` but skip interpolation (raw text)
7. For `${ }` block: execute as single shell session, return result of last command

### Error Propagation (`^`)

`eval(Propagate(inner))`:
1. Evaluate `inner`
2. If `Ok(val)` → return `val`
3. If `Err(e)` → return `Err(PropagatedError { original: e, trace: current_span })`
4. If `Some(val)` → return `val`
5. If `None` → return `Err(NoneError { location: current_span })`
6. Otherwise → runtime type error

Each `^` site appends to the propagation trace for diagnostics.

### Concurrency

**`par` block:**
```
eval(Par(stmts)):
  let mut join_set = JoinSet::new();
  for stmt in stmts:
    let env = self.env.snapshot();  -- immutable clone of current env
    join_set.spawn(async { eval_with_env(env, stmt) });
  collect all results in order
  if any Err, cancel remaining, return first Err
  return Tuple(results)
```

**`sel` block:**
```
eval(Sel(arms)):
  spawn each arm's expression
  tokio::select! on the first to complete
  cancel all others
  bind `it` = result in the winning handler
  evaluate handler
```

**`pmap`:**
```
eval_pmap(func, list):
  let mut join_set = JoinSet::new();
  for (i, elem) in list:
    let f = func.clone();
    join_set.spawn(async { (i, apply(f, elem)) });
  collect results, sort by index
  if any Err and ^ was used, cancel and propagate
```

## Lazy Sequences

Lazy sequences implement `LxIterator`:

```rust
trait LxIterator: Send {
    fn next(&mut self) -> BoxFuture<Option<Value>>;
}
```

Ranges, pipeline stages (`map`, `filter`, `take`), and user-defined iterators (records with `next` field) all implement this trait.

`collect` forces a lazy sequence into a concrete `List`. `sort`, `rev`, `len`, `uniq` also force.

Pipeline stages are lazy wrappers:
- `map f` wraps the upstream iterator, applying `f` to each element on `next()`
- `filter pred` wraps upstream, skipping non-matching elements
- `take n` wraps upstream, returning `None` after n elements

## Implicit Err Early Return

When evaluating the body of a function with a `Result` return annotation (`-> T ^ E`), the interpreter checks each intermediate statement's result. If a bare expression statement (not a binding) evaluates to `Value::Err(e)`, the function immediately returns `Err(e)` without evaluating remaining statements.

```
eval_block(stmts, is_result_annotated):
  for stmt in stmts[..last]:
    val = eval_stmt(stmt)
    if is_result_annotated && val is Err(e) && stmt is ExprStmt:
      return Err(e)
  eval(last_stmt)  -- final expression: implicit Ok wrapping if needed
```

This only applies to function bodies with explicit `-> T ^ E` annotation. Regular blocks and unannotated functions do not short-circuit on Err.

## Division and Index Panics

Division by zero (`/`, `//`, `%`) and out-of-bounds index access (`.0` on empty list) are runtime panics, not recoverable errors. The interpreter prints the diagnostic and aborts. In test mode, panics are caught per-test.

For safe alternatives: `get` returns `Maybe`, `math.safe_div` returns `Result`.

## Defer

`defer () cleanup` registers a closure on the current scope's defer stack. When a block scope exits (normal completion, `^` propagation, `break`), defers run in LIFO order. If a defer itself errors, the error is logged as `error[defer]` but does not replace the original error.

## Cross-References

- AST consumed: [impl-ast.md](impl-ast.md)
- Shell spec: [shell.md](../spec/shell.md)
- Concurrency spec: [concurrency.md](../spec/concurrency.md)
- Runtime semantics: [runtime.md](../spec/runtime.md)
- Built-in functions: [impl-builtins.md](impl-builtins.md)
- Stdlib modules: [impl-stdlib.md](impl-stdlib.md)
