# Primitive Audit Unit 03: Unused Simple UI Wrappers

## Goal

Remove unused simple `lx-desktop` UI wrapper modules whose components have no call sites and whose dedicated styling hooks are dead.

## Why

After clearing the repo verification blocker, the next primitive-audit discovery pass showed a second cluster of dead primitive-like wrappers in `crates/lx-desktop/src/components/ui/`: `Checkbox`, `Input`, `Label`, `Textarea`, and `Tooltip`. Independent search confirmed there are no call sites for those component names outside their own module files, and the dedicated `.checkbox`, `.input`, `.label`, and `.textarea` Tailwind selectors are not referenced by other source files.

Because these wrappers are unused, the lowest-risk and most standard fix is deletion rather than preserving dead abstractions that duplicate primitive concepts.

## Changes

- Delete the unused `ui/checkbox.rs`, `ui/input.rs`, `ui/label.rs`, `ui/textarea.rs`, and `ui/tooltip.rs` modules.
- Remove their exports from `crates/lx-desktop/src/components/ui/mod.rs`.
- Remove the dead `.checkbox`, `.input`, `.label`, and `.textarea` selectors from `crates/lx-desktop/src/tailwind.css`.

## Files Affected

- `work_items/primitives_audit_unit_03_unused_simple_ui_wrappers.md`
- `crates/lx-desktop/src/components/ui/mod.rs`
- `crates/lx-desktop/src/components/ui/checkbox.rs`
- `crates/lx-desktop/src/components/ui/input.rs`
- `crates/lx-desktop/src/components/ui/label.rs`
- `crates/lx-desktop/src/components/ui/textarea.rs`
- `crates/lx-desktop/src/components/ui/tooltip.rs`
- `crates/lx-desktop/src/tailwind.css`

## Task List

1. Verify there are no remaining call sites for `Checkbox`, `Input`, `Label`, `Textarea`, or `Tooltip` outside their own module files.
2. Delete those five module files.
3. Remove `checkbox`, `input`, `label`, `textarea`, and `tooltip` from `crates/lx-desktop/src/components/ui/mod.rs`.
4. Remove the `.checkbox`, `.input`, `.label`, and `.textarea` selectors from `crates/lx-desktop/src/tailwind.css`.
5. Re-audit the source tree to confirm no imports, call sites, or selector references remain for the deleted wrappers.

## Verification

- `rg '\\b(Checkbox|Input|Label|Textarea|Tooltip)\\s*\\{' crates/lx-desktop/src -g '!crates/lx-desktop/src/components/ui/*.rs'`
- `rg 'class:\\s*\"[^\"]*\\b(checkbox|input|label|textarea)\\b[^\"]*\"|data-slot\": \"checkbox|data-slot\": \"tooltip' crates/lx-desktop/src crates/lx-mobile/src -g '!crates/lx-desktop/src/components/ui/*.rs'`
- `just fmt`
- `just rust-diagnose`
