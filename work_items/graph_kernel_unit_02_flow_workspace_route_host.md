# Graph Kernel Unit 02: Flow workspace route host

## Depends On
- `work_items/graph_kernel_unit_01_shared_editor_state_and_protocol.md`

## Goal
Add a route-first workflow editor workspace to `lx-desktop` that hosts the shared graph kernel and gives the DAG editor a real shell surface to grow inside. This unit should create the `/flows` entry point, wire it into navigation, and establish the page-level controller/context that later units will connect to the TS widget, inspector, palette, and persistence layer.

## Why
- `crates/lx-desktop/src/layout/shell.rs` renders routed pages through `LiveUpdatesProvider {}` as the main content surface; the pane tree is not the primary application shell today.
- `DesktopPane::Canvas` exists, but `CanvasView` is intentionally generic and currently ignores widget messages, which makes it the wrong first integration point for an editing surface with real state.
- The editor needs a stable home in the app before widget work starts, otherwise the TS work will be built against a placeholder host and have to be re-integrated later.
- Route-first integration matches the current app structure and keeps the DAG editor independent from future pane-system experiments.

## Changes
- Add a new `pages/flows` module with a route component, workspace shell, and in-memory sample document wiring built on top of `crate::graph_editor`.
- Add `/flows` and `/flows/:flow_id` routes to `crates/lx-desktop/src/routes.rs`.
- Export the new page module from `crates/lx-desktop/src/pages/mod.rs`.
- Add a `Flows` entry to `crates/lx-desktop/src/layout/sidebar.rs` and `crates/lx-desktop/src/components/command_palette.rs`.
- In the new flow workspace, create a page-level controller/context that owns `Signal<GraphDocument>`, the workflow node template list, selected entity state, and simple dispatch helpers that wrap the unit 01 reducer.
- Seed a sample in-memory graph so the page renders a meaningful workflow shell before persistence exists.
- Render a three-region layout now: left rail reserved for the palette, central canvas host region, and top toolbar/status strip. The central region can be a placeholder in this unit; the TS widget arrives later.
- Do not route the page through `CanvasView` yet. The host component should live directly in the flows page so later units can add real state handling instead of fighting the current generic canvas abstraction.

## Files Affected
- `crates/lx-desktop/src/pages/mod.rs`
- `crates/lx-desktop/src/pages/flows/mod.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/sample.rs`
- `crates/lx-desktop/src/routes.rs`
- `crates/lx-desktop/src/layout/sidebar.rs`
- `crates/lx-desktop/src/components/command_palette.rs`

## Task List
1. Add a new `pages/flows` module and export a route component that owns the flow workspace. Keep the module structure explicit from the start with `mod.rs`, `workspace.rs`, `controller.rs`, and `sample.rs`.
2. Add `/flows` and `/flows/:flow_id` to `crates/lx-desktop/src/routes.rs`, then wire the new page into `pages/mod.rs`, the sidebar, and the command palette.
3. In `controller.rs`, create the page-level state wrapper around the unit 01 graph kernel. It should expose the current `GraphDocument`, the workflow template catalog placeholder, the current selection, and a small dispatch API for later UI layers.
4. In `sample.rs`, build a hardcoded in-memory starter document that looks like a workflow instead of an abstract toy graph. Use stable ids so later units can write deterministic UI smoke tests against it.
5. In `workspace.rs`, render the route shell with a toolbar, a left rail reserved for node insertion, a central canvas host container, and lightweight status text for selection and validation counts. Keep the central region intentionally simple in this unit; it is just reserving the surface the widget will replace.
6. Keep this unit route-first. Do not modify `crates/lx-desktop/src/terminal/view.rs`, `panes.rs`, or `tab_bar.rs` yet.

## Verification
- Run `just fmt`.
- Run `just rust-diagnose`.
- Run `just desktop`.
- Open the app, navigate to `/flows`, and confirm the new page renders inside the existing shell with sidebar and command-palette navigation working.
