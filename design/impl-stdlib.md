# Stdlib Loader Design

How `std/*` modules are registered and loaded. Native (Rust) implementations for all stdlib modules.

Implements: [stdlib-modules.md](../spec/stdlib-modules.md), [modules.md](../spec/modules.md)

## Architecture

Stdlib modules are not `.lx` files — they're Rust functions registered as `BuiltinFunc` values inside module namespaces. When user code writes `use std/fs`, the interpreter creates a `Record` value containing all the module's exported functions.

```rust
fn load_stdlib_module(name: &str) -> Result<Value> {
    match name {
        "fs" => Ok(build_fs_module()),
        "net/http" => Ok(build_http_module()),
        "json" => Ok(build_json_module()),
        "csv" => Ok(build_csv_module()),
        "toml" => Ok(build_toml_module()),
        "yaml" => Ok(build_yaml_module()),
        "time" => Ok(build_time_module()),
        "fmt" => Ok(build_fmt_module()),
        "math" => Ok(build_math_module()),
        "env" => Ok(build_env_module()),
        "io" => Ok(build_io_module()),
        "bit" => Ok(build_bit_module()),
        "crypto" => Ok(build_crypto_module()),
        "os" => Ok(build_os_module()),
        "rand" => Ok(build_rand_module()),
        "re" => Ok(build_re_module()),
        "ai" => Ok(build_ai_module()),
        "df" => Ok(build_df_module()),
        "db" => Ok(build_db_module()),
        "num" => Ok(build_num_module()),
        "ml" => Ok(build_ml_module()),
        "plot" => Ok(build_plot_module()),
        _ => Err(LxError::import(format!("unknown stdlib module: std/{name}"))),
    }
}
```

Each `build_*_module()` returns a `Value::Record` with function names as keys and `Value::BuiltinFunc` as values.

## Module as Record

```rust
fn build_fs_module() -> Value {
    let mut fields = IndexMap::new();
    fields.insert("read".into(), BuiltinFunc::new("fs.read", 1, fs_read).into());
    fields.insert("write".into(), BuiltinFunc::new("fs.write", 2, fs_write).into());
    fields.insert("walk".into(), BuiltinFunc::new("fs.walk", 1, fs_walk).into());
    // ...
    Value::Record(Arc::new(fields))
}
```

When user code writes `fs.read "file.txt"`, the interpreter evaluates:
1. `fs` → looks up `fs` in env → gets the module Record
2. `.read` → field access on the Record → gets the `BuiltinFunc`
3. `"file.txt"` → application → calls `fs_read`

## Selective Imports

`use std/fs {read write}` extracts specific functions from the module record and binds them directly in the current scope:

```rust
fn resolve_selective(module: &Value, names: &[String]) -> Result<Vec<(String, Value)>> {
    let record = module.as_record()?;
    names.iter().map(|name| {
        record.get(name)
            .map(|v| (name.clone(), v.clone()))
            .ok_or_else(|| LxError::import(format!("`{name}` not found in module")))
    }).collect()
}
```

## Nested Modules

`use std/net/http` resolves as: load `net` → get field `http`. In practice, `std/net` is a namespace record containing `http` as a sub-record.

```rust
fn build_net_module() -> Value {
    let mut fields = IndexMap::new();
    fields.insert("http".into(), build_http_module());
    Value::Record(Arc::new(fields))
}
```

## User Modules

User `.lx` files are loaded by:
1. Resolve the path relative to the importing file
2. Check the module cache (avoid re-parsing the same file)
3. Lex + parse + (optionally check) the file
4. Evaluate all top-level bindings
5. Collect exported (`+` prefix) bindings into a Record
6. Cache and return

Circular imports are detected by tracking the "currently loading" set. If a module is requested while already loading, emit `error[import]` with the cycle chain.

## Opaque Types

Some stdlib functions return opaque values that can only be consumed by other stdlib functions:

- `fs.Handle` — returned by `fs.open`, consumed by `fs.close` and read/write operations
- `time.Duration` — returned by `time.sec`/`time.ms`/`time.min`, consumed by `time.sleep`, `time.timeout`, and arithmetic

These are represented as `Value::Opaque(Arc<dyn Any + Send + Sync>)` in the interpreter. User code cannot destructure or inspect them.

## Async Implementation

Most stdlib functions are async internally:

- `fs.read` → `tokio::fs::read_to_string`
- `http.get` → `reqwest::get`
- `time.sleep` → `tokio::time::sleep`

The async nature is invisible to lx code — these functions appear synchronous. The interpreter's async eval handles the scheduling.

## Error Mapping

Stdlib functions map Rust errors to lx values:

```rust
async fn fs_read(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
    let path = args[0].as_str()?;
    match tokio::fs::read_to_string(path).await {
        Ok(content) => Ok(Value::Str(content.into())),
        Err(e) => Ok(Value::Err(Box::new(Value::Record(Arc::new({
            let mut m = IndexMap::new();
            m.insert("msg".into(), Value::Str(e.to_string().into()));
            m.insert("path".into(), args[0].clone());
            m
        }))))),
    }
}
```

All stdlib I/O functions return `Result` values. The user handles them with `^`, `??`, or explicit matching.

## Sandboxing

In `--sandbox` or `--deny-*` mode, the interpreter checks capability flags before executing stdlib functions:

```rust
async fn fs_read(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
    interp.check_capability(Capability::FsRead)?;
    // ...
}
```

Denied capabilities return `Err PermissionDenied`.

## Cross-References

- Module system spec: [modules.md](../spec/modules.md)
- Stdlib API spec: [stdlib-modules.md](../spec/stdlib-modules.md)
- Built-in functions: [impl-builtins.md](impl-builtins.md)
- Interpreter integration: [impl-interpreter.md](impl-interpreter.md)
- Phase 9 deliverables: [implementation-phases.md](implementation-phases.md)
