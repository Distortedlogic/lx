# Unit 9: lx-eval

## Scope

Extract the evaluation/runtime layer into `crates/lx-eval`. This is the largest unit (82 files). It includes `interpreter/` (18 files), `builtins/` (16 files incl agent/ subdir), `stdlib/` (38 files incl subdirs), `runtime/` (8 files), `mcp_client.rs`, and `tool_module.rs`. This unit also implements the trait impls for the trait objects introduced in unit 8: `BuiltinCtx` for `RuntimeCtx`, `ToolModuleHandle` for `ToolModule`, and `McpStreamSink` implementing `ExternalStreamSink`.

## Prerequisites

- Units 1-2 complete (lx-span, lx-ast)
- Unit 3 complete (lx-parser: exports `lexer::lex`, `parser::parse`)
- Unit 4 complete (lx-desugar: exports `folder::desugar`)
- Unit 8 complete (lx-value: exports `LxVal`, `LxError`, `Env`, `EventStream`, `ModuleExports`, `BuiltinCtx`, `ExternalStreamSink`, `ToolModuleHandle`, and all builtin fn type aliases use `Arc<dyn BuiltinCtx>`)

## Steps

### Step 1: Create crate skeleton

Create `crates/lx-eval/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-eval"
version = "0.1.0"

[dependencies]
lx-span = { path = "../lx-span" }
lx-ast = { path = "../lx-ast" }
lx-parser = { path = "../lx-parser" }
lx-desugar = { path = "../lx-desugar" }
lx-value = { path = "../lx-value" }
async-recursion.workspace = true
chrono.workspace = true
cron.workspace = true
dashmap.workspace = true
extism.workspace = true
futures.workspace = true
indexmap.workspace = true
itertools.workspace = true
miette.workspace = true
num-bigint.workspace = true
num-integer.workspace = true
num-traits.workspace = true
parking_lot.workspace = true
pulldown-cmark.workspace = true
reqwest.workspace = true
serde.workspace = true
serde_json.workspace = true
similar.workspace = true
smart-default.workspace = true
tokio.workspace = true
tokio-tungstenite.workspace = true
toml.workspace = true

[lints]
workspace = true
```

### Step 2: Create lx-eval/src/lib.rs

```rust
pub use lx_span::{PLUGIN_MANIFEST, LX_MANIFEST};

pub mod builtins;
pub mod interpreter;
pub mod mcp_client;
pub mod mcp_stream_sink;
pub mod runtime;
pub mod stdlib;
pub mod tool_module;
```

Note: `mcp_stream_sink` is declared `pub mod` because lx-cli needs to construct `McpStreamSink` via `lx::mcp_stream_sink::McpStreamSink` (see unit 10).

### Step 3: Move all source directories and files

Move these directories from `crates/lx/src/` to `crates/lx-eval/src/`:

| Source | Destination |
|--------|-------------|
| `crates/lx/src/interpreter/` (18 files) | `crates/lx-eval/src/interpreter/` |
| `crates/lx/src/builtins/` (16 files incl agent/ subdir) | `crates/lx-eval/src/builtins/` |
| `crates/lx/src/stdlib/` (38 files incl subdirs) | `crates/lx-eval/src/stdlib/` |
| `crates/lx/src/runtime/` (8 files) | `crates/lx-eval/src/runtime/` |
| `crates/lx/src/mcp_client.rs` | `crates/lx-eval/src/mcp_client.rs` |
| `crates/lx/src/tool_module.rs` | `crates/lx-eval/src/tool_module.rs` |

Complete file list for interpreter/ (18 files):

- `mod.rs`, `ambient.rs`, `apply.rs`, `apply_helpers.rs`, `collections.rs`, `default_tools.rs`, `eval.rs`, `eval_assert.rs`, `eval_ops.rs`, `exec_stmt.rs`, `hints.rs`, `lx_tool_module.rs`, `messaging.rs`, `modules.rs`, `patterns.rs`, `trait_apply.rs`, `traits.rs`, `type_apply.rs`

Complete file list for builtins/ (16 files):

- `mod.rs`, `call.rs`, `coll.rs`, `coll_transform.rs`, `convert.rs`, `hof.rs`, `hof_extra.rs`, `hof_parallel.rs`, `llm.rs`, `register.rs`, `register_helpers.rs`, `shell.rs`, `str.rs`, `str_extra.rs`, `agent/mod.rs`, `agent/spawn.rs`

Complete file list for stdlib/ (38 files):

- `mod.rs`, `channel.rs`, `checkpoint.rs`, `env.rs`, `events.rs`, `fs.rs`, `helpers.rs`, `http.rs`, `introspect.rs`, `math.rs`, `schema.rs`, `stream.rs`, `time.rs`, `trait_ops.rs`, `wasm.rs`, `wasm_marshal.rs`
- `cron/mod.rs`, `cron/cron_helpers.rs`
- `diag/mod.rs`, `diag/diag_helpers.rs`, `diag/diag_types.rs`, `diag/diag_walk.rs`, `diag/diag_walk_expr.rs`, `diag/echart.rs`, `diag/mermaid.rs`
- `md/mod.rs`, `md/md_parse.rs`, `md/md_render.rs`
- `sandbox/mod.rs`, `sandbox/sandbox_exec.rs`, `sandbox/sandbox_policy.rs`, `sandbox/sandbox_scope.rs`
- `store/mod.rs`, `store/store_dispatch.rs`
- `test_mod/mod.rs`, `test_mod/test_invoke.rs`, `test_mod/test_report.rs`, `test_mod/test_run.rs`

Complete file list for runtime/ (8 files):

- `mod.rs`, `agent_registry.rs`, `channel_registry.rs`, `control.rs`, `control_stdin.rs`, `control_tcp.rs`, `control_ws.rs`, `defaults.rs`

Note: `interpreter/mod.rs` is 304 lines, 4 lines over the 300-line limit. This is a pre-existing condition before this unit's work. Split the file as part of execution (e.g., extract `ModuleExports` removal saves ~5 lines from unit 8, or extract a small helper into a submodule).

### Step 4: Implement BuiltinCtx for RuntimeCtx

In `crates/lx-eval/src/runtime/mod.rs`, after the `RuntimeCtx` struct definition, add:

```rust
impl lx_value::BuiltinCtx for RuntimeCtx {
    fn event_stream(&self) -> &Arc<lx_value::EventStream> {
        &self.event_stream
    }

    fn source_dir(&self) -> Option<std::path::PathBuf> {
        self.source_dir.lock().clone()
    }

    fn network_denied(&self) -> bool {
        self.network_denied
    }

    fn test_threshold(&self) -> Option<f64> {
        self.test_threshold
    }

    fn test_runs(&self) -> Option<u32> {
        self.test_runs
    }
}
```

### Step 5: Implement ToolModuleHandle for ToolModule

In `crates/lx-eval/src/tool_module.rs`, add the trait impl:

```rust
impl lx_value::ToolModuleHandle for ToolModule {
    fn call_tool(
        &self,
        method: &str,
        args: lx_value::LxVal,
        event_stream: &lx_value::EventStream,
        agent_name: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<lx_value::LxVal, lx_value::LxError>> + '_>> {
        Box::pin(self.call_tool(method, args, event_stream, agent_name))
    }

    fn shutdown(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + '_>> {
        Box::pin(self.shutdown())
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn alias(&self) -> &str {
        &self.alias
    }
}
```

Note: The inherent `call_tool` and `shutdown` methods remain as-is on `ToolModule`. The trait impl delegates to them.

### Step 6: Create McpStreamSink

Create `crates/lx-eval/src/mcp_stream_sink.rs`:

```rust
use std::sync::Arc;

use crate::mcp_client::McpClient;

pub struct McpStreamSink {
    client: Arc<tokio::sync::Mutex<McpClient>>,
}

impl McpStreamSink {
    pub fn new(client: Arc<tokio::sync::Mutex<McpClient>>) -> Self {
        Self { client }
    }
}

impl lx_value::ExternalStreamSink for McpStreamSink {
    fn xadd(&self, entry_json: serde_json::Value) {
        let client = Arc::clone(&self.client);
        tokio::task::spawn(async move {
            if let Err(e) = client.lock().await.tools_call("xadd", entry_json).await {
                eprintln!("[stream:external] xadd failed: {e}");
            }
        });
    }

    fn shutdown(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            self.client.lock().await.shutdown().await;
        })
    }
}
```

### Step 7: Rewrite imports in all moved files

This is the bulk of the work. Every file that references `crate::` paths from the old `lx` crate needs updating.

#### Global import rewrite table

| Old import | New import |
|------------|-----------|
| `crate::sym::Sym` | `lx_span::sym::Sym` |
| `crate::sym::intern` | `lx_span::sym::intern` |
| `crate::ast::{...}` | `lx_ast::ast::{...}` |
| `crate::visitor::{...}` | `lx_ast::visitor::{...}` |
| `crate::source::FileId` | `lx_span::source::FileId` |
| `crate::value::LxVal` | `lx_value::LxVal` |
| `crate::value::*` | `lx_value::*` |
| `crate::error::LxError` | `lx_value::LxError` |
| `crate::error::{EvalResult, EvalSignal, LxError}` | `lx_value::{EvalResult, EvalSignal, LxError}` |
| `crate::env::Env` | `lx_value::Env` |
| `crate::event_stream::EventStream` | `lx_value::EventStream` |
| `crate::runtime::RuntimeCtx` | `crate::runtime::RuntimeCtx` (stays same within lx-eval) |
| `crate::interpreter::Interpreter` | `crate::interpreter::Interpreter` (stays same) |
| `crate::interpreter::ModuleExports` | `lx_value::ModuleExports` |
| `crate::builtins::*` | `crate::builtins::*` (stays same) |
| `crate::stdlib::*` | `crate::stdlib::*` (stays same) |
| `crate::tool_module::ToolModule` | `crate::tool_module::ToolModule` (stays same) |
| `crate::mcp_client::McpClient` | `crate::mcp_client::McpClient` (stays same) |
| `crate::lexer::lex` | `lx_parser::lexer::lex` |
| `crate::parser::parse` | `lx_parser::parser::parse` |
| `crate::folder::desugar` | `lx_desugar::folder::desugar` |
| `crate::PLUGIN_MANIFEST` | `crate::PLUGIN_MANIFEST` (stays same) |
| `crate::LX_MANIFEST` | `crate::LX_MANIFEST` (stays same) |

#### File-by-file import changes

**interpreter/mod.rs:**
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`
- `use crate::ast::{...};` -> `use lx_ast::ast::{...};`
- `use crate::env::Env;` -> `use lx_value::Env;`
- `use crate::error::{EvalResult, EvalSignal, LxError};` -> `use lx_value::{EvalResult, EvalSignal, LxError};`
- `use crate::runtime::RuntimeCtx;` -> stays `use crate::runtime::RuntimeCtx;`
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`
- Remove `ModuleExports` struct definition (moved to lx-value in unit 8)
- `pub(crate) tool_modules: Vec<Arc<crate::tool_module::ToolModule>>` -> `pub(crate) tool_modules: Vec<Arc<dyn lx_value::ToolModuleHandle>>`
- `pub(crate) agent_mailbox_rx` type stays, uses `crate::runtime::agent_registry::AgentMessage`

**interpreter/modules.rs:**
- `use crate::ast::{...};` -> `use lx_ast::ast::{...};`
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::folder::desugar;` -> `use lx_desugar::folder::desugar;`
- `use crate::parser::parse;` -> `use lx_parser::parser::parse;`
- `use crate::source::FileId;` -> `use lx_span::source::FileId;`
- `use crate::stdlib::wasm::load_plugin;` -> `use crate::stdlib::wasm::load_plugin;`
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`
- `use super::{Interpreter, ModuleExports};` -> `use super::Interpreter;` and `use lx_value::ModuleExports;`

**interpreter/default_tools.rs:**
- `use crate::error::{EvalSignal, LxError};` -> `use lx_value::{EvalSignal, LxError};`
- `use crate::folder::desugar;` -> `use lx_desugar::folder::desugar;`
- `use crate::parser::parse;` -> `use lx_parser::parser::parse;`
- `use crate::source::FileId;` -> `use lx_span::source::FileId;`

**interpreter/lx_tool_module.rs:**
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::sym::intern;` -> `use lx_span::sym::intern;`
- `use crate::value::{LxVal, mk_dyn_async};` -> `use lx_value::{LxVal, mk_dyn_async};`
- The `_ctx: Arc<crate::runtime::RuntimeCtx>` in the closure becomes `_ctx: Arc<dyn lx_value::BuiltinCtx>`

**interpreter/apply_helpers.rs:**
- `use crate::error::{EvalResult, LxError};` -> `use lx_value::{EvalResult, LxError};`
- `use crate::runtime::RuntimeCtx;` -> `use crate::runtime::RuntimeCtx;`
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`
- `bi_tool_dispatch` signature: `fn bi_tool_dispatch(args: Vec<LxVal>, span: SourceSpan, ctx: Arc<RuntimeCtx>)` -> `fn bi_tool_dispatch(args: Vec<LxVal>, span: SourceSpan, ctx: Arc<dyn lx_value::BuiltinCtx>)`
- Inside `bi_tool_dispatch`: `tm.call_tool(method, arg, &ctx.event_stream, "main")` -> `tm.call_tool(method, arg, ctx.event_stream(), "main")`

**builtins/mod.rs:**
- `use crate::env::Env;` -> `use lx_value::Env;`
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::runtime::RuntimeCtx;` -> change to `use lx_value::BuiltinCtx;`
- `use crate::value::{...};` -> `use lx_value::{...};`
- `call_value` and `call_value_sync` signatures: `ctx: &Arc<RuntimeCtx>` -> `ctx: &Arc<dyn BuiltinCtx>`

**builtins/register.rs:**
- All `fn bi_*(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>)` -> `fn bi_*(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn lx_value::BuiltinCtx>)`
- `ctx.event_stream.xadd(...)` -> `ctx.event_stream().xadd(...)`

**builtins/register_helpers.rs:**
- `ctx: &Arc<RuntimeCtx>` -> `ctx: &Arc<dyn lx_value::BuiltinCtx>`
- `ctx.source_dir.lock().clone()` -> `ctx.source_dir()`

**builtins/call.rs:**
- `use crate::runtime::RuntimeCtx;` -> `use lx_value::BuiltinCtx;`
- `ctx: &Arc<RuntimeCtx>` -> `ctx: &Arc<dyn BuiltinCtx>`

**builtins/agent/spawn.rs:**
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::interpreter::Interpreter;` -> `use crate::interpreter::Interpreter;`
- `use crate::runtime::RuntimeCtx;` -> `use crate::runtime::RuntimeCtx;`
- The function parameter `ctx: Arc<RuntimeCtx>` stays concrete (spawn needs the full RuntimeCtx for `Interpreter::new`)
- `ctx.event_stream.xadd(...)` -> stays as-is (RuntimeCtx still has `event_stream` field directly)

**All hof/hof_extra/hof_parallel/shell/llm/convert/coll/coll_transform/str/str_extra builtins:**
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::runtime::RuntimeCtx;` -> `use lx_value::BuiltinCtx;`
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`
- `ctx: &Arc<RuntimeCtx>` -> `ctx: &Arc<dyn BuiltinCtx>` (sync) or `ctx: Arc<RuntimeCtx>` -> `ctx: Arc<dyn BuiltinCtx>` (async)

**runtime/mod.rs:**
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`
- `use crate::event_stream::EventStream;` -> `use lx_value::EventStream;`

**runtime/control.rs:**
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::event_stream::EventStream;` -> `use lx_value::EventStream;`
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`

**runtime/defaults.rs:**
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`

**runtime/control_stdin.rs, control_tcp.rs, control_ws.rs:**
- Update any `crate::` references to appropriate new paths

**stdlib/mod.rs:**
- `use crate::interpreter::ModuleExports;` -> `use lx_value::ModuleExports;`

**All stdlib/*.rs files:**
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`
- Module-specific crate paths stay as `crate::` since they are within lx-eval

**tool_module.rs:**
- `use crate::error::LxError;` -> `use lx_value::LxError;`
- `use crate::event_stream::EventStream;` -> `use lx_value::EventStream;`
- `use crate::mcp_client::McpClient;` -> `use crate::mcp_client::McpClient;`
- `use crate::sym::intern;` -> `use lx_span::sym::intern;`
- `use crate::value::LxVal;` -> `use lx_value::LxVal;`

### Step 8: Handle the `record!` macro

The `record!` macro is defined in `crates/lx/src/value/impls.rs` via `#[macro_export]`. Unit 8 moves `value/` to lx-value, so the macro moves with it. The macro uses `$crate::sym::intern` and `$crate::value::LxVal`. In lx-value, unit 8 must update these to `lx_span::sym::intern` and `$crate::LxVal` respectively (since `value` is the crate root in lx-value).

15 files in lx-eval use `record!` (14 in stdlib/ and 1 in builtins/shell.rs). Since `record!` is `#[macro_export]`, it is importable by name from lx-value. Add `use lx_value::record;` at the top of each file that uses the macro:

- `builtins/shell.rs`
- `stdlib/channel.rs`
- `stdlib/diag/mod.rs`
- `stdlib/fs.rs`
- `stdlib/helpers.rs`
- `stdlib/http.rs`
- `stdlib/introspect.rs`
- `stdlib/sandbox/mod.rs`
- `stdlib/sandbox/sandbox_policy.rs`
- `stdlib/schema.rs`
- `stdlib/store/mod.rs`
- `stdlib/store/store_dispatch.rs`
- `stdlib/test_mod/mod.rs`
- `stdlib/test_mod/test_run.rs`
- `stdlib/trait_ops.rs`

### Step 9: Add lx-eval to workspace and lx crate

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-eval"` to `workspace.members`.

In `crates/lx/Cargo.toml`, add:

```toml
lx-eval = { path = "../lx-eval" }
```

### Step 10: Re-export shims in lx crate

Delete the moved directories/files from `crates/lx/src/`. Replace with re-export shims:

**crates/lx/src/interpreter.rs** (replaces directory):
```rust
pub use lx_eval::interpreter::*;
```

**crates/lx/src/builtins.rs** (replaces directory):
```rust
pub use lx_eval::builtins::*;
```

**crates/lx/src/stdlib.rs** (replaces directory):
```rust
pub use lx_eval::stdlib::*;
```

**crates/lx/src/runtime.rs** (replaces directory):
```rust
pub use lx_eval::runtime::*;
```

**crates/lx/src/mcp_client.rs:**
```rust
pub use lx_eval::mcp_client::*;
```

**crates/lx/src/tool_module.rs:**
```rust
pub use lx_eval::tool_module::*;
```

Update `crates/lx/src/lib.rs` — keep the `pub mod` declarations but they now resolve to the shim files.

Remove the constants `PLUGIN_MANIFEST` and `LX_MANIFEST` from `crates/lx/src/lib.rs` and add:

```rust
pub use lx_eval::{PLUGIN_MANIFEST, LX_MANIFEST};
```

### Step 11: Update lx-cli imports

lx-cli currently imports everything via `lx::`. The re-export shims make this transparent. Verify these imports work:

- `lx::runtime::RuntimeCtx` (via shim)
- `lx::interpreter::Interpreter` (via shim)
- `lx::lexer::lex` (from lx-parser shim, already done in unit 3)
- `lx::parser::parse` (from lx-parser shim)
- `lx::folder::desugar` (from lx-desugar shim)
- `lx::value::LxVal` (from lx-value shim)
- `lx::error::LxError` (from lx-value shim)
- `lx::mcp_client::McpClient` (from lx-eval shim)
- `lx::runtime::ControlChannelState` (from lx-eval shim)
- `lx::runtime::control_stdin::run_stdin_control` (from lx-eval shim)
- `lx::runtime::control_ws::run_ws_control` (from lx-eval shim)
- `lx::runtime::control_tcp::run_tcp_control` (from lx-eval shim)
- `lx::runtime::ControlYieldBackend` (from lx-eval shim)
- `lx::stdlib::diag::extract_mermaid` (from lx-eval shim)

## Files touched

| Action | File |
|--------|------|
| CREATE | `crates/lx-eval/Cargo.toml` |
| CREATE | `crates/lx-eval/src/lib.rs` |
| CREATE | `crates/lx-eval/src/mcp_stream_sink.rs` |
| MOVE+EDIT | `interpreter/` (18 files) |
| MOVE+EDIT | `builtins/` (16 files incl agent/ subdir) |
| MOVE+EDIT | `stdlib/` (38 files incl subdirs) |
| MOVE+EDIT | `runtime/` (8 files) |
| MOVE+EDIT | `mcp_client.rs` |
| MOVE+EDIT | `tool_module.rs` |
| DELETE | All moved files from `crates/lx/src/` |
| CREATE | `crates/lx/src/interpreter.rs` (shim) |
| CREATE | `crates/lx/src/builtins.rs` (shim) |
| CREATE | `crates/lx/src/stdlib.rs` (shim) |
| CREATE | `crates/lx/src/runtime.rs` (shim) |
| EDIT | `crates/lx/src/mcp_client.rs` (shim) |
| EDIT | `crates/lx/src/tool_module.rs` (shim) |
| EDIT | `crates/lx/src/lib.rs` (update re-exports, remove constants) |
| EDIT | `crates/lx/Cargo.toml` (add lx-eval dep) |
| EDIT | `/home/entropybender/repos/lx/Cargo.toml` (add workspace member) |

## Verification

1. `just diagnose` passes
2. `just test` passes (all .lx suite tests)
3. `RuntimeCtx` implements `BuiltinCtx`
4. `ToolModule` implements `ToolModuleHandle`
5. `McpStreamSink` implements `ExternalStreamSink`
6. lx-eval depends on lx-value but NOT on lx (no cycle)
7. lx-cli compiles without changes (all imports go through lx re-exports)
8. No file exceeds 300 lines
