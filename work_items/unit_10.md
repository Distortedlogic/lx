# Unit 10: Projects & Goals Pages

Port the Projects list/detail and Goals list/detail/tree pages from Paperclip React to Dioxus 0.7.3 in lx-desktop.

## Paperclip Source Files

| Paperclip File | Purpose |
|---|---|
| `reference/paperclip/ui/src/pages/Projects.tsx` | Projects list page with filtering and "Add Project" button |
| `reference/paperclip/ui/src/pages/ProjectDetail.tsx` | Project detail with tabs: overview, issues, configuration, budget |
| `reference/paperclip/ui/src/components/ProjectProperties.tsx` | Configuration panel: status, goals, workspace settings, archive |
| `reference/paperclip/ui/src/components/NewProjectDialog.tsx` | Modal dialog for creating a new project |
| `reference/paperclip/ui/src/pages/Goals.tsx` | Goals list page rendering a GoalTree |
| `reference/paperclip/ui/src/pages/GoalDetail.tsx` | Goal detail with sub-goals and linked projects tabs |
| `reference/paperclip/ui/src/components/GoalTree.tsx` | Recursive tree component for rendering goal hierarchies |
| `reference/paperclip/ui/src/components/GoalProperties.tsx` | Side panel showing goal status, level, owner, parent |
| `reference/paperclip/ui/src/components/NewGoalDialog.tsx` | Modal dialog for creating a new goal |

## Preconditions

1. **Unit 3 is complete:** Unit 3 created stubs `pages/projects.rs` and `pages/goals.rs`. This unit replaces them with real modules. Delete each stub file and create directory modules at those paths (e.g., `src/pages/projects/mod.rs` and `src/pages/goals/mod.rs`). The `routes.rs` Route enum already has `Projects {}`, `ProjectDetail { project_id: String }`, `Goals {}`, and `GoalDetail { goal_id: String }` variants importing from `crate::pages::projects` and `crate::pages::goals` -- no changes to `routes.rs` are needed.
2. `crates/lx-desktop/src/pages/mod.rs` exists declaring page modules
3. `crates/lx-desktop/src/styles.rs` exists with `PAGE_HEADING` and `FLEX_BETWEEN` constants
4. `crates/lx-desktop/src/layout/sidebar.rs` exists with `NavItem` components and `Sidebar`
5. Dioxus 0.7.3 with `use_signal`, `use_store`, `rsx!`, `#[component]`, `Link`, `Routable` is available
6. Precondition verified: `dioxus-storage` is already a dependency in Cargo.toml.
7. Dioxus 0.7.3 passes route parameters as component props. The `Route::ProjectDetail { project_id: String }` variant means the `ProjectDetail` component receives `project_id: String` as a prop via `#[component]`. Same for `GoalDetail { goal_id: String }`.

## Data Types

Since there is no live API backend, all data is local mock state using `use_signal` and `dioxus_storage::use_persistent`. Define the following structs in a new file `crates/lx-desktop/src/pages/projects/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub color: String,
    pub target_date: Option<String>,
    pub goal_ids: Vec<String>,
    pub archived_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub level: String,
    pub parent_id: Option<String>,
    pub owner_agent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

## File Plan

| New File | Lines (est.) | Purpose |
|---|---|---|
| `crates/lx-desktop/src/pages/projects/types.rs` | ~30 | Project and Goal data structs |
| `crates/lx-desktop/src/pages/projects/mod.rs` | ~20 | Module declarations, re-exports |
| `crates/lx-desktop/src/pages/projects/list.rs` | ~90 | Projects list page component |
| `crates/lx-desktop/src/pages/projects/detail.rs` | ~120 | Project detail page with tabs |
| `crates/lx-desktop/src/pages/projects/new_dialog.rs` | ~100 | New project dialog component |
| `crates/lx-desktop/src/pages/goals/mod.rs` | ~15 | Module declarations, re-exports |
| `crates/lx-desktop/src/pages/goals/list.rs` | ~60 | Goals list page component |
| `crates/lx-desktop/src/pages/goals/detail.rs` | ~120 | Goal detail page with sub-goals/projects tabs |
| `crates/lx-desktop/src/pages/goals/tree.rs` | ~90 | Recursive GoalTree + GoalNode components |
| `crates/lx-desktop/src/pages/goals/properties.rs` | ~80 | Goal properties side panel |
| `crates/lx-desktop/src/pages/goals/new_dialog.rs` | ~100 | New goal dialog component |

## Step 1: Create `crates/lx-desktop/src/pages/projects/types.rs`

Create the file with the `Project` and `Goal` structs defined above in the Data Types section. Both structs derive `Clone, Debug, PartialEq, Serialize, Deserialize`.

Add these constants:

```rust
pub const PROJECT_STATUSES: &[&str] = &["backlog", "planned", "in_progress", "completed", "cancelled"];

pub const PROJECT_COLORS: &[&str] = &[
    "#6366f1", "#8b5cf6", "#ec4899", "#f43f5e", "#ef4444",
    "#f97316", "#eab308", "#22c55e", "#14b8a6", "#06b6d4",
];

pub const GOAL_STATUSES: &[&str] = &["planned", "in_progress", "completed", "cancelled"];

pub const GOAL_LEVELS: &[&str] = &["company", "team", "agent", "task"];
```

## Step 2: Create `crates/lx-desktop/src/pages/projects/mod.rs`

```rust
mod detail;
mod list;
mod new_dialog;
pub mod types;

pub use detail::ProjectDetail;
pub use list::Projects;
pub use new_dialog::NewProjectDialog;
```

## Step 3: Create `crates/lx-desktop/src/pages/projects/list.rs`

This component mirrors `Projects.tsx`. It renders a list of non-archived projects with name, description, target date, and status badge.

Structure:
- Import `use_signal`, `rsx!`, `Link`, the `Project` type, and route types
- Use `dioxus_storage::use_persistent("lx_projects", || Vec::<Project>::new())` for storage
- Filter out projects where `archived_at.is_some()`
- Show an "ADD PROJECT" button that sets a `show_dialog: Signal<bool>` to true
- If projects list is empty, show centered text: "No projects yet"
- If projects exist, render a bordered container with one row per project:
  - A colored dot (5x5 rounded div) using `project.color` as inline `background-color`
  - Project name as bold text
  - Optional description as muted text, truncated
  - Optional target date on the right as muted text
  - Status badge: a span with uppercase text, colored per status (use the `status_color` helper function below)
  - Each row is a `Link` to `Route::ProjectDetail { id: project.id.clone() }`
- Below the list, conditionally render `NewProjectDialog { open: show_dialog, projects: projects_signal }`

Add this helper function in the file:

```rust
fn status_color(status: &str) -> &'static str {
    match status {
        "in_progress" => "text-[var(--primary)]",
        "completed" => "text-[var(--success)]",
        "cancelled" => "text-[var(--error)]",
        "planned" => "text-[var(--warning)]",
        _ => "text-[var(--outline)]",
    }
}
```

Use the lx-desktop design system: `var(--surface-container-lowest)` backgrounds, `var(--outline-variant)` borders, `var(--on-surface)` text, uppercase labels, monospace IDs.

## Step 4: Create `crates/lx-desktop/src/pages/projects/new_dialog.rs`

This component mirrors `NewProjectDialog.tsx`. It renders a modal overlay for creating a new project.

Structure:
- Props: `open: Signal<bool>`, `projects: Signal<Vec<Project>>`
- Local signals: `name: Signal<String>`, `description: Signal<String>`, `status: Signal<String>` (default "planned"), `target_date: Signal<String>`, `color: Signal<String>` (default first PROJECT_COLORS entry)
- Render only when `open()` is true
- Overlay: fixed full-screen div with semi-transparent black background, centered content card
- Card layout (mirroring lx-desktop style):
  - Header: "NEW PROJECT" label, close button (sets `open` to false)
  - Name input: text input bound to `name`
  - Description: textarea bound to `description`
  - Status selector: row of buttons, one per `PROJECT_STATUSES`, highlight active
  - Color selector: row of small colored squares from `PROJECT_COLORS`, highlight active with ring
  - Target date: `<input type="date">` bound to `target_date`
  - Footer: "CANCEL" button (clears and closes), "CREATE" button that:
    - Generates a UUID via `uuid::Uuid::new_v4().to_string()`
    - Pushes a new `Project` into the `projects` signal
    - Clears all fields, sets `open` to false

## Step 5: Create `crates/lx-desktop/src/pages/projects/detail.rs`

This component mirrors `ProjectDetail.tsx`. It shows project details with an "OVERVIEW" / "CONFIGURATION" tab bar.

Structure:
- Component props: none (reads `id` from route params)
- Read `id: String` from route via the `Route::ProjectDetail { id }` variant
- Use `dioxus_storage::use_persistent("lx_projects", ...)` to access projects
- Find project by `id` in the list; if not found, render "Project not found"
- Local signal: `active_tab: Signal<&'static str>` defaulting to `"overview"`
- Render:
  - Header row: colored dot (inline bg from `project.color`), project name as h2, status badge
  - Tab bar: two buttons "OVERVIEW" and "CONFIGURATION", styled with active highlight
  - If `active_tab == "overview"`:
    - Description text (or "No description" placeholder)
    - Status and target date in a 2-column grid
  - If `active_tab == "configuration"`:
    - Status picker: row of buttons per PROJECT_STATUSES, clicking updates project in storage
    - Color picker: row of colored squares, clicking updates project in storage
    - Target date input, updating project in storage on change
    - Archive button: sets `archived_at` to current timestamp string, navigates back to projects list

## Step 6: Create `crates/lx-desktop/src/pages/goals/mod.rs`

```rust
mod detail;
mod list;
mod new_dialog;
mod properties;
mod tree;

pub use detail::GoalDetail;
pub use list::Goals;
pub use new_dialog::NewGoalDialog;
```

## Step 7: Create `crates/lx-desktop/src/pages/goals/tree.rs`

This component mirrors `GoalTree.tsx`. It renders a recursive tree of goals with expand/collapse.

Structure:
- `GoalTree` component: props are `goals: Vec<Goal>` (the full flat list)
  - Compute roots: goals where `parent_id.is_none()` or `parent_id` is not in the goals set
  - Render each root through `GoalNode`
  - Wrap in a bordered container div
- `GoalNode` component: props are `goal: Goal`, `all_goals: Vec<Goal>`, `depth: u32`
  - Local signal: `expanded: Signal<bool>` defaulting to `true`
  - Compute `children`: filter `all_goals` where `parent_id == Some(goal.id)`
  - Render a row with:
    - Left padding: `padding-left: {depth * 16 + 12}px` via inline style
    - If has children: a chevron button toggling `expanded`. Use "chevron_right" material icon, rotate 90deg when expanded via conditional class
    - If no children: empty spacer span (w-4)
    - Goal level as muted uppercase text
    - Goal title as flex-1 truncated text
    - Status badge (reuse `status_color` function or inline)
  - Row is a `Link` to `Route::GoalDetail { id: goal.id.clone() }`
  - If `expanded()` and has children, recursively render `GoalNode` for each child at `depth + 1`

## Step 8: Create `crates/lx-desktop/src/pages/goals/list.rs`

This component mirrors `Goals.tsx`. It renders a GoalTree of all goals.

Structure:
- Use `dioxus_storage::use_persistent("lx_goals", || Vec::<Goal>::new())` for storage
- Local signal: `show_dialog: Signal<bool>`
- If goals empty: centered "No goals yet" message
- If goals exist:
  - "NEW GOAL" button (sets `show_dialog` true)
  - Render `GoalTree { goals: goals_list }`
- Conditionally render `NewGoalDialog { open: show_dialog, goals: goals_signal }`

## Step 9: Create `crates/lx-desktop/src/pages/goals/properties.rs`

This component mirrors `GoalProperties.tsx`. It shows metadata for a goal.

Structure:
- Props: `goal: Goal`
- Render a vertical list of property rows, each with a label (w-20, muted, xs text) and value:
  - "Status": colored status text
  - "Level": capitalized level text
  - "Owner": "None" (placeholder, since no agent linkage exists yet)
  - "Parent Goal": if `goal.parent_id` is Some, render as a Link to the parent goal route; else "None"
  - Separator div (border-t)
  - "Created": `goal.created_at` text
  - "Updated": `goal.updated_at` text

## Step 10: Create `crates/lx-desktop/src/pages/goals/detail.rs`

This component mirrors `GoalDetail.tsx`. It shows goal details with sub-goals and linked projects tabs.

Structure:
- Read `id: String` from route params
- Use `dioxus_storage::use_persistent("lx_goals", ...)` for goals
- Use `dioxus_storage::use_persistent("lx_projects", ...)` for projects
- Find goal by `id`; if not found, render "Goal not found"
- Local signal: `active_tab: Signal<&'static str>` defaulting to `"children"`
- Render:
  - Header: level as uppercase muted label, status badge
  - Title as h2
  - Description (or "No description" placeholder)
  - Tab bar: "SUB-GOALS ({count})" and "PROJECTS ({count})" buttons
  - If `active_tab == "children"`:
    - Compute child goals: `all_goals.iter().filter(|g| g.parent_id == Some(id.clone()))`
    - If no children: "No sub-goals" text
    - If children: render `GoalTree { goals: children_vec }`
  - If `active_tab == "projects"`:
    - Compute linked projects: `all_projects.iter().filter(|p| p.goal_ids.contains(&id))`
    - If no projects: "No linked projects" text
    - If projects: bordered list of project rows, each with name, description, status badge, linked to `Route::ProjectDetail`

## Step 11: Create `crates/lx-desktop/src/pages/goals/new_dialog.rs`

This component mirrors `NewGoalDialog.tsx`. It renders a modal overlay for creating a new goal.

Structure:
- Props: `open: Signal<bool>`, `goals: Signal<Vec<Goal>>`, `parent_id: Option<String>` (optional, for sub-goals)
- Local signals: `title: Signal<String>`, `description: Signal<String>`, `status: Signal<String>` (default "planned"), `level: Signal<String>` (default "task"), `selected_parent: Signal<Option<String>>` (initialized from `parent_id` prop)
- Render only when `open()` is true
- Overlay: same pattern as NewProjectDialog
- Card layout:
  - Header: "NEW GOAL" or "NEW SUB-GOAL" (if parent_id is Some)
  - Title input
  - Description textarea
  - Status selector: row of buttons per GOAL_STATUSES
  - Level selector: row of buttons per GOAL_LEVELS, with display labels: "Company", "Team", "Agent", "Task"
  - Parent goal selector: dropdown-like list of existing goals, with "No parent" option
  - Footer: "CANCEL" and "CREATE" buttons
  - CREATE generates UUID, pushes new Goal (with `created_at` and `updated_at` set to current ISO timestamp string), clears fields, closes dialog

## Step 12: Verify `crates/lx-desktop/src/pages/mod.rs`

The `pub mod projects;` and `pub mod goals;` declarations already exist from Unit 3. No changes needed.

## Step 13: Note on routes

Unit 3 already has `Projects`, `ProjectDetail`, `Goals`, and `GoalDetail` route variants with imports pointing at `crate::pages::projects` and `crate::pages::goals`. Creating the real directory modules at those paths replaces the stubs automatically. Do NOT modify `routes.rs` or `pages/mod.rs` -- both already have the correct declarations from Unit 3.

## Step 14: Update `crates/lx-desktop/src/layout/sidebar.rs`

Add two new `NavItem` entries in the sidebar between "TOOLS" and "SETTINGS":

```rust
NavItem {
    to: Route::Projects {},
    label: "PROJECTS",
    icon: "hexagon",
}
NavItem {
    to: Route::Goals {},
    label: "GOALS",
    icon: "target",
}
```

Insert these after the `NavItem` for Tools and before the `NavItem` for Settings.

## Step 15: Add `uuid` dependency

Check `crates/lx-desktop/Cargo.toml`. If `uuid` with feature `v4` is not already a dependency, add it:

```toml
uuid = { version = "1", features = ["v4"] }
```

## Definition of Done

1. `just diagnose` passes with no errors and no warnings
2. The app compiles and the sidebar shows PROJECTS and GOALS nav items
3. Clicking PROJECTS shows the projects list page (empty state initially)
4. The "ADD PROJECT" button opens a modal dialog where a project can be created
5. Created projects appear in the list and persist across app restarts (via `use_persistent`)
6. Clicking a project row navigates to the project detail page with overview and configuration tabs
7. Clicking GOALS shows the goals list page (empty state initially)
8. The "NEW GOAL" button opens a modal dialog where a goal can be created
9. Created goals appear in a tree view and persist across app restarts
10. Clicking a goal navigates to the goal detail page with sub-goals and projects tabs
11. No file exceeds 300 lines
