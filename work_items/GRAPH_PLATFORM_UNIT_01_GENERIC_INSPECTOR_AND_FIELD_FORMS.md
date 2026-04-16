# Graph Platform Unit 01: Generic Inspector And Field Forms

## Goal
Move the node and edge inspector UI, including schema-backed field editors, out of `lx-desktop` and into `lx-graph-editor` so both the n8n-style workflow product and the lx graphical programming product can reuse the same generic property-editing surface.

## Why
- The shared crate currently owns the graph core and canvas, but `crates/lx-desktop/src/pages/flows/inspector.rs` still contains the only node and edge inspector implementation.
- The current field editors are generic over `GraphFieldKind` already, which means this logic belongs in the shared graph package rather than in the workflow product.
- Leaving inspector/forms in the flow feature would force both n8n and lx graph products to either duplicate field editing or route everything through the flow-specific layer.

## Changes
- Add a reusable inspector module to `crates/lx-graph-editor` with:
  - a generic node inspector
  - a generic edge inspector
  - schema-backed field editors for every `GraphFieldKind`
  - generic diagnostic rendering for node and edge diagnostics
- Drive the shared inspector entirely from passed-in `GraphDocument`, `GraphNodeTemplate` registry, diagnostics, selected entity id, and `GraphCommand` dispatch callback.
- Keep workflow-specific shell concerns in `lx-desktop`, but replace the current flow inspector implementation with a thin adapter that passes flow state into the shared inspector component.
- Preserve current editing behavior:
  - updating a field emits `GraphCommand::UpdateField`
  - deleting a node emits `GraphCommand::RemoveNode`
  - deleting an edge emits `GraphCommand::DisconnectEdge`

## Files Affected
- `crates/lx-graph-editor/src/lib.rs`
- `crates/lx-graph-editor/src/inspector.rs`
- `crates/lx-desktop/src/pages/flows/inspector.rs`

## Task List
1. Add `crates/lx-graph-editor/src/inspector.rs` and export it from `crates/lx-graph-editor/src/lib.rs`.
2. Move the generic inspector/header/diagnostic/field-editor logic from `crates/lx-desktop/src/pages/flows/inspector.rs` into the new shared module.
3. Define a shared inspector component API that accepts:
   - `GraphDocument`
   - `Vec<GraphNodeTemplate>`
   - `Vec<GraphWidgetDiagnostic>`
   - selected node or edge id
   - `EventHandler<GraphCommand>` for edits
4. Refactor `crates/lx-desktop/src/pages/flows/inspector.rs` into a thin flow-specific adapter over the shared inspector component, keeping only the `PanelContent` mapping and local flow-state lookup.
5. Audit the final on-disk state so no schema-backed field editor implementation remains in `lx-desktop`.

## Verification
- `cargo fmt --package lx-graph-editor --package lx-desktop`
- `cargo test -p lx-graph-editor`
- `cargo test -p lx-desktop --no-run`
- `rg -n "FlowFieldEditor|GraphFieldKind::Text|GraphFieldKind::TextArea|GraphFieldKind::Number|GraphFieldKind::Integer|GraphFieldKind::Boolean|GraphFieldKind::Select|GraphFieldKind::StringList" crates/lx-desktop/src/pages/flows/inspector.rs`

