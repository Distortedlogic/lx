# Graph Kernel Unit 03: Widget-bridge DAG editor surface

## Depends On
- `work_items/graph_kernel_unit_01_shared_editor_state_and_protocol.md`
- `work_items/graph_kernel_unit_02_flow_workspace_route_host.md`

## Goal
Implement the `dag-editor` widget in `../dioxus-common/ts/widget-bridge` as the first concrete consumer of the shared graph editor kernel. This unit should build the visual editing surface itself: layered DOM/SVG rendering, local pointer interaction state, and typed widget events that match the unit 01 protocol.

## Why
- The TS widget is the reusable rendering/input surface that can later be mounted by the workflow editor and the future lx visual editor without duplicating canvas math.
- Existing widgets such as `agent.ts`, `markdown.ts`, and `json-viewer.ts` prove the widget bridge pattern, but none of them solve pan/zoom, edge rendering, or node editing interaction.
- The DAG editor will be too large for a single flat TS file. Unit 01 fixed build watching so this unit can use a maintainable multi-file widget structure instead of a monolith.
- Keeping drag previews and transient pointer state inside the widget avoids noisy Rust round-trips for every mousemove while still keeping Rust authoritative over committed state.

## Changes
- Add a `dag-editor` widget folder under `../dioxus-common/ts/widget-bridge/widgets/` instead of a single large file.
- Register the widget from `../dioxus-common/ts/widget-bridge/widgets/index.ts`.
- Follow the existing widget lifecycle from `../dioxus-common/ts/widget-bridge/src/registry.ts`: `mount`, `update`, optional `resize`, and `dispose`.
- Split the widget into explicit concerns such as registration, ephemeral UI state, rendering, and pointer interaction handling.
- Render the editor in layers: background grid, SVG edge layer, HTML node layer, and transient overlays for connection previews and selection affordances.
- Drive persistent visual state from `GraphWidgetSnapshot` only. The widget may keep ephemeral interaction state for dragging, viewport preview, and in-progress connection creation, but it must reconcile to the latest snapshot on every update.
- Emit typed `GraphWidgetEvent` payloads through `dx.send` for selection changes, node move commits, viewport commits, edge creation, and selection deletion. Send committed edits on pointer-up or gesture completion rather than flooding Rust with every intermediate move.

## Files Affected
- `../dioxus-common/ts/widget-bridge/widgets/index.ts`
- `../dioxus-common/ts/widget-bridge/widgets/dag-editor/index.ts`
- `../dioxus-common/ts/widget-bridge/widgets/dag-editor/state.ts`
- `../dioxus-common/ts/widget-bridge/widgets/dag-editor/render.ts`
- `../dioxus-common/ts/widget-bridge/widgets/dag-editor/events.ts`
- `../dioxus-common/ts/widget-bridge/widgets/dag-editor/types.ts`

## Task List
1. Create the new `widgets/dag-editor/` folder and register the widget from `widgets/index.ts`. Keep the registration entrypoint small and move actual logic into focused files.
2. In the widget `mount` path, create the layered editor DOM structure, inject local styles, and initialize ephemeral state. Reuse the lifecycle conventions already used by the existing widget-bridge widgets.
3. In `render.ts`, render the snapshot-driven scene: background grid, nodes with visible input/output ports, SVG edges, selection styling, and light-weight validation badges or error markers when the snapshot carries them.
4. In `events.ts`, implement pointer interactions for canvas pan, wheel zoom, node click selection, node drag preview, port-to-port connection preview, and canvas deselect. Keep these interactions local until the gesture commits.
5. In `types.ts`, mirror the unit 01 wire format exactly enough for type-safe widget code. Do not invent parallel payload shapes that drift from the Rust protocol.
6. Implement `update`, `resize`, and `dispose` so the widget fully tears down listeners and can survive repeated mount/unmount cycles without leaked DOM or stale state.

## Verification
- Run `cd ../dioxus-common/ts/widget-bridge && pnpm build`.
- Run `just ts-diagnose`.
- Confirm the widget bundle builds cleanly and the generated `dist/widget-bridge.js` includes the `dag-editor` registration without TypeScript errors.
