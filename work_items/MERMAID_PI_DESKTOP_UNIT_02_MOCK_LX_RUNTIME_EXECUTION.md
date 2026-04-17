# Mermaid Pi Desktop Unit 02: Mock LX Runtime Execution

## Goal
Execute Mermaid product flows through the existing desktop runtime and Pi backend as grouped mock-lx runs, with one runtime session per Mermaid node, graph-level run projection in the Dioxus workspace, and native transcript and tool-activity inspection for the selected node session.

## Why
- Mermaid authoring alone does not satisfy the stepping-stone goal; the chart also needs to run.
- The desktop runtime registry, Pi backend adapter, and grouped `flow_run_id` support already exist and should be reused instead of bypassed.
- Execution semantics must stay lx-shaped and host-owned. Pi should remain only the backend adapter, and the graph UI should continue reading desktop runtime state rather than raw Pi payloads.

## Changes
- Add a Mermaid execution planner and orchestrator in the `flows` host that turns a Mermaid DAG into grouped runtime launches.
- Add runtime-controller helpers for beginning a grouped flow run and launching additional Pi-backed sessions into an existing `flow_run_id` without pushing Mermaid-specific logic into the runtime backend.
- Add Mermaid-specific run snapshot projection so the graph canvas reflects per-node runtime state instead of the current workflow-only heuristic.
- Extend the flow runtime surface so Mermaid mode can launch a run, inspect grouped sessions, and open the existing Pi widget for the selected node session.

## Files Affected
- `crates/lx-desktop/src/runtime/types.rs`
- `crates/lx-desktop/src/runtime/registry.rs`
- `crates/lx-desktop/src/runtime/controller.rs`
- `crates/lx-desktop/src/pages/flows/product.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/pages/flows/runtime_bar.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/mermaid/` new runtime module(s)
- `crates/lx-desktop/src/widgets/pi_widget.rs` only if a Mermaid-specific open-session affordance needs a shared widget seam

## Task List
1. Add a Mermaid execution-plan layer under `crates/lx-desktop/src/pages/flows/mermaid/` that converts the Mermaid domain model into a topologically ordered DAG plan. Each Mermaid node in the plan must carry a stable node id, semantic kind, display label, prompt inputs, dependency list, and one runtime launch spec.
2. Treat execution as DAG dependency scheduling only. Do not implement full Mermaid control-flow semantics in this unit. A node may launch only after every upstream dependency in the same run has completed successfully. Cycles must remain a validation failure. Unsupported control features must not be papered over with best-effort behavior.
3. Add runtime-controller support for grouped multi-session runs. The host layer must be able to create a `DesktopFlowRun` before launching nodes, then launch additional Pi-backed agents into the existing `flow_run_id`. Keep this API lx-shaped and generic. Do not add Mermaid-specific branching or prompt code inside `crates/lx-desktop/src/runtime/`.
4. When launching Mermaid node sessions, set `flow_id` and shared `flow_run_id` on every node session. Set `parent_id` deterministically from the first predecessor in topological order when a node has inputs; nodes with no predecessors are run roots for that grouped run.
5. Add Mermaid prompt shaping in the host layer. Each node prompt must include the chart title, node label, node semantic kind, node-local summary/prompt fields, direct predecessor labels, and predecessor outcome summaries. Predecessor outcome summary must come from desktop runtime state by preferring the final assistant `MessageComplete` text, then the latest tool result text, then the latest backend/tool error text.
6. Add a Mermaid orchestrator that watches runtime registry revisions and launches newly unblocked nodes as dependencies complete. If any predecessor errors or is aborted, downstream nodes must not launch and must render as blocked. Use `GraphRunStatus::Cancelled` with a detail explaining which predecessor blocked execution instead of inventing a new status enum.
7. Add Mermaid-specific graph run snapshots that map runtime sessions back onto Mermaid node ids. The canvas must show each node's own status from the grouped run, not a single root-agent approximation. Completed, running, failed, cancelled, and pending states must all come from desktop runtime state and grouped plan state.
8. Update the flow runtime surface so Mermaid mode shows a `Launch Mermaid Run` control, grouped runs by `flow_run_id`, per-run node/session chips, and the existing native Pi widget for the selected node session. Keep workflow runtime behavior intact; do not regress the current workflow product.
9. Add tests for plan generation order, deterministic `parent_id` selection, grouped flow-run registration, dependency scheduling, blocked downstream nodes after failure, and Mermaid run-snapshot projection onto graph node ids.
10. If this unit needs more Mermaid-specific chrome in `runtime_bar.rs` or `workspace.rs`, add new Mermaid runtime submodules instead of growing those files past the repo limit.

## Verification
- `just fmt`
- `just rust-diagnose`
- `just test`
