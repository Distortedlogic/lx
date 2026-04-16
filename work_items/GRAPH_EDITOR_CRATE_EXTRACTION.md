# Graph Editor Crate Extraction

## Goal
Extract the reusable DAG editor implementation out of `lx-desktop` into its own generic workspace crate so the same graph model and Dioxus editor surface can back both n8n-like workflow builders and an lx-native graphical programming surface.

## Why
- The current graph editor core is only an internal module of `lx-desktop`, so it is not a real package boundary and cannot be consumed cleanly by multiple products.
- The graph model, commands, protocol, and interactive canvas are generic enough to share, while the current flow page chrome, persistence, validation policy, and inspector are workflow-specific.
- Leaving the generic editor inside the desktop app guarantees future coupling and makes it harder to evolve n8n-style workflow UX and lx graph-programming UX separately.

## Changes
- Create a new workspace crate `crates/lx-graph-editor`.
- Move the generic graph core from `crates/lx-desktop/src/graph_editor/` into the new crate:
  - `catalog.rs`
  - `commands.rs`
  - `model.rs`
  - `protocol.rs`
- Add a reusable Dioxus canvas component to the new crate that owns:
  - node dragging
  - pan and zoom
  - edge creation preview
  - edge and node selection
  - fit-view logic
  - graph scene rendering
- Keep flow-specific concerns in `lx-desktop`:
  - workflow template catalog
  - flow persistence
  - flow inspector
  - workflow validation
  - page header and palette copy
  - status-message wording
- Update `lx-desktop` to depend on `lx-graph-editor` and consume the extracted crate from the flows page.
- Remove the old internal `graph_editor` module export from `lx-desktop` once all imports are moved.

## Files Affected
- `Cargo.toml`
- `crates/lx-graph-editor/Cargo.toml`
- `crates/lx-graph-editor/src/lib.rs`
- `crates/lx-graph-editor/src/catalog.rs`
- `crates/lx-graph-editor/src/commands.rs`
- `crates/lx-graph-editor/src/model.rs`
- `crates/lx-graph-editor/src/protocol.rs`
- `crates/lx-graph-editor/src/dioxus.rs`
- `crates/lx-desktop/Cargo.toml`
- `crates/lx-desktop/src/lib.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/inspector.rs`
- `crates/lx-desktop/src/pages/flows/catalog.rs`
- `crates/lx-desktop/src/pages/flows/sample.rs`
- `crates/lx-desktop/src/pages/flows/storage.rs`
- `crates/lx-desktop/src/pages/flows/validation.rs`

## Task List
1. Add `crates/lx-graph-editor` to the workspace and create its package manifest with the exact dependencies needed for the generic graph core and Dioxus canvas.
2. Move the existing graph core modules from `lx-desktop` into `crates/lx-graph-editor/src/` without changing their behavior. Preserve the existing command, model, and protocol tests by relocating them into the new crate.
3. Extract the generic interactive graph scene from `crates/lx-desktop/src/pages/flows/workspace.rs` into `crates/lx-graph-editor/src/dioxus.rs` as a reusable component that accepts graph document data, templates, diagnostics, scene size callbacks, and a command-dispatch callback rather than reading flow-specific context directly.
4. Refactor `crates/lx-desktop/src/pages/flows/workspace.rs` to use the extracted component while keeping the flow page header, node palette, and validation surface local to the flow feature.
5. Update all flow feature imports in `controller.rs`, `inspector.rs`, `catalog.rs`, `sample.rs`, `storage.rs`, and `validation.rs` to consume graph types from `lx_graph_editor` instead of `crate::graph_editor`.
6. Remove the old `crates/lx-desktop/src/graph_editor/` module from the desktop crate only after all call sites compile against the new crate.
7. Audit the final on-disk state to ensure the new crate boundary is real:
   - generic graph core and canvas live in `crates/lx-graph-editor`
   - `lx-desktop` contains only flow-specific product code
   - no remaining `crate::graph_editor` imports exist in `lx-desktop`
   - no desktop-only context dependency leaked into the new crate

## Verification
- `cargo fmt`
- `cargo test -p lx-graph-editor`
- `cargo test -p lx-desktop --no-run`
- `rg -n "crate::graph_editor" crates/lx-desktop/src`
