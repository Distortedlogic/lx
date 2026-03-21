# Tab Bar Surface Type Dropdown

## Goal

Add a dropdown menu to the tab bar's "+" button that lets users choose which surface type to open in a new tab. Left-click on "+" still creates a terminal tab (fast path). Right-click opens a dropdown offering all five surface types with inline prompts for type-specific configuration (URL for browser, file path for editor, model for agent, widget type for canvas).

## Why

- Currently the only way to create non-terminal panes is through the split_pane function which copies the source pane's type — there is no direct UI for opening a browser/editor/agent tab from scratch
- The "+" button is the natural location for surface type selection
- Left-click defaulting to terminal preserves the fast path for the most common action

## What changes

- The "+" button in TabBar gets a right-click (oncontextmenu) handler that opens a dropdown menu
- The dropdown lists all five surface types with icons matching the pane toolbar convention
- Browser shows an inline URL input on selection, Editor shows a file path input, Agent shows a model input (defaulting to claude-sonnet-4-6), Canvas shows a sub-list of available widget types (log-viewer, markdown, json-viewer)
- A generalized add_tab helper replaces the terminal-specific add_terminal_tab for dropdown-initiated creation

## Files affected

| File | Change |
|------|--------|
| `apps/desktop/src/terminal/tab_bar.rs` | Add dropdown state, dropdown rendering with surface type buttons and inline inputs |
| `apps/desktop/src/pages/workflow/terminals.rs` | Generalize create_new_tab or add a new create_tab_with_pane function that accepts a PaneNode |
| `apps/desktop/src/terminal/types.rs` | Update add_terminal_tab to a generic add_tab accepting PaneNode, or add a new function alongside it |

## Task List

### Task 1: Generalize tab creation function

Edit the terminal module (types.rs or wherever add_terminal_tab is defined). Add a new function add_tab that takes tabs_state (TabsStateSignal), id (String), title (String), and root (PaneNode). It creates a TerminalTab with those fields, pushes it onto the tabs vec, and sets it as the active tab. Keep add_terminal_tab as a convenience wrapper that constructs PaneNode::Terminal and calls add_tab. Update create_new_tab in `apps/desktop/src/pages/workflow/terminals.rs` to call add_tab via add_terminal_tab (no behavior change, just uses the new plumbing).

### Task 2: Add dropdown menu to TabBar

Edit `apps/desktop/src/terminal/tab_bar.rs`. Add a dropdown_open signal (use_signal of bool) and a dropdown_input signal (use_signal of Option of a tuple containing PaneKind and String — tracks which surface type is being configured and the current input value). On the "+" button, add an oncontextmenu handler that calls evt.prevent_default() and sets dropdown_open to true. The existing onclick remains unchanged (creates terminal via on_new_tab). When dropdown_open is true, render an absolutely-positioned div below the button (class "absolute top-full right-0 z-30 mt-1 py-1 bg-[var(--surface-container-high)] ghost-border rounded-md shadow-lg min-w-52"). Inside, render five buttons: "▸ Terminal", "🌐 Browser", "◇ Editor", "● Agent", "◻ Canvas". Each button has class "w-full text-left px-3 py-1.5 text-sm hover:bg-[var(--surface-bright)] text-[var(--color-on-surface)]". Terminal button directly creates a terminal tab via the existing on_new_tab and closes the dropdown. Browser, Editor, and Agent buttons set dropdown_input to their PaneKind with an empty string, showing an inline input. Canvas button shows a nested list of widget types (log-viewer, markdown, json-viewer) — clicking one creates the canvas tab directly.

### Task 3: Add inline input handling for dropdown

Continue editing `apps/desktop/src/terminal/tab_bar.rs`. When dropdown_input is Some, render an input row below the surface type list: a text input (flex 1, background --surface-lowest, border --color-border, padding 4px 8px, font-size 13px, border-radius 4px) with placeholder text based on kind ("Enter URL..." for Browser, "Enter file path..." for Editor, "Enter model..." for Agent with default value "claude-sonnet-4-6"), and an "Open" button (gradient-primary, padding 4px 12px, border-radius 4px, font-size 13px). On Enter or Open click: construct the appropriate PaneNode — Browser with the input as url, Editor with input as file_path and None language, Agent with a new UUID session_id and input as model. Call add_tab with the new PaneNode and a title derived from the input (basename for Editor, hostname for Browser, model name for Agent). Close the dropdown and clear dropdown_input. Add a backdrop div (fixed inset-0 z-20) behind the dropdown that closes it on click.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
