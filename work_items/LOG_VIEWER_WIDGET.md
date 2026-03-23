# Goal

Extract the log viewer into a self-contained, well-documented widget with a clear public API that any application can use through the widget-bridge system. Add log level filtering and a clear button so the widget is functional as a standalone tool, not just a passive display.

# Why

- The log viewer currently accepts log lines and displays them, but has no way to filter by level or clear the log — basic features any log viewer needs
- The widget is tightly embedded in widget-bridge with no documented API contract — other apps reusing widget-bridge can't discover or understand it without reading the source
- The placeholder we added ("Log viewer — awaiting log entries") proves the widget mounts correctly, but it needs real interactive features to be useful

# What changes

**Add filter toolbar to `ts/widget-bridge/widgets/log-viewer.ts`:** A compact toolbar above the log container with toggle buttons for each log level (info, warn, error, debug) and a clear button. Toggling a level hides/shows lines of that level via CSS class filtering. Clear empties the container and re-shows the placeholder.

**Add log line count display:** Show a count of total and visible log lines in the toolbar.

# Files affected

- EDIT: `ts/widget-bridge/widgets/log-viewer.ts` — add filter toolbar, level toggles, clear button, line count

# Task List

### Task 1: Add filter toolbar and clear button to log viewer

**Subject:** Add level filtering and clear functionality to log viewer widget

**Description:** In `ts/widget-bridge/widgets/log-viewer.ts`, in the `mount` function, before appending the log container to the element, create a toolbar div with `display: flex; alignItems: center; gap: 4px; padding: 4px 8px; background: #131313; borderBottom: 1px solid #484848; fontSize: 11px;`. Add four toggle buttons (info, warn, error, debug) — each styled with the corresponding level color, initially "on" (opacity 1). Clicking a toggle sets its opacity to 0.3 and adds a CSS class to the container that hides lines of that level (use `data-level` attribute on each log line div, and a container class like `hide-info` with CSS rule `[data-level="info"] { display: none }`). Add a "Clear" button that empties the container and re-shows the placeholder. Add a span showing line count, updated in the `update` method. In the `appendLine` function, add `div.dataset.level = line.level` to each log line div so filtering works. Inject the filter CSS styles via a `<style>` tag in the mount function.

**ActiveForm:** Adding filter toolbar and clear button to log viewer

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
