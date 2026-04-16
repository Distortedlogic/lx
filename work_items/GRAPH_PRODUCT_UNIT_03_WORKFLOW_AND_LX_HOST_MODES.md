# Graph Product Unit 03: Workflow And LX Host Modes

## Goal
Turn the `flows` host into a real product switchboard that can operate as either the workflow/n8n graph surface or the lx graph surface, instead of hardcoding workflow behavior and leaving the lx compiler path unused.

## Why
- The current `flows` host always seeds workflow samples, always loads workflow templates, always runs workflow validation, and always renders the workflow runtime bar.
- The lx semantic registry and lowering path now exist, but they are disconnected from the live UI, which means the lx graph setup is not actually reachable or usable.
- A proper shared graph platform needs a host-side product model that owns templates, diagnostics, sample seeding, and product-specific chrome while reusing the shared editor crate.

## Changes
- Add a host-side flow product model that resolves workflow vs lx behavior from the current flow document or flow id.
- Seed both workflow and lx sample documents through the same persistence path.
- Switch template loading, credential loading, runtime affordances, and diagnostics recomputation from the host product model instead of hardcoded workflow modules.
- Expose an lx sample entry point in the workspace and surface lx compile-state feedback in the header.

## Files Affected
- `crates/lx-desktop/src/pages/flows/mod.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/sample.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/runtime_bar.rs`
- `crates/lx-desktop/src/pages/flows/validation.rs`
- `crates/lx-desktop/src/pages/flows/storage.rs`
- `crates/lx-desktop/src/pages/flows/` new host-product module(s)
- `crates/lx-desktop/assets/flows/` new lx sample asset

## Task List
1. Add a host-side flow product abstraction that can resolve the active product kind, templates, credentials, and diagnostics pipeline for workflow and lx graphs.
2. Seed and persist an lx sample document through the existing flow storage path, alongside the current workflow sample, without breaking current workflow defaults.
3. Update flow editor state to derive product configuration from the active document and use the appropriate diagnostics path, including lx compile feedback from the lowering layer.
4. Update the flow workspace chrome so users can reach both sample setups, see which product mode is active, and avoid workflow-only runtime controls when editing lx graphs.
5. Audit the end-to-end path so workflow behavior still works, lx lowering is no longer dead code, and the shared editor crate remains product-agnostic.

## Verification
- `cargo fmt --package lx-desktop --package lx-graph-editor`
- `cargo test -p lx-desktop graph_editor::lowering`
- `cargo test -p lx-desktop flows::`
- `cargo test -p lx-desktop --no-run`
- `cargo test -p lx-graph-editor`
