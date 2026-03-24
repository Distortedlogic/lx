# Goal

Fix the bug where closures inside `+` (exported) function bindings cannot capture sibling non-exported module bindings. `helper = (x) x * 2` then `+f = () { list | each (x) { helper x } }` silently produces wrong results because `helper` is not forward-registered for `+` bindings.

# Why

- The interpreter's forward declaration scan at `mod.rs:95` has `!b.exported` which excludes exported bindings from pre-registration. Non-exported function bindings get pre-registered as mutable Unit so closures can capture them. Exported bindings are skipped, which means closures inside exported functions cannot capture sibling bindings that are also functions
- This produces a **silent failure** — no error message, `helper` resolves to something unexpected, and `each` processes it without crashing. The user sees wrong results with no indication of what went wrong
- The workaround (`f = ...; +f = f` two-step export) is non-obvious and shouldn't be necessary. The `+` prefix is a visibility modifier — it should not change scoping behavior

# What changes

The `!b.exported` condition in the forward declaration scan is removed. All function bindings — exported or not — are pre-registered as mutable Unit so they can be referenced by closures before their definition is evaluated. The export flag only affects what `collect_exports` picks up for module consumers, which is already gated separately at `modules.rs:161`.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/interpreter/mod.rs` | Remove `!b.exported` from forward declaration filter at line 95 |

# Task List

### Task 1: Remove the exported filter from forward declaration scanning

In `crates/lx/src/interpreter/mod.rs`, find the forward declaration scan in the `exec` method (around lines 90-107):

```rust
let mut forward_names = Vec::new();
for &sid in &program.stmts {
    if let Stmt::Binding(b) = self.arena.stmt(sid)
        && !b.exported
        && let BindTarget::Name(name) = b.target
        && matches!(self.arena.expr(b.value), Expr::Func(_))
    {
        forward_names.push(name);
    }
}
```

Remove the `&& !b.exported` condition:

```rust
let mut forward_names = Vec::new();
for &sid in &program.stmts {
    if let Stmt::Binding(b) = self.arena.stmt(sid)
        && let BindTarget::Name(name) = b.target
        && matches!(self.arena.expr(b.value), Expr::Func(_))
    {
        forward_names.push(name);
    }
}
```

This makes the forward pass treat exported and non-exported function bindings identically. Both are pre-registered as mutable Unit (line 105: `env.bind_mut(*name, LxVal::Unit)`) so closures can capture them. When the binding is later evaluated during sequential execution (exec_stmt.rs line 35), the pre-registered Unit is reassigned to the actual function value via `self.env.reassign(*name, val)`.

The export flag still works correctly: `collect_exports` in `modules.rs:161` independently filters by `b.exported` when building module exports. The forward scan and the export collection are separate passes that should not be coupled.

### Task 2: Add a test file to verify the fix

Create `tests/export_closure_capture.lx`:

```
-- export closure capture test
-- verifies closures inside + functions can capture sibling bindings

double = (x) x * 2

+apply_double = (items) {
  items | map (x) { double x }
}

result = apply_double [1 2 3]
assert (result == [2 4 6])

-- also test with each (side-effect HOF)
-- acc must be declared mutable with := to allow reassignment via <-
acc := []
add_doubled = (items) {
  items | each (x) {
    doubled = double x
    acc <- acc ++ [doubled]
  }
}
add_doubled [10 20]
assert (acc == [20 40])
```

This test defines a non-exported helper (`double`), an exported function (`+apply_double`) that uses `double` inside a closure, and verifies the closure produces correct results. Before the fix, `double` would silently resolve to `Unit` and `map` would produce wrong results.

### Task 3: Compile, format, and verify

Run `just fmt` to format changed files.

Run `just test` to verify:
1. The new test file passes
2. All existing tests still pass
3. No regressions from the forward scan change

If any existing test fails, investigate whether it relied on the `!b.exported` behavior. The only scenario where the old behavior differs: an exported function binding that is also self-recursive AND was previously broken (not forward-registered). Such a function would have silently failed before; now it works. This should not cause regressions.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Do not add, skip, reorder, or combine tasks.**
3. **The fix is removing ONE condition** — `!b.exported` on line 95 of `mod.rs`. Do not change the forward scan logic otherwise.
4. **`collect_exports` at `modules.rs:161` is unchanged.** The export collection is a separate pass that correctly filters by `b.exported`. Do not touch it.
5. **The test file uses `assert`** — the test runner treats any assert failure or runtime error as a test failure.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/EXPORT_CLOSURE_CAPTURE.md" })
```

Then call `next_task` to begin.
