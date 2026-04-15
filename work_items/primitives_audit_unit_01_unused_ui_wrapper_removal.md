# Primitive Audit Unit 01: Unused UI Wrapper Removal

## Goal

Remove unused `lx-desktop` UI wrapper modules that either duplicate available `dioxus-primitives` components or exist as dead exports with no call sites.

## Why

The primitive audit surfaced hand-rolled `Avatar` and `Separator` components in `crates/lx-desktop/src/components/ui/` even though those primitives already exist in the local `dioxus-primitives` source tree. Independent fact-checking showed both modules have zero call sites in `lx-desktop`, so keeping them exported only increases the chance of future drift away from the primitive library.

The Dioxus audit also showed `ui/skeleton.rs` is dead code: it is exported from `ui/mod.rs`, but all real skeleton rendering is handled by the private local `Skeleton` component inside `components/page_skeleton.rs`.

Because none of these wrappers are used, the best validated fix is deletion rather than introducing a new dependency for dead code.

## Changes

- Delete the unused custom `ui/avatar.rs`, `ui/separator.rs`, and `ui/skeleton.rs` modules.
- Remove their module exports from `crates/lx-desktop/src/components/ui/mod.rs`.
- Remove the now-dead `.avatar*` and `.separator` Tailwind component selectors from `crates/lx-desktop/src/tailwind.css`.

## Files Affected

- `work_items/primitives_audit_unit_01_unused_ui_wrapper_removal.md`
- `crates/lx-desktop/src/components/ui/mod.rs`
- `crates/lx-desktop/src/components/ui/avatar.rs`
- `crates/lx-desktop/src/components/ui/separator.rs`
- `crates/lx-desktop/src/components/ui/skeleton.rs`
- `crates/lx-desktop/src/tailwind.css`

## Task List

1. Verify `ui/avatar.rs`, `ui/separator.rs`, and `ui/skeleton.rs` have no call sites outside their own definitions or module exports.
2. Delete those three module files.
3. Remove `avatar`, `separator`, and `skeleton` from `crates/lx-desktop/src/components/ui/mod.rs`.
4. Remove the `.avatar`, `.avatar-fallback`, `.avatar-badge`, `.avatar-group`, and `.separator` component definitions from `crates/lx-desktop/src/tailwind.css`.
5. Re-audit the touched files to ensure the change did not remove any still-referenced selector or module.

## Verification

- `just fmt`
- `just rust-diagnose`
- `rg '\bAvatar\b|AvatarImage|AvatarFallback|AvatarBadge|AvatarGroup|orientation:|Orientation::|components::ui::(avatar|separator|skeleton)|ui::(avatar|separator|skeleton)' crates/lx-desktop/src`
