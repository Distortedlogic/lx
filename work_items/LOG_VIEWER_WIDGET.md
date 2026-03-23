# Goal

Extract the log viewer into a self-contained, well-documented widget with a clear public API that any application can use through the widget-bridge system. Add log level filtering and a clear button so the widget is functional as a standalone tool, not just a passive display.

# Why

- The log viewer currently accepts log lines and displays them, but has no way to filter by level or clear the log — basic features any log viewer needs
- The widget is tightly embedded in widget-bridge with no documented API contract — other apps reusing widget-bridge can't discover or understand it without reading the source
- The placeholder we added ("Log viewer — awaiting log entries") proves the widget mounts correctly, but it needs real interactive features to be useful

# What changes

**Add filter toolbar to `ts/widget-bridge/widgets/log-viewer.ts`:** A compact toolbar above the log container with toggle buttons for each log level (info, warn, error, debug) and a clear button. Toggling a level hides/shows lines of that level via CSS class filtering. Clear empties the container and re-creates the placeholder div.

**Add log line count display:** Show a count of total and visible log lines in the toolbar.

# Files affected

- EDIT: `ts/widget-bridge/widgets/log-viewer.ts` — add filter toolbar, level toggles, clear button, line count

# Task List

### Task 1: Add filter toolbar and clear button to log viewer

**Subject:** Add level filtering and clear functionality to log viewer widget

**Description:** In `ts/widget-bridge/widgets/log-viewer.ts`, in the `mount` function, use flexbox for the outer container layout: set the element to `display: flex; flexDirection: column; height: 100%`. The toolbar is a fixed-height child and the log container uses `flex: 1; overflow-y: auto` — this prevents the container from overflowing when the toolbar is added.

Create a toolbar div with `display: flex; alignItems: center; gap: 4px; padding: 4px 8px; background: #131313; borderBottom: 1px solid #484848; fontSize: 11px;`. Add four toggle buttons (info, warn, error, debug) — each styled with the corresponding level color, initially "on" (opacity 1). Clicking a toggle ALTERNATES between on (opacity 1, class removed, level added back to `hiddenLevels` set) and off (opacity 0.3, class added, level removed from set). The CSS rules must be SCOPED to the container class: `.hide-info [data-level="info"] { display: none }`, `.hide-warn [data-level="warn"] { display: none }`, etc. — NOT bare `[data-level="info"] { display: none }` which would always hide.

Add a "Clear" button that empties the container and re-creates the placeholder div. The placeholder must be created with `document.createElement` matching the same structure as the initial mount (not just "re-shown", since the update method calls `.remove()` on the placeholder). For example: `const placeholder = document.createElement("p"); placeholder.style.color = "#757575"; placeholder.textContent = "Log viewer — awaiting log entries"; container.appendChild(placeholder);` — and store the reference on the state object so `update` can find it.

Add a span showing line count, updated in the `update` method. Track filter state via a `Set<string>` of hidden levels stored on the state object (add `hiddenLevels: Set<string>` to `LogViewerState`). The line count display should reflect total lines and the number hidden by filters.

In the `appendLine` function, add `div.dataset.level = line.level` to each log line div so filtering works. Inject the filter CSS styles via a `<style>` tag in the mount function.

**ActiveForm:** Adding filter toolbar and clear button to log viewer

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
