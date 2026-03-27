# Unit 10: StatusBarState — Signal-per-field → Store

## Problem

`StatusBarState` in `crates/lx-desktop/src/contexts/status_bar.rs` wraps 6 independent fields in `Signal<T>`. This struct should use `#[derive(Store)]` for granular field-level reactivity without manual Signal wrapping.

## Current Code

```rust
// crates/lx-desktop/src/contexts/status_bar.rs
#[derive(Clone, Copy)]
pub struct StatusBarState {
  pub branch: Signal<String>,
  pub line: Signal<u32>,
  pub col: Signal<u32>,
  pub encoding: Signal<String>,
  pub notification_count: Signal<usize>,
  pub pane_label: Signal<String>,
}
```

## Files

| File | Role |
|------|------|
| `crates/lx-desktop/src/contexts/status_bar.rs` | Definition — rewrite struct + provide + update_cursor |
| `crates/lx-desktop/src/layout/status_bar.rs` | Consumer — reads all 6 fields for display |
| `crates/lx-desktop/src/layout/shell.rs` | Provider — calls `StatusBarState::provide()`, sets `notification_count` |
| `crates/lx-desktop/src/terminal/view.rs` | Consumer — calls `ctx.update_cursor(line, col)` at line 121-122 inside async |

## Tasks

### 1. Rewrite `crates/lx-desktop/src/contexts/status_bar.rs`

Replace the entire file with:

```rust
use dioxus::prelude::*;

#[derive(Store, Clone, PartialEq)]
pub struct StatusBarState {
  pub branch: String,
  pub line: u32,
  pub col: u32,
  pub encoding: String,
  pub notification_count: usize,
  pub pane_label: String,
}

#[store]
impl<Lens> Store<StatusBarState, Lens> {
  fn update_cursor(&mut self, line: u32, col: u32) {
    self.line().set(line);
    self.col().set(col);
  }
}
```

The `provide()` method is removed. Provider creates the store directly with `use_store`.

### 2. Update `crates/lx-desktop/src/layout/shell.rs`

**Line 11**: Change import from `crate::contexts::status_bar::StatusBarState` — keep as-is (struct name unchanged).

**Line 34**: Replace `StatusBarState::provide()` with:
```rust
let status_bar_state = use_store(|| StatusBarState {
  branch: "main".into(),
  line: 1,
  col: 1,
  encoding: "UTF-8".into(),
  notification_count: 0,
  pane_label: "READY".into(),
});
use_context_provider(|| status_bar_state);
```

**Lines 37-39**: Replace notification count sync:
```rust
// OLD:
let count = tabs_state.read().notifications.len();
let mut notif = status_bar_state.notification_count;
notif.set(count);

// NEW:
let count = tabs_state.read().notifications.len();
status_bar_state.notification_count().set(count);
```

### 3. Update `crates/lx-desktop/src/layout/status_bar.rs`

**Line 7**: Change context retrieval:
```rust
// OLD:
let state = use_context::<StatusBarState>();

// NEW:
let state = use_context::<Store<StatusBarState>>();
```

**Lines 17-22**: Replace signal call syntax with store field accessors:
```rust
// OLD:
let pane_label = (state.pane_label)();
let branch = (state.branch)();
let line = (state.line)();
let col = (state.col)();
let encoding = (state.encoding)();
let notif_count = (state.notification_count)();

// NEW:
let pane_label = state.pane_label().cloned();
let branch = state.branch().cloned();
let line = state.line().cloned();
let col = state.col().cloned();
let encoding = state.encoding().cloned();
let notif_count = state.notification_count().cloned();
```

**Lines 8-16** (use_future for git branch): Change signal set pattern:
```rust
// OLD:
let mut branch_sig = state.branch;
branch_sig.set(branch);

// NEW:
state.branch().set(branch);
```

### 4. Update `crates/lx-desktop/src/terminal/view.rs`

**Line 121-122** (inside `EditorView` async future): Change context type and method call:
```rust
// OLD:
let ctx = use_context::<StatusBarState>();
ctx.update_cursor(line, col);

// NEW:
let ctx = use_context::<Store<StatusBarState>>();
ctx.update_cursor(line, col);
```

The `update_cursor` method is now a `#[store]` extension method. It takes `&mut self` so it works on any writable store lens. The call site is identical.

## Preconditions

- `dioxus` 0.7.3 with `Store`, `use_store`, `#[store]` available in `dioxus::prelude::*`
- The `Store` type is `Copy` (like `Signal`), so passing by value works
- `Store<T>` can be provided via `use_context_provider` and retrieved via `use_context::<Store<T>>()`

## Verification

`just diagnose` must pass with zero warnings.
