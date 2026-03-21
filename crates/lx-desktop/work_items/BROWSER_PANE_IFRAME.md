# Browser Pane — iframe Implementation

## Goal

Implement the Browser pane surface as an iframe rendered within the Dioxus webview. Includes a URL input bar, navigation controls (back, forward, refresh), and TypeScript widget registration through the existing widget bridge. This replaces the stub BrowserView from the generalize work item with a functional browser pane.

## Why

- Developers running dev servers need to preview localhost URLs alongside terminal panes without leaving the app
- iframe is the simplest browser implementation — no external processes, works within the existing Dioxus webview
- The widget bridge (use_ts_widget + registerWidget) already supports mounting arbitrary JS widgets into pane slots

## How it works

BrowserView calls use_ts_widget("browser", config) with the initial URL. The TS widget creates a container with two children: a toolbar div (URL input, nav buttons) and an iframe. The iframe src is set to the config URL. When the user edits the URL and presses Enter, the widget sends a navigate message back to Rust via dx.send. Navigation buttons call iframe.contentWindow.history methods. When Rust sends an update, the widget changes iframe.src.

## Files affected

| File | Change |
|------|--------|
| `ts/desktop/src/widgets/browser.ts` | New file: browser widget with iframe, URL bar, and nav controls |
| `ts/desktop/src/index.ts` | Side-effect import to register browser widget |
| `apps/desktop/src/terminal/view.rs` or new `browser_view.rs` | Replace stub BrowserView with real implementation |

## Task List

### Task 1: Create browser widget TypeScript implementation

Create `ts/desktop/src/widgets/browser.ts` implementing the Widget interface from the registry. The mount function creates a container div with display flex, flex-direction column, and full height. Inside it, create a toolbar div (height 36px, display flex, align-items center, gap 4px, padding 0 8px, background using CSS variable --surface-container-low, border-bottom 1px solid using --color-border). The toolbar contains: a back button with innerHTML "←", a forward button "→", a refresh button "↻", and a text input (flex 1, background --surface-lowest, border 1px solid --color-border, color --color-on-surface, font-size 13px, padding 4px 8px, border-radius 4px). Below the toolbar, create an iframe with flex 1, width 100%, border none, background white. On mount, set iframe.src to config.url and set the input value to config.url. The input keydown handler: on Enter, set iframe.src to the input value and call dx.send with type "navigate" and the url. Back button calls iframe.contentWindow.history.back() in a try-catch. Forward calls history.forward(). Refresh calls location.reload(). The update function sets iframe.src to the data string and updates the input value. The dispose function removes the container. Import registerWidget and call registerWidget("browser", browserWidget) at module level.

### Task 2: Register browser widget in exports

Edit `ts/desktop/src/index.ts`. Add a side-effect import for the browser widget file (import "./widgets/browser"). Run `just ts-build` to verify the bundle compiles.

### Task 3: Implement BrowserView Rust component

Replace the stub BrowserView in `apps/desktop/src/terminal/view.rs` (or the separate file if one was created). The component takes browser_id (String) and url (String) as props. Call use_ts_widget("browser", serde_json::json!({ "url": url })) to get the element_id and widget handle. Spawn an async task that loops on widget.recv to handle messages — when the message has type "navigate", log the URL for now (state sync with PaneNode will be wired in a future work item). The component renders a div with id set to element_id and class "w-full h-full".

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
