# Goal

Create four new workspace crates — `lx-ui`, `lx-desktop`, `lx-mobile`, `lx-tui` — by migrating `backends/dx` into the workspace as `lx-dx`, extracting reusable terminal/pane infrastructure into `lx-ui`, building an lx-focused Dioxus desktop app, a companion mobile app, and a ratatui terminal UI. Code is copied and adapted from `~/repos/mcp-toolbelt/apps/desktop/` and `~/repos/mcp-toolbelt/apps/mobile/`, stripping context-engine, workflow, voice, and MCP-specific functionality.

# Why

- Running `lx run program.lx` gives zero visibility into AI calls, agent spawns, inter-agent messages, shell executions, or timing data. The only output is whatever the program explicitly `emit`s. Developers need real-time observability into the runtime.
- `backends/dx/` already has the event infrastructure (EventBus, RuntimeEvent, Dx* backend decorators, ANSI formatters, PtyWriter, AgentTerminalManager, ProgramRunner, LangfuseClient) but sits outside the workspace with its own `[workspace]` section and `Cargo.lock`. No other crate can depend on it normally.
- The mcp-toolbelt desktop app has a mature terminal system (PaneNode binary tree, split/close/convert ops, rectangle computation, PTY session management, tab/toolbar UI, WebSocket terminal protocol) and a working lx execution integration (`server/workflow/lx.rs`). This infrastructure is generic but entangled with context-engine pages, workflow APIs, voice agent, and browser automation that have nothing to do with lx.
- The mcp-toolbelt mobile app is almost entirely voice-agent and pipeline-approval UI. The only reusable pattern is the HTTP client + bottom navigation + status badge architecture.
- Without shared UI primitives, each new app (desktop, mobile, TUI) will reimplement pane trees, terminal protocols, and component patterns independently.

# What Changes

## Phase 1: Bring lx-dx into the workspace

Move `backends/dx/` to `crates/lx-dx/`. Remove the standalone `[workspace]` section from its Cargo.toml. Add `"crates/lx-dx"` to the root workspace members list. Delete the standalone `Cargo.lock` and `target/` in the old location. Verify compilation with `just diagnose` (this requires temporarily adding lx-dx to the workspace `exclude` list if the new backend traits it references don't exist yet, or stubbing them — see task details).

## Phase 2: Create lx-ui shared component library

New crate `crates/lx-ui/` — a Dioxus component library with no binary target. Contains:

**From mcp-toolbelt terminal system (adapted):**
- `pane_tree.rs` — `PaneNode` enum (Terminal, Browser, Editor, Agent, Canvas, Split), `SplitDirection`, `Rect`, `DividerInfo` types. Recursive tree operations: split, close, convert, set_ratio, find, compute_pane_rects, compute_dividers, all_pane_ids, first_terminal_id.
- `tab_state.rs` — `TerminalTab`, `TabsState` (Dioxus Store), `NotificationLevel`. Store methods: add_tab, close_tab, split_active_pane, close_pane_in_active_tab, set_active_tab_ratio, set/get/clear notifications.
- `ws_types.rs` — `ClientToTerminal` (Input, Resize, Close), `TerminalToClient` (Output, SessionReady, Closed, Error) enums.
- `pty_session.rs` — PTY session management: `PtySession` struct (input_tx, output_tx broadcast, circular buffer, master handle, child process), global `DashMap<String, Arc<PtySession>>` cache, `get_or_create`, `subscribe`, `send_input`, `resize`, `remove`. Reader/writer threads.
- `ws_endpoint.rs` — Terminal WebSocket handler: query params (terminal_id, cols, rows, working_dir, command), handshake, bidirectional select loop.

**From mcp-toolbelt components (adapted):**
- `components.rs` — `PageHeader` (title, subtitle, actions slot), `StatusBadge` (status string → colored badge), `Skeleton` (loading placeholder with pulse animation).

All `crate::server::*` references stripped. All context-engine, workflow, voice, browser CDP, editor file I/O, and agent WebSocket code excluded. PaneNode keeps all variant names (Terminal, Browser, Editor, Agent, Canvas, Split) for forward compatibility but only Terminal is fully implemented in this phase.

Dependencies: `dioxus`, `portable-pty`, `tokio`, `tokio-tungstenite`, `serde`, `serde_json`, `uuid`, `dashmap`, `parking_lot`.

## Phase 3: Create lx-desktop Dioxus app

New binary crate `crates/lx-desktop/`. A Dioxus desktop app focused on lx program execution and runtime observability.

**Copied and adapted from mcp-toolbelt:**
- `main.rs` — Dioxus desktop launch (from `apps/desktop/src/main.rs`, stripped of server features and context-engine init).
- `app.rs` — Root component with error boundary and routing (from `apps/desktop/src/app.rs`).
- `routes.rs` — Three routes: Run (file picker + execute), Terminals (pane manager), Events (filtered event log). Stripped of all context-engine routes and workflow routes except terminals.
- `layout/shell.rs` — App shell with sidebar + content area (from `apps/desktop/src/layout/shell.rs`, stripped of indexer status, repo listing, terminal.js injection replaced with lx-specific terminal init).
- `layout/sidebar.rs` — Collapsible nav sidebar (from `apps/desktop/src/layout/sidebar.rs`, reduced to 3 nav items).
- `server/lx.rs` — lx execution handler (from `apps/desktop/src/server/workflow/lx.rs`). `LxRunState`, `start_run`, writer factory creating PTY sessions via lx-ui, `AgentTerminalManager` from lx-dx, `ProgramRunner` from lx-dx. Status polling endpoint.
- `pages/run.rs` — File path input, "Run" button, execution status display.
- `pages/terminals.rs` — Pane manager rendering PaneNode tree from lx-ui, tab bar, toolbar, terminal views via lx-ui WebSocket endpoint.
- `pages/events.rs` — EventBus subscriber rendering filtered event log. Hotkey-driven filter toggles (AI, Emit, Log, Shell, Messages, Agents, Progress, Errors, All). Uses ANSI formatters from lx-dx.
- `assets/terminal.ts` — xterm.js terminal widget driver (from `mcp-toolbelt/ts/desktop/src/terminal.ts`, stripped of ToolbeltDesktop namespace).

Everything NOT copied: all `pages/context_engine/`, `pages/workflow/` (except terminals pattern), `server/context_engine.rs`, `server/files.rs`, `server/browser.rs`, `server/agent.rs`, `server/pipelines.rs`, all `server/workflow/` (except lx.rs), `hooks/mod.rs`, `components/code_viewer.rs`, `components/directory_tree.rs`, `components/search_results.rs`, `components/chunk_overlay.rs`, `components/syntax.rs`, `components/stats_card.rs`, `components/task_list.rs`, `ts_widget.rs` (replaced with simpler bridge), `terminal/agent_ws.rs`, `terminal/browser_ws.rs`, `terminal/agent_types.rs`, `terminal/browser_ws_types.rs`.

Dependencies: `lx`, `lx-dx`, `lx-ui`, `dioxus` (desktop feature), `tokio`, `clap`, `serde_json`, `uuid`.

## Phase 4: Create lx-mobile Dioxus app

New binary crate `crates/lx-mobile/`. A Dioxus mobile app for remote lx program monitoring and user prompt response.

**Copied and adapted from mcp-toolbelt:**
- `main.rs` — Dioxus mobile launch (from `apps/mobile/src/main.rs`).
- `app.rs` — Root component (from `apps/mobile/src/app.rs`).
- `routes.rs` — Three routes: Status, Events, Approvals.
- `layout/shell.rs` — Mobile shell with bottom nav (from `apps/mobile/src/layout/shell.rs`, stripped of voice state, transcript, ws_tx context providers).
- `layout/bottom_nav.rs` — Bottom tab bar (from `apps/mobile/src/layout/bottom_nav.rs`, changed tabs to Status/Events/Approvals).
- `api_client.rs` — HTTP client connecting to lx-desktop backend (from `apps/mobile/src/api_client/mod.rs`, stripped of pipeline operations, replaced with lx run status polling and event stream fetch).
- `ws_client.rs` — WebSocket client subscribing to lx-desktop's EventBus WsStream adapter (from `apps/mobile/src/ws_client/mod.rs`, stripped of voice protocol, replaced with RuntimeEvent JSON stream).
- `components/status_badge.rs` — via lx-ui.
- `components/pulse_indicator.rs` — Execution state visual indicator (from `apps/mobile/src/components/pulse_indicator.rs`, adapted from voice states to execution states: Idle/Running/Waiting/Done/Error).
- `pages/status.rs` — Connect to lx-desktop instance, show program name, execution state, elapsed time, cumulative cost.
- `pages/events.rs` — Filtered event stream (same filter categories as desktop/TUI).
- `pages/approvals.rs` — List pending `UserPrompt` events, render confirm/choose/ask UI, post responses back to lx-desktop.

Everything NOT copied: `audio_plugin/`, `ws_client/reconnect.rs` (voice-specific), `components/agent_card.rs` (workflow-specific), `components/pipeline_card.rs` (workflow-specific), `components/transcript_bubble.rs` (voice-specific), `pages/voice.rs`, `pages/dashboard.rs` (replace with status.rs), `pages/hitl.rs` (replace with approvals.rs focused on UserPrompt not pipeline gates), `api_client/pipelines.rs`.

Dependencies: `lx-dx` (RuntimeEvent types only), `lx-ui`, `dioxus` (mobile feature), `reqwest`, `tokio-tungstenite`, `serde_json`, `uuid`.

## Phase 5: Create lx-tui ratatui terminal app

New binary crate `crates/lx-tui/`. A lightweight terminal UI for local lx program observability.

**New code (no mcp-toolbelt source to copy):**
- `main.rs` — Entry point, clap arg parsing (positional file arg), crossterm terminal setup/teardown (raw mode, alternate screen), tokio runtime, run app event loop.
- `app.rs` — `App` state struct: event ring buffer (capped at 10,000), active filter set (HashSet of event categories), scroll offset, agent list, selected agent filter, cumulative cost, elapsed time, program status.
- `ui.rs` — ratatui rendering: three-region vertical layout (header block with file/status/cost/elapsed, scrollable event list filtered by active filters and optional agent, footer with hotkey legend and active filter indicators).
- `input.rs` — Keyboard event dispatch: `a` toggle AI filter, `e` toggle Emit, `l` toggle Log, `s` toggle Shell, `m` toggle Messages, `g` toggle Agents, `p` toggle Progress, `r` toggle Errors, `*` reset to all, `Tab` cycle agent filter, `Up/Down/PgUp/PgDn` scroll, `q` quit.
- `event_loop.rs` — Async select loop merging crossterm terminal events (keyboard/resize at 60Hz poll) with EventBus subscription. On RuntimeEvent: append to ring buffer, update derived state (cost from AiCallComplete, agent list from AgentSpawned/Killed). On keyboard: dispatch to input handler, trigger re-render.

The `ProgramRunner` from lx-dx runs the lx program on a spawned tokio task. Events flow through the EventBus to the TUI subscriber. ANSI formatters from lx-dx are used to render event text into ratatui Spans with appropriate colors.

Dependencies: `lx`, `lx-dx`, `ratatui`, `crossterm`, `tokio`, `clap`, `serde_json`.

# Files Affected

**Moved:**
- `backends/dx/` → `crates/lx-dx/` (entire directory)

**Modified:**
- `Cargo.toml` (workspace root) — add lx-dx, lx-ui, lx-desktop, lx-mobile, lx-tui to members
- `crates/lx-dx/Cargo.toml` — remove standalone `[workspace]` section
- `justfile` — add `tui`, `desktop`, `mobile` recipes

**New crate `crates/lx-ui/`:**
- `Cargo.toml`
- `src/lib.rs`
- `src/pane_tree.rs`
- `src/tab_state.rs`
- `src/ws_types.rs`
- `src/pty_session.rs`
- `src/ws_endpoint.rs`
- `src/components.rs`

**New crate `crates/lx-desktop/`:**
- `Cargo.toml`
- `Dioxus.toml`
- `src/main.rs`
- `src/app.rs`
- `src/routes.rs`
- `src/layout/mod.rs`
- `src/layout/shell.rs`
- `src/layout/sidebar.rs`
- `src/server/mod.rs`
- `src/server/lx.rs`
- `src/pages/mod.rs`
- `src/pages/run.rs`
- `src/pages/terminals.rs`
- `src/pages/events.rs`
- `assets/terminal.ts`

**New crate `crates/lx-mobile/`:**
- `Cargo.toml`
- `Dioxus.toml`
- `src/main.rs`
- `src/app.rs`
- `src/routes.rs`
- `src/layout/mod.rs`
- `src/layout/shell.rs`
- `src/layout/bottom_nav.rs`
- `src/api_client.rs`
- `src/ws_client.rs`
- `src/components/mod.rs`
- `src/components/pulse_indicator.rs`
- `src/pages/mod.rs`
- `src/pages/status.rs`
- `src/pages/events.rs`
- `src/pages/approvals.rs`

**New crate `crates/lx-tui/`:**
- `Cargo.toml`
- `src/main.rs`
- `src/app.rs`
- `src/ui.rs`
- `src/input.rs`
- `src/event_loop.rs`

**Deleted:**
- `backends/dx/Cargo.lock`
- `backends/dx/target/` (build artifacts from standalone compilation)

# Task List

### Task 1: Move backends/dx into crates/lx-dx

**Subject:** Move backends/dx to crates/lx-dx and add to workspace

**Description:** Move the `backends/dx/` directory to `crates/lx-dx/`. In `crates/lx-dx/Cargo.toml`, remove the `[workspace]` section entirely. Update the `lx` dependency path from `path = "../../crates/lx"` to `path = "../lx"`. In the root `Cargo.toml`, add `"crates/lx-dx"` to the workspace members list. Delete `backends/dx/Cargo.lock` and `backends/dx/target/` if they remain after the move. Run `just diagnose` — if lx-dx references backend traits that don't exist yet in the lx crate (EmbedBackend, PaneBackend, etc.), gate those references behind `#[cfg(feature = "full-backends")]` temporarily or remove them (they belong to other work items). The goal is compilation of the workspace.

**ActiveForm:** Moving backends/dx into workspace as lx-dx

---

### Task 2: Create lx-ui crate with pane tree types and operations

**Subject:** Create lx-ui with PaneNode tree, split/close ops, and Rect computation

**Description:** Create `crates/lx-ui/`. In its `Cargo.toml`: package name `lx-ui`, edition 2024, lib-only. Dependencies: `serde = { version = "1", features = ["derive"] }`, `uuid = { version = "1", features = ["v4"] }`.

Create `src/lib.rs` exporting `pub mod pane_tree;` and `pub mod ws_types;`.

Create `src/pane_tree.rs`. Copy and adapt from `~/repos/mcp-toolbelt/apps/desktop/src/terminal/types.rs` and `~/repos/mcp-toolbelt/apps/desktop/src/terminal/tree_ops.rs`. Define `PaneNode` enum (Terminal with id/working_dir/command, Browser with id/url/devtools, Editor with id/file_path/language, Agent with id/session_id/model, Canvas with id/widget_type/config, Split with id/direction/ratio/first/second), `SplitDirection` (Horizontal, Vertical), `Rect` (left/top/width/height f64), `DividerInfo` (rect, parent_rect, direction, split_id). Implement tree operations as methods or free functions: `split(root, target_id, direction, new_pane) -> PaneNode`, `close(root, target_id) -> Option<PaneNode>`, `convert(root, target_id, replacement) -> PaneNode`, `set_ratio(root, split_id, ratio) -> PaneNode`, `compute_pane_rects(root, rect) -> Vec<(PaneNode, Rect)>`, `compute_dividers(root, rect) -> Vec<DividerInfo>`, `all_pane_ids(root) -> Vec<String>`, `first_terminal_id(root) -> Option<String>`, `find_working_dir(root, target_id) -> Option<String>`. Use `DIVIDER_SIZE_PCT = 0.4`. All operations return new trees (no mutation). Derive `Clone`, `Debug`, `Serialize`, `Deserialize` on all types.

Create `src/ws_types.rs`. Define `ClientToTerminal` enum (Input(Vec<u8>), Resize { cols: u16, rows: u16 }, Close) and `TerminalToClient` enum (Output(Vec<u8>), SessionReady { cols: u16, rows: u16 }, Closed, Error(String)). Derive `Clone`, `Debug`, `Serialize`, `Deserialize`.

Add `"crates/lx-ui"` to workspace members. Run `just diagnose`.

**ActiveForm:** Creating lx-ui crate with pane tree infrastructure

---

### Task 3: Add PTY session management to lx-ui

**Subject:** Add pty_session.rs with PTY lifecycle, I/O threads, and session caching

**Description:** Add `portable-pty = "0.8"`, `tokio = { version = "1", features = ["sync", "rt"] }`, `dashmap = "6"`, `parking_lot = "0.12"` to lx-ui dependencies.

Create `src/pty_session.rs`. Copy and adapt from `~/repos/mcp-toolbelt/apps/desktop/src/terminal/session.rs`. Define `PtySession` struct with fields: `input_tx: tokio::sync::mpsc::Sender<Vec<u8>>`, `output_tx: tokio::sync::broadcast::Sender<Vec<u8>>`, `buffer: Arc<std::sync::Mutex<Vec<u8>>>`, `master: parking_lot::Mutex<Box<dyn portable_pty::MasterPty + Send>>`, `_child: Box<dyn portable_pty::Child + Send>`. Static `SESSIONS: LazyLock<DashMap<String, Arc<PtySession>>>`. Functions: `get_or_create(id, cols, rows, working_dir, command) -> Result<Arc<PtySession>>` (reuse existing or spawn new PTY process with reader/writer threads), `subscribe(session) -> (Vec<u8>, broadcast::Receiver<Vec<u8>>)` (returns initial buffer snapshot + receiver), `send_input(session, data) -> Result<()>`, `resize(session, cols, rows) -> Result<()>`, `remove(id)`. Reader thread reads 4KB chunks from PTY output, broadcasts and appends to circular buffer (cap 256KB). Writer thread receives from input_tx mpsc and writes to PTY stdin. Both run in `std::thread` (not async).

Add `pub mod pty_session;` to lib.rs. Run `just diagnose`.

**ActiveForm:** Adding PTY session management to lx-ui

---

### Task 4: Add WebSocket terminal endpoint to lx-ui

**Subject:** Add ws_endpoint.rs with terminal WebSocket handler

**Description:** Add `tokio-tungstenite = "0.24"`, `futures = "0.3"` to lx-ui dependencies.

Create `src/ws_endpoint.rs`. Copy and adapt from `~/repos/mcp-toolbelt/apps/desktop/src/terminal/ws_endpoint.rs`. Define an async function `handle_terminal_ws(ws_stream, terminal_id, cols, rows, working_dir, command)` that: (1) calls `pty_session::get_or_create` to get or spawn a session, (2) sends `TerminalToClient::SessionReady` + any buffered output, (3) enters a `tokio::select!` loop reading from the PTY broadcast receiver and the WebSocket stream simultaneously — PTY output goes to client as `TerminalToClient::Output`, client input/resize/close messages go to the PTY session. Handle broadcast lag by skipping. Clean up on disconnect.

This function is framework-agnostic — it takes an already-upgraded `WebSocketStream` (from tokio-tungstenite), not an axum/dioxus request. The consuming app (lx-desktop) handles the HTTP upgrade and passes the stream.

Add `pub mod ws_endpoint;` to lib.rs. Run `just diagnose`.

**ActiveForm:** Adding WebSocket terminal endpoint to lx-ui

---

### Task 5: Add shared Dioxus components to lx-ui

**Subject:** Add PageHeader, StatusBadge, Skeleton components

**Description:** Add `dioxus = "0.7"` to lx-ui dependencies (no feature flags — components only, no runtime).

Create `src/components.rs`. Copy and adapt from `~/repos/mcp-toolbelt/apps/desktop/src/components/page_header.rs`, `status_badge.rs`, `skeleton.rs`.

`PageHeader` component: takes `title: String`, `subtitle: Option<String>`, children slot for action buttons. Renders as header with h1 title, optional p subtitle, and flex-end actions area.

`StatusBadge` component: takes `status: String`. Maps status strings to colors: "running"/"active" → green, "idle"/"standby" → blue, "error"/"failed" → red, "waiting"/"paused" → amber, default → gray. Renders as inline span with colored dot + text.

`Skeleton` component: takes optional `width: String`, `height: String`. Renders a div with pulse animation placeholder styling.

Add `pub mod components;` to lib.rs. Run `just diagnose`.

**ActiveForm:** Adding shared Dioxus components to lx-ui

---

### Task 6: Create lx-tui crate with app state and main entry

**Subject:** Create lx-tui crate with main.rs and app.rs

**Description:** Create `crates/lx-tui/`. In its `Cargo.toml`: package name `lx-tui`, edition 2024, binary crate. Dependencies: `lx = { path = "../lx" }`, `lx-dx = { path = "../lx-dx" }`, `ratatui = "0.29"`, `crossterm = "0.28"`, `tokio = { version = "1", features = ["rt-multi-thread", "sync", "macros"] }`, `clap = { version = "4", features = ["derive"] }`.

Create `src/main.rs`. Use clap to parse a positional `file` argument. Set up crossterm (enable raw mode, enter alternate screen, enable mouse capture). Create tokio runtime. Instantiate `App` from app.rs. Run the event loop. On exit, restore terminal (disable raw mode, leave alternate screen). Handle panics by restoring terminal before unwinding.

Create `src/app.rs`. Define `EventCategory` enum: Ai, Emit, Log, Shell, Messages, Agents, Progress, Errors. Define `App` struct with fields: `events: Vec<(lx_dx::event::RuntimeEvent, EventCategory)>` ring buffer, `filters: HashSet<EventCategory>` (initialized with all categories), `scroll: usize`, `agent_filter: Option<String>`, `agents: Vec<String>`, `cumulative_cost: f64`, `elapsed_ms: u64`, `program_status: Option<Result<String, String>>`, `source_path: String`, `should_quit: bool`. Method `categorize(event) -> EventCategory` maps RuntimeEvent variants to categories. Method `push_event(event)` appends to buffer (cap 10,000, drop oldest), updates derived state (cost from AiCallComplete, agents from AgentSpawned, status from ProgramFinished). Method `visible_events() -> Vec<&RuntimeEvent>` returns events matching active filters and agent_filter. Method `toggle_filter(cat)` adds or removes from set. Method `reset_filters()` sets all. Method `cycle_agent()` rotates through None → each agent → None.

Add `"crates/lx-tui"` to workspace members. Run `just diagnose`.

**ActiveForm:** Creating lx-tui crate with state management

---

### Task 7: Add TUI rendering to lx-tui

**Subject:** Create ui.rs with ratatui rendering for header, event list, and footer

**Description:** Create `src/ui.rs`. Define `fn render(app: &App, frame: &mut ratatui::Frame)`.

Layout: split frame into three vertical chunks — header (3 lines fixed), event list (fill), footer (2 lines fixed).

Header block: bordered, title "lx-tui". Content: `"{source_path} | {status} | cost: ${cost:.4} | {elapsed}ms"`. Status is "running" (yellow), "ok" (green), "failed" (red), or "starting" (dim).

Event list block: bordered, title "Events ({visible}/{total})". Scrollable list of events from `app.visible_events()`. Each event rendered using `lx_dx::adapters::ansi::format_event` but converted to ratatui Spans with matching Style colors (blue for AI, red for errors, dim for shell, cyan for messages, green for agent lifecycle, plain for emit, yellow for log warn, etc.). Scroll position tracks `app.scroll`. Show scroll indicator if not at bottom.

Footer block: two lines. Line 1: hotkey legend — `"a:AI  e:Emit  l:Log  s:Shell  m:Msg  g:Agent  p:Prog  r:Err  *:All  Tab:Agent  q:Quit"`. Line 2: active filter indicators — each active category shown in its color, inactive shown dim. If agent_filter is Some, show `"agent: {name}"` at right.

**ActiveForm:** Adding ratatui rendering to lx-tui

---

### Task 8: Add input handling and event loop to lx-tui

**Subject:** Create input.rs and event_loop.rs to wire keyboard, EventBus, and rendering

**Description:** Create `src/input.rs`. Define `fn handle_key(app: &mut App, key: crossterm::event::KeyEvent)`. Match on `key.code`: `Char('a')` → toggle Ai, `Char('e')` → toggle Emit, `Char('l')` → toggle Log, `Char('s')` → toggle Shell, `Char('m')` → toggle Messages, `Char('g')` → toggle Agents, `Char('p')` → toggle Progress, `Char('r')` → toggle Errors, `Char('*')` → reset_filters, `Tab` → cycle_agent, `Char('q')` or `Esc` → set should_quit, `Up` → scroll up, `Down` → scroll down, `PageUp` → scroll up 20, `PageDown` → scroll down 20, `Home` → scroll to 0, `End` → scroll to end. Clamp scroll within visible_events bounds.

Create `src/event_loop.rs`. Define `async fn run(app: &mut App, terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>, bus: Arc<lx_dx::event::EventBus>)`. Subscribe to bus. Enter loop using `tokio::select!`:
- Branch 1: `crossterm::event::poll` at 16ms (60Hz). If event available, read it, if KeyEvent call `handle_key`, if Resize mark dirty.
- Branch 2: `rx.recv()` from EventBus subscription. On Ok(event), call `app.push_event(event)`.
- After either branch: if dirty or event received, call `terminal.draw(|f| ui::render(app, f))`.
- Break if `app.should_quit` or bus closed.

Also define `fn start_program(source_path: &str, bus: Arc<lx_dx::event::EventBus>)` which spawns a tokio task that creates a `ProgramRunner` from lx-dx and calls `runner.run(source_path).await`. This runs the lx program in the background while the TUI renders events.

Wire into `main.rs`: create EventBus, start_program, create terminal backend, call `run`.

Run `just diagnose`.

**ActiveForm:** Adding input handling and event loop to lx-tui

---

### Task 9: Create lx-desktop crate with Dioxus app shell

**Subject:** Create lx-desktop with main.rs, app.rs, routes.rs, and layout

**Description:** Create `crates/lx-desktop/`. In its `Cargo.toml`: package name `lx-desktop`, edition 2024, binary crate. Dependencies: `lx = { path = "../lx" }`, `lx-dx = { path = "../lx-dx" }`, `lx-ui = { path = "../lx-ui" }`, `dioxus = { version = "0.7", features = ["desktop"] }`, `tokio = { version = "1", features = ["full"] }`, `clap = { version = "4", features = ["derive"] }`, `serde_json = "1"`, `uuid = { version = "1", features = ["v4"] }`.

Create `src/main.rs`. Launch Dioxus desktop app with `App` component.

Create `src/app.rs`. Root `App` component with `ErrorBoundary` and `Router<Route>`. Copy pattern from `~/repos/mcp-toolbelt/apps/desktop/src/app.rs`, stripping context-engine and workflow initialization.

Create `src/routes.rs`. Define `Route` enum with `#[layout(Shell)]` wrapping three routes: `#[route("/")]` Run page, `#[route("/terminals")]` Terminals page, `#[route("/events")]` Events page.

Create `src/layout/mod.rs` exporting shell and sidebar. Create `src/layout/shell.rs` — `Shell` component providing EventBus context, wrapping Sidebar + Outlet. Copy structure from `~/repos/mcp-toolbelt/apps/desktop/src/layout/shell.rs`, strip indexer status, repo listing, terminal.js injection, context-engine loaders. Create `src/layout/sidebar.rs` — collapsible sidebar with 3 nav items (Run, Terminals, Events). Copy from `~/repos/mcp-toolbelt/apps/desktop/src/layout/sidebar.rs`, reduce to 3 items.

Add `"crates/lx-desktop"` to workspace members. Run `just diagnose`.

**ActiveForm:** Creating lx-desktop Dioxus app shell

---

### Task 10: Add lx-desktop pages and server-side lx execution

**Subject:** Add Run, Terminals, Events pages and lx execution handler

**Description:** Create `src/server/mod.rs` and `src/server/lx.rs`. Copy and adapt from `~/repos/mcp-toolbelt/apps/desktop/src/server/workflow/lx.rs`. Define `LxRunState` (status, source_path, error, timestamps), `start_run` (create EventBus, ProgramRunner from lx-dx, writer factory using lx-ui PTY sessions, AgentTerminalManager from lx-dx, spawn worker thread), status query function. Strip all workflow-types imports, terminal spawn request broadcasting, execution manager references.

Create `src/pages/mod.rs` exporting run, terminals, events.

Create `src/pages/run.rs`. Text input for .lx file path. "Run" button calling `start_run`. Status display showing running/completed/failed with elapsed time and cost. File picker dialog via `rfd` crate.

Create `src/pages/terminals.rs`. Copy pane manager pattern from `~/repos/mcp-toolbelt/apps/desktop/src/pages/workflow/terminals.rs`. Use `lx_ui::pane_tree` for PaneNode ops, `lx_ui::tab_state::TabsState` for store, `lx_ui::ws_endpoint` for terminal WebSocket. Render pane tree by calling `compute_pane_rects` and rendering each pane. Tab bar and toolbar inline (simpler than mcp-toolbelt since fewer pane types). Strip "Run .lx" button (that's on the Run page now), browser/editor/agent/canvas view rendering.

Create `src/pages/events.rs`. Subscribe to EventBus via context. Render scrollable event log with filter toggle buttons (same categories as TUI). Each event formatted using lx-dx ANSI formatters converted to styled HTML spans. Agent filter dropdown.

Run `just diagnose`.

**ActiveForm:** Adding lx-desktop pages and execution handler

---

### Task 11: Create lx-mobile crate

**Subject:** Create lx-mobile Dioxus mobile app with Status, Events, Approvals pages

**Description:** Create `crates/lx-mobile/`. In its `Cargo.toml`: package name `lx-mobile`, edition 2024, binary crate. Dependencies: `lx-dx = { path = "../lx-dx" }`, `lx-ui = { path = "../lx-ui" }`, `dioxus = { version = "0.7", features = ["mobile"] }`, `reqwest = { version = "0.12", features = ["json"] }`, `tokio = { version = "1", features = ["rt-multi-thread", "sync"] }`, `tokio-tungstenite = "0.24"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `uuid = { version = "1", features = ["v4"] }`.

Create `Dioxus.toml` with mobile bundle configuration.

Create `src/main.rs` and `src/app.rs` — Dioxus mobile launch. Copy pattern from `~/repos/mcp-toolbelt/apps/mobile/src/main.rs` and `app.rs`.

Create `src/routes.rs` — three routes: Status (`/`), Events (`/events`), Approvals (`/approvals`).

Create `src/layout/mod.rs`, `src/layout/shell.rs`, `src/layout/bottom_nav.rs`. Copy from `~/repos/mcp-toolbelt/apps/mobile/src/layout/`, change tabs to Status/Events/Approvals, strip voice state and transcript context providers.

Create `src/api_client.rs`. HTTP client with base URL config (env `LX_DESKTOP_URL`, default `http://localhost:3030`). Functions: `fetch_run_status()`, `post_user_response(prompt_id, response)`. Copy reqwest patterns from `~/repos/mcp-toolbelt/apps/mobile/src/api_client/mod.rs`, strip pipeline operations.

Create `src/ws_client.rs`. WebSocket client connecting to lx-desktop's WsStream endpoint. Receives `RuntimeEvent` JSON, deserializes, feeds to app state. Copy reconnection pattern from mcp-toolbelt, strip voice protocol.

Create `src/components/mod.rs` and `src/components/pulse_indicator.rs`. Copy from `~/repos/mcp-toolbelt/apps/mobile/src/components/pulse_indicator.rs`, change states from voice FSM to execution states (Idle, Running, Waiting, Done, Error).

Create `src/pages/mod.rs`, `src/pages/status.rs`, `src/pages/events.rs`, `src/pages/approvals.rs`.

`status.rs`: connection indicator, program name, execution state with pulse indicator, elapsed time, cumulative cost. Poll via api_client.

`events.rs`: filtered event list from WebSocket stream. Same filter categories. Tap to expand event detail.

`approvals.rs`: list pending UserPrompt events. Render confirm (yes/no buttons), choose (radio list), ask (text input). On response, POST to lx-desktop which routes to DxUserBackend oneshot channel.

Add `"crates/lx-mobile"` to workspace members. Run `just diagnose`.

**ActiveForm:** Creating lx-mobile Dioxus app

---

### Task 12: Add justfile recipes and verify full workspace

**Subject:** Add tui, desktop, mobile recipes to justfile and verify everything compiles

**Description:** Edit `justfile`. Add:

```
tui file:
    cargo run -p lx-tui -- {{file}}

desktop:
    dx serve --bin lx-desktop

mobile:
    dx serve --bin lx-mobile --platform mobile
```

Run `just diagnose` to verify the entire workspace compiles without errors or warnings.

Run `just fmt` to format all new code.

Verify `just tui tests/01_basics.lx` launches (may exit immediately if the program has no AI calls, but should render start/finish events and exit cleanly).

**ActiveForm:** Adding justfile recipes and verifying workspace

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/UI_CRATE_MIGRATION.md" })
```

Then call `next_task` to begin.
