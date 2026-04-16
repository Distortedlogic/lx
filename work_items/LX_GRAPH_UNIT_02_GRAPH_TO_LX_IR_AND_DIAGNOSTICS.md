# LX Graph Unit 02: Graph To LX IR And Semantic Diagnostics

## Goal
Compile the lx graph into lx intermediate structures and surface real semantic diagnostics in the graph editor so the graph becomes a true lx programming interface instead of only a structured diagram.

## Why
- Without a compile boundary, the graph is still only an editor shell. A real lx graphical programming environment must produce something the lx toolchain can reason about and execute.
- The shared graph editor already supports diagnostics visually, which makes it a good host for lx type and semantic errors once the graph can lower into lx IR or AST-like structures.
- This unit is the one that turns the lx graph path from “workflow-like UI” into “actual graphical programming.”

## Changes
- Add a graph-to-lx lowering layer in the lx product code.
- Feed checker or compiler diagnostics back into the shared graph diagnostics channel using entity-targeted node and edge errors.
- Update the lx product inspector or supporting surfaces to present semantic diagnostics and compile-state feedback.

## Files Affected
- lx product-side graph semantic modules to be added under the appropriate lx crate
- `crates/lx-graph-editor/src/protocol.rs`
- `crates/lx-graph-editor/src/dioxus.rs`
- `crates/lx-graph-editor/src/inspector.rs`

## Task List
1. Define the lowering boundary from graph document plus lx semantic registry into lx IR or another checker-consumable intermediate form.
2. Implement entity-targeted semantic diagnostics so lx checker results map back to graph nodes and edges.
3. Feed those diagnostics into the shared graph canvas and inspector surfaces without introducing lx-specific behavior into the shared crate.
4. Audit the end-to-end path so the graph can serve as a first-class lx authoring surface rather than a disconnected diagram editor.

## Verification
- `cargo fmt`
- targeted lx checker and graph-editor compile diagnostics once the lx host modules exist
