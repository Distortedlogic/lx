# Unit 3: Tool Module Type + Dispatch

## Goal

Wire `use tool "command" as Name` into the interpreter so it:
1. Spawns an MCP server process via `McpClient` (Unit 2)
2. Creates a module binding in the environment with discovered methods
3. Dispatches method calls through the MCP protocol
4. Handles process lifecycle (shutdown on exit, crash detection)

## Preconditions

- Unit 1 complete: `UseKind::Tool { command, alias }` exists in AST, parser accepts `use tool "cmd" as Name`
- Unit 2 complete: `McpClient` exists in `crates/lx/src/mcp/client.rs` with `spawn`, `call_tool`, `has_tool`, `shutdown`
- `Interpreter` struct at `crates/lx/src/interpreter/mod.rs:41-49` has `env`, `ctx`, `arena`, etc.
- `RuntimeCtx` at `crates/lx/src/runtime/mod.rs:20-39` holds `Arc<dyn Backend>` fields and a `tokio_runtime`
- Field access dispatch at `crates/lx/src/interpreter/apply_helpers.rs:12-70`
- `LxVal` enum at `crates/lx/src/value/mod.rs:57-115` has existing variants including `Record`, `Object`, `BuiltinFunc`
- `BuiltinFunc` at `crates/lx/src/value/func.rs` — check exact definition
- `eval_use` at `crates/lx/src/interpreter/modules.rs:15-62` handles `UseKind` variants

## Step 1: Add tool module storage to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

Add a field to `RuntimeCtx` for tracking active MCP clients so they can be shut down on exit:

```rust
use std::sync::Arc;
use parking_lot::Mutex;

// Add to RuntimeCtx struct:
pub tool_clients: Arc<Mutex<Vec<Arc<tokio::sync::Mutex<crate::mcp::McpClient>>>>>,
```

Initialize with `#[default(Arc::new(Mutex::new(Vec::new())))]`.

## Step 2: Implement eval_use for UseKind::Tool

File: `crates/lx/src/interpreter/modules.rs`

Replace the `UseKind::Tool` placeholder (added in Unit 1) with the implementation below.

### Key API facts (verified):

- `BuiltinFunc` at `value/func.rs:40-46` has `name: &'static str`, `arity: usize`, `kind: BuiltinKind`, `applied: Vec<LxVal>`
- `BuiltinFunc.name` is `&'static str` — for dynamic names, use `Box::leak(name.into_boxed_str())` to get a `&'static str`. This leaks memory but tool names are bounded (one per discovered method per tool module).
- `mk_dyn_async(name, arity, func)` at `value/func.rs:48-50` creates a `LxVal::BuiltinFunc` with `BuiltinKind::DynAsync`
- `DynAsyncBuiltinFn` signature: `Arc<dyn Fn(Vec<LxVal>, SourceSpan, Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> + Send + Sync>`
- `LxVal::from(serde_json::Value)` exists at `value/serde_impl.rs:74-97` — converts JSON to LxVal
- `serde_json::Value::from(&LxVal)` exists at `value/serde_impl.rs:99-103` — converts LxVal to JSON

### Implementation

In the match on `&use_stmt.kind` at `modules.rs:43`, replace the placeholder:

```rust
UseKind::Tool { ref command, alias } => {
  // Split command string: first token is command, rest are args
  let parts: Vec<&str> = command.splitn(2, ' ').collect();
  let cmd = parts[0];
  let cmd_args: Vec<&str> = if parts.len() > 1 {
    parts[1].split_whitespace().collect()
  } else {
    vec![]
  };

  // Spawn MCP process (eager, not lazy — fail-fast on bad commands)
  // eval_use is already async (called from eval loop inside block_on),
  // so use .await directly — do NOT call block_on (would panic: "cannot
  // start a runtime from within a runtime").
  let client = crate::mcp::McpClient::spawn(cmd, &cmd_args).await
    .map_err(|e| LxError::runtime(format!("use tool \"{command}\": {e}"), span))?;

  let client = Arc::new(tokio::sync::Mutex::new(client));
  self.ctx.tool_clients.lock().push(Arc::clone(&client));

  // Discover tools and create module bindings
  let tool_names: Vec<String> = {
    let c = client.lock().await;
    c.available_tools().iter().map(|t| t.name.clone()).collect()
  };

  let mut bindings = IndexMap::new();
  for tool_name in tool_names {
    let client_ref = Arc::clone(&client);
    let method_name = tool_name.clone();

    // Leak the name to get &'static str for BuiltinFunc
    let static_name: &'static str = Box::leak(tool_name.clone().into_boxed_str());

    let func = crate::value::mk_dyn_async(
      static_name,
      1, // arity: one argument (the args record)
      Arc::new(move |args: Vec<LxVal>, span: miette::SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
        let client_ref = Arc::clone(&client_ref);
        let method_name = method_name.clone();
        // DynAsync closures return Pin<Box<dyn Future<...>>> which is
        // awaited by the interpreter's apply path. Use .await directly —
        // do NOT call block_on (we're already inside the tokio runtime).
        Box::pin(async move {
          let args_json = serde_json::Value::from(&args[0]);
          let c = client_ref.lock().await;
          let result = c.call_tool(&method_name, args_json).await;
          drop(c);
          match result {
            Ok(json) => Ok(LxVal::ok(LxVal::from(json))),
            Err(e) => Ok(LxVal::err_str(&e)),
          }
        }) as Pin<Box<dyn std::future::Future<Output = Result<LxVal, LxError>>>>
      }),
    );
    bindings.insert(intern(&tool_name), func);
  }

  let record = LxVal::record(bindings);
  let env = self.env.child();
  env.bind(*alias, record);
  self.env = Arc::new(env);
  return Ok(());
}
```

Add needed imports at the top of `modules.rs`:
```rust
use std::pin::Pin;
use crate::value::mk_dyn_async;
```

## Step 3: JSON-to-LxVal conversion (already exists)

File: `crates/lx/src/value/serde_impl.rs`

JSON↔LxVal conversions already exist:
- `LxVal::from(serde_json::Value)` at lines 74-97 — converts JSON → LxVal
- `serde_json::Value::from(&LxVal)` at lines 99-103 — converts LxVal → JSON (via `serde_json::to_value`)

**Do NOT add new conversion functions.** Use the existing `From` impls:
```rust
// JSON → LxVal
let lx_val = LxVal::from(json_value);

// LxVal → JSON
let json_val = serde_json::Value::from(&lx_val);
```

## Step 4: Tool process shutdown on program exit

File: `crates/lx/src/interpreter/mod.rs`

Add a `shutdown_tools` method to `Interpreter`:

```rust
pub async fn shutdown_tools(&self) {
  let clients = self.ctx.tool_clients.lock().clone();
  for client in clients {
    let mut c = client.lock().await;
    c.shutdown().await;
  }
}
```

File: `crates/lx-cli/src/run.rs`

The `run()` function at line 8 calls `interp.exec(&program).await` at line 22. After exec completes, add shutdown. Change lines 22-29:

```rust
let exec_result = interp.exec(&program).await;
interp.shutdown_tools().await;
match exec_result {
  Ok(val) => {
    if !matches!(val, lx::value::LxVal::Unit) {
      println!("{val}");
    }
    Ok(())
  },
  Err(e) => Err(vec![e]),
}
```

## Step 5: Handle tool process crash

In the `call_tool` builtin function created in Step 2, if the MCP client returns an error indicating the process exited, wrap it as `LxVal::Err`:

The tool call builtin (Step 2) already handles errors by returning `LxVal::ok(LxVal::from(json))` on success and `LxVal::err_str(&e)` on failure. The lx program handles these with `^` propagation or `?` matching.

## Verification

Run `just diagnose`. Write a test that uses `use tool` with a simple echo MCP server (if one is available in the test fixtures). If not, verify compilation only — integration testing requires an actual MCP server binary.
