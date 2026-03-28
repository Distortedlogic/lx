# Work Item: RuntimeCtx Backend Refactor

## Goal

Remove `EmitBackend`, `LogBackend`, `LlmBackend`, and `HttpBackend` traits and all their implementations. Replace their functionality with the event stream (for emit/log) and tool modules (for LLM/HTTP). Keep `YieldBackend`. Update all callsites.

## Preconditions

- **Work item `event_stream_core` is complete:** `EventStream` exists with `xadd` method. `RuntimeCtx` has `event_stream: Arc<EventStream>` field.
- **Work item `tool_module_dispatch` is complete:** `use tool "..." as Name` works at runtime, tool modules dispatch via MCP.
- `crates/lx/src/runtime/mod.rs` currently has: `EmitBackend`, `HttpBackend`, `YieldBackend`, `LogBackend`, `LlmBackend` traits and `RuntimeCtx` struct with `emit`, `http`, `yield_`, `log`, `llm` fields.
- `crates/lx/src/runtime/defaults.rs` has: `StdoutEmitBackend`, `ReqwestHttpBackend`, `StdinStdoutYieldBackend`, `StderrLogBackend`.
- `crates/lx/src/runtime/noop.rs` has: `NoopEmitBackend`, `NoopLogBackend`, `NoopLlmBackend`.
- `crates/lx/src/runtime/restricted.rs` has: `DenyHttpBackend`.

## Callsites to Update

These are every location that uses the backend traits being removed. Each must be rewritten.

### `ctx.emit.emit()`

| File | Line | Current Code | Replacement |
|------|------|-------------|-------------|
| `crates/lx/src/interpreter/mod.rs` | 197 | `self.ctx.emit.emit(&v, span)?;` | `self.ctx.event_stream.xadd("runtime/emit", "main", None, fields);` where `fields` contains `"value" => v.clone()` |
| `crates/lx/src/stdlib/test_mod/test_report.rs` | 34 | `ctx.emit.emit(&LxVal::str(out), span)?;` | `ctx.event_stream.xadd("runtime/emit", "main", None, fields);` where `fields` contains `"value" => LxVal::str(out)`. Also print the value to stdout since test reports need visible output: add `println!("{}", out);` before the xadd. |

### `ctx.log.log()`

| File | Line | Current Code | Replacement |
|------|------|-------------|-------------|
| `crates/lx/src/builtins/register.rs` | 18 | `ctx.log.log(level, s);` | `ctx.event_stream.xadd("runtime/log", "main", None, fields);` where `fields` contains `"level" => LxVal::str(level_str)`, `"msg" => LxVal::str(s)`. Also print to stderr for visibility: `eprintln!("[{level_str}] {s}");` |

### `ctx.llm.prompt()` / `ctx.llm.prompt_with()`

| File | Line | Current Code | Replacement |
|------|------|-------------|-------------|
| `crates/lx/src/builtins/llm.rs` | 11 | `ctx.llm.prompt(text, span)` | Return `Err(LxError::runtime("llm backend removed: use `use tool \"claude-mcp\" as llm` instead", span))` |
| `crates/lx/src/builtins/llm.rs` | 26 | `ctx.llm.prompt_with(&opts, span)` | Same error |
| `crates/lx/src/builtins/llm.rs` | 33 | `ctx.llm.prompt_with(&opts, span)` | Same error |

### `ctx.http.request()`

| File | Line | Current Code | Replacement |
|------|------|-------------|-------------|
| `crates/lx/src/stdlib/http.rs` | 24 | `ctx.http.request("GET", url, &HttpOpts::default(), span)` | Keep the reqwest implementation inline — HTTP is used by stdlib and removing it entirely would break `use std/http`. The `HttpBackend` trait abstraction is removed but the actual reqwest HTTP code moves into the `std/http` module directly. |
| `crates/lx/src/stdlib/http.rs` | 31 | `ctx.http.request("POST", url, &opts, span)` | Same |
| `crates/lx/src/stdlib/http.rs` | 38 | `ctx.http.request("PUT", url, &opts, span)` | Same |
| `crates/lx/src/stdlib/http.rs` | 43 | `ctx.http.request("DELETE", url, &HttpOpts::default(), span)` | Same |
| `crates/lx/src/stdlib/http.rs` | 54 | `ctx.http.request(method, url, &opts, span)` | Same |

## Files to Modify

### 1. `crates/lx/src/runtime/mod.rs`

**Remove traits:** Delete `EmitBackend`, `HttpBackend`, `LogBackend`, `LlmBackend` trait definitions (lines 48-83). Delete `HttpOpts` struct (lines 42-46). Delete `LogLevel` enum (lines 60-66). Delete `LlmOpts` struct (lines 72-78).

**Keep:** `YieldBackend` trait (lines 56-58).

**Update `RuntimeCtx`:** Remove fields `emit`, `http`, `log`, `llm` and their `SmartDefault` annotations. Keep `yield_`, `source_dir`, `workspace_members`, `dep_dirs`, `tokio_runtime`, `test_threshold`, `test_runs`, `event_stream`.

New `RuntimeCtx`:

```rust
#[derive(SmartDefault)]
pub struct RuntimeCtx {
    #[default(Arc::new(StdinStdoutYieldBackend))]
    pub yield_: Arc<dyn YieldBackend>,
    pub source_dir: parking_lot::Mutex<Option<PathBuf>>,
    pub workspace_members: HashMap<String, PathBuf>,
    pub dep_dirs: HashMap<String, PathBuf>,
    #[default(Arc::new(tokio::runtime::Runtime::new().expect("failed to create tokio runtime")))]
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,
    pub test_threshold: Option<f64>,
    pub test_runs: Option<u32>,
    #[default(Arc::new(crate::event_stream::EventStream::new(None)))]
    pub event_stream: Arc<crate::event_stream::EventStream>,
    #[default(false)]
    pub network_denied: bool,
}
```

**Update imports:** Remove `use indexmap::IndexMap;` if no longer needed (it was for `HttpOpts`). Remove `use crate::value::LxVal;` if no longer needed.

**Update module declarations:** Remove `mod defaults;`, `mod noop;`, `mod restricted;` and their `pub use` re-exports, but only the parts related to removed traits. `StdinStdoutYieldBackend` from `defaults.rs` must survive.

### 2. `crates/lx/src/runtime/defaults.rs`

**Remove:** `StdoutEmitBackend` and its `EmitBackend` impl (lines 14-21). `ReqwestHttpBackend` and its `HttpBackend` impl (lines 23-58). `response_to_value` helper (lines 60-74). `StderrLogBackend` and its `LogBackend` impl (lines 95-107).

**Keep:** `StdinStdoutYieldBackend` and its `YieldBackend` impl (lines 76-93).

**Update imports:** Remove `use super::{EmitBackend, HttpBackend, HttpOpts, LogBackend, LogLevel, YieldBackend};` — replace with `use super::YieldBackend;`. Remove `use indexmap::IndexMap;`, `use reqwest::Client;`, `use reqwest::header::CONTENT_TYPE;`, `use crate::record;` if no longer needed.

### 3. `crates/lx/src/runtime/noop.rs`

**Delete this entire file.** All three types (`NoopEmitBackend`, `NoopLogBackend`, `NoopLlmBackend`) are for removed traits.

**Remove** `mod noop;` and `pub use noop::*;` from `crates/lx/src/runtime/mod.rs`.

### 4. `crates/lx/src/runtime/restricted.rs`

**Delete this entire file.** `DenyHttpBackend` is for the removed `HttpBackend` trait.

**Remove** `mod restricted;` and `pub use restricted::*;` from `crates/lx/src/runtime/mod.rs`.

### 5. `crates/lx/src/interpreter/mod.rs`

**Update `Expr::Emit` handling** (line 195-199):

Replace:
```rust
Expr::Emit(ExprEmit { value }) => {
    let v = self.eval(value).await?;
    self.ctx.emit.emit(&v, span)?;
    Ok(LxVal::Unit)
},
```

With:
```rust
Expr::Emit(ExprEmit { value }) => {
    let v = self.eval(value).await?;
    println!("{v}");
    let mut fields = indexmap::IndexMap::new();
    fields.insert(crate::sym::intern("value"), v);
    self.ctx.event_stream.xadd("runtime/emit", "main", None, fields);
    Ok(LxVal::Unit)
},
```

The `println!` preserves the current default behavior (emit prints to stdout). The event stream entry provides observability.

### 6. `crates/lx/src/builtins/register.rs`

**Rewrite `make_log_builtin`** (lines 15-27). Remove the `LogLevel` dependency. Replace with direct event stream writes + stderr output:

```rust
fn make_log_builtin(name: &'static str, level_str: &'static str) -> LxVal {
    mk(name, 1, move |args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>| {
        let s = args[0].require_str(&format!("log.{level_str}"), span)?;
        eprintln!("[{}] {}", level_str.to_uppercase(), s);
        let mut fields = indexmap::IndexMap::new();
        fields.insert(crate::sym::intern("level"), LxVal::str(level_str));
        fields.insert(crate::sym::intern("msg"), LxVal::str(s));
        ctx.event_stream.xadd("runtime/log", "main", None, fields);
        Ok(LxVal::Unit)
    })
}
```

**Update the log registration** (lines 145-150):

Replace:
```rust
let mut log_fields = IndexMap::new();
log_fields.insert(crate::sym::intern("info"), make_log_builtin("log.info", LogLevel::Info));
log_fields.insert(crate::sym::intern("warn"), make_log_builtin("log.warn", LogLevel::Warn));
log_fields.insert(crate::sym::intern("err"), make_log_builtin("log.err", LogLevel::Err));
log_fields.insert(crate::sym::intern("debug"), make_log_builtin("log.debug", LogLevel::Debug));
env.bind_str("log", LxVal::record(log_fields));
```

With:
```rust
let mut log_fields = IndexMap::new();
log_fields.insert(crate::sym::intern("info"), make_log_builtin("log.info", "info"));
log_fields.insert(crate::sym::intern("warn"), make_log_builtin("log.warn", "warn"));
log_fields.insert(crate::sym::intern("err"), make_log_builtin("log.err", "err"));
log_fields.insert(crate::sym::intern("debug"), make_log_builtin("log.debug", "debug"));
env.bind_str("log", LxVal::record(log_fields));
```

**Remove `use crate::runtime::{LogLevel, RuntimeCtx};`** — replace with `use crate::runtime::RuntimeCtx;`.

### 7. `crates/lx/src/builtins/llm.rs`

**Rewrite all three functions** to return errors directing users to `use tool`:

```rust
use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn bi_prompt(args: &[LxVal], span: SourceSpan, _ctx: &std::sync::Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError> {
    Err(LxError::runtime("llm.prompt removed: use `use tool \"claude-mcp\" as llm` and call llm.prompt instead", span))
}

pub fn bi_prompt_with(args: &[LxVal], span: SourceSpan, _ctx: &std::sync::Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError> {
    Err(LxError::runtime("llm.prompt_with removed: use `use tool \"claude-mcp\" as llm` instead", span))
}

pub fn bi_prompt_structured(args: &[LxVal], span: SourceSpan, _ctx: &std::sync::Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError> {
    Err(LxError::runtime("llm.prompt_structured removed: use `use tool \"claude-mcp\" as llm` instead", span))
}
```

Remove all unused imports from this file.

### 8. `crates/lx/src/stdlib/http.rs`

**Inline the reqwest HTTP logic** instead of going through `ctx.http`. The HTTP module keeps working — it just calls reqwest directly instead of through a trait.

**Remove** the import `use crate::runtime::{HttpOpts, RuntimeCtx};` and replace with `use crate::runtime::RuntimeCtx;`.

**Add** a local `HttpOpts` struct and `do_request` helper and `response_to_value` async fn at the top of the file (after imports):

```rust
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;

#[derive(Debug, Clone, Default)]
struct HttpOpts {
    headers: Option<IndexMap<String, String>>,
    query: Option<IndexMap<String, String>>,
    body: Option<serde_json::Value>,
}

fn do_request(method: &str, url: &str, opts: &HttpOpts, span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    if ctx.network_denied {
        return Ok(LxVal::err_str("network access denied by sandbox policy"));
    }
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let c = Client::builder().build().map_err(|e| LxError::runtime(format!("http: client: {e}"), span))?;
            let mut builder = match method {
                "GET" => c.get(url),
                "POST" => c.post(url),
                "PUT" => c.put(url),
                "DELETE" => c.delete(url),
                _ => {
                    return Err(LxError::runtime(format!("http: unknown method '{method}'"), span));
                },
            };
            if let Some(ref hdrs) = opts.headers {
                for (k, v) in hdrs {
                    builder = builder.header(k.as_str(), v.as_str());
                }
            }
            if let Some(ref query) = opts.query {
                let pairs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                builder = builder.query(&pairs);
            }
            if let Some(ref body) = opts.body {
                builder = builder.header(CONTENT_TYPE, "application/json").json(body);
            }
            match builder.send().await {
                Ok(resp) => response_to_value(resp, span).await,
                Err(e) => Ok(LxVal::err_str(e.to_string())),
            }
        })
    })
}

async fn response_to_value(resp: reqwest::Response, span: SourceSpan) -> Result<LxVal, LxError> {
    let status = resp.status().as_u16();
    let mut headers = IndexMap::new();
    for (name, value) in resp.headers() {
        let v = value.to_str().unwrap_or("").to_string();
        headers.insert(crate::sym::intern(name.as_str()), LxVal::str(v));
    }
    let body_str = resp.text().await.map_err(|e| LxError::runtime(format!("http: body: {e}"), span))?;
    let body = if let Ok(jv) = serde_json::from_str::<serde_json::Value>(&body_str) { LxVal::from(jv) } else { LxVal::str(body_str) };
    Ok(LxVal::ok(crate::record! {
        "status" => LxVal::int(status),
        "body" => body,
        "headers" => LxVal::record(headers),
    }))
}
```

**Replace** each function body. Example for `bi_get` (line 22-25):

Replace:
```rust
fn bi_get(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.get", span)?;
  ctx.http.request("GET", url, &HttpOpts::default(), span)
}
```

With:
```rust
fn bi_get(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let url = args[0].require_str("http.get", span)?;
    do_request("GET", url, &HttpOpts::default(), span, ctx)
}
```

Apply the same pattern to `bi_post` (line 27-31), `bi_put` (line 34-38), `bi_delete` (line 41-43), `bi_request` (line 46-54) — each replaces `ctx.http.request(method, url, &opts, span)` with `do_request(method, url, &opts, span, ctx)`.

### 9. `crates/lx/src/stdlib/sandbox/sandbox_scope.rs`

**Remove `DenyHttpBackend` usage.** The sandbox already constructs a new `RuntimeCtx` manually at lines 19-31 of `sandbox_scope.rs`, copying each field from the base context. This means a plain `bool` field works — the sandbox creates its own `RuntimeCtx` instance, so no `AtomicBool` is needed.

**Update imports** (line 6): Replace `use crate::runtime::{DenyHttpBackend, HttpBackend, RuntimeCtx};` with `use crate::runtime::RuntimeCtx;`.

**Rewrite `build_restricted_ctx`** (lines 16-32). Replace the entire function:

```rust
fn build_restricted_ctx(base: &Arc<RuntimeCtx>, policy: &Policy) -> Arc<RuntimeCtx> {
    let network_denied = policy.net_allow.is_empty();

    Arc::new(RuntimeCtx {
        yield_: base.yield_.clone(),
        source_dir: parking_lot::Mutex::new(base.source_dir.lock().clone()),
        workspace_members: base.workspace_members.clone(),
        dep_dirs: base.dep_dirs.clone(),
        tokio_runtime: base.tokio_runtime.clone(),
        test_threshold: base.test_threshold,
        test_runs: base.test_runs,
        event_stream: base.event_stream.clone(),
        network_denied,
    })
}
```

The `network_denied` flag is checked by `do_request` in `crates/lx/src/stdlib/http.rs` (see section 8).

### 10. `crates/lx-cli/src/main.rs`

**Update imports** (line 22): Remove `NoopEmitBackend, NoopLogBackend` from the import.

**Update `apply_manifest_backends`** (lines 200-238): Remove the `emit`, `log`, `llm`, `http` backend configuration blocks. Keep only the `yield_backend` block. The function becomes:

```rust
fn apply_manifest_backends(ctx: &mut RuntimeCtx, file_path: &str) {
    let file_dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
    let Some(root) = manifest::find_manifest_root(file_dir) else {
        return;
    };
    let Ok(m) = manifest::load_manifest(&root) else {
        return;
    };
    let Some(backends) = m.backends else {
        return;
    };
    if let Some(ref backend) = backends.yield_backend {
        match backend {
            manifest::YieldBackend::StdinStdout => {},
        }
    }
}
```

**Update the manifest module** (`crates/lx-cli/src/manifest.rs`):

Delete the following enum definitions:
- `EmitBackend` enum (lines 60-65)
- `LogBackend` enum (lines 67-72)
- `LlmBackend` enum (lines 74-78)
- `HttpBackend` enum (lines 80-84)

Keep `YieldBackend` enum (lines 86-90).

Update `BackendsSection` struct (lines 92-100). Remove fields `llm`, `http`, `emit`, `log`. The struct becomes:

```rust
#[derive(Deserialize)]
pub struct BackendsSection {
    #[serde(rename = "yield")]
    pub yield_backend: Option<YieldBackend>,
}
```

### 11. `crates/lx-cli/src/llm_backend.rs`

**Delete this entire file.** It contains `ClaudeCodeLlmBackend` which implements the removed `LlmBackend` trait.

**Remove** `mod llm_backend;` from `crates/lx-cli/src/main.rs`.

### 12. `crates/lx/src/stdlib/test_mod/test_report.rs`

**Replace `ctx.emit.emit()`** (line 34):

Replace:
```rust
ctx.emit.emit(&LxVal::str(out), span)?;
```

With:
```rust
println!("{out}");
let mut fields = indexmap::IndexMap::new();
fields.insert(crate::sym::intern("value"), LxVal::str(&out));
ctx.event_stream.xadd("runtime/emit", "main", None, fields);
```

## Step-by-Step Instructions

1. Read `crates/lx/src/stdlib/sandbox/sandbox_scope.rs` fully to understand the sandbox `RuntimeCtx` construction pattern.

2. Delete `crates/lx/src/runtime/noop.rs` and `crates/lx/src/runtime/restricted.rs`.

3. Rewrite `crates/lx/src/runtime/mod.rs`: remove four traits, remove `HttpOpts`, `LogLevel`, `LlmOpts`, remove four fields from `RuntimeCtx`, remove module declarations for `noop` and `restricted`, update `defaults` to only re-export yield-related types. Add `network_denied: bool` field.

4. Rewrite `crates/lx/src/runtime/defaults.rs`: keep only `StdinStdoutYieldBackend`.

5. Rewrite `crates/lx/src/stdlib/http.rs`: inline reqwest logic with local `HttpOpts` struct and `do_request` helper. Check `ctx.network_denied` at the top of `do_request`.

6. Rewrite `crates/lx/src/builtins/llm.rs`: all three functions return error messages.

7. Update `crates/lx/src/builtins/register.rs`: rewrite `make_log_builtin` to use event stream + stderr. Remove `LogLevel` import.

8. Update `crates/lx/src/interpreter/mod.rs`: rewrite `Expr::Emit` to use event stream + stdout.

9. Update `crates/lx/src/stdlib/test_mod/test_report.rs`: replace `ctx.emit.emit()` with println + event stream.

10. Update `crates/lx/src/stdlib/sandbox/sandbox_scope.rs`: replace `DenyHttpBackend` with `network_denied` flag on a new `RuntimeCtx` instance.

11. Delete `crates/lx-cli/src/llm_backend.rs`.

12. Update `crates/lx-cli/src/main.rs`: remove noop backend imports, strip `apply_manifest_backends` of emit/log/llm/http branches.

13. Update `crates/lx-cli/src/manifest.rs`: remove backend enum types for emit/log/llm/http.

14. Grep the entire codebase for remaining references to removed types: `EmitBackend`, `HttpBackend`, `LogBackend`, `LlmBackend`, `LogLevel`, `HttpOpts`, `LlmOpts`, `NoopEmitBackend`, `NoopLogBackend`, `NoopLlmBackend`, `DenyHttpBackend`, `StdoutEmitBackend`, `ReqwestHttpBackend`, `StderrLogBackend`. Fix any remaining references.

## Deliverable

After this work item:
- `RuntimeCtx` has three fields that matter: `yield_`, `event_stream`, `tokio_runtime` (plus workspace/dep/source metadata)
- `emit` expression prints to stdout AND writes `runtime/emit` to the event stream
- `log.info "msg"` prints to stderr AND writes `runtime/log` to the event stream
- `llm.prompt "text"` returns a clear error directing users to `use tool "claude-mcp" as llm`
- `use std/http` continues to work via inline reqwest (no trait indirection)
- Sandbox network denial uses a `network_denied` flag instead of `DenyHttpBackend`
- `YieldBackend` trait and `StdinStdoutYieldBackend` remain unchanged
- All noop/restricted backend types are deleted
- All test infrastructure works without noop backends (event stream is always on, no noop needed)
- The `crates/lx-cli/src/llm_backend.rs` file is deleted
