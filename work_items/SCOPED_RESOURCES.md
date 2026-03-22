# Goal

Write tests that verify the existing `with ... as name { body }` cleanup machinery works correctly. The interpreter already calls `close` on resources when a `with` block exits (both normal and error paths, see `eval_with_resource` in `eval.rs` and `close_resource` in `eval_ops.rs`). No stdlib module currently produces records with `close` fields (the `ws` and `http` modules were removed), so the feature is inert. This work item validates the interpreter machinery using mock resources built from plain records with `close` fields, and establishes the test suite at `tests/` which `just test` expects.

# Why

- The `WithResource` interpreter implements acquire/release with `close` field lookup (`eval_ops.rs:close_resource` looks for `crate::sym::intern("close")` on `LxVal::Record`). No stdlib module exercises this path, making it untested.
- Future stdlib resource producers (file handles, network connections, MCP sessions) will rely on this protocol. Tests must exist before building on it.
- The `tests/` directory does not exist yet. `just test` runs `cargo run -p lx-cli -- test tests/` and currently has nothing to run.

# What Changes

## Resource protocol (already implemented in interpreter)

Any record with a `close: Func` field is a managed resource. The interpreter's `eval_with_resource` (in `crates/lx/src/interpreter/eval.rs`, line 141) already:
1. Evaluates each resource expression.
2. Binds the result to the variable name in a child scope.
3. Evaluates the body.
4. Calls `close_resource` (in `crates/lx/src/interpreter/eval_ops.rs`, line 162) on each resource in reverse order on exit — whether body returned Ok or raised an error.
5. If a resource acquisition fails, already-acquired resources are closed before the error propagates.

## Stdlib state

- `ws.rs` — deleted. The `std/ws` module no longer exists in `crates/lx/src/stdlib/mod.rs`.
- `http.rs` — deleted. No `std/http` module exists.
- `fs.rs` — stateless (read/write/exists/remove/mkdir/ls/stat). No resource handles, no changes needed.
- No stdlib module currently returns records with `close` fields.

# Files Affected

**New files:**
- `tests/scoped_resources.lx` — tests for resource cleanup behavior using mock resources

# Task List

### Task 1: Write scoped resource cleanup tests

**Subject:** Create test suite validating with/close resource protocol

**Description:** Create `tests/scoped_resources.lx` with the following test cases, using `Store` to track whether `close` was called. `Store ()` is a global builtin constructor. Store uses method-call syntax: `.set key value` (3 args including self), `.get key` (2 args including self, returns value or `None`).

1. **Normal exit cleanup** — create a mock resource (record with `close` that sets a Store flag), use it in `with`, verify flag is set after block.
2. **Error exit cleanup** — same but body raises an error. Verify `close` still called. Verify original error propagates.
3. **No close field** — `with` on a plain record (no `close`), verify works as scoped binding without error.
4. **Nested with** — two nested `with` blocks, verify both resources cleaned up in reverse order.

Example pattern for tests:
```
tracker = Store ()
resource = {value: 42, close: () { tracker.set "closed" true }}
with resource as r {
  assert (r.value == 42) "can use resource"
}
assert (tracker.get "closed" == true) "resource was closed"
```

Run `just diagnose` and `just test`.

**ActiveForm:** Writing scoped resource cleanup tests

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
