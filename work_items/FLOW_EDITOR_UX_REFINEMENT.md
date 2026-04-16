# Flow Editor UX Refinement

## Goal
Refine the Dioxus flow editor surface so it reads as a finished primary workspace instead of a stack of internal-tool panels. The changes should shift visual weight toward the canvas, reduce dead space and duplicate status surfaces, improve first-load framing of the sample graph, and make node insertion and validation feel like secondary support systems rather than co-equal slabs.

## Why
- The screenshot shows the canvas still losing priority to surrounding chrome, especially the always-open node palette and the healthy-state validation slab.
- The graph is framed conservatively enough that the editor still feels undersized relative to the available space.
- The current canvas HUD mixes instructional text and viewport controls in the same lane, which creates clutter without improving usability.
- The palette cards still consume too much width and height for the amount of actionable information they provide.
- Healthy-state validation is over-reported in multiple places while still consuming vertical space better spent on editing.

## Changes
- Rework the flow workspace header in `crates/lx-desktop/src/pages/flows/workspace.rs` into a denser editor header with only high-value metadata visible by default, a clear primary save action, and no redundant healthy-state messaging.
- Replace the permanently heavy left palette column with a lower-weight node insertion surface that can stay collapsed by default while preserving fast access to search and template insertion.
- Rebalance the main editor body so the canvas gets more width and height in the common healthy-state case.
- Simplify the canvas HUD by separating persistent controls from ephemeral help text, reducing overlay crowding, and keeping viewport helpers subordinate to the graph.
- Tighten initial framing logic so the sample flow opens at a more assertive readable scale when the saved viewport under-fills the canvas.
- Remove the large healthy-state validation slab and show validation details only when there are actual issues, while retaining compact positive-state feedback elsewhere in the page.
- Continue polishing node and palette typography, spacing, and contrast where the screenshot still shows cramped or low-signal presentation.

## How It Works
- The workspace surface should carry one primary mode at a time: editing the graph. Supporting surfaces such as palette and validation should compress or hide when they are not actively needed.
- First-load framing should remain deterministic and local to the flow workspace. It should inspect the rendered scene dimensions against graph bounds and only auto-fit when the stored viewport clearly wastes space or leaves the graph badly framed.
- Validation should follow a severity-driven display model: compact in the healthy case, expanded only when diagnostics exist.
- Node insertion should stay available through the same controller path and viewport-center insertion logic that already exists. The UX change is presentation and hierarchy, not a new insertion mechanism.

## Files Affected
- `crates/lx-desktop/src/pages/flows/workspace.rs`
  Rework header hierarchy, palette presentation, canvas controls, validation surface, and remaining node/canvas styling.
- `crates/lx-desktop/src/pages/flows/controller.rs`
  Keep status messaging aligned with the refined UX so low-value load or interaction messages do not reintroduce noise.

## Task List
1. Rework the flow workspace shell in `crates/lx-desktop/src/pages/flows/workspace.rs` so the header is denser, lower-value metadata is demoted or removed, and the action cluster has a clear primary save affordance. Remove any remaining duplicated healthy-state messaging from the header.
2. Replace the current always-open palette column in `crates/lx-desktop/src/pages/flows/workspace.rs` with a lower-weight insertion surface that can stay collapsed by default while still supporting search, template scanning, and add-node actions through the existing controller.
3. Rebalance the canvas container and overlays in `crates/lx-desktop/src/pages/flows/workspace.rs` so the graph owns more of the available space, the viewport helper controls are grouped coherently, and the steady-state instructional pill no longer crowds the top of the canvas.
4. Tighten graph framing in `crates/lx-desktop/src/pages/flows/workspace.rs` by improving the auto-fit heuristics and fit padding so the sample graph opens larger and more intentionally framed when the stored viewport under-fills the scene.
5. Remove the large healthy-state validation slab in `crates/lx-desktop/src/pages/flows/workspace.rs`. Keep validation visible when diagnostics exist, but collapse the healthy case to either a compact signal or no separate panel at all.
6. Audit the final on-disk state of `crates/lx-desktop/src/pages/flows/workspace.rs` and `crates/lx-desktop/src/pages/flows/controller.rs` against this work item. Fix any remaining low-value metadata, oversized support surfaces, cramped palette affordances, or duplicated healthy-state messaging before running project diagnostics.

## Verification
- Run `cargo fmt --package lx-desktop`.
- Run `cargo test -p lx-desktop flows --no-run`.
- Run `cargo test -p lx-desktop`.
- Confirm the flow page no longer shows an always-heavy palette, no longer burns vertical space on empty validation, and opens with the graph framed more assertively than in the provided screenshot.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. Call `complete_task` after each task. The MCP handles formatting, committing, and diagnostics automatically.
2. Call `next_task` to get the next task. Do not look ahead in the task list.
3. Do not add tasks, skip tasks, reorder tasks, or combine tasks. Execute the task list exactly as written.
4. Tasks are implementation-only. No commit, verify, format, or cleanup tasks — the MCP handles these.
