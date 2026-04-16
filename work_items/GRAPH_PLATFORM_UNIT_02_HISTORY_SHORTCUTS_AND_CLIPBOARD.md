# Graph Platform Unit 02: History, Shortcuts, And Clipboard Editing

## Goal
Add shared editing fundamentals to `lx-graph-editor` so both the n8n-style workflow builder and the lx graphical programming environment get the same baseline power-user editing model.

## Why
- The current editor supports drag, connect, select, delete, and fit-view, but it does not yet provide undo/redo, copy/paste, duplicate, select-all, or box select.
- Those interactions are standard expectations for both workflow builders and graphical programming editors.
- If these capabilities are implemented separately per product, command history and selection semantics will fork early and become harder to stabilize.

## Changes
- Introduce shared graph editing capabilities in `lx-graph-editor` for:
  - undo/redo
  - copy/paste and duplicate selection
  - select-all
  - marquee or box selection
  - keyboard shortcuts for the above
- Keep the underlying operations command-based and reusable from multiple hosts.
- Avoid introducing workflow-specific assumptions about node categories, execution, or persistence.

## Files Affected
- `crates/lx-graph-editor/src/lib.rs`
- `crates/lx-graph-editor/src/commands.rs`
- `crates/lx-graph-editor/src/dioxus.rs`
- `crates/lx-graph-editor/src/history.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`

## Task List
1. Add a shared history module in `lx-graph-editor` that can record and replay graph mutations without baking in desktop-only state.
2. Extend the shared Dioxus editor surface to support keyboard-first history and clipboard actions.
3. Implement marquee selection in the shared canvas rather than in a flow-specific wrapper.
4. Update the flow controller and workspace to host the shared history state and connect it to the existing command dispatch path.
5. Audit the result to ensure the shortcut and history behavior is not wired through flow-specific one-off logic.

## Verification
- `cargo fmt --package lx-graph-editor --package lx-desktop`
- `cargo test -p lx-graph-editor`
- `cargo test -p lx-desktop --no-run`

