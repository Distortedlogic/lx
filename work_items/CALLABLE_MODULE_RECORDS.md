# Goal

Make `Value::Record` callable via a `__call` field, and auto-set `__call` on module records when an exported function name matches the module name. This eliminates the name collision where `use ./main` shadows the exported `+main` function, making `main()` fail with "cannot call Record, not a function."

# Why

- `use ./main` followed by `main()` is the natural pattern for lx entry points, but it fails because `use ./main` binds the module record to `main`, shadowing the exported `+main` function
- The workaround (`use ./main : alias` or `use ./main {main}`) is non-obvious and every new lx user will hit this
- Python, Lua, and JavaScript solve this with callable objects (`__call__` protocol) — a proven, general-purpose pattern
- The fix is not special-cased to module imports — callable records are useful independently (factory patterns, configurable handlers, etc.)

# What changes

**Interpreter — callable records:** In `apply_func` in `crates/lx/src/interpreter/apply.rs`, add a `Value::Record` arm that checks for a `__call` field. If the field is a callable value (Func, BuiltinFunc, etc.), delegate the call to it. If no `__call` field exists, return the existing "cannot call Record" error.

**Module imports — auto-set `__call`:** In `eval_use` in `crates/lx/src/interpreter/modules.rs`, after collecting module exports into a record, check if any exported name matches the module name (the last segment of the import path). If so, insert a `__call` field pointing to that export's value. This only applies to `UseKind::Whole` and `UseKind::Alias` imports — selective imports (`use ./main {main}`) already bring names directly into scope.

**No changes to module export collection:** The `collect_exports` function stays the same. The `__call` insertion happens at the import site, not the export site.

# How it works

When `use ./main` is evaluated:
1. Module exports are collected: `{main: <fn>, run: <fn>}`
2. Module name is `main` (last path segment)
3. Exports contain a key `main` matching the module name — insert `__call: <fn>` pointing to the same value
4. Final record bound to `main`: `{main: <fn>, run: <fn>, __call: <fn>}`

When `main()` is called:
1. `apply_func` receives `Value::Record` as the function
2. Checks for `__call` field — found
3. Delegates to `apply_func(call_fn, arg, span)`

When `main.run()` is called:
1. Field access `main.run` returns the `run` function
2. Called normally — no change

# Files affected

- `crates/lx/src/interpreter/apply.rs` — add `Value::Record` arm in `apply_func` that checks for `__call` and delegates
- `crates/lx/src/interpreter/modules.rs` — in `eval_use`, after building the module record for `UseKind::Whole` and `UseKind::Alias`, check if any export key matches the module name and insert `__call`

# Task List

### Task 1: Add callable record support in apply_func

**Subject:** Make Value::Record callable via __call field

**Description:** In `crates/lx/src/interpreter/apply.rs`, in the `apply_func` method, add a match arm for `Value::Record(r)` before the final `other` catch-all. Check if the record has a field named `__call`. If so, extract the value and recursively call `self.apply_func(call_value, arg, span)`. If no `__call` field exists, fall through to the existing error: "cannot call Record, not a function". The error message should be updated to suggest: "cannot call Record — add a __call field to make it callable, or use selective import: use ./module {name}".

**ActiveForm:** Adding callable record support to apply_func

### Task 2: Auto-insert __call on module import

**Subject:** Set __call on module records when export name matches module name

**Description:** In `crates/lx/src/interpreter/modules.rs`, in the `eval_use` method, for `UseKind::Whole` and `UseKind::Alias` branches: after building the `record` from `exports.bindings`, compute the module name (for Whole: last segment of `use_stmt.path`, for Alias: the alias itself). Check if `exports.bindings` contains a key matching the module name. If so, clone that value and insert it into the record's IndexMap with key `__call` before binding. Do not insert `__call` if the module name does not match any export.

**ActiveForm:** Auto-inserting __call field on module import

### Task 3: Add tests

**Subject:** Test callable records and module import __call

**Description:** In `tests/`, add assertions covering: (1) a record with a `__call` field can be called as a function, (2) a record without `__call` produces a clear error, (3) `use ./module` where the module exports `+module` — calling `module()` works, (4) `use ./module` where no export matches the module name — calling `module()` produces the existing error with the new suggestion, (5) `use ./module : alias` — the alias name is checked against exports, not the original module name.

**ActiveForm:** Adding callable record and module __call tests

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
mcp__workflow__load_work_item({ path: "work_items/CALLABLE_MODULE_RECORDS.md" })
```

Then call `next_task` to begin.
