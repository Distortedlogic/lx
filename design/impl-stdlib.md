# Stdlib Loader Design

How `std/*` modules are registered and loaded. Native (Rust) implementations for all stdlib modules.

Implements: [stdlib-modules.md](../spec/stdlib-modules.md), [modules.md](../spec/modules.md)

## Architecture

Stdlib modules are Rust functions registered as `BuiltinFunc` values inside module namespaces. When user code writes `use std/fs`, the interpreter creates a `Record` value containing all the module's exported functions.

Each module has a `pub fn build() -> IndexMap<String, Value>` that returns the module's functions via `mk("module.fn_name", arity, bi_fn)`.

Registration in `crates/lx/src/stdlib/mod.rs`:

```rust
pub(crate) fn get_std_module(path: &[String]) -> Option<ModuleExports> {
    if path.len() < 2 || path[0] != "std" { return None; }
    let bindings = if path[1] == "agents" && path.len() >= 3 {
        match path[2].as_str() {
            "auditor" => agents_auditor::build(),
            "grader" => agents_grader::build(),
            "monitor" => agents_monitor::build(),
            "planner" => agents_planner::build(),
            "reviewer" => agents_reviewer::build(),
            "router" => agents_router::build(),
            _ => return None,
        }
    } else {
        match path[1].as_str() {
            "json" | "ctx" | "math" | "fs" | "env" | "re" | "md"
            | "agent" | "mcp" | "http" | "time" | "cron" | "ai"
            | "tasks" | "audit" | "circuit" | "diag" | "knowledge"
            | "memory" | "plan" | "saga" | "introspect" | "trace"
            => /* module::build() */,
            _ => return None,
        }
    };
    Some(ModuleExports { bindings, variant_ctors: Vec::new() })
}
```

## 29 Modules (all implemented)

| Layer | Modules |
|---|---|
| Data | json, md, re, math, time |
| System | fs, env, http |
| Communication | agent (incl. reconcile), mcp, ai |
| Scheduling | cron |
| Orchestration | ctx, tasks, audit, circuit, plan, saga |
| Intelligence | knowledge, introspect |
| Standard Agents | agents/auditor, agents/router, agents/grader, agents/planner, agents/monitor, agents/reviewer |
| Infrastructure | memory, trace (incl. improvement_rate, should_stop) |
| Visualization | diag |

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

## Splitting Large Modules

When a module grows past 300 lines, split into multiple Rust files while keeping one `build()` entry point. Pattern (used by trace, agent, mcp, md, diag):

```rust
// trace.rs — core types + build()
pub(crate) static STORES: LazyLock<DashMap<u64, TraceStore>> = ...;
pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("trace.create", 1, bi_create));
    // ... core functions ...
    super::trace_query::register(&mut m);     // adds export/summary/filter
    super::trace_progress::register(&mut m);  // adds improvement_rate/should_stop
    m
}

// trace_query.rs — query functions
pub(crate) fn register(m: &mut IndexMap<String, Value>) { ... }

// trace_progress.rs — progress tracking functions
pub(crate) fn register(m: &mut IndexMap<String, Value>) { ... }
```

Register sibling files in `stdlib/mod.rs`: `mod trace_query; mod trace_progress;`

## Builtin Function Signature

All stdlib functions are synchronous with the signature:

```rust
fn bi_example(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>
```

No async. Functions receive args by slice, call-site span for error reporting, and `&Arc<RuntimeCtx>` for backend access (AI, HTTP, shell, etc.).

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
fn bi_read(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
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
