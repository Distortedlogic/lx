# Pane Type Conversion

## Goal

Allow converting an existing pane from one surface type to another without changing the layout tree. The pane slot stays in the same position within the binary split tree — only the leaf PaneNode variant is swapped. The pane toolbar's surface icon becomes a clickable dropdown for type switching.

## Why

- An agent starts in a terminal, discovers it needs a browser to verify — converting the pane in-place is faster than splitting, opening a new pane, and closing the old one
- Preserves spatial layout: the user's carefully arranged splits remain stable when they need a different surface in one slot
- Agents can convert panes programmatically (terminal → browser to show test results)

## What changes

- New tree operation convert_pane that replaces a leaf node while preserving its position in the tree
- New TabsState method convert_pane_in_active_tab wired through the state extension trait
- PaneToolbar's surface icon becomes clickable, opening a type selector dropdown that excludes the current type
- Conversion disposes the old TS widget (the component unmounts) and mounts a new one (the new component mounts)

## How it works

The convert operation is structurally simple: traverse the tree, find the leaf with matching id, replace it with a new PaneNode. The Dioxus rendering handles the rest — the old view component unmounts (disposing the TS widget), and the new view component mounts (creating the new TS widget). No explicit dispose/mount orchestration is needed because PaneNode is the source of truth for rendering dispatch.

## Files affected

| File | Change |
|------|--------|
| `apps/desktop/src/terminal/tree_ops.rs` | Add convert_pane method on PaneNode |
| `apps/desktop/src/terminal/types.rs` or extension trait file | Add convert_pane_in_active_tab on TabsState |
| `apps/desktop/src/terminal/toolbar.rs` | Make surface icon clickable with type selector dropdown |
| `apps/desktop/src/pages/workflow/terminals.rs` | Wire convert callback through to PaneToolbar |

## Task List

### Task 1: Add convert_pane tree operation

Edit `apps/desktop/src/terminal/tree_ops.rs`. Add a method convert on PaneNode that takes target_id (String reference) and replacement (PaneNode), returning a new PaneNode. For leaf variants: if pane_id matches target_id, return the replacement. Otherwise return self clone. For Split: recursively call convert on first and second children, constructing a new Split with the results. This follows the same recursive pattern as close but substitutes instead of collapsing.

### Task 2: Add convert_pane_in_active_tab to TabsState

Edit the TabsState extension trait (same file where split_active_pane and close_pane_in_active_tab are defined). Add a method convert_pane_in_active_tab that takes pane_id (String reference) and new_node (PaneNode). It finds the active tab, calls convert on its root with the pane_id and new_node, and updates the tab's root with the result.

### Task 3: Add conversion dropdown to PaneToolbar

Edit `apps/desktop/src/terminal/toolbar.rs`. Add an on_convert prop (EventHandler taking PaneNode) to PaneToolbar. Add a conversion_open signal (use_signal of bool). Make the surface icon span (▸/🌐/◇/●/◻) clickable — on click, toggle conversion_open. When conversion_open is true, render an absolutely-positioned dropdown below the icon (class "absolute top-full left-0 z-30 mt-1 py-1 bg-[var(--surface-container-high)] ghost-border rounded-md shadow-lg min-w-36"). List all five surface types except the current one (determined by pane_node.pane_kind()). Each entry is a button with the type icon and name. On click: construct a default PaneNode for the selected type — Terminal with working_dir "." and no command, Browser with url "about:blank", Editor with empty file_path and None language, Agent with new UUID session_id and "claude-sonnet-4-6" model, Canvas with "markdown" widget_type and empty object config. Call on_convert with the new PaneNode. Close the dropdown.

### Task 4: Wire convert through Terminals page

Edit `apps/desktop/src/pages/workflow/terminals.rs`. In the PaneToolbar instantiation, pass on_convert as a closure that calls tabs_state.convert_pane_in_active_tab with the current pane's id and the new PaneNode received from the toolbar.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
