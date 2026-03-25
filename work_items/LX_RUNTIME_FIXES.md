# Goal

Fix three runtime bugs: trait default method self binding, source_dir shared state corruption, and type_of inconsistency.

# Why

- Trait default methods (inherited by Classes via `inject_traits`) don't bind `self` to the instance. Calling `self.values()` from an inherited Collection method returns `[]`. Every Store/Collection user works around this by calling `self.entries.values()` directly.
- `source_dir()` returns wrong values after module imports because child interpreters overwrite the shared RuntimeCtx.source_dir.
- `type_of` returns `"BuiltinFunc"` for stdlib functions and `"Func"` for user functions. Users can't reliably check if something is callable.

# What Changes

**Trait self binding — `crates/lx/src/interpreter/apply_helpers.rs`:**

When a method is accessed on an Object via field access (`obj.method`), `inject_self` creates a child environment with `self` bound to the Object. For class-defined methods, this works because the method's closure was created during class evaluation and the child env properly chains.

For trait default methods, the method's closure was created during TRAIT evaluation. The closure's parent env is the trait's scope, not the class's scope. When `inject_self` creates a child env from this closure, `self` is bound, but the closure parent doesn't have the class's bindings.

However — `self` IS correctly bound in the child env. The issue might be elsewhere. Read the `inject_self` function fully. Then write a minimal reproduction:

```lx
+Trait HasValues = {
  entries: Store = Store ()
  get_count = () { self.entries.values () | len }
}
Class Impl : [HasValues] = {}
obj = Impl {}
obj.entries.set "a" 1
assert (obj.get_count () == 1)
```

If this fails, the bug is confirmed. Debug by adding prints inside `inject_self`. The fix depends on what exactly is wrong — likely the method closure needs to be reconstructed with the instance's environment as the parent, not the original trait scope.

Read `crates/lx/src/interpreter/apply_helpers.rs` `inject_self` function and `crates/lx/src/interpreter/traits.rs` `inject_traits` function. Understand the full flow from trait default → class method → instance method call.

**source_dir shared state — `crates/lx/src/interpreter/modules.rs`:**

`Interpreter::new()` at line 54 does `*ctx.source_dir.lock() = source_dir.clone()`. When `load_module` creates a child interpreter at line 146, it calls `Interpreter::new` with the module's directory, which overwrites the parent's source_dir.

Fix: save and restore source_dir around module loading. In `load_module`:
```rust
let saved_source_dir = self.ctx.source_dir.lock().clone();
// ... create child interpreter, exec module ...
*self.ctx.source_dir.lock() = saved_source_dir;
```

Same for `load_module_from_source` (which passes `None` as source_dir).

**type_of inconsistency — `crates/lx/src/builtins/register.rs`:**

The `bi_type_of` function returns `val.type_name()` which is derived from `IntoStaticStr` on `LxVal`. The enum variant `BuiltinFunc` produces `"BuiltinFunc"` while `Func` produces `"Func"`.

Fix: in `bi_type_of`, after getting the type name, map `"BuiltinFunc"` to `"Func"`:
```rust
fn bi_type_of(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let name = match &args[0] {
        LxVal::BuiltinFunc(_) => "Func",
        other => other.type_name(),
    };
    Ok(LxVal::str(name))
}
```

Or modify the `IntoStaticStr` derive/impl for `LxVal::BuiltinFunc` to return `"Func"`.

# Files Affected

- `crates/lx/src/interpreter/apply_helpers.rs` — fix trait self binding
- `crates/lx/src/interpreter/traits.rs` — may need changes for method closure reconstruction
- `crates/lx/src/interpreter/modules.rs` — save/restore source_dir
- `crates/lx/src/builtins/register.rs` — normalize type_of output

# Task List

### Task 1: Fix trait default method self binding

**Subject:** Ensure inherited trait methods bind self to the instance

**Description:** First, write a minimal reproduction to confirm the bug:

```lx
+Trait Counter = {
  count: Int = 0
  increment = () { self.count <- self.count + 1 }
  get_count = () { self.count }
}
Class MyCounter : [Counter] = {}
c = MyCounter {}
c.increment ()
assert (c.get_count () == 1)
```

Run this. If it fails, the bug is confirmed. If it passes, try with Store:

```lx
+Trait HasStore = {
  entries: Store = Store ()
  add = (k v) { self.entries.set k v }
  count = () { self.entries.values () | len }
}
Class Impl : [HasStore] = {}
obj = Impl {}
obj.add "a" 1
assert (obj.count () == 1)
```

Read `crates/lx/src/interpreter/apply_helpers.rs`. Find `inject_self`. Read how it creates the child environment. The method is a `LxFunc` with a `closure: Arc<Env>`. `inject_self` does:
```rust
let child = lf.closure.child();
child.bind(intern("self"), self_val.clone());
```

The child env's PARENT is `lf.closure` — the trait's evaluation scope. When the method runs `self.count`, it looks up `self` in the child env (found — the Object), then `.count` accesses the Object's store field. This should work because `self` IS the Object.

If the bug is NOT in `inject_self`, check whether trait defaults are cloned correctly in `inject_traits`. The `v.clone()` of a `LxFunc` clones the closure Arc — both the original and clone share the same env. This is correct for immutable closures but might cause issues if the method body mutates variables in the closure scope.

Debug by adding trace prints in `inject_self` and the field access path in `apply_helpers.rs`.

**ActiveForm:** Fixing trait self binding

---

### Task 2: Fix source_dir shared state

**Subject:** Save and restore source_dir around module loads

**Description:** Edit `crates/lx/src/interpreter/modules.rs`.

In `load_module` (around line 119), before creating the child interpreter:
```rust
let saved_source_dir = self.ctx.source_dir.lock().clone();
```

After the child finishes (after `mod_interp.exec` completes and exports are collected):
```rust
*self.ctx.source_dir.lock() = saved_source_dir;
```

Same in `load_module_from_source` (around line 98):
```rust
let saved_source_dir = self.ctx.source_dir.lock().clone();
// ... create child, exec, collect exports ...
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

This imports std/time (a module load), then checks source_dir is still correct after the import. Currently sd2 would be wrong (overwritten by the time module's source_dir).

**ActiveForm:** Fixing source_dir shared state

---

### Task 3: Fix type_of inconsistency

**Subject:** Normalize BuiltinFunc to Func in type_of

**Description:** Edit `crates/lx/src/builtins/register.rs`. Find the `bi_type_of` function.

Change it to return `"Func"` for BuiltinFunc values:
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

Also normalize `MultiFunc` to `"Func"` — the user shouldn't see the internal distinction between single and multi-clause functions.

Write test `tests/type_of_consistency.lx`:
```lx
f = (x) { x + 1 }
assert (type_of f == "Func")
assert (type_of len == "Func")
assert (type_of to_str == "Func")

g = (x) & (x > 0) { x }
g = (x) { 0 - x }
assert (type_of g == "Func")
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
