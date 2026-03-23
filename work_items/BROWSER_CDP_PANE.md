# Goal

Create a reusable `browser-cdp` Rust crate that provides headless Chrome browser session management via the Chrome DevTools Protocol, and wire it into the lx-desktop `BrowserView` so the browser pane renders live page screenshots on a canvas with full user interaction (click, type, navigate, back, forward, refresh). The crate has no Dioxus dependency — it exposes a pure async API that any Rust application can use. The desktop app only handles composition into its pane system.

# Why

- The browser pane currently shows "CDP backend not connected" — the TypeScript canvas renderer and event capture are in place but there is no Rust backend driving the headless browser
- Dioxus desktop's navigation handler blocks ALL http/https navigations (including iframe src changes on WebKitGTK), making iframe-based browsing architecturally impossible
- Dioxus does not support creating child webviews from component code (WebView2 reentrancy blocker, no public API)
- A production-ready reference implementation exists at `mcp-toolbelt/apps/desktop/src/server/browser.rs` using `chromiumoxide` — this should be adapted into an independent crate
- The CDP approach gives full programmatic control over the browser: agents can automate browsing, users can watch and interact, and the UX is embedded in the pane system

# What changes

**New crate `crates/browser-cdp/`:** Independent Rust crate with no Dioxus dependency. Contains `BrowserSession` (navigate, click, type_text, screenshot, go_back, go_forward, reload), global browser instance management via `OnceCell`, per-session page tracking via `DashMap`. Adapted from `mcp-toolbelt/apps/desktop/src/server/browser.rs`.

**BrowserView rewrite in `crates/lx-desktop/src/terminal/view.rs`:** Replace the current no-op `BrowserView` with an async loop following the `TerminalView` pattern. On mount: creates a browser session via `browser_cdp::get_or_create_session`, navigates to initial URL. Async loop uses `tokio::select!` with two branches — screenshot timer (every 500ms) sends base64 JPEG to the TypeScript widget via `widget.send_update()`, and message receiver handles click/type/navigate/back/forward/refresh events from the widget via `widget.recv()`.

**Wire PaneToolbar on_navigate in `terminals.rs`:** The `on_navigate` handler for Browser panes is currently `None`. Wire it to send navigate/back/forward/refresh commands through the widget handle to the BrowserView async loop.

# How it works

1. User creates a Browser pane → `BrowserView` component mounts
2. `BrowserView` calls `browser_cdp::get_or_create_session(&browser_id)` → launches headless Chrome (auto-detects binary), creates a new page
3. Navigates to initial URL (default "about:blank")
4. Enters `tokio::select!` loop:
   - Every 500ms: `session.screenshot()` → base64 JPEG → `widget.send_update(b64)` → TypeScript `update()` draws JPEG on canvas
   - On `widget.recv()`: dispatches `{ type: "click", x, y }` → `session.click(x, y)`, `{ type: "type", text }` → `session.type_text(text)`, `{ type: "navigate", url }` → `session.navigate(url)`, etc.
5. User sees live browser output on canvas, clicks are forwarded to headless Chrome, typing is forwarded, navigation works via PaneToolbar address bar

# Files affected

- NEW: `crates/browser-cdp/Cargo.toml` — new crate with chromiumoxide, dashmap, tokio, futures, base64, anyhow dependencies
- NEW: `crates/browser-cdp/src/lib.rs` — BrowserSession, get_browser, get_or_create_session, remove_session (adapted from mcp-toolbelt reference)
- EDIT: `Cargo.toml` (workspace) — add browser-cdp to workspace members and dependencies
- EDIT: `crates/lx-desktop/Cargo.toml` — add browser-cdp dependency
- EDIT: `crates/lx-desktop/src/terminal/view.rs` — rewrite BrowserView with async CDP loop
- EDIT: `crates/lx-desktop/src/pages/terminals.rs` — wire on_navigate for Browser panes

# Reference implementation

The complete `BrowserSession` API is at `mcp-toolbelt/apps/desktop/src/server/browser.rs` (120 lines). Chrome binary detection is handled by chromiumoxide internally via `BrowserConfig::builder().build()`. The async handler loop pattern is `tokio::spawn(async move { while handler.next().await.is_some() {} })`.

# Task List

### Task 1: Create browser-cdp crate

**Subject:** Create independent browser-cdp crate with CDP session management

**Description:** Create `crates/browser-cdp/` with `Cargo.toml` and `src/lib.rs`. The Cargo.toml should have dependencies: `chromiumoxide = "0.9"`, `dashmap = "6"`, `tokio = { version = "1", features = ["sync"] }`, `futures = "0.3"`, `base64 = "0.22"`, `anyhow = "1"`. The `src/lib.rs` should contain: (1) `static BROWSER_INSTANCE: OnceCell<Arc<Mutex<Browser>>>` for global browser singleton, (2) `static SESSIONS: LazyLock<DashMap<String, Arc<BrowserSession>>>` for per-pane sessions, (3) `async fn get_browser()` that initializes headless Chrome with args `--headless=new --no-sandbox --disable-gpu --disable-dev-shm-usage`, spawns the handler loop, (4) `pub struct BrowserSession` with `page: Arc<Mutex<Page>>`, (5) methods on BrowserSession: `navigate(&self, url) -> Result<(String, String)>`, `click(&self, x, y) -> Result<()>`, `type_text(&self, text) -> Result<()>`, `screenshot(&self) -> Result<String>` (base64 JPEG, quality 70), `go_back -> Result<()>`, `go_forward -> Result<()>`, `reload -> Result<()>`, (6) `pub async fn get_or_create_session(id: &str) -> Result<Arc<BrowserSession>>`, (7) `pub fn remove_session(id: &str)`. Adapt directly from `mcp-toolbelt/apps/desktop/src/server/browser.rs`. Add `browser-cdp` to workspace members in the root `Cargo.toml`.

**ActiveForm:** Creating browser-cdp crate with CDP session management

### Task 2: Add browser-cdp dependency to lx-desktop

**Subject:** Wire browser-cdp into lx-desktop Cargo.toml

**Description:** In `crates/lx-desktop/Cargo.toml`, add `browser-cdp = { path = "../browser-cdp" }` under `[dependencies]`. Ensure `tokio` has the `time` feature enabled (needed for the screenshot interval timer).

**ActiveForm:** Adding browser-cdp dependency to lx-desktop

### Task 3: Rewrite BrowserView with async CDP loop

**Subject:** Replace BrowserView placeholder with live CDP backend

**Description:** In `crates/lx-desktop/src/terminal/view.rs`, rewrite the `BrowserView` component. Change `let (element_id, _widget)` to `let (element_id, widget)` to use the widget handle. Add a `use_future` block (following the `TerminalView` pattern at lines 22-77 in the same file). Inside the future: (1) call `browser_cdp::get_or_create_session(&browser_id).await`, return early on error, (2) if url is not empty and not "about:blank", call `session.navigate(&url).await`, (3) create `let mut interval = tokio::time::interval(std::time::Duration::from_millis(500))`, (4) enter `loop { tokio::select! { _ = interval.tick() => { if let Ok(b64) = session.screenshot().await { widget.send_update(b64); } }, result = widget.recv::<serde_json::Value>() => { match result { Ok(msg) => match msg["type"].as_str() { Some("click") => { let x = msg["x"].as_f64().unwrap_or(0.0); let y = msg["y"].as_f64().unwrap_or(0.0); let _ = session.click(x, y).await; }, Some("type") => { if let Some(text) = msg["text"].as_str() { let _ = session.type_text(text).await; } }, Some("navigate") => { if let Some(url) = msg["url"].as_str() { let _ = session.navigate(url).await; } }, Some("back") => { let _ = session.go_back().await; }, Some("forward") => { let _ = session.go_forward().await; }, Some("refresh") => { let _ = session.reload().await; }, _ => {} }, Err(_) => break, } } } }`. Add `use_drop` to call `browser_cdp::remove_session(&browser_id)` on cleanup.

**ActiveForm:** Rewriting BrowserView with async CDP backend loop

### Task 4: Wire on_navigate for Browser panes

**Subject:** Connect PaneToolbar browser controls to BrowserView

**Description:** In `crates/lx-desktop/src/pages/terminals.rs`, in the `render_pane_item` function, the `on_navigate` prop is currently `None::<EventHandler<String>>`. For Browser panes, this should send navigation commands through the widget bridge. However, the PaneToolbar's on_navigate sends string commands ("back", "forward", "refresh", or a URL), while the BrowserView's async loop receives structured JSON from `widget.recv()`. The simplest correct approach: keep `on_navigate` as `None` for now — the TypeScript CDP mode's canvas already captures clicks and the hidden textarea captures typing, and navigation via the PaneToolbar address bar's keypress handler already works through the existing toolbar.rs code which calls `on_navigate`. Change the Browser match arm in `render_pane_item` to pass an `on_navigate` handler that uses `document::eval` to send a message to the widget's eval context. Alternatively, this can be deferred since the TypeScript side already has a toolbar with address bar in the PaneToolbar (Rust side) that can be wired later. For MVP, the canvas click/type interaction is sufficient.

**ActiveForm:** Wiring PaneToolbar navigate handler for Browser panes

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
