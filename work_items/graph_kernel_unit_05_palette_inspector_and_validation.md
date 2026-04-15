# Graph Kernel Unit 05: Palette, inspector, and validation

## Depends On
- `work_items/graph_kernel_unit_01_shared_editor_state_and_protocol.md`
- `work_items/graph_kernel_unit_02_flow_workspace_route_host.md`
- `work_items/graph_kernel_unit_04_rust_widget_command_bridge.md`

## Goal
Make the DAG editor usable as a workflow builder instead of a raw graph toy by adding a workflow node catalog, node insertion palette, typed properties inspector, and validation surface. This is the unit that turns the shared graph kernel into the first n8n-style workflow authoring experience.

## Why
- A graph editor without insertion, editing, and validation only proves rendering; it does not prove the product direction.
- The workflow domain needs its own node catalog and validation rules, but those should sit on top of the shared kernel instead of leaking into it.
- `crates/lx-desktop/src/layout/properties_panel.rs` is still a stub that only prints `Panel: {id}`, so the editor cannot mature until the shell properties panel can host real flow controls.
- Validation must be visible while editing or the graph becomes a visually appealing but semantically ambiguous canvas.

## Changes
- Add a workflow-specific node catalog under `pages/flows` that instantiates the generic template types from unit 01.
- Add a left-side palette/search surface that inserts nodes using the controller and places them relative to the current viewport center instead of hardcoded coordinates.
- Replace the stringly shell panel placeholder with typed flow inspector content so the properties panel can render the selected node or edge.
- Add field editors driven by template schema metadata: text, multiline text, number, boolean, select, and list-like fields as needed by the initial workflow nodes.
- Add workflow validation rules in `pages/flows/validation.rs`. Cover at minimum: cycle detection, missing required inputs, missing required property values, duplicate node ids, and incompatible port connection types when the template metadata declares them.
- Show validation output in the flow workspace so users can see graph problems without opening dev tools.

## Files Affected
- `crates/lx-desktop/src/pages/flows/mod.rs`
- `crates/lx-desktop/src/pages/flows/catalog.rs`
- `crates/lx-desktop/src/pages/flows/inspector.rs`
- `crates/lx-desktop/src/pages/flows/validation.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/contexts/panel.rs`
- `crates/lx-desktop/src/layout/properties_panel.rs`

## Task List
1. Add `pages/flows/catalog.rs` and define the first workflow node catalog using the generic template system from unit 01. Include realistic starter nodes for the news/research workflow rather than abstract placeholder blocks.
2. Add palette UI to the flow workspace so users can browse and insert nodes. New nodes should spawn at the visible viewport center and become the active selection immediately.
3. Replace the current string-only panel targeting with a typed `PanelContent` enum in `crates/lx-desktop/src/contexts/panel.rs`, then implement `pages/flows/inspector.rs` for selected node and edge editing. The shell properties panel should render this component instead of a plain id string.
4. In the inspector, drive field editors from schema metadata instead of hardcoding per-node forms. Field updates must flow through the same controller and reducer path as every other edit.
5. Add `pages/flows/validation.rs` and compute validation results whenever the graph changes. Keep the validation engine workflow-specific, not part of the generic graph kernel.
6. Surface validation results in the flow workspace with enough detail to identify the broken node or edge directly from the editor.

## Verification
- Run `just fmt`.
- Run `just rust-diagnose`.
- Run `just desktop`.
- In the running app, insert multiple node types, edit their properties through the right-hand panel, intentionally create invalid graphs, and confirm the validation surface updates immediately with actionable problem messages.
