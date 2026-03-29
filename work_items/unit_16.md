# Unit 16: Command Palette & Keyboard Shortcuts

## Scope

Port the Cmd+K command palette (fuzzy search over routes, agents, issues, projects), global keyboard shortcut handler, and three hooks (inbox badge, autosave indicator, date range) from Paperclip React to Dioxus 0.7.3 in lx-desktop.

## Paperclip Source Files

| Paperclip file | Purpose |
|---|---|
| `reference/paperclip/ui/src/components/CommandPalette.tsx` | Cmd+K overlay dialog with fuzzy search across actions, pages, issues, agents, projects |
| `reference/paperclip/ui/src/hooks/useKeyboardShortcuts.ts` | Global key handler: C=new issue, [=toggle sidebar, ]=toggle panel |
| `reference/paperclip/ui/src/hooks/useInboxBadge.ts` | Aggregates badge count from approvals, join requests, dashboard, heartbeats, issues |
| `reference/paperclip/ui/src/hooks/useAutosaveIndicator.ts` | State machine (idle/saving/saved/error) with debounced transitions |
| `reference/paperclip/ui/src/hooks/useDateRange.ts` | Date preset picker (mtd/7d/30d/ytd/all/custom) with minute-tick refresh |

## Preconditions

- **Unit 1 is complete:** `src/components/mod.rs` already exists. `src/lib.rs` already contains `pub mod components;`.
- **Units 1-15 are complete:** `lib.rs` already contains `pub mod components;`, `pub mod hooks;` (if added by Unit 15), `pub mod plugins;`.
- `crates/lx-desktop/src/routes.rs` exists with `Route` enum (from Unit 3, with all route variants)
- `crates/lx-desktop/src/layout/shell.rs` exists with `Shell` component
- `crates/lx-desktop/src/layout/sidebar.rs` exists with `Sidebar` component
- `crates/lx-desktop/src/contexts/mod.rs` exists with all context modules from Units 3+

## Files Affected

| File | Action |
|---|---|
| `crates/lx-desktop/src/components/mod.rs` | Create: module declarations for command_palette and hooks submodules |
| `crates/lx-desktop/src/components/command_palette.rs` | Create: CommandPalette component |
| `crates/lx-desktop/src/hooks/mod.rs` | Create: module declarations |
| `crates/lx-desktop/src/hooks/keyboard_shortcuts.rs` | Create: use_keyboard_shortcuts hook |
| `crates/lx-desktop/src/hooks/autosave_indicator.rs` | Create: use_autosave_indicator hook |
| `crates/lx-desktop/src/hooks/date_range.rs` | Create: use_date_range hook |
| `crates/lx-desktop/src/hooks/inbox_badge.rs` | Create: use_inbox_badge hook |
| `crates/lx-desktop/src/lib.rs` | Modify: add `pub mod hooks;` (`pub mod components;` already exists from Unit 1) |
| `crates/lx-desktop/src/layout/shell.rs` | Modify: add `CommandPalette {}` and `use_keyboard_shortcuts()` call |

## Tasks

### 1. Create `crates/lx-desktop/src/hooks/mod.rs`

```rust
pub mod autosave_indicator;
pub mod date_range;
pub mod inbox_badge;
pub mod keyboard_shortcuts;
```

### 2. Create `crates/lx-desktop/src/hooks/keyboard_shortcuts.rs`

Port `useKeyboardShortcuts.ts`. The hook registers a global `keydown` listener via `use_effect` and dispatches to caller-provided callbacks.

Struct and function signatures:

```rust
use dioxus::prelude::*;

pub struct ShortcutHandlers {
    pub on_new_issue: Option<EventHandler<()>>,
    pub on_toggle_sidebar: Option<EventHandler<()>>,
    pub on_toggle_panel: Option<EventHandler<()>>,
}

pub fn use_keyboard_shortcuts(handlers: ShortcutHandlers) { ... }
```

Implementation details:

- Use `use_effect` with a `document::eval` call to register a JS `keydown` listener (Dioxus desktop runs in a webview; direct DOM event listeners require JS eval or `onkeydown` on a root element).
- Alternative approach: attach `onkeydown` to the outermost `div` in `Shell` and pass it down. This is simpler in Dioxus 0.7.3.
- Chosen approach: add `onkeydown` handler to the Shell root div in task 8 instead of JS eval. The `use_keyboard_shortcuts` function returns an `EventHandler<KeyboardEvent>` that Shell wires to `onkeydown`.

Revised signature:

```rust
pub fn use_keyboard_shortcuts(handlers: ShortcutHandlers) -> EventHandler<KeyboardEvent>
```

Logic inside the returned handler:
- If `event.target` is an input/textarea element (check via `event.data().code()` is insufficient; use a `prevent_in_input` signal set to false), skip. Since Dioxus `KeyboardEvent` does not expose target tag, use a Signal<bool> `input_focused` that the Shell can set.
- If key is `c` (no meta/ctrl/alt): call `on_new_issue`
- If key is `[` (no meta/ctrl): call `on_toggle_sidebar`
- If key is `]` (no meta/ctrl): call `on_toggle_panel`

### 3. Create `crates/lx-desktop/src/hooks/autosave_indicator.rs`

Port `useAutosaveIndicator.ts`. This is a state machine tracking save lifecycle.

```rust
use std::time::Duration;
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutosaveState {
    Idle,
    Saving,
    Saved,
    Error,
}

pub struct AutosaveIndicator {
    pub state: Signal<AutosaveState>,
    save_id: Signal<u64>,
}

impl AutosaveIndicator {
    pub fn state(&self) -> AutosaveState { ... }
    pub fn mark_dirty(&self) { ... }
    pub fn reset(&self) { ... }
    pub async fn run_save<F, Fut>(&self, save: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    { ... }
}

pub fn use_autosave_indicator() -> AutosaveIndicator { ... }
```

Constants: `SAVING_DELAY: Duration = Duration::from_millis(250)`, `SAVED_LINGER: Duration = Duration::from_millis(1600)`.

`run_save` logic:
1. Increment `save_id`
2. Spawn a delayed task that sets state to `Saving` after `SAVING_DELAY` if `save_id` matches
3. Await the provided future
4. On success: set `Saved`, spawn a delayed task to set `Idle` after `SAVED_LINGER`
5. On error: set `Error`, re-return the error
6. All delayed tasks check `save_id` matches before mutating state

### 4. Create `crates/lx-desktop/src/hooks/date_range.rs`

Port `useDateRange.ts`. Provides date preset selection with computed ISO range strings.

```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePreset {
    Mtd,
    Last7d,
    Last30d,
    Ytd,
    All,
    Custom,
}

impl DatePreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Mtd => "Month to Date",
            Self::Last7d => "Last 7 Days",
            Self::Last30d => "Last 30 Days",
            Self::Ytd => "Year to Date",
            Self::All => "All Time",
            Self::Custom => "Custom",
        }
    }

    pub fn all() -> &'static [DatePreset] {
        &[Self::Mtd, Self::Last7d, Self::Last30d, Self::Ytd, Self::All, Self::Custom]
    }
}

pub struct DateRangeState {
    pub preset: Signal<DatePreset>,
    pub custom_from: Signal<String>,
    pub custom_to: Signal<String>,
    pub from: Memo<String>,
    pub to: Memo<String>,
    pub custom_ready: Memo<bool>,
}

pub fn use_date_range() -> DateRangeState { ... }
```

`compute_range` is a pure function taking `DatePreset` and returning `(String, String)` using `chrono::Utc::now()`. Add `chrono` to Cargo.toml dependencies if not already present (check first; if it is a transitive dep, add it directly).

Minute-tick: use `use_future` that sleeps until the next minute boundary, then loops every 60 seconds, writing to a `Signal<String>` that the `Memo` depends on.

### 5. Create `crates/lx-desktop/src/hooks/inbox_badge.rs`

Port `useInboxBadge.ts`. In the Paperclip version, this aggregates data from 5 API queries (approvals, join requests, dashboard, heartbeats, issues). The lx-desktop does not have these API endpoints. Create a placeholder that returns a `Signal<usize>` badge count.

```rust
use dioxus::prelude::*;

pub struct InboxBadge {
    pub count: Signal<usize>,
    pub has_unread: Memo<bool>,
}

pub fn use_inbox_badge() -> InboxBadge { ... }
```

Implementation: initialize `count` to `0`. `has_unread` is a `Memo` returning `count() > 0`. The body contains a `// TODO: wire to real API queries when lx-api exposes inbox endpoints` comment is NOT allowed per code style rules. Instead, just return the zero-initialized signals. When API endpoints are added in Unit 17, this hook will be updated to consume them.

### 6. Edit `crates/lx-desktop/src/components/mod.rs` (already exists)

Add `pub mod command_palette;` to the existing `components/mod.rs`. Do NOT recreate the file.

### 7. Create `crates/lx-desktop/src/components/command_palette.rs`

Port `CommandPalette.tsx`. The Dioxus version uses signals and conditional RSX.

```rust
use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn CommandPalette() -> Element { ... }
```

State signals:
- `open: Signal<bool>` - whether palette is visible
- `query: Signal<String>` - current search text

The component:

1. Registers a `Cmd+K` / `Ctrl+K` keyboard handler. Use `use_effect` with `document::eval` to add a global JS keydown listener that posts a message back. Alternatively, rely on the Shell-level `onkeydown` handler. Chosen approach: the Shell `onkeydown` will check for Cmd+K and set a context signal. The `CommandPalette` reads that signal.

   Create a context signal `CommandPaletteOpen: Signal<bool>` provided by `CommandPalette` via `use_context_provider`, and consumed by Shell's keydown handler.

   Revised approach: `CommandPalette` provides `use_context_provider(|| Signal::new(false))` with a newtype `CommandPaletteOpen(Signal<bool>)`. Shell reads this context in its keydown handler.

2. When `open` is false, render nothing (return `None`).

3. When `open` is true, render:
   - A full-screen backdrop div with `onclick` to close
   - A centered modal div with:
     - A text input at the top (search box) with placeholder "Search pages, actions..."
     - Filtered result list

4. The result list contains these static groups:

   **Actions group:**
   - "Create new issue" (navigates nowhere yet, placeholder)
   - "Create new agent" (placeholder)

   **Pages group** (always shown, filtered by query):
   - "Agents" -> `Route::Agents {}`
   - "Activity" -> `Route::Activity {}`
   - "Tools" -> `Route::Tools {}`
   - "Settings" -> `Route::Settings {}`
   - "Accounts" -> `Route::Accounts {}`

5. Filtering: if `query` is non-empty, filter items whose label contains the query (case-insensitive).

6. Each item is a `div` with `onclick` that navigates using `navigator().push(route)` and closes the palette.

7. Material icon for each entry: `smart_toy` for Agents, `pulse_alert` for Activity, `build` for Tools, `settings` for Settings, `account_circle` for Accounts, `add` for create actions.

8. Clear query when palette closes (via `use_effect` watching `open`).

RSX structure:

```rust
if *open.read() {
    div { class: "fixed inset-0 z-50 bg-black/50",
        onclick: move |_| open.set(false),
        div { class: "fixed top-[20%] left-1/2 -translate-x-1/2 w-full max-w-md bg-[var(--surface-container)] border border-[var(--outline)] shadow-2xl z-50",
            onclick: move |e| e.stop_propagation(),
            input {
                class: "w-full px-4 py-3 bg-transparent border-b border-[var(--outline-variant)] text-[var(--on-surface)] text-sm outline-none placeholder:text-[var(--outline)]",
                placeholder: "Search pages, actions...",
                value: "{query}",
                oninput: move |e| query.set(e.value()),
                autofocus: true,
            }
            div { class: "max-h-64 overflow-y-auto py-1",
                // Render filtered items here
            }
        }
    }
}
```

Each result item:

```rust
div {
    class: "flex items-center gap-3 px-4 py-2 text-sm cursor-pointer hover:bg-[var(--surface-container-highest)] text-[var(--on-surface)]",
    onclick: move |_| { /* navigate and close */ },
    span { class: "material-symbols-outlined text-lg text-[var(--outline)]", "{icon}" }
    span { "{label}" }
}
```

### 8. Modify `crates/lx-desktop/src/layout/shell.rs`

Add the command palette and keyboard shortcut integration.

**Import additions** (add after existing imports at top of file):

```rust
use crate::components::command_palette::CommandPalette;
```

**Inside the `Shell` component**, add `CommandPalette {}` as the last child of the root div (after `StatusBar {}`):

```rust
StatusBar {}
CommandPalette {}
```

### 9. Modify `crates/lx-desktop/src/lib.rs`

Edit `lib.rs` -- add `pub mod hooks;` after `pub mod contexts;`. Note: `pub mod components;` already exists (added by Unit 1). Do NOT re-add it.

## Line Count Verification

| File | Estimated lines |
|---|---|
| `components/mod.rs` | 2 |
| `components/command_palette.rs` | ~120 |
| `hooks/mod.rs` | 5 |
| `hooks/keyboard_shortcuts.rs` | ~45 |
| `hooks/autosave_indicator.rs` | ~80 |
| `hooks/date_range.rs` | ~100 |
| `hooks/inbox_badge.rs` | ~20 |
| `layout/shell.rs` (modified) | 205 (was 204, +1 line) |
| `lib.rs` (modified) | 14 (was 12, +2 lines) |

All under 300 lines.

## Definition of Done

1. `just diagnose` passes with zero warnings
2. `CommandPalette` renders when Cmd+K / Ctrl+K is pressed (verified by running the desktop app)
3. Typing in the palette filters the pages list
4. Clicking a page item navigates to that route and closes the palette
5. Clicking the backdrop closes the palette
6. `use_autosave_indicator` compiles and returns valid state transitions
7. `use_date_range` compiles and `compute_range` returns correct ISO strings for each preset
8. `use_inbox_badge` compiles and returns a zero count
9. All new files are under 300 lines
10. No code comments or doc strings in new files
