# Goal

Make the menu bar functional: each menu item opens a dropdown with actionable commands. FILE manages tabs, VIEW toggles layout elements, RUN executes the focused pane's primary action, TERMINAL manages terminal panes, and remaining menus show labeled stubs.

# Why

All eight menu items (FILE, EDIT, SELECTION, VIEW, GO, RUN, TERMINAL, HELP) are inert spans with hover styling but no onclick handlers. Clicking them does nothing. A desktop application's menu bar is the primary command surface — users expect it to work.

# Architecture

A single `Signal<Option<usize>>` tracks which menu is open (by index). Clicking a menu item toggles it open/closed. Clicking outside closes it. Each dropdown renders a list of `MenuItem` structs with label, optional keyboard shortcut display text, and an action closure.

Menu actions that operate on panes use the `TabsState` context (already provided at Shell level). The existing helper functions `create_new_tab`, `split_pane`, and `close_pane` in `pages/terminals.rs` are the right operations but they're private functions — the menu bar needs access to the same operations. Rather than making those public and creating a cross-module dependency, the menu bar operates directly on `TabsState` via `use_context`.

This unit depends on WU-2 because RUN menu actions dispatch pane operations. But the RUN menu can start as a stub and be enhanced later.

# Files Affected

| File | Change |
|------|--------|
| `src/layout/menu_bar.rs` | Rewrite with dropdown menus and action handlers |

# Task List

### Task 1: Implement menu bar dropdown state and MenuDropdown component

**Subject:** Add dropdown open/close state management and a reusable dropdown renderer

**Description:** Rewrite `crates/lx-desktop/src/layout/menu_bar.rs`. The new implementation needs:

1. A signal tracking which menu is open: `let mut open_menu: Signal<Option<usize>> = use_signal(|| None);`

2. A struct for menu items:

```rust
struct MenuItem {
    label: &'static str,
    shortcut: Option<&'static str>,
    action: Option<EventHandler>,
}
```

3. A struct for each menu:

```rust
struct Menu {
    label: &'static str,
    items: Vec<MenuItem>,
}
```

4. Build the menus inside the `MenuBar` component. Access `TabsState` via context for tab operations:

```rust
let tabs_state: Signal<TabsState<DesktopPane>> = use_context();
```

Define the menus:

**FILE menu:**
- "New Tab" (Ctrl+T) — creates a new terminal tab: generates a UUID, creates a `DesktopPane::Terminal` with working_dir from `std::env::current_dir`, calls `crate::terminal::add_terminal_tab`
- "Close Tab" (Ctrl+W) — calls `tabs_state.write().close_tab(id)` where `id` is the active tab ID
- separator (render as an `hr` or `div` with border-t)
- "Quit" (Ctrl+Q) — calls `dioxus::desktop::window().close()` (cfg-gated)

**EDIT menu:**
- "Undo" (Ctrl+Z) — stub (no action)
- "Redo" (Ctrl+Shift+Z) — stub
- "Cut" (Ctrl+X) — stub
- "Copy" (Ctrl+C) — stub
- "Paste" (Ctrl+V) — stub

**VIEW menu:**
- "Toggle Status Bar" — stub (future: signal to show/hide status bar)

**TERMINAL menu:**
- "New Terminal" — same as FILE > New Tab
- "Split Right" — calls `tabs_state.write().split_pane(focused_id, SplitDirection::Horizontal, new_pane)` where `focused_id` is from `tabs_state.read().focused_pane_id`
- "Split Down" — same but `SplitDirection::Vertical`

**RUN, SELECTION, GO, HELP:** — each gets a single stub item: "No actions available"

5. Render the menu bar. For each menu, render a `span` that on click toggles `open_menu`. When `open_menu` matches the menu's index, render the dropdown:

```rust
for (idx, menu) in menus.iter().enumerate() {
    div { class: "relative",
        span {
            class: "px-2 py-1 rounded cursor-pointer ...",
            onclick: move |_| {
                if open_menu() == Some(idx) {
                    open_menu.set(None);
                } else {
                    open_menu.set(Some(idx));
                }
            },
            "{menu.label}"
        }
        if open_menu() == Some(idx) {
            // backdrop to close on click-away
            div {
                class: "fixed inset-0 z-20",
                onclick: move |_| open_menu.set(None),
            }
            // dropdown panel
            div {
                class: "absolute top-full left-0 z-30 mt-1 py-1 bg-[var(--surface-container-high)]/80 backdrop-blur-[12px] rounded-md shadow-ambient min-w-48",
                for item in menu.items.iter() {
                    // render each item
                }
            }
        }
    }
}
```

Each menu item renders as a button with label on the left and shortcut on the right. Clicking calls the action (if Some) and closes the menu. Items with `action: None` are grayed out.

6. Keep the existing title bar drag behavior (`onmousedown` on the outer div calls `dioxus::desktop::window().drag()`), the "TERMINAL_MONOLITH" brand, and the window control buttons (minimize, maximize, close) unchanged. Only the middle section with menu items changes.

The full file will be near the 300-line limit. If it exceeds 300 lines, extract the menu definitions (the `Vec<Menu>` construction) into a separate `menu_items.rs` file and import it.

Required imports:
```rust
use common_pane_tree::{PaneNode, SplitDirection, TabsState};
use crate::panes::DesktopPane;
use crate::terminal::add_terminal_tab;
```

**ActiveForm:** Implementing menu bar dropdowns with action handlers

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_MENU_BAR.md" })
```
