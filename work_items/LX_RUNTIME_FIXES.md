# Goal

Fix two runtime bugs: source_dir shared state corruption and type_of inconsistency.

Note: trait default method self binding was investigated and confirmed to work correctly. The failures previously attributed to it were caused by the coalesce bug (LX_COALESCE_FIX) and pipe-after-block bug (LX_PARSER_FIXES). Verified with both simple fields and Store fields — inherited trait methods bind `self` to the instance correctly.

# Why

- `source_dir()` returns wrong values after module imports because child interpreters overwrite the shared RuntimeCtx.source_dir.
- `type_of` returns `"BuiltinFunc"` for stdlib functions and `"Func"` for user functions. Users can't reliably check if something is callable.

# What Changes

**source_dir shared state — `crates/lx/src/interpreter/modules.rs`:**

`Interpreter::new()` at line 54 does `*ctx.source_dir.lock() = source_dir.clone()`. When `load_module` creates a child interpreter at line 146, it calls `Interpreter::new` with the module's directory, which overwrites the parent's source_dir.

Fix: save and restore source_dir around module loading.

**type_of inconsistency — `crates/lx/src/builtins/register.rs`:**

The `bi_type_of` function returns `val.type_name()` which produces `"BuiltinFunc"` for builtins and `"Func"` for user functions. Both are callable — the distinction is internal.

Fix: normalize `BuiltinFunc` and `MultiFunc` to `"Func"`.

# Files Affected

- `crates/lx/src/interpreter/modules.rs` — save/restore source_dir
- `crates/lx/src/builtins/register.rs` — normalize type_of output

# Task List

### Task 1: Fix source_dir shared state

**Subject:** Save and restore source_dir around module loads

**Description:** Edit `crates/lx/src/interpreter/modules.rs`.

In `load_module` (line 119), before creating the child interpreter at line 146:
```rust
let saved_source_dir = self.ctx.source_dir.lock().clone();
```

After the child finishes (after line 149 `mod_interp.exec` and line 150 `collect_exports`):
```rust
*self.ctx.source_dir.lock() = saved_source_dir;
```

Same in `load_module_from_source` (line 98), before line 110:
```rust
let saved_source_dir = self.ctx.source_dir.lock().clone();
```

After line 113 `mod_interp.exec` and line 114 `collect_exports`:
```rust
*self.ctx.source_dir.lock() = saved_source_dir;
```

Write test `tests/source_dir_modules.lx`:
```lx
use std/time

sd1 = source_dir ()
now = time.now ()
sd2 = source_dir ()
assert (sd1 == sd2)
```

**ActiveForm:** Fixing source_dir shared state

---

### Task 2: Fix type_of inconsistency

**Subject:** Normalize BuiltinFunc and MultiFunc to Func in type_of

**Description:** Edit `crates/lx/src/builtins/register.rs`. Find `bi_type_of` (search for `type_of`).

Add special cases before the default:
```rust
fn bi_type_of(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let name = match &args[0] {
        LxVal::BuiltinFunc(_) => "Func",
        LxVal::MultiFunc(_) => "Func",
        other => other.type_name(),
    };
    Ok(LxVal::str(name))
}
```

Write test `tests/type_of_consistency.lx`:
```lx
f = (x) { x + 1 }
assert (type_of f == "Func")
assert (type_of len == "Func")
assert (type_of to_str == "Func")
assert (type_of map == "Func")
```

**ActiveForm:** Normalizing type_of output

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/LX_RUNTIME_FIXES.md" })
```
