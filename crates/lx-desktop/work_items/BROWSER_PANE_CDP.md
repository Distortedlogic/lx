# Browser Pane — Chrome DevTools Protocol Upgrade

## Goal

Upgrade the Browser pane to support a CDP-backed mode using chromiumoxide. In CDP mode, a headless Chrome instance runs server-side, and the pane displays screenshots streamed to the client via WebSocket. Agents can programmatically navigate, click, type, and take screenshots. This is an additive upgrade — the existing iframe mode continues to work when devtools is false.

## Why

- iframe mode only works for same-origin or CORS-friendly URLs and provides zero programmatic agent control
- CDP mode enables agents to interact with any web page: fill forms, click buttons, read DOM, take screenshots
- This is the foundation for autonomous agent testing workflows (build → deploy → browser-verify → screenshot evidence)
- chromiumoxide is already available in reference/chromiumoxide as a submodule

## How it works

When a Browser pane is created with devtools: true, the server launches a headless Chrome instance via chromiumoxide (singleton — one Chrome process, multiple pages). A CDP session is created per browser pane, wrapping a chromiumoxide Page. The server takes screenshots at a configurable interval (default every 500ms) and streams them as base64 JPEG over WebSocket. The TS widget renders screenshots on a canvas element instead of an iframe. Click events on the canvas compute viewport coordinates and send them as CDP click commands. Keyboard input is captured by a transparent overlay and sent as type commands. The URL bar and navigation controls in the PaneToolbar work the same way — they send messages to Rust which forwards to the CDP session.

## Files affected

| File | Change |
|------|--------|
| `apps/desktop/Cargo.toml` | Add chromiumoxide path dependency under server feature |
| `apps/desktop/src/server/browser.rs` | New file: Chrome lifecycle, CDP session management, screenshot streaming, WebSocket endpoint |
| `apps/desktop/src/terminal/browser_ws_types.rs` | New file: ClientToBrowser and BrowserToClient message enums |
| `apps/desktop/src/terminal/mod.rs` | Export browser_ws_types |
| `ts/desktop/src/widgets/browser.ts` | Add CDP rendering mode with canvas and screenshot display |
| `apps/desktop/src/terminal/view.rs` or `browser_view.rs` | BrowserView detects devtools flag and connects to CDP WebSocket when true |

## Task List

### Task 1: Add chromiumoxide dependency

Edit `apps/desktop/Cargo.toml`. Read `reference/chromiumoxide/Cargo.toml` to determine the exact crate name and available features (likely needs a tokio runtime feature). Add it as a path dependency pointing to reference/chromiumoxide, gated on the server feature. Verify the workspace compiles with `just rust-diagnose`.

### Task 2: Define browser WebSocket message types

Create `apps/desktop/src/terminal/browser_ws_types.rs`. Define ClientToBrowser enum with variants: Navigate (url: String), Click (x: f64, y: f64), Type (text: String), Screenshot (no fields — request immediate capture), Back, Forward, Refresh, Close. Define BrowserToClient enum with variants: ScreenshotFrame (data: String — base64 JPEG), PageLoaded (url: String, title: String), ConsoleMessage (level: String, text: String), Error (message: String), Closed. Derive Serialize and Deserialize on both. Export the module from terminal/mod.rs.

### Task 3: Create CDP session manager and WebSocket endpoint

Create `apps/desktop/src/server/browser.rs`. Define a BrowserManager struct stored as a global OnceCell that holds the chromiumoxide Browser instance (launched once as headless, no-sandbox). Define a BrowserSession struct wrapping a chromiumoxide Page, stored in a DashMap keyed by session_id. Implement get_or_create on BrowserManager: if no Browser exists, launch Chrome with chromiumoxide::Browser::launch using BrowserConfig::builder() with headless and no-sandbox options. Create a new Page for the session. Implement methods on BrowserSession: navigate(url) calls page.goto(url), click(x, y) calls page.click_point(x, y), type_text(text) calls page.type_str(text), screenshot() calls page.screenshot with CaptureScreenshot params for JPEG format and quality 70 returning the bytes, go_back/go_forward/reload call the corresponding page navigation methods. Create a WebSocket endpoint at /api/browser/ws with query param session_id. The handler creates a BrowserSession, sends an initial screenshot, then runs two concurrent tasks: one receives ClientToBrowser messages from the socket and executes the corresponding BrowserSession method (Navigate calls navigate then sends a PageLoaded response, Click/Type call their methods, Screenshot sends an immediate frame, Back/Forward/Refresh call nav methods); the other takes a screenshot every 500ms and sends it as a ScreenshotFrame. Base64-encode screenshot bytes before sending. Register the endpoint in the server router. Export from server/mod.rs.

### Task 4: Add CDP mode to browser widget

Edit `ts/desktop/src/widgets/browser.ts`. In the mount function, check config.mode. If config.mode is "cdp", skip creating the iframe. Instead create a canvas element (flex 1, width 100%, cursor crosshair). Keep the toolbar (URL bar, nav buttons) the same. In CDP mode, toolbar actions send messages via dx.send instead of manipulating an iframe: back sends { type: "back" }, forward sends { type: "forward" }, refresh sends { type: "refresh" }, URL enter sends { type: "navigate", url }. Add a click handler on the canvas: compute coordinates as (event.offsetX / canvas.clientWidth) times the viewport width (from config.viewport.width, default 1280), same for Y with height (default 720), and call dx.send with { type: "click", x, y }. Add a transparent textarea overlay (position absolute, opacity 0, pointer-events none, activated on double-click of canvas — set pointer-events auto and focus). On textarea input, send { type: "type", text: textarea.value } and clear it. The update function in CDP mode: if data is a string (base64 JPEG), decode it to a blob via atob and Uint8Array, create an ImageBitmap from a Blob of the bytes, and draw it on the canvas 2D context using drawImage scaled to canvas dimensions. If config.mode is not "cdp", use the existing iframe behavior unchanged.

### Task 5: Update BrowserView for CDP mode

Edit BrowserView in the terminal view file. Check the devtools field from the PaneNode::Browser variant (currently just a bool, but this was defined in the generalize work item — if it does not exist yet, the prop is the devtools bool). When devtools is true, pass mode: "cdp" and viewport: { width: 1280, height: 720 } in the config to use_ts_widget. Connect to /api/browser/ws using the same WebSocket pattern as TerminalView (use_websocket with session_id query param set to browser_id). Run a bidirectional forwarding loop: messages from widget.recv (navigate, click, type, back, forward, refresh) are deserialized and sent to the WebSocket as ClientToBrowser messages; messages from the WebSocket (ScreenshotFrame, PageLoaded, Error) are forwarded to the widget via widget.send_update. When devtools is false, behavior is unchanged from the iframe-only implementation.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
