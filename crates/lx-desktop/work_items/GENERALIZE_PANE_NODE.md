# Generalize PaneNode for Heterogeneous Surfaces

## Goal

Extend the PaneNode enum from terminal-only to support five leaf types: Terminal (existing), Browser, Editor, Agent, and Canvas. Update all tree operations and rendering dispatch so any surface type can occupy any position in the binary split tree. This is the structural foundation for all subsequent heterogeneous pane work items.

## Why

- PaneNode currently has only Terminal and Split variants — the entire pane system is hard-coded to terminals
- The widget bridge (use_ts_widget) is already surface-agnostic but tree_ops.rs and the Terminals page hard-code terminal assumptions (all_terminal_ids, the four-element tuple from compute_pane_rects, split_pane always creating PaneNode::Terminal)
- Every subsequent pane surface work item depends on this structural change
- Tree operations like all_terminal_ids and find_working_dir assume all leaves are terminals, causing exhaustive match failures as soon as a new variant is added

## What changes

**types.rs:**

- Four new PaneNode variants: Browser with id (String) and url (String); Editor with id (String), file_path (String), and language (Option of String); Agent with id (String), session_id (String), and model (String); Canvas with id (String), widget_type (String), and config (serde_json::Value)
- A PaneKind enum with five unit variants (Terminal, Browser, Editor, Agent, Canvas) for APIs that reference pane type without carrying full variant data
- A pane_id method on PaneNode returning the id String reference for any leaf variant (None for Split)
- A pane_kind method on PaneNode returning Option of PaneKind (None for Split)

**tree_ops.rs:**

- Rename all_terminal_ids to all_pane_ids — collect ids from all five leaf variants
- Update compute_pane_rects to return Vec of (PaneNode, Rect) tuples instead of the current (String, String, Option of String, Rect) terminal-specific tuple
- Update find_working_dir: Terminal returns its working_dir, Editor returns the parent directory of file_path, Browser/Agent/Canvas return None
- Ensure every match on PaneNode is exhaustive with no wildcard fallthrough

**terminals.rs (page):**

- The pane rendering loop match-dispatches on PaneNode variant to create TerminalView, BrowserView, EditorView, AgentView, or CanvasView
- Four new stub placeholder components (simple div with pane type label and id) — actual implementations come in subsequent work items
- split_pane creates a new leaf matching the source pane's kind instead of always creating Terminal

## Files affected

| File | Change |
|------|--------|
| `apps/desktop/src/terminal/types.rs` | Add Browser, Editor, Agent, Canvas variants; add PaneKind enum; add pane_id and pane_kind methods |
| `apps/desktop/src/terminal/tree_ops.rs` | Rename all_terminal_ids to all_pane_ids; change compute_pane_rects return type; exhaustive matches everywhere |
| `apps/desktop/src/pages/workflow/terminals.rs` | Match-dispatch rendering by variant; update split_pane; add stub view components |
| `apps/desktop/src/terminal/mod.rs` | Export new types and stub components |
| `apps/desktop/src/terminal/view.rs` | Add stub BrowserView, EditorView, AgentView, CanvasView components |

## Task List

### Task 1: Add new PaneNode variants and PaneKind enum

Edit `apps/desktop/src/terminal/types.rs`. Add four new variants to PaneNode: Browser with fields id (String) and url (String); Editor with fields id (String), file_path (String), and language (Option of String); Agent with fields id (String), session_id (String), and model (String); Canvas with fields id (String), widget_type (String), and config (serde_json::Value). Add serde_json to the imports. Add a PaneKind enum with unit variants Terminal, Browser, Editor, Agent, Canvas — derive Clone, Copy, Debug, PartialEq, Eq. Add a pane_id method on PaneNode that matches each leaf variant and returns Some with a reference to the id field, returning None for Split. Add a pane_kind method returning Option of PaneKind — Terminal returns Some(PaneKind::Terminal), Browser returns Some(PaneKind::Browser), and so on, Split returns None.

### Task 2: Update tree operations for generic leaf handling

Edit `apps/desktop/src/terminal/tree_ops.rs`. Rename all_terminal_ids to all_pane_ids and update its match to collect ids from all five leaf variants using the pane_id method. Update compute_pane_rects: change the return type from the current terminal-specific tuple to Vec of (PaneNode, Rect) where PaneNode is cloned for each leaf. Update find_working_dir: match all five variants — Terminal returns Some of its working_dir clone, Editor returns Some of the parent directory of file_path (use std::path::Path::parent and convert to string), Browser/Agent/Canvas return None. Split recurses into children as before. Ensure every other match on PaneNode in this file is exhaustive — no wildcard arms.

### Task 3: Create stub view components

Edit `apps/desktop/src/terminal/view.rs` (or create separate files if view.rs would exceed 200 lines). Add four stub components: BrowserView with props browser_id (String) and url (String); EditorView with props editor_id (String), file_path (String), and language (Option of String); AgentView with props agent_id (String), session_id (String), and model (String); CanvasView with props canvas_id (String), widget_type (String), and config (serde_json::Value). Each stub renders a div with class "flex items-center justify-center h-full text-muted-foreground" containing a paragraph showing the pane type name and id (for example "Browser: " followed by the browser_id). Export all four from the terminal module in mod.rs.

### Task 4: Update Terminals page rendering dispatch

Edit `apps/desktop/src/pages/workflow/terminals.rs`. The pane rendering loop iterates over compute_pane_rects results which now return (PaneNode, Rect) tuples. Match on the PaneNode variant: Terminal destructures to get id, working_dir, command and renders TerminalView (existing behavior); Browser destructures to get id, url and renders BrowserView; Editor destructures to id, file_path, language and renders EditorView; Agent destructures to id, session_id, model and renders AgentView; Canvas destructures to id, widget_type, config and renders CanvasView. The pane wrapper div (absolute positioning, click handler for focus, group class) remains the same for all variants — only the inner view component changes. Update the existing split_pane function: instead of always creating PaneNode::Terminal, read the kind of the focused pane and create a new leaf of the same kind with a fresh UUID. Terminal gets the same working_dir; Browser gets the same url; Editor gets an empty file_path; Agent gets a new session_id UUID and same model; Canvas gets the same widget_type and empty config.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
