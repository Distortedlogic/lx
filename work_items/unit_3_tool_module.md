# Unit 3: Tool Module -- Interpreter Integration

Wire `use tool "command" as Name` into the interpreter: spawn MCP server, connect, discover tools, create module bindings, dispatch method calls via MCP.

## Prerequisites

- **Unit 1** (MCP Client Library) must be complete -- provides `McpClient` at `crates/lx/src/mcp/client.rs`
- **Unit 2** (Parser) must be complete -- provides `UseKind::Tool { command, alias }` variant

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify

## Current State

- `Interpreter` struct is in `crates/lx/src/interpreter/mod.rs` (lines 41-49) with fields: `env`, `source`, `source_dir`, `module_cache`, `loading`, `ctx`, `arena`
- `RuntimeCtx` is in `crates/lx/src/runtime/mod.rs` (lines 20-39) with backend traits and config fields
- `eval_use` is in `crates/lx/src/interpreter/modules.rs` (lines 15-61)
- `eval_field_access` is in `crates/lx/src/interpreter/apply_helpers.rs` (lines 13-70)
- Field access on `LxVal::Record(r)` returns `r.get(name)` (line 17)
- `LxVal` is defined in `crates/lx/src/value/mod.rs` (lines 56-115)
- `BuiltinFunc` and `BuiltinKind` are in `crates/lx/src/value/func.rs`
- `BuiltinFunc.name` is `&'static str` (not `Sym`)
- `mk_dyn_async(name: &'static str, arity: usize, func: DynAsyncBuiltinFn) -> LxVal` returns an `LxVal` wrapping a `BuiltinFunc`
- Tool modules are bound as `LxVal::Record` with builtin functions as methods
- The conversion `serde_json::Value -> LxVal` exists via `From` impl in `crates/lx/src/value/serde_impl.rs`
- The conversion `&LxVal -> serde_json::Value` exists via `From` impl in `crates/lx/src/value/serde_impl.rs`

## Files to Create

- `crates/lx/src/interpreter/tool_module.rs` -- tool module creation logic (extracted from modules.rs to stay under 300-line limit)

## Files to Modify

- `crates/lx/src/interpreter/modules.rs` -- implement `UseKind::Tool` in `eval_use` (delegates to `tool_module.rs`)
- `crates/lx/src/interpreter/mod.rs` -- add `mod tool_module;`
- `crates/lx/src/runtime/mod.rs` -- add `tool_connections` field to RuntimeCtx

## Step 1: Add tool_connections to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

Add a field to `RuntimeCtx` (after the `test_runs` field at line 38):

```rust
#[default(Arc::new(parking_lot::Mutex::new(HashMap::new())))]
pub tool_connections: Arc<parking_lot::Mutex<HashMap<String, Arc<crate::mcp::McpClient>>>>,
```

`HashMap` is already imported at line 9: `use std::collections::HashMap;`.

## Step 2: Implement UseKind::Tool in eval_use (delegate to tool_module.rs)

File: `crates/lx/src/interpreter/modules.rs`

Replace the placeholder `UseKind::Tool` handling (added in Unit 2) with a call to the extracted helper in `tool_module.rs`.

The early-return guard added in Unit 2 looks like:
```rust
if let UseKind::Tool { ref command, alias } = use_stmt.kind {
    return Err(LxError::runtime("use tool not yet implemented", span));
}
```

Replace with:

```rust
if let UseKind::Tool { ref command, alias } = use_stmt.kind {
    let module_record = self.build_tool_module(command, span).await?;
    let env = self.env.child();
    env.bind(alias, module_record);
    self.env = Arc::new(env);
    return Ok(());
}
```

This keeps the addition to modules.rs minimal (~6 lines), well under the 300-line limit.

## Step 2b: Create tool_module.rs with the tool module builder

File: `crates/lx/src/interpreter/tool_module.rs`

Add `mod tool_module;` to `crates/lx/src/interpreter/mod.rs` (after `mod modules;`).

```rust
use std::sync::Arc;

use indexmap::IndexMap;
use miette::SourceSpan;

use crate::error::LxError;
use crate::value::LxVal;

impl super::Interpreter {
    pub(super) async fn build_tool_module(
        &mut self,
        command: &str,
        span: SourceSpan,
    ) -> Result<LxVal, LxError> {
        let client = crate::mcp::McpClient::spawn(command)
            .await
            .map_err(|e| LxError::runtime(format!("use tool \"{command}\": {e}"), span))?;

        let tool_names: Vec<String> = client
            .available_tools()
            .iter()
            .map(|t| t.name.clone())
            .collect();

        let client_arc = Arc::new(client);
        self.ctx.tool_connections.lock().insert(command.to_string(), Arc::clone(&client_arc));

        let mut bindings = IndexMap::new();
        for tool_name in &tool_names {
            let client_ref = Arc::clone(&client_arc);
            let method_name = tool_name.clone();
            let tool_module_name = command.to_string();
            let leaked_name: &'static str = Box::leak(tool_name.clone().into_boxed_str());
            let val = crate::value::mk_dyn_async(
                leaked_name,
                1,
                Arc::new(move |args: Vec<LxVal>, span: SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
                    let client_ref = Arc::clone(&client_ref);
                    let method_name = method_name.clone();
                    let tool_module_name = tool_module_name.clone();
                    Box::pin(async move {
                        if !client_ref.is_alive().await {
                            return Ok(LxVal::err_str(format!(
                                "tool '{}' process exited unexpectedly",
                                tool_module_name
                            )));
                        }
                        let arg_json = serde_json::Value::from(&args[0]);
                        match client_ref.call_tool(&method_name, arg_json).await {
                            Ok(result) => Ok(LxVal::from(result)),
                            Err(e) => Ok(LxVal::err_str(format!(
                                "tool '{}' method '{}': {e}",
                                tool_module_name, method_name
                            ))),
                        }
                    })
                }),
            );
            bindings.insert(
                crate::sym::intern(tool_name),
                val,
            );
        }

        Ok(LxVal::record(bindings))
    }
}
```

## Step 3: Add shutdown on program exit

File: `crates/lx-cli/src/run.rs`

After the interpreter finishes execution (line 22-29), add shutdown logic:

Current code (lines 20-31):
```rust
ctx.tokio_runtime.block_on(async {
    interp.load_default_tools().await.map_err(|e| vec![e])?;
    match interp.exec(&program).await {
      Ok(val) => {
        if !matches!(val, lx::value::LxVal::Unit) {
          println!("{val}");
        }
        Ok(())
      },
      Err(e) => Err(vec![e]),
    }
  })
```

Change to:
```rust
ctx.tokio_runtime.block_on(async {
    interp.load_default_tools().await.map_err(|e| vec![e])?;
    let result = match interp.exec(&program).await {
      Ok(val) => {
        if !matches!(val, lx::value::LxVal::Unit) {
          println!("{val}");
        }
        Ok(())
      },
      Err(e) => Err(vec![e]),
    };
    for (_name, client) in ctx.tool_connections.lock().drain() {
        client.shutdown().await;
    }
    result
  })
```

## Step 4: Handle dead tool process

The builtin closure in Step 2 checks `client_ref.is_alive()` before calling, returning `LxVal::err_str(...)` if the process has exited. If the process crashes mid-call, the `McpClient`'s reader task ends and subsequent `call_tool` calls return errors via the oneshot channel being dropped.

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. Write a test `.lx` file that does `use tool "echo" as Echo` -- it fails with "command 'echo' not found" or similar (since `echo` is not an MCP server), confirming the code path is exercised
3. All existing tests pass unchanged
