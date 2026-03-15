# Stdlib Loader Design

How `std/*` modules are registered and loaded. Native (Rust) implementations for all stdlib modules.

Implements: [stdlib-modules.md](../spec/stdlib-modules.md), [modules.md](../spec/modules.md)

## Architecture

Stdlib modules are Rust functions registered as `BuiltinFunc` values inside module namespaces. When user code writes `use std/fs`, the interpreter creates a `Record` value containing all the module's exported functions.

Each module has a `pub fn build() -> IndexMap<String, Value>` that returns the module's functions via `mk("module.fn_name", arity, bi_fn)`.

Registration in `crates/lx/src/stdlib/mod.rs`:

```rust
pub fn get_std_module(name: &str) -> Option<IndexMap<String, Value>> {
    match name {
        "json" => Some(json::build()),
        "ctx" => Some(ctx::build()),
        "math" => Some(math::build()),
        "fs" => Some(fs::build()),
        "env" => Some(env::build()),
        "re" => Some(re::build()),
        "md" => Some(md::build()),
        "agent" => Some(agent::build()),
        "mcp" => Some(mcp::build()),
        "http" => Some(http::build()),
        "time" => Some(time::build()),
        "cron" => Some(cron::build()),
        _ => None,
    }
}
```

## Module as Record

```rust
fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("read".into(), mk("fs.read", 1, bi_read));
    m.insert("write".into(), mk("fs.write", 2, bi_write));
    m
}
```

When user code writes `fs.read "file.txt"`, the interpreter evaluates:
1. `fs` → looks up `fs` in env → gets the module Record
2. `.read` → field access on the Record → gets the `BuiltinFunc`
3. `"file.txt"` → application → calls `bi_read`

## Builtin Function Signature

All stdlib functions are synchronous with the signature:

```rust
fn bi_example(args: &[Value], span: Span) -> Result<Value, LxError>
```

No async. No interpreter reference. Functions receive args by slice and the call-site span for error reporting.

## Selective Imports

`use std/fs {read write}` extracts specific functions from the module record and binds them directly in the current scope.

## User Modules

User `.lx` files are loaded by:
1. Resolve the path relative to the importing file
2. Check the module cache (avoid re-parsing the same file)
3. Lex + parse + (optionally check) the file
4. Evaluate all top-level bindings
5. Collect exported (`+` prefix) bindings into a Record
6. Cache and return

Circular imports are detected by tracking the "currently loading" set.

## Error Mapping

Stdlib functions map Rust errors to lx `Result` values:

```rust
fn bi_read(args: &[Value], span: Span) -> Result<Value, LxError> {
    let path = args[0].as_str()
        .ok_or_else(|| LxError::type_err("fs.read expects Str", span))?;
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Value::Ok(Box::new(Value::Str(Arc::from(content.as_str()))))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(e.to_string().as_str()))))),
    }
}
```

All stdlib I/O functions return `Result` values. The user handles them with `^`, `??`, or explicit matching.

## Cross-References

- Module system spec: [modules.md](../spec/modules.md)
- Stdlib API spec: [stdlib-modules.md](../spec/stdlib-modules.md)
- Built-in functions: [impl-builtins.md](impl-builtins.md)
- Interpreter integration: [impl-interpreter.md](impl-interpreter.md)
