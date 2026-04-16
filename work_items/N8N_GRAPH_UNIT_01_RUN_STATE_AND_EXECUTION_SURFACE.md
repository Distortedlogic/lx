# N8N Graph Unit 01: Run State And Execution Surface

## Goal
Add execution-state primitives and runtime visualization to the shared graph editor and wire them into the workflow product so the graph can present step status, per-node results, and run-oriented feedback instead of only static graph structure.

## Why
- A proper n8n-like product is not just a static DAG authoring surface. Users expect to run flows and inspect what happened at each step.
- The current graph editor only exposes static validation and property editing. It does not model live run state, outputs, retries, or failure surfaces.
- This execution layer is the first meaningful product divergence from the lx graphical programming path.

## Changes
- Introduce run-state data structures that can be rendered by `lx-graph-editor` without assuming a specific backend transport.
- Extend the shared node rendering and inspector surface to show per-node execution status, timing, recent output summary, and failure state.
- Add a workflow-host layer in `lx-desktop` that can attach run-state snapshots to the current graph and present run-oriented controls.
- Preserve the generic graph editor as a renderer/interaction layer; workflow execution semantics remain in the product layer.

## Files Affected
- `crates/lx-graph-editor/src/protocol.rs`
- `crates/lx-graph-editor/src/dioxus.rs`
- `crates/lx-graph-editor/src/inspector.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/inspector.rs`

## Task List
1. Define shared run-state snapshot types for node status, edge activity, last-run metadata, and optional output summaries.
2. Render run-state badges and node-level execution affordances in the shared graph canvas without hardcoding workflow-only copy into the crate.
3. Extend the shared inspector to render run details when supplied by the host.
4. Add workflow-host state in `lx-desktop` that can populate and clear run-state snapshots for the active graph.
5. Audit the implementation to ensure the shared crate remains generic while the workflow product owns execution semantics.

## Verification
- `cargo fmt --package lx-graph-editor --package lx-desktop`
- `cargo test -p lx-graph-editor`
- `cargo test -p lx-desktop --no-run`

