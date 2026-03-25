# Goal

Create `std/http` stdlib module wrapping the existing `HttpBackend` runtime trait, then implement HTTP keyword desugaring. `HTTP MyApi = { base_url: "https://api.example.com", headers: {Authorization: "Bearer ..."} }` desugars to `Class MyApi : [Connector]` with generated methods that dispatch HTTP requests via `std/http`.

# Why

HTTP APIs are the most common remote integration. The runtime already has `HttpBackend` trait with a `reqwest`-based default implementation. The `std/http` module wraps it for lx programs. The HTTP keyword eliminates boilerplate for declaring API connectors.

# HttpBackend exact API

From `crates/lx/src/runtime/mod.rs`:

```rust
#[derive(Debug, Clone, Default)]
pub struct HttpOpts {
    pub headers: Option<IndexMap<String, String>>,
    pub query: Option<IndexMap<String, String>>,
    pub body: Option<serde_json::Value>,
}

pub trait HttpBackend: Send + Sync {
    fn request(&self, method: &str, url: &str, opts: &HttpOpts, span: SourceSpan) -> Result<LxVal, LxError>;
}
```

This is a **synchronous** trait (not async). The default implementation is `ReqwestHttpBackend`. The return type is `Result<LxVal, LxError>`. The returned `LxVal` on success is a Record with status, body, headers.

# Async builtin pattern

From `crates/lx/src/value/func.rs`, sync builtins use:
```rust
pub type SyncBuiltinFn = fn(&[LxVal], SourceSpan, &Arc<RuntimeCtx>) -> Result<LxVal, LxError>;
```
Registered via `mk("name", arity, fn_ptr)`.

Since HttpBackend.request is sync, std/http builtins should be **sync** (not async). Use `mk` not `mk_async`.

# What Changes

**`crates/lx/src/stdlib/http.rs` — new stdlib module:**

Build function returns a record with:
- `"get"` → sync builtin arity 1: takes url string. Calls `ctx.http.request("GET", url, &HttpOpts::default(), span)`.
- `"post"` → sync builtin arity 2: takes url string + body. Serializes body LxVal to serde_json::Value, sets `opts.body = Some(json)`. Calls with "POST".
- `"put"` → sync builtin arity 2: same as post with "PUT".
- `"delete"` → sync builtin arity 1: same as get with "DELETE".
- `"request"` → sync builtin arity 1: takes opts Record. Extracts `method` (default "GET"), `url`, `body`, `headers`, `query` from the record. Builds `HttpOpts`. Calls `ctx.http.request`.

LxVal to serde_json::Value conversion: use the existing `lx_val_to_json` utility if it exists, or implement a simple recursive conversion (Int→Number, Float→Number, Str→String, Bool→Bool, Record→Object, List→Array, Unit→Null).

**`crates/lx/src/stdlib/mod.rs`:**

Add `"http"` to `get_std_module()` and `std_module_exists()`. The module is `mod http;` in the stdlib directory. Check the existing module structure — some modules are files (e.g., `channel.rs`), some are directories (e.g., `store/mod.rs`). Use whichever pattern the simplest modules follow.

**Desugar — `crates/lx/src/folder/desugar.rs`:**

Add Http branch to `desugar_keyword`. Generate methods using gen_ast helpers:

```lx
connect = () Ok ()
disconnect = () Ok ()
call = (req) {
  url = self.base_url ++ req.tool
  http.request {method: req.args.method ?? "GET", url: url, body: req.args.body, headers: self.headers}
}
tools = () self.endpoints
```

Inject default fields: `base_url: ""`, `headers: {}`, `endpoints: []`.
Inject imports: `use pkg/core/connector {Connector}`, `use std/http`.

**Validate — `crates/lx/src/folder/validate_core.rs`:**

Add Http to desugared assertion. After this, ALL 12 keyword kinds are covered.

# Files Affected

- `crates/lx/src/stdlib/http.rs` — New file: HTTP stdlib module
- `crates/lx/src/stdlib/mod.rs` — Register http module
- `crates/lx/src/folder/desugar.rs` — Add HTTP desugaring
- `crates/lx/src/folder/validate_core.rs` — Add Http assertion (completes all 12)
- `tests/keyword_http.lx` — New test
- `tests/stdlib_http.lx` — New test

# Task List

### Task 1: Create std/http stdlib module

**Subject:** Implement std/http wrapping HttpBackend

**Description:** First, check how existing stdlib modules are structured. Read `crates/lx/src/stdlib/mod.rs` to see the dispatch pattern. Read one simple sync module (likely `std/env` or `std/math`) for the `build()` function pattern.

Create `crates/lx/src/stdlib/http.rs`. Implement a `pub fn build(ctx: &Arc<RuntimeCtx>) -> IndexMap<Sym, LxVal>` function (or whatever signature the other modules use).

Register five sync builtins:

`bi_get`: extract `args[0]` as string url. Call `ctx.http.request("GET", &url, &HttpOpts::default(), span)`. Wrap result: on Ok, return `LxVal::ok(result)`. On Err, return the error.

`bi_post`: extract `args[0]` as url, `args[1]` as body. Convert body to `serde_json::Value`. Call with `HttpOpts { body: Some(json), ..default() }` and method "POST".

`bi_put`: same as post with "PUT".

`bi_delete`: same as get with "DELETE".

`bi_request`: extract `args[0]` as Record. Get fields: `method` (default "GET"), `url`, `body` (optional), `headers` (optional Record→IndexMap<String,String>), `query` (optional Record→IndexMap<String,String>). Build `HttpOpts`. Call `ctx.http.request`.

For LxVal-to-JSON conversion: check if `crate::value` has an existing `to_json` or `to_serde` method. If not, write a simple conversion function handling Int, Float, Str, Bool, List, Record, Unit→Null.

Register in `crates/lx/src/stdlib/mod.rs`: add `"http" => http::build(ctx)` in `get_std_module()` and `"http"` in `std_module_exists()`.

**ActiveForm:** Creating std/http module

---

### Task 2: Write std/http test

**Subject:** Test std/http module functions exist and are callable

**Description:** Create `tests/stdlib_http.lx`:

```lx
use std/http

-- Verify functions exist
assert (type_of http.get) == "BuiltinFunc"
assert (type_of http.post) == "BuiltinFunc"
assert (type_of http.request) == "BuiltinFunc"

-- Actual HTTP call (may fail if no network)
result = try (http.get "https://httpbin.org/get")
result ? {
  Ok r -> {
    assert (r | ok?)
    log.info "stdlib_http: GET ok"
  }
  Err _ -> log.info "stdlib_http: GET skipped (network unavailable)"
}
```

Run `just test`.

**ActiveForm:** Writing std/http test

---

### Task 3: Implement HTTP desugaring

**Subject:** Generate Connector methods for HTTP keyword

**Description:** Edit `crates/lx/src/folder/desugar.rs`. Add the Http branch to `desugar_keyword`.

Using gen_ast helpers:

`connect()`: `gen_ok_unit` — HTTP is stateless.
`disconnect()`: `gen_ok_unit`.

`call(req)`: Generate a block:
1. `url = self.base_url ++ req.tool` — string concatenation via Binary(Concat) operator
2. Build a record `{method: req.args.method ?? "GET", url: url, body: req.args.body, headers: self.headers}` — use gen_record with appropriate field access expressions and a coalesce for the method default
3. Call `http.request opts`

Since coalesce (`??`) is desugared by the same Desugarer in `leave_expr`, you can emit `Expr::Coalesce` AST nodes and they'll be handled.

`tools()`: `self.endpoints` — a field access.

Inject default fields if not present:
- `base_url: ""` (empty string literal)
- `headers: {}` (empty record)
- `endpoints: []` (empty list)

Emit imports: `use pkg/core/connector {Connector}`, `use std/http` (UseKind::Whole).

**ActiveForm:** Implementing HTTP desugaring

---

### Task 4: Finalize validate_core

**Subject:** Complete validate_core for all 12 keywords

**Description:** Edit `crates/lx/src/folder/validate_core.rs`. Add `KeywordKind::Http` to the assertion list. After this change, ALL 12 KeywordKind variants are checked: Agent, Tool, Prompt, Connector, Store, Session, Guard, Workflow, Schema, Mcp, Cli, Http. No KeywordDecl should ever survive into Core AST.

**ActiveForm:** Completing validate_core

---

### Task 5: Write HTTP keyword test

**Subject:** Test HTTP keyword end-to-end

**Description:** Create `tests/keyword_http.lx`:

```lx
HTTP TestApi = {
  base_url: "https://httpbin.org"
  headers: {Accept: "application/json"}
  endpoints: [{name: "/get", method: "GET"}]
}

api = TestApi {}
assert api.base_url == "https://httpbin.org"
assert api.headers.Accept == "application/json"
assert (methods_of api | any? (== "connect"))
assert (methods_of api | any? (== "call"))
assert (methods_of api | any? (== "tools"))
result = api.tools ()
assert (result | len) == 1
connect_result = api.connect ()
assert (connect_result | ok?)
```

Run `just test`.

**ActiveForm:** Writing HTTP keyword test

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_5_HTTP.md" })
```
