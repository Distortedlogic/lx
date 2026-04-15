# Rust Audit Unit 01: Self-Assignments and `&String` Cleanup

## Goal

Remove verified Rust audit violations for self-assignment bindings and the one `&String` container type that should use `&str`.

## Why

`./rules/rust-audit.md` flags `let x = x;` / `let mut x = x;` shadow bindings as low-signal capture patterns that should be restructured, and it flags `&String` usage where `&str` suffices. These are binary findings and they can be fixed without changing behavior.

## Changes

- Replace same-name rebinding patterns with clearer capture names or by making the original binding mutable.
- Change the dependency target list in `lx-cli` from `&String` keys to `&str`.
- Preserve behavior exactly.

## Files Affected

- `crates/lx-api/src/ws_events.rs`
- `crates/lx-cli/src/install.rs`
- `crates/lx-desktop/src/pages/issues/new_issue.rs`
- `crates/lx-desktop/src/components/editor_textarea.rs`
- `crates/lx-desktop/src/components/ui/dropdown_menu.rs`
- `crates/lx-desktop/src/components/ui/collapsible.rs`
- `crates/lx-desktop/src/components/ui/tabs.rs`
- `crates/lx-desktop/src/components/ui/popover.rs`
- `crates/lx-desktop/src/components/ui/sheet.rs`
- `crates/lx-desktop/src/components/ui/dialog.rs`

## Task List

1. Remove each verified self-assignment binding in the listed files by renaming the captured binding or restructuring the surrounding code so the shadow copy is no longer needed.
2. Update the `lx-cli` install target collection to use `&str` names instead of `&String`.
3. Re-run the self-assignment and `&String` search commands and confirm the listed violations are gone.
4. Run formatting and Rust diagnostics.

## Verification

- `rg -n 'let (mut )?([A-Za-z_][A-Za-z0-9_]*) = \2;' crates --type rust -P`
- `rg -n '&String|&Vec<' crates --type rust`
- `just fmt`
- `just rust-diagnose`
