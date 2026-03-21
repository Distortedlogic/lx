# Surface-Aware Pane Toolbar

## Goal

Replace the current floating hover buttons (split right, split down, close) with a thin toolbar at the top of each pane that adapts its controls to the surface type. Terminal panes show working directory, Browser panes show URL and navigation, Editor panes show file path, Agent panes show model and session status, Canvas panes show widget type.

## Why

- The current hover buttons are identical for all pane types and lack surface-specific controls
- Browser panes need navigation (back/forward/refresh/URL bar) that does not belong as floating buttons
- Editor panes need file path and save status display
- A consistent toolbar location across all surface types reduces discoverability friction compared to scattered hover buttons

## What changes

- A new PaneToolbar component renders a 32px toolbar at the top of every pane, visible on hover (uses the existing group/group-hover pattern)
- Left side: surface type icon plus context label — working_dir for Terminal, URL for Browser, file_path basename for Editor, model for Agent, widget_type for Canvas
- Right side: universal action buttons (split right, split down, close) plus surface-specific buttons
- The toolbar renders in the Rust pane wrapper in terminals.rs, not inside individual TS widgets — keeps toolbar consistent and avoids duplicating UI across five widget implementations
- For Browser panes, navigation controls (back/forward/refresh) move from the TS-side toolbar into the Rust-rendered PaneToolbar — the TS widget's toolbar div is removed or hidden when the Rust toolbar handles navigation
- The current floating hover button div in terminals.rs is removed entirely

## Files affected

| File | Change |
|------|--------|
| `apps/desktop/src/terminal/toolbar.rs` | New file: PaneToolbar component with surface-specific rendering |
| `apps/desktop/src/terminal/mod.rs` | Export PaneToolbar |
| `apps/desktop/src/pages/workflow/terminals.rs` | Remove floating hover buttons; add PaneToolbar inside each pane wrapper div; change pane wrapper to flex column layout |

## Task List

### Task 1: Create PaneToolbar component

Create `apps/desktop/src/terminal/toolbar.rs`. The PaneToolbar component takes props: pane_node (PaneNode), on_split_h (EventHandler), on_split_v (EventHandler), on_close (EventHandler), and on_navigate (Option of EventHandler taking String — used by Browser panes). The component renders a div with class "flex items-center h-8 px-2 gap-1 bg-[var(--surface-container-low)] border-b border-[var(--color-border)] opacity-0 group-hover:opacity-100 transition-opacity text-xs shrink-0". Left section: match on pane_node variant. Terminal renders a span with "▸" in monospace and a span showing the working_dir truncated to the last two path components. Browser renders a span "🌐", three small buttons (← → ↻) that send navigation commands via on_navigate with "back"/"forward"/"refresh" as the string, and a text input for the URL (flex 1, background --surface-lowest, border --color-border, font-size 12px, padding 2px 6px, border-radius 3px) — on Enter, call on_navigate with the input value. Editor renders a span "◇" and a span showing the basename of file_path. Agent renders a span "●" and a span showing the model. Canvas renders a span "◻" and a span showing the widget_type. Right section (all variants): three buttons for split right ("⇥"), split down ("⇤"), and close ("×") — each with class "px-1.5 py-0.5 bg-[var(--surface-container-highest)]/80 rounded hover:bg-[var(--surface-bright)]", calling the corresponding EventHandler. Export PaneToolbar from terminal/mod.rs.

### Task 2: Integrate PaneToolbar into Terminals page

Edit `apps/desktop/src/pages/workflow/terminals.rs`. In the pane rendering loop, remove the entire "absolute top-1 right-1 z-10 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity" div that contains the current ⇥ ⇤ × buttons. Change the pane wrapper div class from "group absolute" to "group absolute flex flex-col" so toolbar and view stack vertically. Add a PaneToolbar component as the first child inside the pane wrapper, before the view component (TerminalView/BrowserView/etc.). Pass pane_node as the current PaneNode clone, wire on_split_h to the existing split_pane call with SplitDirection::Horizontal, on_split_v with SplitDirection::Vertical, on_close to close_pane. For Browser panes, pass on_navigate that sends a URL update message to the browser widget handle (this requires access to the widget handle — either lift it to shared state or pass a callback). For non-Browser panes, pass on_navigate as None. The view component div should have class "flex-1 min-h-0" added to allow it to fill remaining space below the toolbar.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
