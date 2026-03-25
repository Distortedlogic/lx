# Goal

Create two shell-level context structs — `StatusBarState` and `ActivityLog` — provided at the Shell component, consumed by the status bar and activity page respectively. Rewrite the StatusBar component to display live data from `StatusBarState` instead of hardcoded strings.

# Why

The status bar currently shows hardcoded values: "SYSTEM_READY_V1.0.4", "main*", "Ln 1, Col 1", "UTF-8", "Notifications (0)". None react to application state. Pane views, the activity page, and the status bar all need shared state channels to communicate. These two contexts are the foundation that WU-2 (pane engine), WU-5 (menu bar), and WU-6 (leaf pages) depend on.

# Architecture

Both contexts follow the established `VoiceContext` pattern in `pages/agents/voice_context.rs`: a `#[derive(Clone, Copy)]` struct containing `Signal<T>` fields, with a `provide()` class method that calls `use_context_provider`. Consumers use `use_context::<T>()`.

`StatusBarState` holds signals for branch name, cursor position, encoding, notification count, and a pane-type label. Any focused pane component writes to these signals; the StatusBar reads them.

`ActivityLog` holds a `Signal<VecDeque<ActivityEvent>>` capped at a configurable max. Any component (TerminalView, BrowserView, voice pipeline, etc.) pushes events; the Activity page reads them.

Both are provided in `Shell` alongside the existing `TabsState` and spawn channel context providers.

# Files Affected

| File | Change |
|------|--------|
| `src/contexts/mod.rs` | New file — module declarations |
| `src/contexts/status_bar.rs` | New file — StatusBarState context struct |
| `src/contexts/activity_log.rs` | New file — ActivityLog context struct |
| `src/lib.rs` | Add `pub mod contexts;` |
| `src/layout/shell.rs` | Provide both contexts |
| `src/layout/status_bar.rs` | Rewrite to consume StatusBarState |

# Task List

### Task 1: Create StatusBarState context struct

**Subject:** Define the StatusBarState context with signals for all status bar fields

**Description:** Create `crates/lx-desktop/src/contexts/status_bar.rs`. Define a `#[derive(Clone, Copy)]` struct `StatusBarState` with these fields:

- `branch: Signal<String>` — git branch name
- `line: Signal<u32>` — cursor line (1-based)
- `col: Signal<u32>` — cursor column (1-based)
- `encoding: Signal<String>` — file encoding label
- `notification_count: Signal<usize>` — active notification count
- `pane_label: Signal<String>` — focused pane's type/status label

Add an `impl StatusBarState` block with a `pub fn provide() -> Self` method that creates a new instance with these defaults:

- `branch`: Signal::new("main".into())
- `line`: Signal::new(1)
- `col`: Signal::new(1)
- `encoding`: Signal::new("UTF-8".into())
- `notification_count`: Signal::new(0)
- `pane_label`: Signal::new("READY".into())

The `provide` method calls `use_context_provider(|| ctx)` and returns `ctx`, exactly matching `VoiceContext::provide()` in `src/pages/agents/voice_context.rs`.

Add a `pub fn update_cursor(&self, line: u32, col: u32)` convenience method that sets both `line` and `col` signals.

Import `dioxus::prelude::*`.

**ActiveForm:** Creating StatusBarState context struct

---

### Task 2: Create ActivityLog context struct

**Subject:** Define the ActivityLog context with a capped event queue

**Description:** Create `crates/lx-desktop/src/contexts/activity_log.rs`. Define:

A `#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]` struct `ActivityEvent` with fields:
- `pub timestamp: String`
- `pub kind: String` — event category (e.g., "terminal", "browser", "voice", "pane")
- `pub message: String` — human-readable description

The `Serialize`/`Deserialize` derives are required because WU-7 (server API) uses `ActivityEvent` in JSON request/response bodies.

A `#[derive(Clone, Copy)]` struct `ActivityLog` with fields:
- `pub events: Signal<std::collections::VecDeque<ActivityEvent>>`

Add an `impl ActivityLog` block with:

- `pub fn provide() -> Self` — creates `events: Signal::new(VecDeque::new())`, calls `use_context_provider(|| ctx)`, returns `ctx`.

- `pub fn push(&self, kind: &str, message: &str)` — constructs an `ActivityEvent` with timestamp from `std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0).to_string()`. Pushes to the front of the VecDeque via `push_front`. If the length exceeds 500, calls `pop_back()` to cap the queue.

Import `dioxus::prelude::*` and `std::collections::VecDeque`.

**ActiveForm:** Creating ActivityLog context struct

---

### Task 3: Create contexts module and register in lib.rs

**Subject:** Wire the new contexts module into the crate

**Description:** Create `crates/lx-desktop/src/contexts/mod.rs` with:

```rust
pub mod activity_log;
pub mod status_bar;
```

Then edit `crates/lx-desktop/src/lib.rs`. Add `pub mod contexts;` after the existing `pub mod app;` line. The final lib.rs module list should be:

```
pub mod app;
pub mod contexts;
pub mod layout;
pub mod pages;
pub mod panes;
pub mod routes;
pub mod server; (feature-gated)
pub mod terminal;
pub mod voice_backend;
pub mod webview_permissions; (feature-gated)
```

**ActiveForm:** Wiring contexts module into crate

---

### Task 4: Provide both contexts at Shell level

**Subject:** Initialize StatusBarState and ActivityLog in Shell component

**Description:** Edit `crates/lx-desktop/src/layout/shell.rs`. Add two imports:

```rust
use crate::contexts::status_bar::StatusBarState;
use crate::contexts::activity_log::ActivityLog;
```

In the `Shell` component function body, after the existing `let tabs_state = use_provide_tabs();` line (currently line 31), add:

```rust
let status_bar_state = StatusBarState::provide();
let _activity_log = ActivityLog::provide();
```

`_activity_log` has an underscore prefix because Shell provides it but never reads it. `status_bar_state` does NOT have an underscore because it is read in the effect below. Both must be provided before the `rsx!` block.

Add a `use_effect` after the context providers that keeps `notification_count` in sync with `TabsState::notifications`:

```rust
use_effect(move || {
    let count = tabs_state.read().notifications.len();
    status_bar_state.notification_count.set(count);
});
```

**ActiveForm:** Providing shared contexts at Shell level

---

### Task 5: Rewrite StatusBar to consume StatusBarState

**Subject:** Replace all hardcoded status bar text with signal-driven values

**Description:** Edit `crates/lx-desktop/src/layout/status_bar.rs`. Replace the entire file content.

Add import:
```rust
use crate::contexts::status_bar::StatusBarState;
```

In the `StatusBar` component, get the context once at the top: `let state = use_context::<StatusBarState>();`

Read each signal in the rsx:
- Replace `"SYSTEM_READY_V1.0.4"` with `"{pane_label}"` where `let pane_label = (state.pane_label)();`
- Replace `"main*"` with `"{branch}"` where `let branch = (state.branch)();`
- Replace `"Ln 1, Col 1"` with `"Ln {line}, Col {col}"` where `let line = (state.line)();` and `let col = (state.col)();`
- Replace `"UTF-8"` with `"{encoding}"` where `let encoding = (state.encoding)();`
- Replace `"Notifications (0)"` with `"Notifications ({notif_count})"` where `let notif_count = (state.notification_count)();`

Keep the same Tailwind classes and layout structure. The only change is replacing static strings with signal reads.

Add a `use_effect` after the `use_context` call that reads the git branch on mount (reuse the `state` variable from above):

```rust
use_effect(move || {
    spawn(async move {
        if let Ok(output) = tokio::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .await
        {
            if output.status.success() {
                let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                state.branch.set(branch);
            }
        }
    });
});
```

This runs once on mount and sets the branch signal. The `tokio::process` feature is already enabled in Cargo.toml.

**ActiveForm:** Rewriting StatusBar with live signal data

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_SHELL_SHARED_STATE.md" })
```
