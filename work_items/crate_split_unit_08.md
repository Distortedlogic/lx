# Unit 8: lx-value

## Scope

Extract the value layer into `crates/lx-value`. This includes `value/`, `error.rs`, `env.rs`, `event_stream.rs`, and the `ModuleExports` struct. Cycle-breaking refactors replace concrete types with trait objects:

- `EventStream.external_client` changes from `Arc<tokio::sync::Mutex<McpClient>>` to `Arc<dyn ExternalStreamSink>`
- Builtin fn signatures change from `Arc<RuntimeCtx>` to `Arc<dyn BuiltinCtx>`
- `LxVal::ToolModule` changes from `Arc<ToolModule>` to `Arc<dyn ToolModuleHandle>`
- `ModuleExports` moves from `interpreter/mod.rs` into lx-value

## Prerequisites

- Units 1-2 complete (lx-span, lx-ast exist)
- `lx-span` exports `sym::Sym`, `sym::intern`
- `lx-ast` exports `ast::AstArena`, `ast::ExprId`, `ast::Program`, `ast::Core`, `ast::Field`, `ast::MethodSpec`

## Steps

### Step 1: Create crate skeleton

Create `crates/lx-value/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-value"
version = "0.1.0"

[dependencies]
lx-span = { path = "../lx-span" }
lx-ast = { path = "../lx-ast" }
chrono.workspace = true
dashmap.workspace = true
derive_more.workspace = true
indexmap.workspace = true
itertools.workspace = true
miette.workspace = true
num-bigint.workspace = true
num-traits.workspace = true
parking_lot.workspace = true
serde.workspace = true
serde_json.workspace = true
strum.workspace = true
thiserror.workspace = true
tokio.workspace = true

[lints]
workspace = true
```

### Step 2: Create lx-value/src/lib.rs

```rust
mod env;
pub mod error;
mod event_stream;
mod value;

pub use env::Env;
pub use error::{AssertError, EvalResult, EvalSignal, LxError, LxResult};
pub use event_stream::{entry_to_lxval, EventStream, SpanInfo, StreamEntry};
pub use value::*;
```

### Step 3: Define the ExternalStreamSink trait

Create `crates/lx-value/src/external_sink.rs` and add `mod external_sink;` plus `pub use external_sink::ExternalStreamSink;` to `lib.rs`.

Contents of `external_sink.rs`:

```rust
use std::sync::Arc;

pub trait ExternalStreamSink: Send + Sync {
    fn xadd(&self, entry_json: serde_json::Value);
    fn shutdown(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;
}
```

### Step 4: Define the BuiltinCtx trait

Create `crates/lx-value/src/builtin_ctx.rs` and add `mod builtin_ctx;` plus `pub use builtin_ctx::BuiltinCtx;` to `lib.rs`.

The trait must expose every `RuntimeCtx` field that builtins actually access. Reading all builtins, they access:

- `ctx.event_stream` (used by `log_at` in `register.rs`, `bi_tool_dispatch` in `apply_helpers.rs`)
- `ctx.source_dir` (used by `bi_source_dir` in `register_helpers.rs`)
- `ctx.network_denied` (used by `http.rs` stdlib)
- `ctx.test_threshold` (used by `test_mod/`)
- `ctx.test_runs` (used by `test_mod/`)
- `ctx.workspace_members` (used by `modules.rs` via `self.ctx.workspace_members`)
- `ctx.dep_dirs` (used by `modules.rs` via `self.ctx.dep_dirs`)
- `ctx.tokio_runtime` (used by `call_value_sync`)
- `ctx.yield_` (used by interpreter `eval` for `Expr::Yield`)
- `ctx.global_pause` / `ctx.cancel_flag` (used by interpreter `check_control_flags`)
- `ctx.inject_tx` (used by control channel)

Since the builtins themselves only use `event_stream` and `source_dir` directly, and all other fields are accessed by the interpreter (which stays in lx-eval and has the concrete `RuntimeCtx`), the `BuiltinCtx` trait only needs methods the builtin function signatures require. The actual type parameter in `SyncBuiltinFn`, `AsyncBuiltinFn`, `DynAsyncBuiltinFn` changes from `Arc<RuntimeCtx>` to `Arc<dyn BuiltinCtx>`.

Contents of `builtin_ctx.rs`:

```rust
use std::path::PathBuf;
use std::sync::Arc;

use crate::EventStream;

pub trait BuiltinCtx: Send + Sync {
    fn event_stream(&self) -> &Arc<EventStream>;
    fn source_dir(&self) -> Option<PathBuf>;
    fn network_denied(&self) -> bool;
    fn test_threshold(&self) -> Option<f64>;
    fn test_runs(&self) -> Option<u32>;
}
```

### Step 5: Define the ToolModuleHandle trait

Create `crates/lx-value/src/tool_module_handle.rs` and add `mod tool_module_handle;` plus `pub use tool_module_handle::ToolModuleHandle;` to `lib.rs`.

Reading `tool_module.rs`, the `ToolModule` is used via:
- `LxVal::ToolModule(Arc<ToolModule>)` in value enum
- `tm.call_tool(method, arg, &ctx.event_stream, agent_name)` in `apply_helpers.rs` `bi_tool_dispatch`
- `tm.shutdown()` in `interpreter/mod.rs` `exec()`

Contents of `tool_module_handle.rs`:

```rust
use std::future::Future;
use std::pin::Pin;

use crate::error::LxError;
use crate::event_stream::EventStream;
use crate::LxVal;

pub trait ToolModuleHandle: std::fmt::Debug + Send + Sync {
    fn call_tool(
        &self,
        method: &str,
        args: LxVal,
        event_stream: &EventStream,
        agent_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>> + '_>>;

    fn shutdown(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>;

    fn command(&self) -> &str;
    fn alias(&self) -> &str;
}
```

### Step 6: Move error.rs

Move `crates/lx/src/error.rs` to `crates/lx-value/src/error.rs`.

Rewrite imports:

| Old | New |
|-----|-----|
| `use crate::value::LxVal;` | `use crate::value::LxVal;` (stays same, now within lx-value) |

The `LxError::Propagate` variant references `crate::value::LxVal` — this becomes `crate::value::LxVal` within lx-value, no change needed.

### Step 7: Move env.rs

Move `crates/lx/src/env.rs` to `crates/lx-value/src/env.rs`.

Rewrite imports:

| Old | New |
|-----|-----|
| `use crate::sym::{Sym, intern};` | `use lx_span::sym::{Sym, intern};` |
| `use crate::value::LxVal;` | `use crate::value::LxVal;` |

### Step 8: Move event_stream.rs

Move `crates/lx/src/event_stream.rs` to `crates/lx-value/src/event_stream.rs`.

Rewrite imports and replace `McpClient` with `ExternalStreamSink`:

| Old | New |
|-----|-----|
| `use crate::mcp_client::McpClient;` | REMOVE |
| `use crate::sym::{Sym, intern};` | `use lx_span::sym::{Sym, intern};` |
| `use crate::value::LxVal;` | `use crate::value::LxVal;` |

Change the `external_client` field in `EventStream`:

Old:
```rust
external_client: Mutex<Option<Arc<tokio::sync::Mutex<McpClient>>>>,
```

New:
```rust
external_client: Mutex<Option<Arc<dyn crate::ExternalStreamSink>>>,
```

Change `set_external_client`:

Old:
```rust
pub fn set_external_client(&self, client: Arc<tokio::sync::Mutex<McpClient>>) {
    *self.external_client.lock() = Some(client);
}
```

New:
```rust
pub fn set_external_client(&self, client: Arc<dyn crate::ExternalStreamSink>) {
    *self.external_client.lock() = Some(client);
}
```

Change the external client usage in `xadd()`:

Old (lines 118-125):
```rust
if let Some(client) = self.external_client.lock().clone() {
    let entry_json = serde_json::to_value(&entry).unwrap_or(serde_json::Value::Null);
    tokio::task::spawn(async move {
        if let Err(e) = client.lock().await.tools_call("xadd", entry_json).await {
            eprintln!("[stream:external] xadd failed: {e}");
        }
    });
}
```

New:
```rust
if let Some(client) = self.external_client.lock().clone() {
    let entry_json = serde_json::to_value(&entry).unwrap_or(serde_json::Value::Null);
    tokio::task::spawn(async move {
        client.xadd(entry_json);
    });
}
```

Change `shutdown_external()`:

Old:
```rust
pub async fn shutdown_external(&self) {
    let client = self.external_client.lock().take();
    if let Some(client) = client {
        client.lock().await.shutdown().await;
    }
}
```

New:
```rust
pub async fn shutdown_external(&self) {
    let client = self.external_client.lock().take();
    if let Some(client) = client {
        client.shutdown().await;
    }
}
```

### Step 9: Move value/ directory

Move the entire `crates/lx/src/value/` directory to `crates/lx-value/src/value/`.

Files moved:
- `value/mod.rs`
- `value/display.rs`
- `value/func.rs`
- `value/impls.rs`
- `value/methods.rs`
- `value/serde_impl.rs`

#### Rewrite value/mod.rs imports

| Old | New |
|-----|-----|
| `use crate::ast::{AstArena, ExprId, Field, MethodSpec};` | `use lx_ast::ast::{AstArena, ExprId, Field, MethodSpec};` |
| `use crate::error::LxError;` | `use crate::error::LxError;` |
| `use crate::sym::Sym;` | `use lx_span::sym::Sym;` |

Change the `ToolModule` variant:

Old (line 120):
```rust
ToolModule(Arc<crate::tool_module::ToolModule>),
```

New:
```rust
ToolModule(Arc<dyn crate::ToolModuleHandle>),
```

#### Rewrite value/func.rs imports

| Old | New |
|-----|-----|
| `use crate::ast::{AstArena, ExprId};` | `use lx_ast::ast::{AstArena, ExprId};` |
| `use crate::env::Env;` | `use crate::env::Env;` |
| `use crate::error::LxError;` | `use crate::error::LxError;` |
| `use crate::runtime::RuntimeCtx;` | REMOVE |
| `use crate::sym::Sym;` | `use lx_span::sym::Sym;` |
| `use crate::value::LxVal;` | `use crate::value::LxVal;` |

Change builtin fn type aliases:

Old:
```rust
pub type SyncBuiltinFn = fn(&[LxVal], SourceSpan, &Arc<RuntimeCtx>) -> Result<LxVal, LxError>;
pub type AsyncBuiltinFn = fn(Vec<LxVal>, SourceSpan, Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>>;
pub type DynAsyncBuiltinFn = Arc<dyn Fn(Vec<LxVal>, SourceSpan, Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> + Send + Sync>;
```

New:
```rust
pub type SyncBuiltinFn = fn(&[LxVal], SourceSpan, &Arc<dyn crate::BuiltinCtx>) -> Result<LxVal, LxError>;
pub type AsyncBuiltinFn = fn(Vec<LxVal>, SourceSpan, Arc<dyn crate::BuiltinCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>>;
pub type DynAsyncBuiltinFn = Arc<dyn Fn(Vec<LxVal>, SourceSpan, Arc<dyn crate::BuiltinCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> + Send + Sync>;
```

#### Rewrite value/impls.rs: fix record! macro

The `record!` macro (line 7-13 of impls.rs) uses `$crate::sym::intern($key)`. When this file lives in lx-value, `$crate` resolves to `lx_value`, but `sym` is in `lx_span`. Change:

```rust
$crate::sym::intern($key)
```

to:

```rust
lx_span::sym::intern($key)
```

Also change `$crate::value::LxVal::Record` to `$crate::LxVal::Record` (since `value` is an internal module of lx-value, not re-exported as a path).

#### Rewrite value/mod.rs: inline crate::sym::intern paths

The `typed_field_methods!` macro (line 147) expands `crate::sym::intern(key)` -- change to `lx_span::sym::intern(key)`.

The following methods have inline `crate::sym::intern(...)` calls that must become `lx_span::sym::intern(...)`:

| Method | Line | Call |
|--------|------|------|
| `typ()` | 187 | `crate::sym::intern(name)` |
| `list_field()` | 265 | `crate::sym::intern(key)` |
| `record_field()` | 272 | `crate::sym::intern(key)` |
| `get_field()` | 282 | `crate::sym::intern(key)` |

#### Rewrite value/methods.rs, value/display.rs, value/serde_impl.rs

These files reference `crate::sym::intern` — change to `lx_span::sym::intern`.
Any reference to `crate::sym::Sym` changes to `lx_span::sym::Sym`.

### Step 10: Move ModuleExports into lx-value

Create `crates/lx-value/src/module_exports.rs`:

```rust
use indexmap::IndexMap;
use lx_span::sym::Sym;
use crate::LxVal;

#[derive(Debug, Clone)]
pub struct ModuleExports {
    pub bindings: IndexMap<Sym, LxVal>,
    pub variant_ctors: Vec<Sym>,
}
```

Add `mod module_exports;` and `pub use module_exports::ModuleExports;` to `lib.rs`.

Remove the `ModuleExports` struct from `crates/lx/src/interpreter/mod.rs` (lines 38-42).

### Step 11: Add lx-value to workspace and lx crate

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-value"` to `workspace.members`.

In `crates/lx/Cargo.toml`, add:

```toml
lx-value = { path = "../lx-value" }
```

### Step 12: Re-export shims in lx crate

Replace `crates/lx/src/error.rs` with:

```rust
pub use lx_value::error::*;
```

Replace `crates/lx/src/env.rs` with:

```rust
pub use lx_value::Env;
```

Replace `crates/lx/src/event_stream.rs` with:

```rust
pub use lx_value::*;
```

Note: `event_stream.rs` re-export needs to be selective. Better:

```rust
pub use lx_value::{entry_to_lxval, EventStream, SpanInfo, StreamEntry, ExternalStreamSink};
```

Replace `crates/lx/src/value/` directory — delete all 6 files, create `crates/lx/src/value.rs`:

```rust
pub use lx_value::*;
```

Update `crates/lx/src/lib.rs`: the `pub mod value;` line now resolves to the `value.rs` shim file. Same for `pub mod error;`, `pub mod env;`, `pub mod event_stream;`.

Also add to `lib.rs`:

```rust
pub use lx_value::{BuiltinCtx, ExternalStreamSink, ModuleExports, ToolModuleHandle};
```

### Step 13: Update interpreter/mod.rs

Remove the `ModuleExports` struct definition. Add import:

```rust
use crate::value::ModuleExports;
```

This works because the lx crate re-exports it. Also update stdlib/mod.rs: change `use crate::interpreter::ModuleExports` to `use crate::value::ModuleExports` (the re-export path).

## Files touched

| Action | File |
|--------|------|
| CREATE | `crates/lx-value/Cargo.toml` |
| CREATE | `crates/lx-value/src/lib.rs` |
| CREATE | `crates/lx-value/src/external_sink.rs` |
| CREATE | `crates/lx-value/src/builtin_ctx.rs` |
| CREATE | `crates/lx-value/src/tool_module_handle.rs` |
| CREATE | `crates/lx-value/src/module_exports.rs` |
| MOVE+EDIT | `crates/lx/src/error.rs` -> `crates/lx-value/src/error.rs` |
| MOVE+EDIT | `crates/lx/src/env.rs` -> `crates/lx-value/src/env.rs` |
| MOVE+EDIT | `crates/lx/src/event_stream.rs` -> `crates/lx-value/src/event_stream.rs` |
| MOVE+EDIT | `crates/lx/src/value/*.rs` (6 files) -> `crates/lx-value/src/value/*.rs` |
| CREATE | `crates/lx/src/error.rs` (re-export shim) |
| CREATE | `crates/lx/src/env.rs` (re-export shim) |
| CREATE | `crates/lx/src/event_stream.rs` (re-export shim) |
| CREATE | `crates/lx/src/value.rs` (re-export shim, replaces directory) |
| EDIT | `crates/lx/src/lib.rs` (add re-exports) |
| EDIT | `crates/lx/src/interpreter/mod.rs` (remove ModuleExports, update import) |
| EDIT | `crates/lx/src/stdlib/mod.rs` (update ModuleExports import) |
| EDIT | `crates/lx/Cargo.toml` (add lx-value dep) |
| EDIT | `/home/entropybender/repos/lx/Cargo.toml` (add workspace member) |

## Verification

1. `just diagnose` passes
2. `lx::value::LxVal` is accessible
3. `lx::error::LxError` is accessible
4. `lx::env::Env` is accessible
5. `lx::event_stream::EventStream` is accessible
6. `lx_value::BuiltinCtx` trait exists with methods: `event_stream`, `source_dir`, `network_denied`, `test_threshold`, `test_runs`
7. `lx_value::ExternalStreamSink` trait exists with methods: `xadd`, `shutdown`
8. `lx_value::ToolModuleHandle` trait exists with methods: `call_tool`, `shutdown`, `command`, `alias`
9. `SyncBuiltinFn` signature uses `Arc<dyn BuiltinCtx>` not `Arc<RuntimeCtx>`
10. `LxVal::ToolModule` holds `Arc<dyn ToolModuleHandle>` not `Arc<ToolModule>`
11. lx-value has NO dependency on lx-eval, lx (no cycle)
12. No file exceeds 300 lines
