# Canvas Pane — Generic Widget Surface

## Goal

Implement the Canvas pane surface as a generic container for pluggable widget types. Ships with three built-in widgets: log-viewer (scrolling log lines with level coloring), markdown (rendered markdown preview), and json-viewer (collapsible JSON tree). Additional widget types can be registered in TypeScript without any Rust changes. This replaces the stub CanvasView with a functional surface.

## Why

- Many visualization needs (logs, metrics, diffs, documentation) share the same pattern: receive JSON data from Rust, render it — they do not warrant their own PaneNode variant
- Canvas is the extensibility escape hatch: any future widget type is just a new registerWidget call in TypeScript
- The widget_type field on PaneNode::Canvas maps directly to the registerWidget key, so no Rust enum changes are needed for new widget types

## How it works

CanvasView calls use_ts_widget(widget_type, config) where widget_type is the string from PaneNode::Canvas. The TS side looks up the registered widget by name and mounts it. Each widget implements the same Widget interface (mount, update, resize, dispose). Data is pushed from Rust via widget.send_update with arbitrary JSON — the widget decides how to render it.

## Files affected

| File | Change |
|------|--------|
| `ts/desktop/src/widgets/log-viewer.ts` | New file: scrolling log viewer widget |
| `ts/desktop/src/widgets/markdown.ts` | New file: markdown preview widget |
| `ts/desktop/src/widgets/json-viewer.ts` | New file: collapsible JSON tree widget |
| `ts/desktop/src/index.ts` | Side-effect imports to register all three canvas widgets |
| `apps/desktop/src/terminal/view.rs` or new `canvas_view.rs` | Replace stub CanvasView with real implementation |

## Task List

### Task 1: Create log-viewer widget

Create `ts/desktop/src/widgets/log-viewer.ts` implementing the Widget interface. The mount function creates a container div (overflow-y auto, full height, background --surface-lowest, font-family monospace, font-size 13px, padding 8px, line-height 1.4). The update function receives data that is either a single log line object or an array of log line objects. Each log line has a level field (info, warn, error, debug) and a message field, with an optional ts (timestamp) field. For each line, append a div to the container. Text color by level: info uses --color-on-surface, warn uses amber (#F59E0B), error uses red (#EF4444), debug uses --color-outline. If a ts field exists, prefix the message with the timestamp in gray. Auto-scroll to bottom unless the user has manually scrolled up (check if scrollTop + clientHeight is less than scrollHeight minus 50). The dispose function removes the container. Import registerWidget and register as "log-viewer".

### Task 2: Create markdown widget

Create `ts/desktop/src/widgets/markdown.ts` implementing the Widget interface. Import markdown-it (already a dependency from the agent pane work item). The mount function creates a container div (overflow-y auto, full height, padding 24px, color --color-on-surface, background --surface-lowest). Apply inline styles for markdown elements: set a class on the container and use a style element injected into it — h1 through h6 with decreasing sizes, code elements with --surface-container-low background and monospace font and 2px 6px padding, pre elements with --surface-container-low background and 12px padding and overflow-x auto, links with --color-primary color, blockquotes with 3px left border in --color-outline and padding-left 12px and --color-on-surface-variant color. Initialize a markdown-it instance. The update function sets container innerHTML to the markdown-it render output of the data string (data is the raw markdown string). The dispose function removes the container. Register as "markdown".

### Task 3: Create json-viewer widget

Create `ts/desktop/src/widgets/json-viewer.ts` implementing the Widget interface. The mount function creates a container div (overflow-y auto, full height, padding 16px, font-family monospace, font-size 13px, background --surface-lowest, color --color-on-surface). Implement a recursive renderNode function that takes a value and an indentation level. For objects and arrays: create a div with a clickable toggle span (▶ when collapsed, ▼ when expanded) and a label (the key name if inside an object, or the type and count like "Object {3}" or "Array [5]"). Children are in a div with 16px margin-left, initially visible. Clicking the toggle hides/shows the children div and swaps the arrow. For primitives: strings display in green (#4ADE80) wrapped in quotes, numbers in amber (#F59E0B), booleans in purple (#A78BFA), null in --color-outline. Keys are colored with --color-primary. The update function clears the container and calls renderNode with the data (already parsed JSON from the bridge). Register as "json-viewer".

### Task 4: Register all canvas widgets in exports

Edit `ts/desktop/src/index.ts`. Add side-effect imports for log-viewer, markdown, and json-viewer widget files. Run `just ts-build` to verify the bundle compiles with all new widgets.

### Task 5: Implement CanvasView Rust component

Replace the stub CanvasView. The component takes canvas_id (String), widget_type (String), and config (serde_json::Value) as props. Call use_ts_widget with the widget_type string and the config value as the config parameter. Optionally spawn an async loop on widget.recv for widgets that send messages back (not needed for the initial three, but wire it up for future extensibility). The component renders a div with id element_id and class "w-full h-full".

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
