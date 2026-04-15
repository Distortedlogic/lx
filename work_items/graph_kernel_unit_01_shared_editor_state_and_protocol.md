# Graph Kernel Unit 01: Shared editor state and protocol

## Depends On
- None

## Goal
Create a reusable Rust-side graph editor kernel for `lx-desktop` before any workflow-specific page or widget logic lands. This unit defines the canonical graph document model, command reducer, template metadata, and Rust/TS widget protocol so the first workflow DAG editor and the later lx visual editor can share the same editing core without sharing workflow semantics.

## Why
- `lx-desktop` has no generic graph editing state today; the existing graph-like UI in `pages/org/chart.rs` is a page-specific layout and cannot be reused as an editor kernel.
- `crates/lx-desktop/src/terminal/view.rs` mounts arbitrary TS widgets, but `CanvasView` currently ignores widget messages, so the graph editor needs an explicit protocol instead of improvised message payloads.
- `crates/lx-desktop/build.rs` only watches top-level `.ts` files in `../dioxus-common/ts/widget-bridge`, which will miss changes once the DAG widget is split into multiple files.
- Locking the model and protocol first keeps Rust as the source of truth and the TS widget as a rendering/input surface, which is the right split for a reusable kernel.

## Changes
- Add `crates/lx-desktop/src/graph_editor/mod.rs` exporting `model`, `catalog`, `commands`, and `protocol`.
- In `model.rs`, define `GraphDocument`, `GraphNode`, `GraphEdge`, `GraphPortRef`, `GraphViewport`, `GraphSelection`, and lightweight metadata structs. Keep these types domain-neutral; nodes should carry `template_id`, position, optional label override, and generic JSON-backed properties.
- In `catalog.rs`, define generic template metadata used by any graph domain: `GraphNodeTemplate`, `GraphPortTemplate`, `GraphFieldSchema`, `GraphFieldKind`, `PortDirection`, and helpers for default property materialization.
- In `commands.rs`, define `GraphCommand` plus pure reducer functions for add/remove/move/select/connect/disconnect/set_viewport/update_field. Invalid operations should return typed errors instead of mutating silently.
- In `protocol.rs`, define the Rust-side wire types for `GraphWidgetSnapshot` and `GraphWidgetEvent`. Use one full-snapshot update payload from Rust to TS and a small user-intent event set from TS to Rust.
- Export the new module from `crates/lx-desktop/src/lib.rs`.
- Update `crates/lx-desktop/build.rs` to recurse through `../dioxus-common/ts/widget-bridge/src/**`, `widgets/**`, and `package.json` so nested DAG widget files trigger a rebuild reliably.
- Add focused reducer/protocol tests covering add-node, move-node, connect ports, reject invalid connects, delete selection, and snapshot serialization.

## Files Affected
- `crates/lx-desktop/src/lib.rs`
- `crates/lx-desktop/src/graph_editor/mod.rs`
- `crates/lx-desktop/src/graph_editor/model.rs`
- `crates/lx-desktop/src/graph_editor/catalog.rs`
- `crates/lx-desktop/src/graph_editor/commands.rs`
- `crates/lx-desktop/src/graph_editor/protocol.rs`
- `crates/lx-desktop/build.rs`

## Task List
1. Add the new `graph_editor` module and define the graph document types, selection model, viewport model, and generic JSON-backed node properties without introducing workflow execution semantics or lx-specific language semantics.
2. Add template/catalog types and default-property helpers so later workflow nodes can be declared from metadata rather than hardcoded per-node behavior in the widget.
3. Implement `GraphCommand` and pure reducer functions that apply edits against `GraphDocument` and return typed errors for duplicate ids, unknown nodes, unknown ports, incompatible port directions, and invalid selection targets.
4. Define the Rust-side widget protocol around one authoritative snapshot payload and a compact user-intent event set. Keep the wire format stable and document each event directly beside its type definition.
5. Update `crates/lx-desktop/build.rs` to recurse through the widget-bridge source tree instead of only top-level `.ts` files so the DAG widget can safely live in multiple files.
6. Add reducer and protocol tests inside the new module. Cover at minimum: creating a node from a template, moving a node, connecting and disconnecting edges, rejecting an invalid connection, deleting the current selection, and serializing a snapshot payload.

## Verification
- Run `just fmt`.
- Run `cargo test -p lx-desktop graph_editor`.
- Run `just rust-diagnose`.
- Confirm the new graph kernel tests pass and the desktop crate still type-checks cleanly.
