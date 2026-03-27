# Structural — Free Functions to Methods + Single-Impl Trait

Move free functions that take a struct as first parameter into impl blocks on those types. Remove single-implementation trait.

**Extends:** structural_cleanup.md (which covers workspace_member_map, h_level, build_args, and extract functions).

---

## Task 1: derive_pane_title → DesktopPane method

**File:** `crates/lx-desktop/src/terminal/toolbar.rs:221`

Current: `fn derive_pane_title(pane: &DesktopPane) -> String`

`DesktopPane` already has an impl block in `crates/lx-desktop/src/panes.rs`.

Move this function into the `impl DesktopPane` block as:
```rust
pub fn title(&self) -> String { ... }
```

Update the single call site in `toolbar.rs` from `derive_pane_title(&pane)` to `pane.title()`.

---

## Task 2: count_errors → CheckResult method

**File:** `crates/lx-cli/src/check.rs:93`

Current: `fn count_errors(result: &CheckResult, strict: bool) -> u32`

Move into `impl CheckResult` (defined in `crates/lx/src/checker/mod.rs`). Add:
```rust
pub fn count_errors(&self, strict: bool) -> u32 { ... }
```

Update call site in `check.rs` from `count_errors(&result, strict)` to `result.count_errors(strict)`.

---

## Task 3: Keep print_diagnostics as free function

**File:** `crates/lx-cli/src/check.rs:214`

`print_diagnostics` depends on CLI-specific imports (`NamedSource`, `Report` from miette, and local `print_fix` function). It cannot move to `impl CheckResult` in the `lx` crate.

**No change needed** — this function correctly lives in lx-cli as a free function. Remove from audit scope.

Note: `count_errors` (Task 2) uses only `CheckResult` public fields and CAN move to `impl CheckResult`.

---

## Task 4: validate_manifest → RootManifest method

**File:** `crates/lx-cli/src/manifest.rs:116`

Current: `fn validate_manifest(manifest: &RootManifest, path: &Path) -> Result<(), String>`

Move into `impl RootManifest`:
```rust
pub fn validate(&self, path: &Path) -> Result<(), String> { ... }
```

Update call site from `validate_manifest(&manifest, &path)` to `manifest.validate(&path)`.

---

## Task 5: inject_self_for_method → LxVal::bind_self method

**File:** `crates/lx/src/builtins/register.rs:250`

Current: `fn inject_self_for_method(method: &LxVal, self_val: &LxVal) -> LxVal`

The function creates a child environment from the function's closure, binds `"self"` to the provided `self_val`, and returns a new `LxVal::Func` with the updated closure. If the method is not a `Func`, it just clones it.

Move into `impl LxVal` with the name `bind_self`:
```rust
pub fn bind_self(&self, self_val: &LxVal) -> LxVal { ... }
```

Update call sites from `inject_self_for_method(&method, &args[0])` to `method.bind_self(&args[0])`.

---

## Task 6: highest_notification_level → extension trait on TabsState

**File:** `crates/lx-desktop/src/terminal/tab_bar.rs:166`

Current: `fn highest_notification_level(state: &TabsState<DesktopPane>, tab_id: &str) -> Option<NotificationLevel>`

`TabsState` is defined in the external `common-pane-tree` crate, so the function cannot be moved there directly. Use an extension trait in `tab_bar.rs`:

```rust
trait TabsStateExt {
    fn highest_notification_level(&self, tab_id: &str) -> Option<NotificationLevel>;
}

impl TabsStateExt for TabsState<DesktopPane> {
    fn highest_notification_level(&self, tab_id: &str) -> Option<NotificationLevel> {
        // move function body here, replacing `state` with `self`
    }
}
```

Update call site from `highest_notification_level(&state, tab_id)` to `state.highest_notification_level(tab_id)`.

---

## Task 7: AstTransformer — architecturally justified, no change

The `AstTransformer` trait at `visitor/transformer.rs:10` has only one implementation (`Desugarer`). However, it exists for **architectural separation**: the generic `<T: AstTransformer>` bound on `Expr::recurse_children`, `Stmt::recurse_children`, and `Pattern::recurse_children` in `ast/walk_impls.rs` allows the AST types to be recursed without depending on `folder::desugar::Desugarer`. Removing the trait would create an inverted dependency (`ast/` → `folder/`).

**No change needed.** Remove from audit scope.
