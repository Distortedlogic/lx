# Mermaid Pi Desktop Unit 01: MMD Product Mode And Graph Roundtrip

## Goal
Add a Mermaid-backed flow product mode in `lx-desktop` whose canonical on-disk format is `.mmd`, parse a constrained Mermaid flowchart subset into a dedicated Mermaid domain model, and round-trip that model through the shared graph editor without making raw `GraphDocument` the source of truth.

## Why
- The current `flows` host only understands JSON-backed workflow and lx graph documents.
- The requested stepping stone needs Mermaid charts to exist both as `.mmd` files and as a Dioxus graph-editor surface.
- Pi-backed execution will need a stable chart model later; storing Mermaid state implicitly in generic graph labels and ports would create knotty follow-on work.
- The shared graph editor should stay a renderer and interaction layer, not become a Mermaid parser.

## Changes
- Add `FlowProductKind::Mermaid` to the `flows` host and tag Mermaid documents explicitly so product inference is stable.
- Add a `pages/flows/mermaid/` host module that owns Mermaid parsing, normalized emission, validation, graph mapping, and Mermaid-specific node templates.
- Use an established Mermaid parser crate for the supported subset, but emit normalized `.mmd` from the app-owned Mermaid domain model instead of trying to preserve original formatting.
- Persist Mermaid flows as `.mmd` files under the existing flow storage root while leaving workflow and lx graph persistence on their current JSON path.
- Seed one Mermaid mock-lx sample and expose it in the workspace alongside the existing workflow and lx sample entries.

## Files Affected
- `Cargo.toml` or `crates/lx-desktop/Cargo.toml`
- `crates/lx-desktop/src/pages/flows/mod.rs`
- `crates/lx-desktop/src/pages/flows/product.rs`
- `crates/lx-desktop/src/pages/flows/storage.rs`
- `crates/lx-desktop/src/pages/flows/sample.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/` new `mermaid/` module(s)
- `crates/lx-desktop/assets/flows/` new Mermaid sample asset

## Task List
1. Add `FlowProductKind::Mermaid` to `crates/lx-desktop/src/pages/flows/product.rs` and update host product resolution so Mermaid documents are detected from explicit document metadata or storage format, not by empty-canvas guesswork. Mermaid mode must have its own labels, palette copy, empty-state copy, diagnostics title, and runtime support flag.
2. Add a dedicated Mermaid domain model under `crates/lx-desktop/src/pages/flows/mermaid/`. Do not use raw `GraphDocument` as the canonical Mermaid representation. The model must carry at least chart direction, nodes, edges, subgraphs, semantic kind, display labels, and edge labels.
3. Add Mermaid parsing and validation for a strict supported subset. Use an established parser crate such as `mmdflux` for detection and parse support, then convert into the app-owned Mermaid model. The supported subset for this unit is:
   - `flowchart TD` and `flowchart LR`
   - node ids with display labels
   - directed edges `-->` with optional edge labels
   - `subgraph ... end`
   - node semantic classes from this fixed set: `step`, `agent`, `decision`, `tool`, `io`
   Any Mermaid construct outside that subset must surface diagnostics instead of being silently dropped.
4. Add a normalized Mermaid emitter for the supported subset. Saving a Mermaid flow must rewrite canonical `.mmd` text from the Mermaid domain model. The save path does not preserve original whitespace, comments, or custom class styling.
5. Add Mermaid-to-graph and graph-to-Mermaid mapping. Introduce Mermaid-specific graph node templates with stable template ids and generic in/out ports so the shared graph editor can render and edit the chart. Mermaid-specific state must live in explicit fields or metadata; do not hide it in display labels.
6. Update `crates/lx-desktop/src/pages/flows/storage.rs` so Mermaid flows load from and save to `.mmd` files under the existing flow storage root, while workflow and lx flows continue using `.json`. The repository must reopen Mermaid files as Mermaid product documents without requiring manual mode switching.
7. Seed a Mermaid sample asset such as `crates/lx-desktop/assets/flows/mermaid-mock-lx.mmd`, load it through the same persistence path as other samples, and surface it in the workspace sample controls.
8. Extend diagnostics and tests to cover parse failures, unsupported Mermaid constructs, `.mmd` save/load roundtrip, Mermaid-to-graph roundtrip, subgraph preservation, semantic class preservation, and edge-label preservation.
9. `crates/lx-desktop/src/pages/flows/workspace.rs` already exceeds the repo file-size rule. If this unit needs more than a minimal touch there, split workspace chrome into new `workspace/` submodules before adding Mermaid-specific UI so no touched file ends above 300 lines.

## Verification
- `just fmt`
- `just rust-diagnose`
- `just test`
