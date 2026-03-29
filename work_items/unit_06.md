# Unit 6: Dashboard & Activity Pages

## Scope

Create a new Dashboard page (metrics summary, active agents panel) and rewrite the existing Activity page.

## Preconditions

- **Unit 3 is complete:** Unit 3 created a stub `pages/dashboard.rs`. This unit replaces it with a real dashboard module. The `routes.rs` Route enum already has `Dashboard {}` and `DashboardAlt {}` variants importing from `crate::pages::dashboard` -- no changes to `routes.rs` are needed.
- Unit 4 is complete: layout components (`Shell`, `Sidebar`, `CompanyRail`, `BreadcrumbBar`, `PropertiesPanel`) are functional. Shared components (`StatusIcon`, `StatusBadge`, `PriorityIcon`, `Identity`, `EmptyState`, `EntityRow`, `status_colors`) exist.
- Unit 5 is complete: `MetricCard`, `PageSkeleton`, `MarkdownBody`, `FilterBar`, `PageTabBar` exist in `src/components/`.
- The breadcrumb context is provided in Shell via `crate::contexts::breadcrumb::BreadcrumbState` (from Unit 3). Use `crate::contexts::breadcrumb::BreadcrumbEntry` for breadcrumb entries.
- `lx_api::types::ActivityEvent` has fields: `timestamp: String`, `kind: String`, `message: String`.
- `lx_api::activity_api::get_activity` is a Dioxus server function at `GET /api/activity?limit`.
- The existing activity page is at `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/activity.rs`.

## Paperclip Source References

| lx-desktop target file | Paperclip reference file |
|---|---|
| `pages/dashboard/mod.rs` | `reference/paperclip/ui/src/pages/Dashboard.tsx` |
| `pages/dashboard/active_agents_panel.rs` | `reference/paperclip/ui/src/components/ActiveAgentsPanel.tsx` |
| `pages/dashboard/activity_charts.rs` | `reference/paperclip/ui/src/components/ActivityCharts.tsx` |
| `pages/activity.rs` | `reference/paperclip/ui/src/pages/Activity.tsx` |

## Steps

### Step 1: Create the dashboard directory

**Create directory:** `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/dashboard/`

### Step 2: Create the ActiveAgentsPanel component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/dashboard/active_agents_panel.rs`

Reference: `reference/paperclip/ui/src/components/ActiveAgentsPanel.tsx`

This is an lx-specific adaptation. In lx, "agents" are lx runtime agents, not Paperclip company agents. The panel shows currently running agent processes.

```rust
use dioxus::prelude::*;
use crate::contexts::activity_log::ActivityLog;

#[component]
pub fn ActiveAgentsPanel() -> Element
```

Behavior:
- Read from `ActivityLog` context: filter events where `kind == "agent_start"` or `kind == "agent_running"` (the most recent events suggesting active agents).
- This is a simplified version. Count distinct agent names from recent "agent" kind events.
- Render:
  ```
  div {
    h3 (class: "mb-3 text-sm font-semibold uppercase tracking-wide text-gray-400") {
      "Agents"
    }
    if no_agents {
      div (class: "rounded-xl border border-gray-700 p-4") {
        p (class: "text-sm text-gray-400") { "No recent agent runs." }
      }
    } else {
      div (class: "grid grid-cols-1 gap-2 sm:grid-cols-2 xl:grid-cols-4") {
        for agent in active_agents.iter() {
          AgentRunCard { name: agent.name.clone(), status: agent.status.clone(), last_seen: agent.last_seen.clone() }
        }
      }
    }
  }
  ```

Define a helper struct and component:
```rust
struct ActiveAgent {
    name: String,
    status: String,
    last_seen: String,
}

#[component]
fn AgentRunCard(name: String, status: String, last_seen: String) -> Element
```

AgentRunCard renders:
```
div (class: "flex h-[200px] flex-col overflow-hidden rounded-xl border border-gray-700 shadow-sm bg-[var(--surface-container)]") {
  div (class: "border-b border-gray-700/60 px-3 py-3") {
    div (class: "flex items-center gap-2") {
      if status == "running" {
        span (class: "relative flex h-2.5 w-2.5 shrink-0") {
          span (class: "absolute inline-flex h-full w-full animate-ping rounded-full bg-cyan-400 opacity-70") {}
          span (class: "relative inline-flex h-2.5 w-2.5 rounded-full bg-cyan-500") {}
        }
      } else {
        span (class: "inline-flex h-2.5 w-2.5 rounded-full bg-gray-500") {}
      }
      Identity { name: name.clone(), size: "sm".to_string() }
    }
    div (class: "mt-2 text-[11px] text-gray-400") {
      "{last_seen}"
    }
  }
  div (class: "min-h-0 flex-1 overflow-y-auto p-3") {
    p (class: "text-xs text-gray-500") { "No transcript available." }
  }
}
```

Import `Identity` from `crate::components::identity::Identity`.

### Step 3: Create the ActivityCharts module

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/dashboard/activity_charts.rs`

Reference: `reference/paperclip/ui/src/components/ActivityCharts.tsx`

This module provides chart card wrappers. The actual charts use the existing ECharts bridge already loaded in Shell (via `ECHARTS_JS` and `CHARTS_JS`). For this port, create static placeholder chart components that render the card frame.

```rust
use dioxus::prelude::*;

#[component]
pub fn ChartCard(title: String, #[props(optional)] subtitle: Option<String>, children: Element) -> Element
```

Renders:
```
div (class: "border border-gray-700 rounded-lg p-4 space-y-3") {
  div {
    h3 (class: "text-xs font-medium text-gray-400") { "{title}" }
    if subtitle.is_some() {
      span (class: "text-[10px] text-gray-500") { "{subtitle}" }
    }
  }
  {children}
}
```

```rust
#[component]
pub fn ActivitySummaryChart() -> Element
```

Renders a placeholder:
```
div (class: "flex items-end gap-[3px] h-20") {
  for i in 0..14 {
    div (class: "flex-1 bg-gray-700/30 rounded-sm", style: "height: {pseudo_random_height(i)}%") {}
  }
}
```

Where `pseudo_random_height` is a simple function: `((i * 17 + 5) % 80 + 10)` to give varied bar heights.

```rust
#[component]
pub fn EventBreakdownChart() -> Element
```

Renders a placeholder:
```
div (class: "flex items-end gap-[3px] h-20") {
  for i in 0..14 {
    div (class: "flex-1 bg-emerald-700/30 rounded-sm", style: "height: {pseudo_random_height(i)}%") {}
  }
}
```

### Step 4: Create the Dashboard page

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/dashboard/mod.rs`

Reference: `reference/paperclip/ui/src/pages/Dashboard.tsx`

```rust
pub mod active_agents_panel;
pub mod activity_charts;

use dioxus::prelude::*;
use crate::components::empty_state::EmptyState;
use crate::components::metric_card::MetricCard;
use crate::components::page_skeleton::PageSkeleton;
use crate::contexts::activity_log::ActivityLog;
use crate::contexts::breadcrumb::BreadcrumbEntry;
use self::active_agents_panel::ActiveAgentsPanel;
use self::activity_charts::{ChartCard, ActivitySummaryChart, EventBreakdownChart};

#[component]
pub fn Dashboard() -> Element
```

Behavior:
- Set breadcrumbs on mount:
  ```rust
  let breadcrumb_state = use_context::<crate::contexts::breadcrumb::BreadcrumbState>();
  use_effect(move || { breadcrumb_state.set(vec![BreadcrumbEntry { label: "Dashboard".into(), href: None }]); });
  ```
- Read `ActivityLog` to compute summary metrics:
  - `total_events`: count of all events.
  - `agent_events`: count of events where `kind` contains "agent".
  - `tool_events`: count of events where `kind` contains "tool".
  - `error_events`: count of events where `kind` contains "error" or `message` contains "error" (case-insensitive).
- If `total_events == 0`, render:
  ```
  EmptyState { icon: "dashboard", message: "No activity recorded yet. Run an agent to see metrics here." }
  ```
- Otherwise render:
  ```
  div (class: "space-y-6") {
    ActiveAgentsPanel {}

    div (class: "grid grid-cols-2 xl:grid-cols-4 gap-2") {
      MetricCard { icon: "pulse_alert", value: "{total_events}", label: "Total Events" }
      MetricCard { icon: "smart_toy", value: "{agent_events}", label: "Agent Events" }
      MetricCard { icon: "build", value: "{tool_events}", label: "Tool Events" }
      MetricCard { icon: "error", value: "{error_events}", label: "Errors" }
    }

    div (class: "grid grid-cols-2 lg:grid-cols-4 gap-4") {
      ChartCard { title: "Activity".into(), subtitle: "Last 14 events".into(),
        ActivitySummaryChart {}
      }
      ChartCard { title: "Event Breakdown".into(), subtitle: "By type".into(),
        EventBreakdownChart {}
      }
    }

    div (class: "min-w-0") {
      h3 (class: "text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3") {
        "Recent Activity"
      }
      div (class: "border border-gray-700 divide-y divide-gray-700 overflow-hidden") {
        for event in recent_events.iter().take(10) {
          div (class: "px-4 py-2.5 text-sm hover:bg-white/5 transition-colors") {
            div (class: "flex gap-3") {
              p (class: "flex-1 min-w-0 truncate") {
                span (class: "text-gray-400 font-mono text-xs") { "{event.kind}" }
                span (class: "ml-2") { "{event.message}" }
              }
              span (class: "text-xs text-gray-500 shrink-0") { "{event.timestamp}" }
            }
          }
        }
      }
    }
  }
  ```

### Step 5: Rewrite the Activity page

**Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/activity.rs`

Replace the entire contents. Reference: `reference/paperclip/ui/src/pages/Activity.tsx`

```rust
use dioxus::prelude::*;
use crate::components::empty_state::EmptyState;
use crate::components::page_skeleton::PageSkeleton;
use crate::contexts::activity_log::ActivityLog;
use crate::contexts::breadcrumb::BreadcrumbEntry;

#[component]
pub fn Activity() -> Element
```

Behavior:
- Set breadcrumbs on mount:
  ```rust
  let breadcrumb_state = use_context::<crate::contexts::breadcrumb::BreadcrumbState>();
  use_effect(move || { breadcrumb_state.set(vec![BreadcrumbEntry { label: "Activity".into(), href: None }]); });
  ```
- Read `ActivityLog` context.
- Add a filter signal: `use_signal(|| "all".to_string())`.
- Compute `entity_types`: collect unique `kind` values from all events, sorted.
- Compute `filtered`: if filter is `"all"`, show all events; otherwise filter by `kind == filter`.
- Render:
  ```
  div (class: "space-y-4") {
    div (class: "flex items-center justify-end") {
      select (class: "h-8 rounded-md border border-gray-600 bg-gray-800 px-2 py-1 text-xs focus:outline-none focus:ring-1 focus:ring-blue-500",
              value: "{filter}",
              onchange: move |evt| filter.set(evt.value())) {
        option (value: "all") { "All types" }
        for kind in entity_types.iter() {
          option (value: "{kind}") { "{kind}" }
        }
      }
    }

    if filtered.is_empty() {
      EmptyState { icon: "history", message: "No activity recorded yet.".into() }
    } else {
      div (class: "border border-gray-700 divide-y divide-gray-700 overflow-hidden") {
        for event in filtered.iter() {
          div (class: "flex items-center px-4 py-2.5 hover:bg-white/5 transition-colors text-sm") {
            span (class: "w-40 shrink-0 text-gray-500 font-mono text-xs") { "{event.timestamp}" }
            span (class: "w-28 shrink-0 text-[var(--primary)] uppercase font-semibold text-xs") { "{event.kind}" }
            span (class: "flex-1 text-gray-300 truncate") { "{event.message}" }
          }
        }
      }
    }
  }
  ```

### Step 6: Replace the dashboard stub

Unit 3 created a stub `pages/dashboard.rs`. This unit replaces it with a real dashboard module. Delete `src/pages/dashboard.rs` (the Unit 3 stub) and create `src/pages/dashboard/mod.rs` with the real Dashboard component. The `pub mod dashboard;` declaration already exists in `pages/mod.rs` from Unit 3, and `routes.rs` already imports `Dashboard` and `DashboardAlt` from `crate::pages::dashboard` -- no changes to either file are needed. The Rust module system automatically resolves the directory module (`dashboard/mod.rs`) in place of the former single-file module (`dashboard.rs`).

### Step 8: Add Dashboard to Sidebar navigation

**Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/sidebar.rs`

In the main nav section (the `div` with `class: "flex flex-col gap-0.5"` containing the top-level nav items), add a `SidebarNavItem` for Dashboard before Agents:

```rust
SidebarNavItem { to: Route::Dashboard {}, label: "Dashboard", icon: "dashboard" }
SidebarNavItem { to: Route::Agents {}, label: "Agents", icon: "smart_toy" }
SidebarNavItem { to: Route::Activity {}, label: "Activity", icon: "pulse_alert" }
```

## Files Created

| File | Lines (approx) |
|---|---|
| `src/pages/dashboard/mod.rs` | ~90 |
| `src/pages/dashboard/active_agents_panel.rs` | ~80 |
| `src/pages/dashboard/activity_charts.rs` | ~55 |

## Files Modified

| File | Change |
|---|---|
| `src/pages/activity.rs` | Full rewrite — Paperclip-style with filter dropdown, breadcrumb setting, EmptyState |
| `src/pages/dashboard.rs` | Delete Unit 3 stub (replaced by `dashboard/mod.rs` directory module) |
| `src/layout/sidebar.rs` | Add Dashboard nav item |

## Definition of Done

1. `just diagnose` passes with no errors.
2. Navigating to `/dashboard` renders the Dashboard page with:
   - An ActiveAgentsPanel section showing either "No recent agent runs." or agent cards.
   - A 4-column metric cards row (Total Events, Agent Events, Tool Events, Errors).
   - Chart card placeholders (Activity, Event Breakdown).
   - A "Recent Activity" list showing the last 10 activity events.
3. Navigating to `/activity` renders the rewritten Activity page with:
   - A filter dropdown in the top-right showing "All types" plus unique event kinds.
   - Filtering by kind shows only matching events.
   - An EmptyState when no events exist.
   - Each event row shows timestamp, kind, and message in a three-column layout.
4. The Sidebar shows "Dashboard" as the first nav item, navigating to `/dashboard`.
5. Both pages set breadcrumbs via the `BreadcrumbState` context (from `crate::contexts::breadcrumb`).
6. No file exceeds 300 lines.
