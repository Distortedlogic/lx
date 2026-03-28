# Work Item 9: Pure lx Backends

Handle `use tool "./file.lx" as Name` -- load `.lx` files through tool dispatch so they get the same auto-logging and observability as MCP tool modules.

## Prerequisites

- **use_tool_parser** must be complete -- provides `UseKind::Tool { command, alias }` variant in `crates/lx/src/ast/types.rs`
- **tool_module_dispatch** must be complete -- provides `build_tool_module` in `crates/lx/src/interpreter/tool_module.rs`, tool dispatch path exists, `McpClient` at `crates/lx/src/mcp_client.rs`
- **unit_4_event_stream** must be complete -- provides `EventStream` trait, `StreamEntry`, `IdGenerator` in `crates/lx/src/runtime/event_stream.rs`
- **unit_5_stream_module** must be complete -- provides `RuntimeCtx.event_stream` (`parking_lot::Mutex<Option<Arc<dyn EventStream>>>`), `RuntimeCtx.call_id_counter` (`Arc<AtomicU64>`), `RuntimeCtx.replay_cache`, `RuntimeCtx.id_gen` (`IdGenerator`)
- **unit_6_auto_logging** must be complete -- tool dispatch closures emit `tool.call`, `tool.result`, `tool.error` to the event stream via `StreamEntry::new` + `ctx.xadd`

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify

## Current State

- `UseKind::Tool { command: String, alias: Sym }` is the AST variant (in `crates/lx/src/ast/types.rs` lines 16-17). The `command` field is a plain string -- it holds `"agent-browser"` for MCP backends or `"./my_browser.lx"` for pure lx backends.
- `Interpreter::build_tool_module` in `crates/lx/src/interpreter/tool_module.rs` currently spawns an MCP child process via `McpClient::spawn(command)` for every `use tool` invocation. It creates a `LxVal::Record` with one `BuiltinFunc` entry per discovered MCP tool, then binds the record to the alias name.
- `eval_use` in `crates/lx/src/interpreter/modules.rs` delegates to `build_tool_module` for `UseKind::Tool`.
- `Interpreter::load_module` in `crates/lx/src/interpreter/modules.rs` (lines 156-193) loads a `.lx` file: reads source, lexes, parses, desugars, creates a child `Interpreter`, executes, and returns `ModuleExports { bindings, variant_ctors }`.
- `collect_exports` in `crates/lx/src/interpreter/modules.rs` (lines 219-259) extracts exported bindings (bindings with `exported: true`, exported class/trait/typedef declarations) from a program.
- In lx, the `+` prefix on a binding makes it exported. For example `+open = (url) { ... }` produces a `Stmt::Binding` with `exported: true` and name `open`.
- `mk_dyn_async(name: &'static str, arity: usize, func: DynAsyncBuiltinFn) -> LxVal` creates an `LxVal::BuiltinFunc` wrapping an async closure.
- `DynAsyncBuiltinFn` is `Arc<dyn Fn(Vec<LxVal>, SourceSpan, Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> + Send + Sync>`.
- `LxVal::Func(Box<LxFunc>)` is the representation for user-defined functions. `LxFunc` has fields: `params: Vec<Sym>`, `body: ExprId`, `closure: Arc<Env>`, `arena: Arc<AstArena>`.
- `Interpreter::apply_func` in `crates/lx/src/interpreter/apply.rs` dispatches function calls for both `LxVal::Func` and `LxVal::BuiltinFunc`.

## Architecture

When `use tool` encounters a `command` string ending in `.lx`:
1. The interpreter loads the `.lx` file as a module (same as `load_module`) to discover its exported bindings
2. For each exported binding that is a function (`LxVal::Func` or `LxVal::MultiFunc`), it wraps the function in a tool dispatch closure
3. The tool dispatch closure emits `tool.call` before invoking the lx function, and `tool.result` or `tool.error` after -- identical to the MCP tool dispatch path
4. The resulting `LxVal::Record` of dispatch closures is bound to the alias name
5. The calling code sees the same module interface regardless of whether the backend is MCP or lx

The key difference from MCP dispatch: instead of sending JSON-RPC to a child process, the dispatch closure directly evaluates the lx function using `Interpreter::apply_func`.

Recursive tool loading (a `.lx` tool file that itself uses `use tool`) is handled by the existing `self.loading` cycle detection in `load_module`. If a `.lx` tool file loads another `.lx` tool file, it goes through the same `build_lx_tool_module` path recursively, which is safe because `load_module` already tracks loading state.

## Files to Create

- `crates/lx/src/interpreter/lx_tool_module.rs` -- pure lx backend tool module builder

## Files to Modify

- `crates/lx/src/interpreter/mod.rs` -- add `mod lx_tool_module;`
- `crates/lx/src/interpreter/tool_module.rs` -- add `.lx` extension check before MCP spawn, delegate to `build_lx_tool_module`

## Step 1: Add module declaration

File: `crates/lx/src/interpreter/mod.rs`

Add after the `mod modules;` line:

```rust
mod lx_tool_module;
```

## Step 2: Branch on .lx extension in build_tool_module

File: `crates/lx/src/interpreter/tool_module.rs`

In `build_tool_module`, before the `McpClient::spawn(command)` call, add a check for `.lx` extension:

```rust
pub(super) async fn build_tool_module(
    &mut self,
    command: &str,
    span: SourceSpan,
) -> Result<LxVal, LxError> {
    if command.ends_with(".lx") {
        return self.build_lx_tool_module(command, span).await;
    }

    // ... existing MCP spawn logic unchanged ...
}
```

This is a 3-line addition at the top of the existing method body.

## Step 3: Create lx_tool_module.rs

File: `crates/lx/src/interpreter/lx_tool_module.rs`

This file implements `build_lx_tool_module` on `Interpreter`. It:
1. Resolves the `.lx` path relative to `self.source_dir`
2. Loads and executes the module to collect exports
3. Wraps each exported function in a tool dispatch closure with auto-logging
4. Returns a `LxVal::Record` of dispatch closures

```rust
use std::sync::Arc;

use indexmap::IndexMap;
use miette::SourceSpan;

use crate::error::LxError;
use crate::sym::intern;
use crate::value::LxVal;

impl super::Interpreter {
    pub(super) async fn build_lx_tool_module(
        &mut self,
        command: &str,
        span: SourceSpan,
    ) -> Result<LxVal, LxError> {
        let file_path = self.resolve_lx_tool_path(command, span)?;
        let exports = self.load_module(&file_path, span).await?;

        let tool_module_name = command.to_string();
        let mut bindings = IndexMap::new();

        for (name, val) in &exports.bindings {
            if !matches!(val, LxVal::Func(_) | LxVal::MultiFunc(_) | LxVal::BuiltinFunc(_)) {
                bindings.insert(*name, val.clone());
                continue;
            }

            let func_val = val.clone();
            let method_name = name.as_str().to_string();
            let module_name = tool_module_name.clone();
            let source = self.source.clone();
            let arena = Arc::clone(&self.arena);
            let env = Arc::clone(&self.env);
            let ctx_ref = Arc::clone(&self.ctx);
            let module_cache = Arc::clone(&self.module_cache);
            let loading = Arc::clone(&self.loading);
            let source_dir = self.source_dir.clone();

            let dispatch = crate::value::mk_dyn_async(
                "lx_tool.call",
                1,
                Arc::new(move |args: Vec<LxVal>, span: SourceSpan, ctx: Arc<crate::runtime::RuntimeCtx>| {
                    let func_val = func_val.clone();
                    let method_name = method_name.clone();
                    let module_name = module_name.clone();
                    let source = source.clone();
                    let arena = Arc::clone(&arena);
                    let env = Arc::clone(&env);
                    let ctx_ref = Arc::clone(&ctx_ref);
                    let module_cache = Arc::clone(&module_cache);
                    let loading = Arc::clone(&loading);
                    let source_dir = source_dir.clone();
                    Box::pin(async move {
                        let call_id = ctx.call_id_counter.fetch_add(
                            1,
                            std::sync::atomic::Ordering::SeqCst,
                        );

                        {
                            let cache = ctx.replay_cache.lock();
                            if let Some(cached) = cache.get(&call_id) {
                                return Ok(cached.clone());
                            }
                        }

                        let arg_json = serde_json::Value::from(&args[0]);
                        {
                            let mut entry = crate::runtime::StreamEntry::new(
                                "tool.call", "main", &ctx.id_gen,
                            );
                            entry = entry.with_field(
                                "call_id",
                                serde_json::Value::Number(call_id.into()),
                            );
                            entry = entry.with_field(
                                "tool",
                                serde_json::Value::String(module_name.clone()),
                            );
                            entry = entry.with_field(
                                "method",
                                serde_json::Value::String(method_name.clone()),
                            );
                            entry = entry.with_field("args", arg_json);
                            ctx.xadd(entry);
                        }

                        let mut interp = super::Interpreter::new(
                            &source, source_dir, ctx_ref,
                        );
                        interp.arena = arena;
                        interp.env = env;
                        interp.module_cache = module_cache;
                        interp.loading = loading;

                        let result = interp.apply_func(func_val, args[0].clone(), span).await;

                        match result {
                            Ok(val) => {
                                let result_json = serde_json::Value::from(&val);
                                let mut entry = crate::runtime::StreamEntry::new(
                                    "tool.result", "main", &ctx.id_gen,
                                );
                                entry = entry.with_field(
                                    "call_id",
                                    serde_json::Value::Number(call_id.into()),
                                );
                                entry = entry.with_field(
                                    "tool",
                                    serde_json::Value::String(module_name.clone()),
                                );
                                entry = entry.with_field(
                                    "method",
                                    serde_json::Value::String(method_name.clone()),
                                );
                                entry = entry.with_field("result", result_json);
                                ctx.xadd(entry);
                                Ok(val)
                            },
                            Err(signal) => {
                                let err_msg = match &signal {
                                    crate::error::EvalSignal::Error(e) => e.to_string(),
                                    crate::error::EvalSignal::Break(_) => "unexpected break in lx tool".to_string(),
                                    _ => format!("{signal:?}"),
                                };
                                let mut entry = crate::runtime::StreamEntry::new(
                                    "tool.error", "main", &ctx.id_gen,
                                );
                                entry = entry.with_field(
                                    "call_id",
                                    serde_json::Value::Number(call_id.into()),
                                );
                                entry = entry.with_field(
                                    "tool",
                                    serde_json::Value::String(module_name.clone()),
                                );
                                entry = entry.with_field(
                                    "method",
                                    serde_json::Value::String(method_name.clone()),
                                );
                                entry = entry.with_field(
                                    "error",
                                    serde_json::Value::String(err_msg.clone()),
                                );
                                ctx.xadd(entry);
                                Ok(LxVal::err_str(format!(
                                    "tool '{}' method '{}': {}",
                                    module_name, method_name, err_msg,
                                )))
                            },
                        }
                    })
                }),
            );

            bindings.insert(*name, dispatch);
        }

        Ok(LxVal::record(bindings))
    }

    fn resolve_lx_tool_path(
        &self,
        command: &str,
        span: SourceSpan,
    ) -> Result<std::path::PathBuf, LxError> {
        let path = std::path::Path::new(command);
        if path.is_absolute() {
            return Ok(path.to_path_buf());
        }
        let source_dir = self.source_dir.as_ref().ok_or_else(|| {
            LxError::runtime(
                format!("cannot resolve lx tool path '{command}': no source directory"),
                span,
            )
        })?;
        let resolved = source_dir.join(command);
        if !resolved.exists() {
            return Err(LxError::runtime(
                format!("lx tool file not found: {}", resolved.display()),
                span,
            ));
        }
        Ok(resolved)
    }
}
```

## Step 4: Verify file lengths

- `lx_tool_module.rs`: ~140 lines. Under 300.
- `tool_module.rs`: gains 3 lines (the `.lx` extension check). Stays under 300.
- `mod.rs`: gains 1 line (`mod lx_tool_module;`). Stays under 300.

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. All existing tests pass unchanged
3. Write two test `.lx` files:

File `test_lx_backend.lx`:
```lx
+greet = (name) { "Hello, " ++ name }
+add = (args) { args.a + args.b }
```

File `test_use_lx_tool.lx`:
```lx
use tool "./test_lx_backend.lx" as Backend
result = Backend.greet "world"
assert result == "Hello, world"
sum = Backend.add {a: 1, b: 2}
assert sum == 3
```

This verifies: path resolution, module loading, export discovery, function wrapping, dispatch, and return value propagation. If auto-logging is active (unit_6_auto_logging complete), the event stream also contains `tool.call` and `tool.result` entries for each call.
