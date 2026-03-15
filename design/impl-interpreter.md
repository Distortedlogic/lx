# Interpreter Design

Tree-walking interpreter. Evaluates AST nodes to produce runtime values.

Implements: [runtime.md](../spec/runtime.md), [shell.md](../spec/shell.md)

## Architecture

The interpreter is `fn eval(&mut self, expr: &SExpr) -> Result<Value, LxError>`. Synchronous — no async runtime.

```rust
struct Interpreter {
    env: Arc<Env>,
    source: Arc<str>,
}
```

## Value Representation

```rust
enum Value {
    Int(BigInt),
    Float(f64),
    Bool(bool),
    Str(Arc<str>),
    Regex(Arc<regex::Regex>),
    Unit,

    List(Arc<Vec<Value>>),
    Record(Arc<IndexMap<String, Value>>),
    Map(Arc<IndexMap<ValueKey, Value>>),
    Tuple(Arc<Vec<Value>>),

    Func(LxFunc),
    BuiltinFunc(BuiltinFunc),

    Ok(Box<Value>),
    Err(Box<Value>),
    Some(Box<Value>),
    None,

    Tagged { tag: Arc<str>, values: Arc<Vec<Value>> },
    TaggedCtor { tag: Arc<str>, arity: usize, applied: Vec<Value> },
    Range { start: i64, end: i64, inclusive: bool },
    Protocol { name: Arc<str>, fields: Arc<Vec<ProtoFieldDef>> },
    McpDecl { name: Arc<str>, tools: Arc<Vec<McpToolDef>> },
}
```

### Why Arc

Collections use `Arc` for cheap cloning. lx is immutable-by-default — `{..p x: 5.0}` creates a new record by cloning the inner map and inserting.

Mutable bindings (`:=`) hold a `Value` directly. Reassignment (`<-`) replaces the value in the environment slot.

### LxFunc

```rust
struct LxFunc {
    params: Vec<String>,
    defaults: Vec<Option<Value>>,
    body: Arc<SExpr>,
    closure: Arc<Env>,
    arity: usize,
    applied: Vec<Value>,
}
```

`applied` tracks partial application for auto-currying.

### BuiltinFunc

```rust
struct BuiltinFunc {
    name: &'static str,
    arity: usize,
    func: fn(&[Value], Span) -> Result<Value, LxError>,
    applied: Vec<Value>,
}
```

## Environment

```rust
struct Env {
    bindings: parking_lot::Mutex<HashMap<String, Value>>,
    parent: Option<Arc<Env>>,
}
```

Variable lookup walks the parent chain. Mutable bindings are regular values that can be replaced via `<-`. The Env uses `parking_lot::Mutex` for interior mutability.

## Expression Evaluation

### Pipe

`eval(Pipe { left, right })`:
1. Evaluate `left` → `val`
2. Evaluate `right` → `func`
3. Apply `func` with `val` as the last argument

### Tuple Auto-Spread

When applying a function with arity N to a single tuple of size N, the tuple is spread into the parameters. Enables `enumerate | each (i item) body`.

### Pattern Matching

`eval_match(scrutinee, arms)`:
1. Evaluate scrutinee
2. For each arm: try to match the pattern against the value
3. If matched, bind pattern variables and evaluate the arm body
4. If guard exists, evaluate guard — if false, continue to next arm
5. If no arm matches, runtime error

### Shell Execution

`eval_shell(ShellExpr)`:
1. Evaluate interpolation holes and concatenate into a command string
2. Spawn via `std::process::Command::new("sh").arg("-c").arg(&cmd_str)`
3. Capture stdout, stderr, exit code
4. `$cmd`: return `Ok({out err code})` or `Err(msg)`
5. `$^cmd`: exit 0 → `Ok(stdout)`, else `Err({msg code})`
6. `${ }` block: execute as single shell session

### Error Propagation (`^`)

`eval(Propagate(inner))`:
1. Evaluate `inner`
2. `Ok(val)` → return `val`
3. `Err(e)` → return `Err(PropagatedError { ... })`
4. `Some(val)` → return `val`
5. `None` → return `Err`
6. Otherwise → runtime type error

### Concurrency

`par`, `sel`, `pmap`, `pmap_n`, `timeout` are implemented but **sequential**. `par` evaluates each statement in order and collects results into a tuple. Real async (tokio) is planned.

## Division and Index Panics

Division by zero (`/`, `//`, `%`) is a runtime panic. For safe alternatives: `math.safe_div` returns `Result`.

## Cross-References

- AST consumed: [impl-ast.md](impl-ast.md)
- Built-in functions: [impl-builtins.md](impl-builtins.md)
- Stdlib modules: [impl-stdlib.md](impl-stdlib.md)
- Shell spec: [shell.md](../spec/shell.md)
