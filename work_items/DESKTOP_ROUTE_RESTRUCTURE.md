# Goal

Remove the Terminals and Repos routes. Add a Tools route. Make Agents the default route at `/`. Update the sidebar. Create a placeholder Tools page. Delete repos page files.

# Why

The Terminals page is being absorbed into the Agents page (WU-C). The Repos page is being replaced by a Tools page (WU-C). This unit clears the routing plumbing so WU-C can populate the pages.

# Files Affected

| File | Change |
|------|--------|
| `src/routes.rs` | Remove Terminals, Repos variants. Add Tools. Move Agents to `/` |
| `src/layout/sidebar.rs` | Remove PANES. Replace REPOS with TOOLS. |
| `src/pages/mod.rs` | Remove repos, terminals modules. Add tools. |
| `src/pages/tools/mod.rs` | New file — placeholder Tools page |
| `src/pages/repos/` | Delete entire directory (5 files) |
| `src/pages/terminals.rs` | Delete file |

# Task List

### Task 1: Create placeholder Tools page

**Subject:** Create the tools module so the route has something to render

**Description:** Create the directory `crates/lx-desktop/src/pages/tools/`. Create `crates/lx-desktop/src/pages/tools/mod.rs` with:

```rust
use dioxus::prelude::*;

#[component]
pub fn Tools() -> Element {
  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]",
        "TOOLS"
      }
    }
  }
}
```

This is a temporary placeholder. WU-C will populate it with McpPanel.

**ActiveForm:** Creating placeholder Tools page

---

### Task 2: Update routes.rs

**Subject:** Remove old routes, add new routes, change default

**Description:** Edit `crates/lx-desktop/src/routes.rs`. Replace the entire file with:

```rust
use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::accounts::Accounts;
use crate::pages::activity::Activity;
use crate::pages::agents::Agents;
use crate::pages::settings::Settings;
use crate::pages::tools::Tools;

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Agents {},
        #[route("/activity")]
        Activity {},
        #[route("/tools")]
        Tools {},
        #[route("/settings")]
        Settings {},
        #[route("/accounts")]
        Accounts {},
}
```

Changes from current:
- Removed `use crate::pages::terminals::Terminals` and `use crate::pages::repos::Repos`
- Added `use crate::pages::tools::Tools`
- `Agents {}` moved from `#[route("/agents")]` to `#[route("/")]`
- `Terminals {}` variant removed entirely
- `Repos {}` variant removed entirely
- `Tools {}` variant added at `#[route("/tools")]`

**ActiveForm:** Updating routes to remove Terminals/Repos and add Tools

---

### Task 3: Update sidebar.rs

**Subject:** Remove PANES nav item, replace REPOS with TOOLS

**Description:** Edit `crates/lx-desktop/src/layout/sidebar.rs`. In the `Sidebar` component RSX, make these changes:

1. The `NavItem` for AGENTS currently points to `Route::Agents {}`. Since Agents is now at `/`, this still works — no change needed to the NavItem itself.

2. Remove the PANES NavItem entirely (the one with `to: Route::Terminals {}`, label: "PANES", icon: "dashboard").

3. Replace the REPOS NavItem (`to: Route::Repos {}`, label: "REPOS", icon: "database") with:
```rust
NavItem { to: Route::Tools {}, label: "TOOLS", icon: "build" }
```

The final nav item order should be: AGENTS, ACTIVITY, TOOLS, SETTINGS, (spacer), ACCOUNTS.

**ActiveForm:** Updating sidebar navigation items

---

### Task 4: Update pages/mod.rs

**Subject:** Remove old page modules, add tools module

**Description:** Edit `crates/lx-desktop/src/pages/mod.rs`. Replace the entire file with:

```rust
pub mod accounts;
pub mod activity;
pub mod agents;
pub mod settings;
pub mod tools;
```

This removes `pub mod repos;` and `pub mod terminals;`, and adds `pub mod tools;`.

**ActiveForm:** Updating pages module declarations

---

### Task 5: Delete repos directory and terminals.rs

**Subject:** Remove the files that are no longer referenced

**Description:** Delete these files:
- `crates/lx-desktop/src/pages/repos/mod.rs`
- `crates/lx-desktop/src/pages/repos/state.rs`
- `crates/lx-desktop/src/pages/repos/file_tree.rs`
- `crates/lx-desktop/src/pages/repos/ast_config.rs`
- `crates/lx-desktop/src/pages/repos/chunks_panel.rs`
- `crates/lx-desktop/src/pages/terminals.rs`

After deletion, remove the empty `crates/lx-desktop/src/pages/repos/` directory.

The content of `terminals.rs` will be recreated as `pages/agents/pane_area.rs` in WU-C. The repos files are permanently deleted.

**ActiveForm:** Deleting removed page files

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_ROUTE_RESTRUCTURE.md" })
```
