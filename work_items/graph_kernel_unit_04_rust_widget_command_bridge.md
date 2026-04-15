# Graph Kernel Unit 04: Rust widget command bridge

## Depends On
- `work_items/graph_kernel_unit_01_shared_editor_state_and_protocol.md`
- `work_items/graph_kernel_unit_02_flow_workspace_route_host.md`
- `work_items/graph_kernel_unit_03_widget_bridge_dag_editor_surface.md`

## Goal
Wire the flows workspace to the `dag-editor` widget so the page becomes a functioning editor instead of a shell with a placeholder canvas. This unit should make Rust own the canonical `GraphDocument`, translate committed widget events into reducer commands, and push fresh snapshots back into the widget after every accepted edit.

## Why
- The widget surface is not useful until it is attached to real Dioxus state and the shared reducer.
- `CanvasView` in `crates/lx-desktop/src/terminal/view.rs` currently drops widget messages on the floor, which is why the bridge needs to live in the flows route host first.
- The architecture only stays reusable if the widget emits user intents and Rust decides whether they become document mutations.
- Selection and properties-panel state need to move through the same authoritative controller so later inspector work does not bolt on a second state system.

## Changes
- Replace the central placeholder region from unit 02 with a dedicated host component that calls `use_ts_widget("dag-editor", ...)` directly.
- Add snapshot construction helpers that translate `GraphDocument`, workflow templates, and validation summaries into the unit 01 `GraphWidgetSnapshot`. Until unit 05 lands, the validation list can be an explicit empty placeholder rather than a deferred TODO.
- Add widget-event handling that maps `GraphWidgetEvent` values into `GraphCommand` reducer calls. Ignore or surface rejected events explicitly; do not let the widget mutate canonical state by itself.
- Keep transient pointer-preview state in the widget and only commit document changes on discrete events such as selection, delete-selection, completed node drag, completed viewport move, and completed edge creation.
- Open and close the shell properties panel based on the current graph selection so later inspector work has the right shell behavior already in place.
- Update the flows workspace status strip to show live selection and validation counts sourced from the real controller state.

## Files Affected
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/mod.rs`
- `crates/lx-desktop/src/contexts/panel.rs`

## Task List
1. In the flows controller, add helpers that convert the current graph document and workflow template metadata into a `GraphWidgetSnapshot` suitable for the TS widget.
2. Replace the unit 02 canvas placeholder with a dedicated host component that mounts `use_ts_widget("dag-editor", initial_config)` directly from the flows route instead of going through `CanvasView`.
3. Add the async receive loop that listens for widget events, translates them into `GraphCommand` reducer calls, and re-sends a fresh snapshot after every successful mutation.
4. Keep Rust authoritative over committed state. The widget should never be treated as the source of truth for nodes, edges, selection, or viewport.
5. Update flow-page selection handling so selecting a node or edge from the widget updates controller state and toggles the shell properties panel open when appropriate.
6. Surface rejected reducer operations in the flow workspace status strip or a lightweight toast path so protocol or validation mistakes do not fail silently during editing.

## Verification
- Run `just fmt`.
- Run `just rust-diagnose`.
- Run `just ts-diagnose`.
- Run `just desktop`.
- In the running app, open `/flows`, select nodes, drag nodes, pan and zoom the canvas, and create at least one edge. Confirm every committed action is reflected after the next snapshot and survives a page re-render.
