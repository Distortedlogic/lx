# Work Item 11: External Stream Backend

Optional MCP server for cross-process event stream access. Configured in `lx.toml` manifest. When configured, the interpreter writes to both the in-memory event stream AND the external backend.

## Prerequisites

- **tool_module_dispatch** must be complete -- provides `McpClient` at `crates/lx/src/mcp_client.rs`, tool module dispatch path exists
- **unit_4_event_stream** must be complete -- provides `EventStream` trait, `StreamEntry`, `IdGenerator` in `crates/lx/src/runtime/event_stream.rs`
- **unit_5_stream_module** must be complete -- provides `RuntimeCtx.event_stream` (`parking_lot::Mutex<Option<Arc<dyn EventStream>>>`), `RuntimeCtx.id_gen`
- **jsonl_persistence** must be complete -- JSONL persistence initializes on interpreter creation

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify

## Current State

- `RootManifest` in `crates/lx-cli/src/manifest.rs` (line 8) has `backends: Option<BackendsSection>` (line 13)
- `BackendsSection` in `crates/lx-cli/src/manifest.rs` (lines 93-100) has fields for llm, http, emit, yield, log
- `apply_manifest_backends` in `crates/lx-cli/src/main.rs` (lines 200-238) reads the manifest and configures `RuntimeCtx` backends
- `McpClient` in `crates/lx/src/mcp_client.rs` provides `spawn(command) -> Result<Self, String>` and `tools_call(tool_name, arguments) -> Result<serde_json::Value, String>`
- `RuntimeCtx.event_stream` is `parking_lot::Mutex<Option<Arc<dyn EventStream>>>` (set to `JsonlBackend` by jsonl_persistence)
- `RuntimeCtx.xadd(entry)` calls `self.event_stream.lock().as_ref().and_then(|s| s.xadd(entry).ok())`
- `EventStream` trait in `crates/lx/src/runtime/event_stream.rs` defines `xadd`, `xrange`, `xread`, `xlen`, `xtrim`
- `StreamEntry` has `Serialize` + `Deserialize` derives
- `Interpreter.tool_modules` is `Vec<Arc<ToolModule>>` (added by tool_module_dispatch) -- tool modules are shut down at end of `exec`

## Architecture

The external stream backend wraps the existing in-memory/JSONL event stream with an additional MCP-based backend. It implements `EventStream` by:
1. Delegating all operations to the inner (JSONL) backend first
2. Then forwarding `xadd` calls to the external MCP server

The MCP server speaks JSON-RPC and handles tool methods named `xadd`, `xrange`, `xread`, `xlen`, `xtrim`. The interpreter's MCP client forwards each call.

The external backend is additive -- if it is slow or crashes, the inner event stream continues working. `xadd` to the external backend is fire-and-forget: failures are logged to stderr but do not propagate to the caller.

Configuration in `lx.toml`:
```toml
[stream]
command = "valkey-stream-mcp"
```

## Files to Create

- `crates/lx/src/runtime/external_stream.rs` -- `ExternalStreamBackend` implementing `EventStream`, wrapping inner + MCP client

## Files to Modify

- `crates/lx/src/runtime/mod.rs` -- add `mod external_stream;`, re-export `ExternalStreamBackend`
- `crates/lx-cli/src/manifest.rs` -- add `StreamSection` and `stream` field to `RootManifest`
- `crates/lx-cli/src/main.rs` -- read `[stream]` config in `apply_manifest_backends`, spawn MCP client, wrap event stream

## Step 1: Add StreamSection to manifest

File: `crates/lx-cli/src/manifest.rs`

Add after `BackendsSection` (after line 100):

```rust
#[derive(Deserialize)]
pub struct StreamSection {
    pub command: String,
}
```

Add a field to `RootManifest` (after `backends` on line 13):

```rust
pub stream: Option<StreamSection>,
```

## Step 2: Create ExternalStreamBackend

File: `crates/lx/src/runtime/external_stream.rs`

```rust
use std::sync::Arc;

use super::event_stream::{EventStream, StreamEntry};

pub struct ExternalStreamBackend {
    inner: Arc<dyn EventStream>,
    client: Arc<tokio::sync::Mutex<crate::mcp_client::McpClient>>,
}

impl ExternalStreamBackend {
    pub fn new(
        inner: Arc<dyn EventStream>,
        client: Arc<tokio::sync::Mutex<crate::mcp_client::McpClient>>,
    ) -> Self {
        Self { inner, client }
    }
}

impl EventStream for ExternalStreamBackend {
    fn xadd(&self, entry: StreamEntry) -> Result<String, String> {
        let id = self.inner.xadd(entry.clone())?;

        let entry_json = serde_json::to_value(&entry)
            .unwrap_or(serde_json::Value::Null);

        let client = Arc::clone(&self.client);
        tokio::task::spawn(async move {
            if let Err(e) = client.lock().await.tools_call("xadd", entry_json).await {
                eprintln!("[stream:external] xadd failed: {e}");
            }
        });

        Ok(id)
    }

    fn xrange(
        &self,
        start: &str,
        end: &str,
        count: Option<usize>,
    ) -> Result<Vec<StreamEntry>, String> {
        self.inner.xrange(start, end, count)
    }

    fn xread(
        &self,
        last_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<Option<StreamEntry>, String> {
        self.inner.xread(last_id, timeout_ms)
    }

    fn xlen(&self) -> Result<usize, String> {
        self.inner.xlen()
    }

    fn xtrim(&self, maxlen: usize) -> Result<usize, String> {
        self.inner.xtrim(maxlen)
    }
}
```

The `xadd` method writes to the inner backend synchronously (preserving the in-memory + JSONL guarantees), then spawns a tokio task to forward to the external MCP server asynchronously. The external forward is fire-and-forget -- failures print to stderr but do not fail the `xadd` call.

The `xrange`, `xread`, `xlen`, `xtrim` methods delegate to the inner backend only. Cross-process consumers that need to read the stream connect to the external MCP server directly -- the external server maintains its own copy of the data.

## Step 3: Register module in runtime/mod.rs

File: `crates/lx/src/runtime/mod.rs`

Add module declaration after the existing ones:

```rust
mod external_stream;
```

Add re-export after the existing ones:

```rust
pub use external_stream::ExternalStreamBackend;
```

## Step 4: Wire external backend in CLI

File: `crates/lx-cli/src/main.rs`

In the `apply_manifest_backends` function (lines 200-238), add handling for the `[stream]` section. This must run after JSONL persistence is initialized (jsonl_persistence handles that in `Interpreter::new`), so the external backend wraps the already-initialized JSONL backend.

The external stream setup must be async because `McpClient::spawn` is async. Place it inside the existing `ctx.tokio_runtime.block_on` in `run_file`, before `run::run` is called. This avoids a nested `block_on` deadlock.

In `run_file` (lines 166-198 of `crates/lx-cli/src/main.rs`), add an async helper and call it inside the existing `block_on` block, after `let ctx = Arc::new(ctx_val);` (line 180) and before `run::run`:

```rust
async fn setup_external_stream(ctx: &Arc<RuntimeCtx>, file_path: &str) {
    let file_dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
    let Some(root) = manifest::find_manifest_root(file_dir) else {
        return;
    };
    let Ok(m) = manifest::load_manifest(&root) else {
        return;
    };
    let Some(stream_config) = m.stream else {
        return;
    };

    let command = stream_config.command;
    let lx_dir = file_dir.join(".lx");
    let _ = std::fs::create_dir_all(&lx_dir);
    let jsonl_path = lx_dir.join("stream.jsonl");
    let Ok(jsonl_backend) = lx::runtime::JsonlBackend::new(
        &jsonl_path.to_string_lossy(),
    ) else {
        return;
    };
    let inner: Arc<dyn lx::runtime::EventStream> = Arc::new(jsonl_backend);

    match lx::mcp_client::McpClient::spawn(&command).await {
        Ok(client) => {
            let client_arc = Arc::new(tokio::sync::Mutex::new(client));
            let external = lx::runtime::ExternalStreamBackend::new(
                inner, client_arc,
            );
            *ctx.event_stream.lock() = Some(Arc::new(external));
        },
        Err(e) => {
            eprintln!("[stream:external] failed to connect to '{command}': {e}");
            *ctx.event_stream.lock() = Some(inner);
        },
    }
}
```

This sets up the external backend before the interpreter runs. If external connection fails, the JSONL-only backend is used as a fallback. The `Interpreter::new` call (inside `run::run`) checks `ctx.event_stream.lock().is_none()` (from jsonl_persistence) -- since we set it here, it skips its own JSONL initialization.

## Step 5: Shutdown external stream MCP client

The external stream's MCP client is held inside the `ExternalStreamBackend` struct, which is stored in `RuntimeCtx.event_stream`. When `RuntimeCtx` is dropped at the end of `run_file`, the `Arc<dyn EventStream>` holding `ExternalStreamBackend` is dropped, which drops the `Arc<tokio::sync::Mutex<McpClient>>`. When the last `Arc` reference is dropped, `McpClient`'s `Drop` impl closes stdin, which signals the child process to exit. No additional shutdown code needed beyond `McpClient`'s existing drop behavior.

## Step 6: Verify file lengths

- `external_stream.rs`: ~55 lines. Under 300.
- `manifest.rs`: gains ~5 lines (struct + field). Under 300.
- `main.rs`: gains ~30 lines for external stream setup. Current is ~258 lines. Under 300.

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. All existing tests pass unchanged (no `[stream]` section in test manifests means no external backend is spawned)
3. Manual test: create a `lx.toml` with `[stream]\ncommand = "echo"` -- the interpreter attempts to spawn "echo" as an MCP server. It fails the MCP handshake and prints `[stream:external] failed to connect to 'echo': ...`. The program runs normally with JSONL-only persistence. This confirms the error path and graceful fallback.
4. Integration test (requires an actual MCP stream server): create a mock MCP server that accepts `xadd` tool calls. Configure it in `lx.toml`. Run a program with `emit "hello"`. Verify the mock server received an `xadd` call with the entry data.
