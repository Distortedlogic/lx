# Pane and Tab Notification Indicators

## Goal

Add visual notification indicators to tabs and panes that signal when a pane needs attention: command completed, error occurred, agent waiting for approval, or new output in an unfocused pane. Tab dots show the highest-severity notification across all panes in that tab. Pane borders change color based on notification level.

## Why

- With multiple tabs and split panes, users lose track of which panes have new activity
- Agent sessions run for minutes — users need to know when an agent finishes or needs tool approval without watching every pane
- The developer value data identifies notification rings and attention signals as key purpose-built agent terminal features (Ghostty cmux pattern)

## What changes

**State:**

- A notifications HashMap of String to PaneNotification on TabsState, keyed by pane_id
- PaneNotification has a level (NotificationLevel enum) and an optional message String
- NotificationLevel variants: Info, Success, Warning, Error, Attention — ordered by severity

**Tab indicators:**

- Tabs render a small colored dot when any pane in that tab has an active notification
- Dot color follows level: green for Success, amber for Warning and Attention (Attention also pulses), red for Error, blue for Info
- Dot shows the highest severity across all panes in the tab
- Info and Success notifications clear when the tab becomes active

**Pane indicators:**

- Pane border color reflects notification level, taking precedence over the focus border
- Attention level adds a subtle pulse animation

**Notification sources (initial):**

- Terminal: TerminalToClient::Closed triggers Success notification
- Agent: ToolCall triggers Attention, AssistantDone triggers Success, Error triggers Error
- Browser, Editor, Canvas: no built-in sources initially, but the API supports manual notification setting

## Files affected

| File | Change |
|------|--------|
| `apps/desktop/src/terminal/types.rs` | Add NotificationLevel enum, PaneNotification struct, notifications field on TabsState, notification methods on extension trait |
| `apps/desktop/src/terminal/tab_bar.rs` | Render notification dot on tabs based on highest severity |
| `apps/desktop/src/pages/workflow/terminals.rs` | Apply notification-based border classes to pane wrapper divs |
| `apps/desktop/src/terminal/view.rs` | TerminalView emits notification on PTY close |
| `apps/desktop/src/terminal/agent_view.rs` | AgentView emits notifications on tool_call, completion, and error |

## Task List

### Task 1: Add notification types and state

Edit `apps/desktop/src/terminal/types.rs`. Add a NotificationLevel enum with variants Info, Success, Warning, Error, Attention — derive Clone, Copy, Debug, PartialEq, Eq, and implement PartialOrd and Ord so Error is highest, then Attention, Warning, Success, Info lowest. Add a PaneNotification struct with fields level (NotificationLevel) and message (Option of String) — derive Clone, Debug, PartialEq. Add a notifications field to TabsState of type HashMap of String to PaneNotification (add std::collections::HashMap import). Initialize it as HashMap::new() in the Default impl. Add methods on the TabsState extension trait: set_notification taking pane_id (String) and notification (PaneNotification) that inserts into the map; clear_notification taking pane_id (String reference) that removes from the map; get_notification taking pane_id (String reference) returning Option of PaneNotification clone; clear_tab_notifications taking tab_id (String reference) that clears Info and Success notifications for all pane_ids in that tab.

### Task 2: Render notification dots on tabs

Edit `apps/desktop/src/terminal/tab_bar.rs`. For each tab rendered in the tab list, collect all pane ids via all_pane_ids on the tab root. Check the notifications map for each pane id. If any notification exists, find the one with the highest level (use the Ord impl). Render a span (width 6px, height 6px, border-radius full, inline-block, margin-left 6px) next to the tab title. Set the background class based on level: Error uses "bg-red-500", Attention uses "bg-amber-500 animate-pulse", Warning uses "bg-amber-400", Success uses "bg-emerald-500", Info uses "bg-blue-400". When a tab becomes the active tab (on click), call clear_tab_notifications to remove Info and Success level notifications for that tab.

### Task 3: Apply notification borders to panes

Edit `apps/desktop/src/pages/workflow/terminals.rs`. In the pane rendering loop, after determining the focus border class, also check for an active notification on the current pane id via get_notification. If a notification exists, override the border class: Error uses "ghost-border outline-red-500", Attention uses "ghost-border outline-amber-500 animate-pulse", Warning uses "ghost-border outline-amber-400", Success uses "ghost-border outline-emerald-500", Info uses "ghost-border outline-blue-400". Notification borders take precedence over the focus border. The notification border is always visible (not gated on group-hover or focus state).

### Task 4: Emit notifications from TerminalView and AgentView

Edit `apps/desktop/src/terminal/view.rs`. In the TerminalView component's WebSocket receive loop, when a TerminalToClient::Closed message arrives, call tabs_state.set_notification with the terminal_id and a PaneNotification with level Success and message None. Access tabs_state from Dioxus context (use use_tabs_state or consume_context). Edit `apps/desktop/src/terminal/agent_view.rs`. In the AgentView WebSocket receive loop: when a ToolCall message arrives, set notification with level Attention and message "Tool approval needed"; when AssistantDone arrives, set notification with level Success and clear any existing Attention notification; when Error arrives, set notification with level Error and the error message. When the user sends a ToolDecision (in the widget recv loop), clear the Attention notification for this pane.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
