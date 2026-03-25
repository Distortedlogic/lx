# Goal

Add `std/http` stdlib module wrapping the existing `HttpBackend` runtime trait, then implement HTTP keyword desugaring. `HTTP MyApi = { base_url: "https://api.example.com", headers: {Authorization: "Bearer ..."} }` desugars to a `Class MyApi : [Connector]` with auto-generated methods that dispatch HTTP requests via `std/http`.

# Why

- HTTP APIs are the most common remote integration in agentic systems. Every external service (search, CRM, deployment, monitoring) exposes HTTP.
- The runtime already has `HttpBackend` trait and a default `reqwest`-based implementation. The `std/http` module wraps it for lx programs to call.
- The HTTP keyword eliminates boilerplate for declaring API connectors — user provides base_url + headers + endpoints, gets a full Connector.

# What Changes

**`crates/lx/src/stdlib/http/mod.rs` — new stdlib module:**

Expose functions via `std/http`:
- `http.get(url)` — GET request, returns `Ok {status, body, headers}` or `Err`
- `http.post(url, body)` — POST with JSON body
- `http.put(url, body)` — PUT with JSON body
- `http.delete(url)` — DELETE request
- `http.request(opts)` — Generic request, opts record: `{method: Str, url: Str, body?: Any, headers?: Record, timeout_ms?: Int}`

Each function calls `ctx.http.request(method, url, &HttpOpts { ... })` from the runtime. HttpOpts is the existing struct used by HttpBackend.

**`crates/lx/src/stdlib/mod.rs`:**

Register `"http"` in `get_std_module()` and `std_module_exists()`.

**Desugar — `crates/lx/src/folder/desugar.rs`:**

In `transform_stmts`, handle `KeywordDecl { keyword: Http, ... }`:

1. Create `Stmt::Use` for `pkg/core/connector {Connector}` and `std/http`.
2. Inject default fields if not present: `base_url: ""`, `headers: {}`, `endpoints: []`.
3. Generate method ASTs:

   `connect = () { Ok () }` (stateless — HTTP is request/response)

   `disconnect = () { Ok () }`

   `call = (req) { ... }` — builds full URL from `self.base_url ++ req.tool`, calls `http.request { method: req.args.method ?? "GET", url: full_url, body: req.args.body, headers: self.headers }`.

   `tools = () { self.endpoints }`

4. User-provided methods override generated ones.
5. Create `Stmt::ClassDecl` with Connector trait.

**Validate — `crates/lx/src/folder/validate_core.rs`:**

Remove Http pass-through. All 12 keyword kinds now must be desugared.

# Files Affected

- `crates/lx/src/stdlib/http/mod.rs` — New file: HTTP stdlib module
- `crates/lx/src/stdlib/mod.rs` — Register http module
- `crates/lx/src/folder/desugar.rs` — Add HTTP desugaring branch
- `crates/lx/src/folder/validate_core.rs` — Remove Http pass-through
- `tests/keyword_http.lx` — New test file
- `tests/stdlib_http.lx` — New test file

# Task List

### Task 1: Create std/http stdlib module

**Subject:** Implement std/http wrapping the existing HttpBackend runtime trait

**Description:** Create `crates/lx/src/stdlib/http/mod.rs` (or `crates/lx/src/stdlib/http.rs` following existing stdlib file conventions — check how other modules like `std/fs` or `std/time` are structured).

Read the existing `HttpBackend` trait and its default implementation to understand the request/response shapes. Read how other stdlib modules (e.g., `std/fs`, `std/time`) are structured — specifically their `build()` function that returns exports.

Implement `build()` returning a record with:
- `"get"` → async builtin arity 1: `(url: Str)`. Calls `ctx.http.request("GET", url, &HttpOpts::default())`. Converts response to `Ok {status: Int, body: Str, headers: Record}` or `Err`.
- `"post"` → async builtin arity 2: `(url: Str, body: Any)`. Serializes body to JSON string if not already string. Calls with method "POST".
- `"put"` → async builtin arity 2: same as post but method "PUT".
- `"delete"` → async builtin arity 1: same as get but method "DELETE".
- `"request"` → async builtin arity 1: `(opts: Record)`. Extracts `method`, `url`, `body`, `headers`, `timeout_ms` from opts record. Builds `HttpOpts`, calls `ctx.http.request`.

Register in `crates/lx/src/stdlib/mod.rs`: add `"http"` match arm in `get_std_module()` and `std_module_exists()`.

**ActiveForm:** Creating std/http stdlib module

---

### Task 2: Write std/http tests

**Subject:** Create test file for std/http module

**Description:** Create `tests/stdlib_http.lx`:

```
use std/http

-- http module exists and has expected functions
assert (http.get | type_of) == "BuiltinFunc" ?? true
assert (http.post | type_of) == "BuiltinFunc" ?? true
assert (http.request | type_of) == "BuiltinFunc" ?? true

-- Actual HTTP calls may fail if no server running, so test gracefully
result = try (http.get "https://httpbin.org/get")
result ? {
  Ok r -> {
    assert r.status == 200
    log.info "stdlib_http: GET passed"
  }
  Err _ -> log.info "stdlib_http: GET skipped (network unavailable)"
}
```

Run `just test`.

**ActiveForm:** Writing std/http tests

---

### Task 3: Implement HTTP desugaring

**Subject:** Add HTTP keyword desugaring with generated Connector methods

**Description:** Edit `crates/lx/src/folder/desugar.rs`. In `transform_stmts`, add the Http branch.

Generate methods using the gen_ast helpers from Unit 4:

`connect = () { Ok () }` — stateless

`disconnect = () { Ok () }` — stateless

`call = (req) { ... }` — generate a block that:
1. Builds `full_url = self.base_url ++ req.tool` (concatenation)
2. Builds opts record: `{method: req.args.method ?? "GET", url: full_url, body: req.args.body, headers: self.headers}`
3. Calls `http.request opts`
Use gen_ast helpers for string concatenation, record construction, field access, function application.

`tools = () { self.endpoints }`

Inject default fields: `base_url: ""`, `headers: {}`, `endpoints: []`.

Same override logic: user-provided methods take precedence.

Emit `Stmt::Use` for `pkg/core/connector {Connector}` and `std/http` (whole).

Emit `Stmt::ClassDecl` with Connector trait.

**ActiveForm:** Implementing HTTP keyword desugaring

---

### Task 4: Finalize validate_core

**Subject:** Remove Http pass-through, assert all 12 keywords desugared

**Description:** Edit `crates/lx/src/folder/validate_core.rs`. Remove the Http keyword from the temporary pass-through list. All 12 KeywordKind variants should now be checked: if any `Stmt::KeywordDecl` survives into Core AST, panic. This completes the validate_core coverage for all keywords.

**ActiveForm:** Finalizing validate_core for all keywords

---

### Task 5: Write HTTP keyword test

**Subject:** Create test file validating HTTP keyword works end-to-end

**Description:** Create `tests/keyword_http.lx`:

```
HTTP TestApi = {
  base_url: "https://httpbin.org"
  headers: {Accept: "application/json"}
  endpoints: [{name: "get", method: "GET"}, {name: "post", method: "POST"}]
}

api = TestApi {}
assert api.base_url == "https://httpbin.org"
assert (api.headers.Accept) == "application/json"

-- Verify Connector methods exist
assert (method_of api "connect" | some?)
assert (method_of api "disconnect" | some?)
assert (method_of api "call" | some?)
assert (method_of api "tools" | some?)

-- tools() returns endpoints
assert (api.tools () | len) == 2

-- connect is no-op
connect_result = api.connect ()
assert (connect_result | ok?)
```

Run `just test`.

**ActiveForm:** Writing HTTP keyword test

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_5_HTTP.md" })
```

Then call `next_task` to begin.
