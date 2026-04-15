# Rust Audit Unit 03: Single-Implementation Local Trait Removal

## Goal

Remove the verified local trait that has exactly one implementation and no polymorphic use.

## Why

`./rules/rust-audit.md` flags traits with a single concrete implementation when they are not used as `dyn Trait`, `impl Trait`, or generic bounds. `TabsStateExt` in `crates/lx-desktop/src/terminal/tab_bar.rs` is a local extension trait with one implementation on `TabsState<DesktopPane>` and one call site in the same file, so it should become a plain helper function.

## Changes

- Replace `TabsStateExt` with a free helper function in `tab_bar.rs`.
- Keep behavior and call-site semantics unchanged.

## Files Affected

- `crates/lx-desktop/src/terminal/tab_bar.rs`

## Task List

1. Remove the local `TabsStateExt` trait and its single implementation.
2. Add a helper function that takes `&TabsState<DesktopPane>` and `tab_id: &str` and returns the same `Option<NotificationLevel>`.
3. Update the existing call site in `TabBar` to use the helper function.
4. Run formatting and Rust diagnostics.

## Verification

- `rg -n 'trait TabsStateExt|impl TabsStateExt|\\bTabsStateExt\\b' crates --type rust`
- `just fmt`
- `just rust-diagnose`
