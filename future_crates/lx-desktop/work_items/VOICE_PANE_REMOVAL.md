# Voice Pane Removal

## Goal

Remove the Voice variant from the pane system entirely. Delete the VoiceView component, remove DesktopPane::Voice and PaneKind::Voice from all enum definitions and match arms, and clean up the module declarations and re-exports. After this, voice is only accessible from the Agents page.

## Prerequisites

VOICE_AGENTS_PAGE_INTEGRATION must be completed first. The voice pipeline must be working on the Agents page before the pane surface is removed.

## Why

- Voice now lives on the Agents page — the pane variant is dead code
- Keeping it creates confusion about where voice lives and adds maintenance burden to every match arm across 4 files
- The tab bar "+" menu should not offer Voice as a pane type when it belongs on the Agents page

## What changes

One file deleted, four files edited.

## Files affected

| File | Change |
|------|--------|
| `crates/lx-desktop/src/terminal/voice_view.rs` | Delete file |
| `crates/lx-desktop/src/terminal/mod.rs` | Remove `pub mod voice_view;` line |
| `crates/lx-desktop/src/terminal/view.rs` | Remove `pub use super::voice_view::VoiceView;` re-export |
| `crates/lx-desktop/src/panes.rs` | Remove Voice variant from DesktopPane enum, PaneKind enum, and all 8 match arms |
| `crates/lx-desktop/src/pages/terminals.rs` | Remove DesktopPane::Voice arm from render_pane_view, remove VoiceView from import |

## Task List

### Task 1: Delete voice_view.rs

Delete the file `crates/lx-desktop/src/terminal/voice_view.rs`.

### Task 2: Remove voice_view module declaration

Edit `crates/lx-desktop/src/terminal/mod.rs`. Remove the line:

```rust
pub mod voice_view;
```

### Task 3: Remove VoiceView re-export

Edit `crates/lx-desktop/src/terminal/view.rs`. Remove the line:

```rust
pub use super::voice_view::VoiceView;
```

### Task 4: Remove Voice from pane enums and match arms

Edit `crates/lx-desktop/src/panes.rs`. Make these changes:

In the `DesktopPane` enum, remove the variant:
```rust
  Voice { id: String, name: Option<String> },
```

In `impl Pane for DesktopPane`, in the `pane_id` method, remove:
```rust
      | Self::Voice { id, .. } => id,
```
from the match arm chain (keep the remaining arms chained with `|`).

In `impl DesktopPane`, in the `kind` method, remove:
```rust
      Self::Voice { .. } => PaneKind::Voice,
```

In the `name` method, remove:
```rust
      | Self::Voice { name, .. } => name.as_deref(),
```
from the match arm chain.

In the `make_default` method, remove:
```rust
      PaneKind::Voice => Self::Voice { id, name: None },
```

In the `icon` method, remove:
```rust
      Self::Voice { .. } => "\u{1F3A4}",
```

In the `PaneKind` enum, remove the `Voice` variant.

In `PaneKind::ALL`, remove `PaneKind::Voice` from the array.

In `PaneKind::icon`, remove:
```rust
      Self::Voice => "\u{1F3A4}",
```

In `PaneKind::label`, remove:
```rust
      Self::Voice => "Voice",
```

### Task 5: Remove Voice from terminals page

Edit `crates/lx-desktop/src/pages/terminals.rs`. In the import line that reads:

```rust
use crate::terminal::view::{AgentView, BrowserNavCtx, BrowserView, CanvasView, ChartView, EditorView, TerminalView, VoiceView};
```

Remove `VoiceView` from the import list:

```rust
use crate::terminal::view::{AgentView, BrowserNavCtx, BrowserView, CanvasView, ChartView, EditorView, TerminalView};
```

In the `render_pane_view` function, remove the entire match arm:

```rust
    DesktopPane::Voice { id, .. } => rsx! {
      VoiceView { voice_id: id.clone() }
    },
```

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "future_crates/lx-desktop/work_items/VOICE_PANE_REMOVAL.md" })
```

Then call `next_task` to begin.
