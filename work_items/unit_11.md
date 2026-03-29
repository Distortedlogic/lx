# Unit 11: Routines & Org Chart Pages

Port the Routines list/detail pages with schedule editor, and Org Chart visualization page from Paperclip React to Dioxus 0.7.3 in lx-desktop.

## Paperclip Source Files

| Paperclip File | Purpose |
|---|---|
| `reference/paperclip/ui/src/pages/Routines.tsx` | Routines list with table, enable/disable toggle, create dialog |
| `reference/paperclip/ui/src/pages/RoutineDetail.tsx` | Routine detail with triggers, runs, activity tabs |
| `reference/paperclip/ui/src/components/ScheduleEditor.tsx` | Cron schedule editor with presets and custom expression |
| `reference/paperclip/ui/src/pages/OrgChart.tsx` | SVG-based org chart with pan/zoom and card nodes |
| `reference/paperclip/ui/src/pages/Org.tsx` | Tree-view org chart (text-based, collapsible) |
| `reference/paperclip/ui/src/components/ReportsToPicker.tsx` | Dropdown picker for selecting an agent's manager |

## Preconditions

1. **Unit 3 is complete:** Unit 3 created stubs `pages/routines.rs` and `pages/org.rs`. This unit replaces them with real modules. Delete each stub file and create directory modules at those paths (e.g., `src/pages/routines/mod.rs` and `src/pages/org/mod.rs`). The `routes.rs` Route enum already has `Routines {}`, `RoutineDetail { routine_id: String }`, and `OrgChart {}` variants importing from `crate::pages::routines` and `crate::pages::org` -- no changes to `routes.rs` are needed.
2. Unit 10 is complete: `pages/mod.rs` declares modules, sidebar has been extended
3. `crates/lx-desktop/src/styles.rs` has `PAGE_HEADING` and `FLEX_BETWEEN`
4. Precondition verified: `dioxus-storage` is already a dependency in Cargo.toml.
5. Material Symbols icon font is loaded (used in sidebar and throughout)

## Data Types

Create `crates/lx-desktop/src/pages/routines/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Routine {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub project_id: Option<String>,
    pub assignee_agent_id: Option<String>,
    pub priority: String,
    pub concurrency_policy: String,
    pub catch_up_policy: String,
    pub cron_expression: Option<String>,
    pub last_run_at: Option<String>,
    pub last_run_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrgNode {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub reports_to: Option<String>,
}
```

## File Plan

| New File | Lines (est.) | Purpose |
|---|---|---|
| `crates/lx-desktop/src/pages/routines/types.rs` | ~35 | Routine and OrgNode data structs |
| `crates/lx-desktop/src/pages/routines/mod.rs` | ~20 | Module declarations, re-exports |
| `crates/lx-desktop/src/pages/routines/list.rs` | ~180 | Routines list page with table and create dialog |
| `crates/lx-desktop/src/pages/routines/detail.rs` | ~160 | Routine detail page with triggers tab |
| `crates/lx-desktop/src/pages/routines/schedule_editor.rs` | ~200 | Cron schedule editor component (main component + presets) |
| `crates/lx-desktop/src/pages/routines/cron_utils.rs` | ~100 | `parse_cron_to_preset` and `build_cron` functions |
| `crates/lx-desktop/src/pages/org/mod.rs` | ~10 | Module declarations, re-exports |
| `crates/lx-desktop/src/pages/org/chart.rs` | ~200 | Org chart SVG rendering + card component |
| `crates/lx-desktop/src/pages/org/chart_layout.rs` | ~100 | Layout algorithm functions (subtree_width, layout_tree, layout_forest, flatten, collect_edges) |
| `crates/lx-desktop/src/pages/org/tree_view.rs` | ~100 | Text-based collapsible org tree |

## Step 1: Create `crates/lx-desktop/src/pages/routines/types.rs`

Create the file with the `Routine` and `OrgNode` structs as specified in the Data Types section above.

Add these constants:

```rust
pub const CONCURRENCY_POLICIES: &[(&str, &str)] = &[
    ("coalesce_if_active", "If a run is already active, keep just one follow-up run queued"),
    ("always_enqueue", "Queue every trigger occurrence, even if already running"),
    ("skip_if_active", "Drop new trigger occurrences while a run is still active"),
];

pub const CATCH_UP_POLICIES: &[(&str, &str)] = &[
    ("skip_missed", "Ignore windows that were missed while paused"),
    ("enqueue_missed_with_cap", "Catch up missed windows in capped batches"),
];

pub const PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];
```

## Step 2: Create `crates/lx-desktop/src/pages/routines/mod.rs`

```rust
pub mod cron_utils;
mod detail;
mod list;
pub mod schedule_editor;
pub mod types;

pub use detail::RoutineDetail;
pub use list::Routines;
```

## Step 3: Create schedule editor (split into two files for 300-line compliance)

Create `crates/lx-desktop/src/pages/routines/cron_utils.rs` (~100 lines) containing the cron parsing and building functions, and `crates/lx-desktop/src/pages/routines/schedule_editor.rs` (~200 lines) containing the component and presets.

### cron_utils.rs

Contains `parse_cron_to_preset` and `build_cron` functions:

- `parse_cron_to_preset(cron: &str) -> (SchedulePreset, hour, minute, day_of_week, day_of_month)`: parse a 5-field cron string to determine which preset matches. Logic mirrors `parseCronToPreset` in the TSX source exactly.
- `build_cron(preset, hour, minute, dow, dom) -> String`: construct cron from preset values. Logic mirrors `buildCron` in the TSX source.

### schedule_editor.rs

This component mirrors `ScheduleEditor.tsx`. It provides a cron expression editor with presets. Imports `parse_cron_to_preset` and `build_cron` from `super::cron_utils`.

Structure:
- Props: `value: String`, `on_change: EventHandler<String>`
- Internal types and constants:
  - `SchedulePreset` enum: `EveryMinute`, `EveryHour`, `EveryDay`, `Weekdays`, `Weekly`, `Monthly`, `Custom`
  - `PRESETS` array: `[("every_minute", "Every minute"), ("every_hour", "Every hour"), ("every_day", "Every day"), ("weekdays", "Weekdays"), ("weekly", "Weekly"), ("monthly", "Monthly"), ("custom", "Custom (cron)")]`
  - `HOURS`: 24 entries, labels like "12 AM", "1 AM", ..., "11 PM"
  - `MINUTES`: entries 0,5,10,...,55 with zero-padded labels
  - `DAYS_OF_WEEK`: `[("1","Mon"),("2","Tue"),("3","Wed"),("4","Thu"),("5","Fri"),("6","Sat"),("0","Sun")]`
  - `DAYS_OF_MONTH`: entries 1-31
- Local signals: `preset`, `hour`, `minute`, `day_of_week`, `day_of_month`, `custom_cron`
- Initialize signals from `parse_cron_to_preset(&value)` on mount
- Render:
  - Preset selector: a `<select>` element with options from PRESETS. On change, update `preset` signal and call `on_change` with the built cron.
  - If preset is Custom: a text input bound to `custom_cron`, with placeholder "0 10 * * *" and helper text "Five fields: minute hour day-of-month month day-of-week". On change, call `on_change` directly.
  - Otherwise, render time/day pickers contextually:
    - For EveryHour: minute selector only (labeled "at minute")
    - For EveryDay/Weekdays: hour selector + ":" + minute selector
    - For Weekly: hour + minute + day-of-week button row (Mon-Sun), active day highlighted
    - For Monthly: hour + minute + day-of-month `<select>`
    - For EveryMinute: no additional pickers
  - Each picker change calls `on_change` with `build_cron(...)` result

Style selectors with lx-desktop conventions: `bg-[var(--surface-container)]` for select backgrounds, `border border-[var(--outline-variant)]`, `text-xs uppercase`.

## Step 4: Create `crates/lx-desktop/src/pages/routines/list.rs`

This component mirrors `Routines.tsx`. It renders a table of routines with a create dialog.

Structure:
- Use `dioxus_storage::use_persistent("lx_routines", || Vec::<Routine>::new())` for storage
- Local signals: `show_composer: Signal<bool>`, `draft_title`, `draft_description`, `draft_priority` (default "medium"), `draft_concurrency_policy` (default "coalesce_if_active"), `draft_catch_up_policy` (default "skip_missed")
- Header: "ROUTINES" heading with "CREATE ROUTINE" button
- If no routines: centered "No routines yet" empty state
- If routines exist: render an HTML table with columns:
  - "NAME": routine title, with status label below if paused/archived
  - "LAST RUN": `last_run_at` or "Never", with `last_run_status` below
  - "ENABLED": a toggle switch (styled div with click handler). Toggling changes `routine.status` between "active" and "paused". Render a `button[role="switch"]` with a sliding dot, green when active.
  - Row is clickable, navigates to `Route::RoutineDetail { id }`
- Create dialog (rendered when `show_composer` is true):
  - Fixed overlay, centered card
  - Header: "NEW ROUTINE" label
  - Title textarea (auto-resizing not needed in Dioxus, use regular text input)
  - Description textarea
  - Advanced section (collapsible via signal `show_advanced: Signal<bool>`):
    - Concurrency policy: `<select>` with options from `CONCURRENCY_POLICIES`, show description text below
    - Catch-up policy: `<select>` with options from `CATCH_UP_POLICIES`, show description text below
  - Footer: "CANCEL" and "CREATE ROUTINE" buttons
  - CREATE generates UUID, pushes new Routine with status "active" and default fields, clears draft, closes dialog

## Step 5: Create `crates/lx-desktop/src/pages/routines/detail.rs`

This component mirrors `RoutineDetail.tsx` (simplified — no live API, no webhook triggers).

Structure:
- Read `id: String` from route params
- Use `dioxus_storage::use_persistent("lx_routines", ...)` for routines
- Find routine by `id`; if not found, render "Routine not found"
- Local signals: `active_tab: Signal<&'static str>` (default "triggers"), `draft_cron: Signal<String>` (initialized from `routine.cron_expression`)
- Render:
  - Header: routine title as h2, status badge, enable/disable toggle
  - Editable title: text input showing routine title, on blur update routine in storage
  - Description: textarea, on blur update routine in storage
  - Tab bar: "TRIGGERS" and "SETTINGS" buttons
  - If `active_tab == "triggers"`:
    - If `routine.cron_expression` is Some: render `ScheduleEditor { value: cron, on_change: handler }` where handler updates routine's `cron_expression` in storage
    - If None: "No schedule configured" text with "ADD SCHEDULE" button that sets `cron_expression` to `Some("0 10 * * *".into())`
    - A "RUN NOW" button that sets `last_run_at` to current ISO timestamp and `last_run_status` to "completed"
  - If `active_tab == "settings"`:
    - Concurrency policy: `<select>` with current value, on change update routine
    - Catch-up policy: `<select>` with current value, on change update routine
    - Priority: `<select>` with options from PRIORITIES, on change update routine
    - "ARCHIVE" / "RESTORE" button toggling `routine.status` between "archived" and "active"

## Step 6: Create `crates/lx-desktop/src/pages/org/mod.rs`

```rust
mod chart;
mod chart_layout;
mod tree_view;

pub use chart::OrgChart;
pub use tree_view::OrgTreeView;
```

## Step 7: Create `crates/lx-desktop/src/pages/org/tree_view.rs`

This component mirrors `Org.tsx`. It renders a text-based collapsible tree of organizational nodes.

Structure:
- Props: `nodes: Vec<OrgNode>`
- Compute tree structure from flat list:
  - Build a `HashMap<String, Vec<OrgNode>>` mapping parent_id to children
  - Roots are nodes where `reports_to.is_none()`
- `OrgTreeNode` component: props `node: OrgNode`, `children_map: HashMap ref`, `all_nodes: Vec ref`, `depth: u32`
  - Signal: `expanded: Signal<bool>` defaulting to `true`
  - Look up children from `children_map`
  - Render a row with:
    - Left padding via inline style: `padding-left: {depth * 16 + 12}px`
    - Chevron button if has children (material icon "chevron_right", rotated when expanded)
    - Status dot: colored circle (2x2 rounded-full), green for "active", yellow for "paused", red for "error", gray for others
    - Node name as bold text
    - Role as muted text
  - If expanded, recursively render children

## Step 8: Create org chart (split into two files for 300-line compliance)

Create `crates/lx-desktop/src/pages/org/chart_layout.rs` (~100 lines) containing the layout algorithm functions, and `crates/lx-desktop/src/pages/org/chart.rs` (~200 lines) containing the SVG rendering and card component.

### chart_layout.rs

Contains layout structs and functions:

```rust
pub struct LayoutNode {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub x: f64,
    pub y: f64,
    pub children: Vec<LayoutNode>,
}
```

Layout functions (mirror the TSX implementations):
- `pub fn subtree_width(nodes: &[OrgNode], node_id: &str, children_map: &HashMap) -> f64`
- `pub fn layout_tree(node: &OrgNode, x: f64, y: f64, children_map: &HashMap, all: &[OrgNode]) -> LayoutNode`
- `pub fn layout_forest(roots: &[OrgNode], children_map: &HashMap, all: &[OrgNode]) -> Vec<LayoutNode>`
- `pub fn flatten_layout(nodes: &[LayoutNode]) -> Vec<&LayoutNode>`
- `pub fn collect_edges(nodes: &[LayoutNode]) -> Vec<(&LayoutNode, &LayoutNode)>`

Also includes layout constants: `CARD_W: f64 = 200.0`, `CARD_H: f64 = 80.0`, `GAP_X: f64 = 32.0`, `GAP_Y: f64 = 80.0`, `PADDING: f64 = 60.0`.

### chart.rs

This component mirrors `OrgChart.tsx`. It renders an SVG-based org chart with pan and zoom. Imports layout functions from `super::chart_layout`.

Structure:
- Use `dioxus_storage::use_persistent("lx_org_nodes", || default_org_nodes())` where `default_org_nodes()` returns a sample `Vec<OrgNode>` with 3-4 nodes to demonstrate the layout
- Pan/zoom state: signals for `pan_x: f64`, `pan_y: f64`, `zoom: f64` (default 1.0), `dragging: bool`, `drag_start_x/y/pan_x/pan_y: f64`
- Render:
  - Container div: `class="w-full flex-1 min-h-0 overflow-hidden relative"`, styled with `cursor: grab` (or `grabbing` when dragging)
  - Mouse event handlers on the container:
    - `onmousedown`: set `dragging` true, record start positions (only if not clicking a card)
    - `onmousemove`: if dragging, update `pan_x`/`pan_y` by delta
    - `onmouseup` / `onmouseleave`: set `dragging` false
    - `onwheel`: adjust zoom by factor 1.1/0.9, clamp 0.2-2.0, zoom toward mouse position
  - Zoom controls: three small buttons in top-right corner: "+", "-", "Fit"
    - "Fit" recalculates zoom to fit all nodes in the container
  - SVG layer: `<svg>` covering the container, with a `<g>` transform for pan/zoom
    - For each edge (parent -> child): render a `<path>` with the orthogonal connector pattern: `M x1 y1 L x1 midY L x2 midY L x2 y2`, stroke `var(--outline-variant)`, stroke-width 1.5
  - Card layer: absolute-positioned div with CSS transform for pan/zoom
    - For each node: a positioned div card at `(node.x, node.y)` with `width: CARD_W`, `min-height: CARD_H`
    - Card content: status dot (colored circle), node name, role label
    - Card styled: `bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg shadow-sm`

The status dot colors:

```rust
fn status_dot_color(status: &str) -> &'static str {
    match status {
        "running" => "#22d3ee",
        "active" => "#4ade80",
        "paused" | "idle" => "#facc15",
        "error" => "#f87171",
        "terminated" => "#a3a3a3",
        _ => "#a3a3a3",
    }
}
```

## Step 9: Verify `crates/lx-desktop/src/pages/mod.rs`

The `pub mod routines;` and `pub mod org;` declarations already exist from Unit 3. No changes needed.

## Step 10: Note on routes

Unit 3 already has `Routines`, `RoutineDetail`, and `OrgChart` route variants with imports pointing at `crate::pages::routines` and `crate::pages::org`. Creating the real directory modules at those paths replaces the stubs automatically. Do NOT modify `routes.rs` or `pages/mod.rs`.

## Step 11: Update `crates/lx-desktop/src/layout/sidebar.rs`

Add two new `NavItem` entries after GOALS and before SETTINGS:

```rust
NavItem {
    to: Route::Routines {},
    label: "ROUTINES",
    icon: "repeat",
}
NavItem {
    to: Route::OrgChart {},
    label: "ORG",
    icon: "account_tree",
}
```

## Definition of Done

1. `just diagnose` passes with no errors and no warnings
2. Sidebar shows ROUTINES and ORG nav items
3. Clicking ROUTINES shows the routines list page (empty state initially)
4. The "CREATE ROUTINE" button opens a dialog where a routine can be created with title, description, and advanced settings
5. Created routines appear in a table with title, last run, and enable/disable toggle
6. The toggle switch changes routine status between active/paused and persists
7. Clicking a routine row navigates to the routine detail page
8. The routine detail page shows a triggers tab with a ScheduleEditor component
9. The ScheduleEditor renders preset options (Every minute, Every hour, Every day, Weekdays, Weekly, Monthly, Custom)
10. Selecting a preset shows the appropriate time/day pickers and produces valid cron expressions
11. The Custom preset shows a raw cron text input
12. Clicking ORG shows an SVG org chart with sample nodes, pan/zoom via mouse drag/wheel, and zoom controls
13. Org chart cards show node name, role, and status dot
14. No file exceeds 300 lines
