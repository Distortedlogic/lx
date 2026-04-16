# N8N Graph Unit 02: Expressions, Connectors, And Credentials

## Goal
Turn the workflow graph into a practical n8n-style automation product by adding connector-backed nodes, expression or mapping support, and credential-aware configuration surfaces.

## Why
- Run-state UX alone does not produce an n8n-class system. The product also needs real integration nodes and a way to map data between steps.
- The current node catalog is a demo workflow vocabulary. A serious workflow product needs a scalable connector and credential model.
- Expression-driven field values and data mapping are one of the main reasons users adopt a flow-automation tool over a static graph editor.

## Changes
- Define a connector-backed node registry strategy for the workflow product.
- Add schema support for credential-bound fields and expression-capable fields.
- Extend the shared inspector/forms surface only where the behavior is generic, and keep connector resolution plus secret lookup in the workflow product layer.
- Replace the current fixed sample-oriented workflow catalog with a host-driven registry that can grow into real connector packs.

## Files Affected
- `crates/lx-graph-editor/src/catalog.rs`
- `crates/lx-graph-editor/src/inspector.rs`
- `crates/lx-desktop/src/pages/flows/catalog.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/inspector.rs`
- additional workflow-host connector and credential modules to be added under `crates/lx-desktop/src/pages/flows/`

## Task List
1. Design and implement a host-driven workflow node registry that can supply connector metadata into the shared graph editor.
2. Extend the field schema model to represent expression-enabled values and credential-bound inputs where those concepts are generic.
3. Add the workflow-host logic for connector discovery, credential lookup, and secure field population.
4. Replace the current fixed sample catalog path with the host-driven registry while retaining a sample pack for local development.
5. Audit the result so credential and connector semantics are not leaked into the shared graph core.

## Verification
- `cargo fmt --package lx-graph-editor --package lx-desktop`
- `cargo test -p lx-graph-editor`
- `cargo test -p lx-desktop --no-run`

