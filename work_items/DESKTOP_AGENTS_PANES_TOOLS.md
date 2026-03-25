# Goal

Merge the pane system into the Agents page (agent controls top, panes bottom). Move McpPanel from agents to the Tools page.

# Why

Agents spawn and use panes. Separating them into different pages forces context-switching. Merging them makes the Agents page the primary workspace. MCP server configuration is a tools concern, not an agent concern.

# Prerequisites

WU-B (Route Restructure) must be completed first. The Terminals route and terminals.rs are already deleted. The Tools page placeholder exists.

# Current state after WU-B

- `pages/terminals.rs` is deleted — its 280-line content needs to be recreated in agents
- `pages/agents/mod.rs` still imports and renders McpPanel
- `pages/tools/mod.rs` is a placeholder with just a header
- `pages/agents/mcp_panel.rs` still exists in agents directory

# Files Affected

| File | Change |
|------|--------|
| `src/pages/agents/pane_area.rs` | New file — pane system from old terminals.rs |
| `src/pages/agents/mod.rs` | Remove McpPanel, add PaneArea, new layout |
| `src/pages/tools/mcp_panel.rs` | New file — moved from agents |
| `src/pages/tools/mod.rs` | Rewrite with McpPanel |
| `src/pages/agents/mcp_panel.rs` | Delete after moving |

# Task List

### Task 1: Create pane_area.rs from old terminals.rs content

**Subject:** Recreate the pane system as a component inside the agents page

**Description:** Create `crates/lx-desktop/src/pages/agents/pane_area.rs`. This file contains the entire pane rendering system that was previously in `pages/terminals.rs` (deleted in WU-B). The component is renamed from `Terminals` to `PaneArea`.

Read the git history to get the exact content: run `git show HEAD~3:crates/lx-desktop/src/pages/terminals.rs` (the file existed 3 commits ago, before the route restructure deleted it). Copy the entire content into `pane_area.rs` with ONE change: rename the `Terminals` component to `PaneArea`:

```rust
#[component]
pub fn PaneArea() -> Element {
```

Everything else stays identical: `create_new_tab`, `split_pane`, `close_pane`, `render_tab`, `PaneItem`, `render_pane_view`, `render_divider_item` — all unchanged.

The imports at the top reference `crate::panes::DesktopPane`, `crate::terminal::tab_bar::TabBar`, `crate::terminal::toolbar::PaneToolbar`, `crate::terminal::use_tabs_state`, and `crate::terminal::view::*` — these all still exist and are valid.

**ActiveForm:** Creating pane_area.rs with pane system from old terminals.rs

---

### Task 2: Rewrite Agents page with PaneArea

**Subject:** Merge agent controls and pane system into one page layout

**Description:** Edit `crates/lx-desktop/src/pages/agents/mod.rs`.

First, change the module declarations. Remove `mod mcp_panel;` and its import. Add `mod pane_area;` and `use self::pane_area::PaneArea;`.

The module declarations should be:

```rust
mod agent_card;
mod pane_area;
mod voice_banner;
mod voice_context;
```

The imports should be:

```rust
use dioxus::prelude::*;

use self::agent_card::AgentCard;
use self::pane_area::PaneArea;
use self::voice_banner::VoiceBanner;
use self::voice_context::VoiceContext;
```

Rewrite the `Agents` component RSX to a vertical split layout — agent controls in a collapsible top section, pane system filling the remaining space:

```rust
#[component]
pub fn Agents() -> Element {
  let ctx = VoiceContext::provide();
  let session_short = &crate::voice_backend::SESSION_ID[..8];
  let status_text = (ctx.status)().to_string();

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "shrink-0 p-4 flex flex-col gap-4 border-b border-[var(--outline-variant)]/15 max-h-[40%] overflow-auto",
        div { class: "flex items-center justify-between",
          div {
            h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]",
              "AGENT_MANAGER"
            }
            p { class: "text-xs text-[var(--outline)] uppercase tracking-wider mt-1",
              "SESSION: {session_short}"
            }
          }
          span { class: "text-xs text-[var(--outline)] uppercase tracking-wider",
            "STATUS: {status_text}"
          }
        }
        VoiceBanner {}
        AgentCard {}
      }
      div { class: "flex-1 min-h-0",
        PaneArea {}
      }
    }
  }
}
```

The top section has `shrink-0` so it doesn't collapse, `max-h-[40%]` so it never takes more than 40% of the page, and `overflow-auto` for scrolling if content overflows. The bottom section has `flex-1 min-h-0` so it fills all remaining space — this is where the tab bar and panes render.

**ActiveForm:** Rewriting Agents page with merged pane system

---

### Task 3: Move mcp_panel.rs to tools directory

**Subject:** Relocate McpPanel from agents to tools

**Description:** Read the current content of `crates/lx-desktop/src/pages/agents/mcp_panel.rs`. Create `crates/lx-desktop/src/pages/tools/mcp_panel.rs` with identical content. Then delete `crates/lx-desktop/src/pages/agents/mcp_panel.rs`.

The file content does not change — it's a pure file move. The McpPanel component, the `load_mcp_servers` async function, and all imports stay exactly the same.

**ActiveForm:** Moving mcp_panel.rs from agents to tools

---

### Task 4: Populate Tools page with McpPanel

**Subject:** Replace the placeholder Tools page with actual content

**Description:** Edit `crates/lx-desktop/src/pages/tools/mod.rs`. Replace the placeholder content with:

```rust
mod mcp_panel;

use dioxus::prelude::*;

use self::mcp_panel::McpPanel;

#[component]
pub fn Tools() -> Element {
  rsx! {
    div { class: "flex flex-col h-full gap-4 p-4 overflow-auto",
      div { class: "flex items-center justify-between",
        h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]",
          "TOOLS"
        }
      }
      McpPanel {}
    }
  }
}
```

**ActiveForm:** Populating Tools page with McpPanel

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_AGENTS_PANES_TOOLS.md" })
```
