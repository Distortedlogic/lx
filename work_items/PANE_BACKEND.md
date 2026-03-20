# Goal

Add `PaneBackend` trait to RuntimeCtx and `std/pane` stdlib module. Agents can programmatically request UI panes (browser, editor, terminal, canvas) from the host environment. Default backend: `YieldPaneBackend` — serializes pane requests over the existing yield/JSON-line protocol so the orchestrator decides rendering.

# Why

- Agentic IDEs (Cursor, Devin, Windsurf) give agents the ability to open browser tabs, editor views, terminal sessions, and visual canvases. lx agents have no way to request UI surfaces — they can only emit text.
- The yield protocol already handles agent↔orchestrator communication. Pane requests are a natural extension: the agent yields a structured pane request, the orchestrator renders it in whatever host UI exists (desktop app, web UI, CLI).
- A backend trait means the mechanism is host-agnostic: the DX desktop backend can route to mcp-toolbelt's pane manager, a CLI can open `$BROWSER`, tests can record requests.

# What Changes

**`crates/lx/src/backends/mod.rs` — new PaneBackend trait:**

```rust
pub trait PaneBackend: Send + Sync {
    fn open(&self, kind: &str, config: &Value, span: Span) -> Result<Value, LxError>;
    fn update(&self, pane_id: &str, content: &Value, span: Span) -> Result<(), LxError>;
    fn close(&self, pane_id: &str, span: Span) -> Result<(), LxError>;
    fn list(&self, span: Span) -> Result<Value, LxError>;
}
```

Add `pub pane: Arc<dyn PaneBackend>` field to `RuntimeCtx`. Set default to `YieldPaneBackend` in `Default` impl.

**`crates/lx/src/backends/pane.rs` — YieldPaneBackend implementation:**

`YieldPaneBackend` serializes pane operations as JSON-line yield messages. `open` writes `{"__pane": {"action": "open", "kind": "...", "config": {...}}}` to stdout, reads the response from stdin (expects `{"pane_id": "..."}`), returns a handle Record. `update` writes `{"__pane": {"action": "update", "pane_id": "...", "content": {...}}}`, no response needed. `close` writes `{"__pane": {"action": "close", "pane_id": "..."}}`. `list` writes `{"__pane": {"action": "list"}}`, reads response.

This follows the exact same pattern as `StdinStdoutYieldBackend` in `defaults.rs` — JSON-line over stdin/stdout.

**New file `crates/lx/src/stdlib/pane.rs`:** Module with `build()` registering `pane.open`, `pane.update`, `pane.close`, `pane.list`. Each function delegates to `ctx.pane.method()`.

# Files Affected

- `crates/lx/src/backends/mod.rs` — Add PaneBackend trait, add field to RuntimeCtx
- `crates/lx/src/backends/pane.rs` — New file: YieldPaneBackend
- `crates/lx/src/stdlib/pane.rs` — New file: std/pane module
- `crates/lx/src/stdlib/mod.rs` — Register module
- `tests/100_pane.lx` — New test file

# Task List

### Task 1: Add PaneBackend trait and YieldPaneBackend

**Subject:** Add PaneBackend trait to backends and implement YieldPaneBackend

**Description:** Edit `crates/lx/src/backends/mod.rs`:

Add the trait after the existing backend traits:

```rust
pub trait PaneBackend: Send + Sync {
    fn open(&self, kind: &str, config: &Value, span: Span) -> Result<Value, LxError>;
    fn update(&self, pane_id: &str, content: &Value, span: Span) -> Result<(), LxError>;
    fn close(&self, pane_id: &str, span: Span) -> Result<(), LxError>;
    fn list(&self, span: Span) -> Result<Value, LxError>;
}
```

Add `pub pane: Arc<dyn PaneBackend>,` to the `RuntimeCtx` struct.

Add `mod pane;` and `pub use pane::*;` at the top of mod.rs.

In the `Default` impl for `RuntimeCtx`, add: `pane: Arc::new(YieldPaneBackend),`.

Create `crates/lx/src/backends/pane.rs`:

Imports: `std::io::{Write, BufRead}`, `std::sync::Arc`, `indexmap::IndexMap`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`, `crate::stdlib::json_conv::{lx_to_json, json_to_lx}`.

`pub struct YieldPaneBackend;`

Implement `PaneBackend for YieldPaneBackend`:

`fn open(&self, kind: &str, config: &Value, span: Span) -> Result<Value, LxError>`: Serialize config to JSON via `lx_to_json(config, span)?`. Build the message: `serde_json::json!({"__pane": {"action": "open", "kind": kind, "config": config_json}})`. Print the message via `println!("{msg}")`. Flush stdout. Read one line from stdin. Parse as JSON. Extract `pane_id` field. Return `Ok(record! { "__pane_id" => Value::Str(Arc::from(pane_id)), "kind" => Value::Str(Arc::from(kind)) })`.

`fn update(&self, pane_id: &str, content: &Value, span: Span) -> Result<(), LxError>`: Serialize content. Build and print `{"__pane": {"action": "update", "pane_id": pane_id, "content": content_json}}`. Flush stdout. Return `Ok(())`.

`fn close(&self, pane_id: &str, _span: Span) -> Result<(), LxError>`: Build and print `{"__pane": {"action": "close", "pane_id": pane_id}}`. Flush stdout. Return `Ok(())`.

`fn list(&self, _span: Span) -> Result<Value, LxError>`: Build and print `{"__pane": {"action": "list"}}`. Flush stdout. Read response line. Parse as JSON. Convert to lx Value via `json_to_lx`. Return `Ok(value)`.

**ActiveForm:** Adding PaneBackend trait and YieldPaneBackend

---

### Task 2: Create std/pane stdlib module

**Subject:** Create pane.rs stdlib module with build() and 4 functions

**Description:** Create `crates/lx/src/stdlib/pane.rs`.

Imports: `std::sync::Arc`, `indexmap::IndexMap`, `crate::backends::RuntimeCtx`, `crate::builtins::mk`, `crate::error::LxError`, `crate::span::Span`, `crate::value::Value`.

`pub fn build() -> IndexMap<String, Value>`: register:
- `"open"` → `bi_open` arity 2
- `"update"` → `bi_update` arity 2
- `"close"` → `bi_close` arity 1
- `"list"` → `bi_list` arity 1

`fn bi_open(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is kind (Str via `as_str()`), args[1] is config (any Value). Call `ctx.pane.open(kind, &args[1], span)`. Wrap result in `Value::Ok`.

`fn bi_update(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is pane handle Record, extract `__pane_id` via `str_field`. args[1] is content. Call `ctx.pane.update(pane_id, &args[1], span)?`. Return `Ok(Value::Unit)`.

`fn bi_close(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is pane handle, extract `__pane_id`. Call `ctx.pane.close(pane_id, span)?`. Return `Ok(Value::Unit)`.

`fn bi_list(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is unit (ignored). Call `ctx.pane.list(span)`.

**ActiveForm:** Creating std/pane stdlib module

---

### Task 3: Register std/pane and write tests

**Subject:** Register pane module in mod.rs and write tests

**Description:** Edit `crates/lx/src/stdlib/mod.rs`:

Add `mod pane;` alongside the other module declarations.

In `get_std_module`, add: `"pane" => pane::build(),` in the match arm.

In `std_module_exists`, add `| "pane"` to the matches! pattern.

Create `tests/100_pane.lx`. Since YieldPaneBackend requires a running orchestrator on stdin/stdout, tests need to verify the module loads and functions exist without actually calling the backend. Use `type_of` checks:

```
use std/pane

assert (type_of pane.open == "Fn") "pane.open is a function"
assert (type_of pane.update == "Fn") "pane.update is a function"
assert (type_of pane.close == "Fn") "pane.close is a function"
assert (type_of pane.list == "Fn") "pane.list is a function"

log.info "100_pane: all passed"
```

The actual pane operations are integration-tested when run under an orchestrator. The unit test verifies the module loads and exports the correct API.

Run `just diagnose` to verify compilation.

**ActiveForm:** Registering pane module and writing tests

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
mcp__workflow__load_work_item({ path: "work_items/PANE_BACKEND.md" })
```

Then call `next_task` to begin.
