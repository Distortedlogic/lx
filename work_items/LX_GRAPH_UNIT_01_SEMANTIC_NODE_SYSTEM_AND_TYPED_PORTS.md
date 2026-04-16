# LX Graph Unit 01: Semantic Node System And Typed Ports

## Goal
Introduce an lx-native graph vocabulary and replace the current stringly port typing with a type model that can represent real lx semantics rather than only workflow-demo data tags.

## Why
- The current graph data model uses free-form port type strings like `"topics"` and `"articles"`, which is enough for a demo workflow but not enough for a graphical programming environment.
- A proper lx graph surface needs node kinds and port types that correspond to actual lx concepts, not just product-specific steps.
- This is the point where the lx product diverges from the n8n product in semantics while still reusing the same graph editing surface.

## Changes
- Define an lx-native node registry and richer port type model.
- Update shared graph validation interfaces so product-specific semantic validation can reason over structured types instead of opaque strings.
- Keep generic graph rendering in `lx-graph-editor`, but let the lx product own the semantic registry and validation policy.

## Files Affected
- `crates/lx-graph-editor/src/catalog.rs`
- `crates/lx-graph-editor/src/commands.rs`
- `crates/lx-graph-editor/src/model.rs`
- lx semantic host modules to be added under an appropriate lx product crate

## Task List
1. Replace or extend the current stringly `data_type` model with a structured port-type representation that can support lx semantics.
2. Define a product-side lx node registry that maps node kinds and fields to actual lx concepts.
3. Update shared connection validation to use the richer port-type model.
4. Audit the shared crate so it remains graph-generic while supporting richer product-level type information.

## Verification
- `cargo fmt --package lx-graph-editor`
- `cargo test -p lx-graph-editor`

