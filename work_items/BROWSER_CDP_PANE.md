# Goal

Create a reusable `browser-cdp` Rust crate that provides headless Chrome browser session management via the Chrome DevTools Protocol, and wire it into the lx-desktop `BrowserView` so the browser pane renders live page screenshots on a canvas with full user interaction (click, type, navigate, back, forward, refresh). The crate has no Dioxus dependency — it exposes a pure async API that any Rust application can use. The desktop app only handles composition into its pane system.

Note: the `devtools: bool` prop on BrowserView is ignored for this implementation — all browser sessions use headless Chrome.

# Why

- The browser pane currently shows "CDP backend not connected" — the TypeScript canvas renderer and event capture are in place but there is no Rust backend driving the headless browser
- Dioxus desktop's navigation handler blocks ALL http/https navigations (including iframe src changes on WebKitGTK), making iframe-based browsing architecturally impossible
- Dioxus does not support creating child webviews from component code (WebView2 reentrancy blocker, no public API)
- A production-ready reference existed using `chromiumoxide` — the core API is adapted into an independent crate
- The CDP approach gives full programmatic control over the browser: agents can automate browsing, users can watch and interact, and the UX is embedded in the pane system

# What changes

**New crate `crates/browser-cdp/`:** Independent Rust crate with no Dioxus dependency. Contains `BrowserSession` (navigate, click, type_text, screenshot, go_back, go_forward, reload), global browser instance management via `tokio::sync::OnceCell`, per-session page tracking via `DashMap`. Key API details below.

**BrowserSession API:**
- `struct BrowserSession { page: Mutex<Page> }` — wraps a single chromiumoxide `Page`
- `navigate(&self, url: &str) -> Result<(String, String)>` — navigates page, returns (final_url, title)
- `click(&self, x: f64, y: f64) -> Result<()>` — dispatches mouse press+release via CDP `DispatchMouseEventParams`
- `type_text(&self, text: &str) -> Result<()>` — types text via chromiumoxide's `page.type_str(text)` high-level API (handles key events internally)
- `screenshot(&self) -> Result<String>` — captures JPEG screenshot (quality 70) via `ScreenshotParams`, returns base64
- `go_back(&self) -> Result<()>`, `go_forward(&self) -> Result<()>`, `reload(&self) -> Result<()>` — history/reload via CDP
- `get_browser()` — initializes headless Chrome singleton with `--headless=new --no-sandbox --disable-gpu --disable-dev-shm-usage`, spawns handler loop as `tokio::spawn(async move { while handler.next().await.is_some() {} })`
- `get_or_create_session(id: &str) -> Result<Arc<BrowserSession>>` — creates or retrieves a session from the DashMap
- `remove_session(id: &str)` — drops session and its page

**BrowserView rewrite in `crates/lx-desktop/src/terminal/view.rs`:** Replace the current no-op `BrowserView` with an async loop following the `TerminalView` pattern. On mount: creates a browser session via `browser_cdp::get_or_create_session`, navigates to initial URL. Async loop uses `tokio::select!` with two branches — screenshot timer (every 500ms) sends base64 JPEG to the TypeScript widget via `widget.send_update()`, and message receiver handles click/type/navigate/back/forward/refresh events from the widget via `widget.recv()`.

**Keep `on_navigate` as `None` for Browser panes.** The BrowserView async loop already receives all navigation commands directly from the TypeScript widget via `widget.recv()`. The PaneToolbar address bar is visual-only for now. Wiring PaneToolbar to widget bridge is a future enhancement.

# How it works

1. User creates a Browser pane -> `BrowserView` component mounts
2. `BrowserView` calls `browser_cdp::get_or_create_session(&browser_id)` -> launches headless Chrome (auto-detects binary), creates a new page
3. Navigates to initial URL (default "about:blank")
4. Enters `tokio::select!` loop:
   - Every 500ms: `session.screenshot()` -> base64 JPEG -> `widget.send_update(b64)` -> TypeScript `update()` draws JPEG on canvas
   - On `widget.recv()`: dispatches `{ type: "click", x, y }` -> `session.click(x, y)`, `{ type: "type", text }` -> `session.type_text(text)`, `{ type: "navigate", url }` -> `session.navigate(url)`, etc.
5. User sees live browser output on canvas, clicks are forwarded to headless Chrome, typing is forwarded

# Files affected

- NEW: `crates/browser-cdp/Cargo.toml` — new crate with chromiumoxide, dashmap, tokio, futures, base64, anyhow dependencies
- NEW: `crates/browser-cdp/src/lib.rs` — BrowserSession, get_browser, get_or_create_session, remove_session
- EDIT: `Cargo.toml` (workspace) — add browser-cdp to workspace members and dependencies
- EDIT: `crates/lx-desktop/Cargo.toml` — add browser-cdp dependency
- EDIT: `crates/lx-desktop/src/terminal/view.rs` — rewrite BrowserView with async CDP loop

# Task List

### Task 1: Create browser-cdp crate

**Subject:** Create independent browser-cdp crate with CDP session management

**Description:** Create `crates/browser-cdp/` with `Cargo.toml` and `src/lib.rs`. The Cargo.toml should have dependencies: `chromiumoxide = "0.9"`, `dashmap = "6"`, `tokio = { version = "1", features = ["sync"] }`, `futures = "0.3"`, `base64 = "0.22"`, `anyhow = "1"`. The `src/lib.rs` should contain: (1) `static BROWSER_INSTANCE: tokio::sync::OnceCell<Arc<Mutex<Browser>>>` for global browser singleton, (2) `static SESSIONS: std::sync::LazyLock<DashMap<String, Arc<BrowserSession>>>` for per-pane sessions, (3) `async fn get_browser()` that initializes headless Chrome with args `--headless=new --no-sandbox --disable-gpu --disable-dev-shm-usage`, spawns the handler loop, (4) `pub struct BrowserSession` with `page: Mutex<Page>`, (5) methods on BrowserSession: `navigate(&self, url) -> Result<(String, String)>`, `click(&self, x, y) -> Result<()>`, `type_text(&self, text) -> Result<()>`, `screenshot(&self) -> Result<String>` (base64 JPEG, quality 70), `go_back -> Result<()>`, `go_forward -> Result<()>`, `reload -> Result<()>`, (6) `pub async fn get_or_create_session(id: &str) -> Result<Arc<BrowserSession>>`, (7) `pub fn remove_session(id: &str)`. Import types: `chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat`, `chromiumoxide::page::ScreenshotParams`, `chromiumoxide::cdp::browser_protocol::input::{DispatchMouseEventParams, DispatchMouseEventType}`. The `type_text` method uses `page.type_str(text)` — a high-level chromiumoxide API, not raw DispatchKeyEvent. Add `browser-cdp` to workspace members in the root `Cargo.toml`.

**ActiveForm:** Creating browser-cdp crate with CDP session management

### Task 2: Add browser-cdp dependency to lx-desktop

**Subject:** Wire browser-cdp into lx-desktop Cargo.toml

**Description:** In `crates/lx-desktop/Cargo.toml`, add `browser-cdp = { path = "../browser-cdp" }` under `[dependencies]`. Ensure `tokio` has the `time` feature enabled (needed for the screenshot interval timer).

**ActiveForm:** Adding browser-cdp dependency to lx-desktop

### Task 3: Rewrite BrowserView with async CDP loop

**Subject:** Replace BrowserView placeholder with live CDP backend

**Description:** In `crates/lx-desktop/src/terminal/view.rs`, rewrite the `BrowserView` component. Change `let (element_id, _widget)` to `let (element_id, widget)` to use the widget handle. Add a `use_future` block following the `TerminalView` pattern (lines 22-77). Clone `browser_id` and `url` inside the future closure (same pattern as TerminalView clones `working_dir` and `command`). Inside the async block: (1) call `browser_cdp::get_or_create_session(&browser_id).await`, return on error, (2) if url is not empty and not "about:blank", navigate and return on error, (3) create a 500ms interval, (4) enter a `tokio::select!` loop — screenshot branch sends base64 via `widget.send_update(b64)`, message branch matches on `msg["type"]` for click/type/navigate/back/forward/refresh and dispatches to session methods. Use `.is_err()` for error checks (not `if let Err(e)` which creates unused variable warnings): `if session.click(x, y).await.is_err() { break; }`. Add `use_drop` to call `browser_cdp::remove_session(&browser_id)`.

**ActiveForm:** Rewriting BrowserView with async CDP backend loop

### Task 4: Verify on_navigate stays None

**Subject:** No code change — on_navigate is already None for Browser panes

**Description:** No action needed. The `on_navigate` prop in `terminals.rs` `render_pane_item` is already `None::<EventHandler<String>>` for all pane types. The BrowserView async loop receives navigation commands directly from the TypeScript widget via `widget.recv()`. This task exists only to document the decision — mark it complete immediately.

**ActiveForm:** Rewriting BrowserView with async CDP backend loop

### Task 4: Keep on_navigate as None for Browser panes

**Subject:** Leave PaneToolbar on_navigate unwired for Browser panes

**Description:** Keep `on_navigate` as `None` for Browser panes. The BrowserView async loop already receives all navigation commands directly from the TypeScript widget via `widget.recv()`. The PaneToolbar address bar is visual-only for now — it shows the current URL but navigation happens through the canvas interaction. Wiring PaneToolbar -> widget bridge is a future enhancement.

**ActiveForm:** Confirming on_navigate remains None for Browser panes

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
