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
| `crates/lx-desktop/src/terminal/mod.rs` | Remove `pub mod voice_view;` declaration (line 6) |
| `crates/lx-desktop/src/terminal/view.rs` | Remove `pub use super::voice_view::VoiceView;` re-export (line 16) |
| `crates/lx-desktop/src/panes.rs` | Remove Voice variant from DesktopPane enum (line 12), PaneKind enum (line 37), and all 8 match arms: pane_id (line 24), kind (line 49), name (line 61), make_default (line 73), icon (line 85), PaneKind::ALL (line 91), PaneKind::icon (line 101), PaneKind::label (line 113) |
| `crates/lx-desktop/src/pages/terminals.rs` | Remove VoiceView from import (line 13), remove DesktopPane::Voice match arm from render_pane_view (lines 225-227) |

## Task List

### Task 1: Delete voice_view.rs

Delete the file `crates/lx-desktop/src/terminal/voice_view.rs`.

### Task 2: Remove voice_view module declaration

Edit `crates/lx-desktop/src/terminal/mod.rs`. Remove the line:

```rust
pub mod voice_view;
```

The remaining module declarations are: `browser_view`, `status_badge`, `tab_bar`, `toolbar`, `view`.

### Task 3: Remove VoiceView re-export

Edit `crates/lx-desktop/src/terminal/view.rs`. Remove the line:

```rust
pub use super::voice_view::VoiceView;
```

The remaining re-export (`pub use super::browser_view::{BrowserNavCtx, BrowserView};`) stays.

### Task 4: Remove Voice from pane enums and match arms

Edit `crates/lx-desktop/src/panes.rs`. Make all of the following changes in a single edit:

**DesktopPane enum** — remove the Voice variant. Before:

```rust
  Chart { id: String, chart_json: String, title: Option<String>, name: Option<String> },
  Voice { id: String, name: Option<String> },
```

After:

```rust
  Chart { id: String, chart_json: String, title: Option<String>, name: Option<String> },
```

**pane_id match** — remove `| Self::Voice { id, .. }` from the chain. Before:

```rust
      | Self::Chart { id, .. }
      | Self::Voice { id, .. } => id,
```

After:

```rust
      | Self::Chart { id, .. } => id,
```

**kind match** — remove the Voice arm. Before:

```rust
      Self::Chart { .. } => PaneKind::Chart,
      Self::Voice { .. } => PaneKind::Voice,
```

After:

```rust
      Self::Chart { .. } => PaneKind::Chart,
```

**name match** — remove `| Self::Voice { name, .. }` from the chain. Before:

```rust
      | Self::Chart { name, .. }
      | Self::Voice { name, .. } => name.as_deref(),
```

After:

```rust
      | Self::Chart { name, .. } => name.as_deref(),
```

**make_default match** — remove the Voice arm. Before:

```rust
      PaneKind::Chart => Self::Chart { id, chart_json: String::new(), title: None, name: None },
      PaneKind::Voice => Self::Voice { id, name: None },
```

After:

```rust
      PaneKind::Chart => Self::Chart { id, chart_json: String::new(), title: None, name: None },
```

**icon match** — remove the Voice arm. Before:

```rust
      Self::Chart { .. } => "\u{25A3}",
      Self::Voice { .. } => "\u{1F3A4}",
```

After:

```rust
      Self::Chart { .. } => "\u{25A3}",
```

**PaneKind enum** — remove the Voice variant. Before:

```rust
  Chart,
  Voice,
```

After:

```rust
  Chart,
```

**PaneKind::ALL** — remove PaneKind::Voice from the array. Before:

```rust
  pub const ALL: &[PaneKind] = &[PaneKind::Terminal, PaneKind::Browser, PaneKind::Editor, PaneKind::Agent, PaneKind::Canvas, PaneKind::Chart, PaneKind::Voice];
```

After:

```rust
  pub const ALL: &[PaneKind] = &[PaneKind::Terminal, PaneKind::Browser, PaneKind::Editor, PaneKind::Agent, PaneKind::Canvas, PaneKind::Chart];
```

**PaneKind::icon** — remove the Voice arm. Before:

```rust
      Self::Chart => "\u{25A3}",
      Self::Voice => "\u{1F3A4}",
```

After:

```rust
      Self::Chart => "\u{25A3}",
```

**PaneKind::label** — remove the Voice arm. Before:

```rust
      Self::Chart => "Chart",
      Self::Voice => "Voice",
```

After:

```rust
      Self::Chart => "Chart",
```

### Task 5: Remove Voice from terminals page

Edit `crates/lx-desktop/src/pages/terminals.rs`. Make two changes:

**Import** — remove VoiceView from the import. Before:

```rust
use crate::terminal::view::{AgentView, BrowserNavCtx, BrowserView, CanvasView, ChartView, EditorView, TerminalView, VoiceView};
```

After:

```rust
use crate::terminal::view::{AgentView, BrowserNavCtx, BrowserView, CanvasView, ChartView, EditorView, TerminalView};
```

**render_pane_view** — remove the Voice match arm. Before:

```rust
    DesktopPane::Chart { id, chart_json, title, .. } => rsx! {
      ChartView {
        chart_id: id.clone(),
        chart_json: chart_json.clone(),
        title: title.clone(),
      }
    },
    DesktopPane::Voice { id, .. } => rsx! {
      VoiceView { voice_id: id.clone() }
    },
```

After:

```rust
    DesktopPane::Chart { id, chart_json, title, .. } => rsx! {
      ChartView {
        chart_id: id.clone(),
        chart_json: chart_json.clone(),
        title: title.clone(),
      }
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
