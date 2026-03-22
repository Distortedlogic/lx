# Goal

Wire the `close` resource protocol into stdlib modules that produce resources (WebSocket connections) so they work with the existing `with ... as name { body }` cleanup machinery. The interpreter already calls `close` on resources when a `with` block exits (both normal and error paths, see `eval_with_resource` and `close_resource` in `eval_ops.rs`). What's missing is that stdlib resource producers don't return records with a `close` field.

# Why

- The `WithResource` interpreter already implements acquire/release with `close` field lookup (`eval_ops.rs:close_resource`). But no stdlib module actually returns records with `close` fields, making the feature inert.
- Agentic workflows open WebSocket connections, HTTP sessions, and temporary files. Adding `close` fields to these resource records activates the existing cleanup machinery.
- The `close` field convention is established in the interpreter — this work item just connects stdlib to it.

# What Changes

## Resource protocol (already implemented in interpreter)

Any record with a `close: Func` field is a managed resource. The interpreter's `eval_with_resource` already:
1. Evaluates the resource expression.
2. Binds the result to the variable name.
3. Evaluates the body.
4. Calls `resource.close()` on exit — whether body returned Ok or raised an error.

## Stdlib changes

Add `close` fields to resource-producing functions in stdlib:
- `ws.connect` returns `Ok({__ws_id, url})` — add a `close` field to the inner record that calls `bi_close` on the connection.
- `fs` module has no resource handles (all operations are stateless read/write/exists) — no changes needed.

# Files Affected

**Modified files:**
- `crates/lx/src/stdlib/ws.rs` — add `close` field to the record inside the `Ok(...)` returned by `ws.connect`

**New files:**
- `tests/scoped_resources.lx` — tests for resource cleanup behavior

# Task List

### Task 1: Add close to stdlib resource producers and write tests

**Subject:** Add resource protocol to ws.connect and create test suite

**Description:** In `crates/lx/src/stdlib/ws.rs`, modify the `bi_connect` function to include a `close` field in the `Ok(record { __ws_id, url })` it returns. The `close` field should be a builtin function that calls the existing `bi_close` logic for that connection.

Create `tests/scoped_resources.lx`:
1. **Normal exit cleanup** — create a mock resource (record with `close` that sets a flag), use it in `with`, verify flag is set after block.
2. **Error exit cleanup** — same but body raises an error. Verify `close` still called. Verify original error propagates.
3. **No close field** — `with` on a plain record (no `close`), verify works as scoped binding without error.
4. **Nested with** — two nested `with` blocks, verify both resources cleaned up.

For testing, use a `Store` to track whether `close` was called. Store uses method-call syntax (`.set`, `.get`), and `.get` returns the value directly or `None`:
```
tracker = Store ()
resource = {value: 42, close: () { tracker.set "closed" true }}
with resource as r {
  assert (r.value == 42) "can use resource"
}
assert (tracker.get "closed" == true) "resource was closed"
```

Run `just diagnose` and `just test`.

**ActiveForm:** Adding resource protocol and writing tests

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
mcp__workflow__load_work_item({ path: "work_items/SCOPED_RESOURCES.md" })
```

Then call `next_task` to begin.
