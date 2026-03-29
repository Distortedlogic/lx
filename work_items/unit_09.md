# Unit 9: Issues List & Issue Detail

## Scope

Port the Paperclip Issues page (list view with filters, kanban board view) and Issue Detail page (properties sidebar, comment thread, documents section, workspace card, live run widget) into Dioxus 0.7.3 components under `crates/lx-desktop/src/pages/issues/`.

## Preconditions

- **Unit 3 is complete:** Unit 3 created a stub `pages/issues.rs`. This unit replaces it with a real issues module. Delete `src/pages/issues.rs` (the Unit 3 stub) and create `src/pages/issues/mod.rs` with the real Issues component. The `routes.rs` Route enum already has `Issues {}` and `IssueDetail { issue_id: String }` variants importing from `crate::pages::issues` -- no changes to `routes.rs` are needed.
- Unit 7 and Unit 8 are complete.
- `crates/lx-desktop/src/pages/agents/list.rs` exports `StatusBadge`.
- `crates/lx-desktop/src/pages/agents/live_run_widget.rs` exports `LiveRunWidget` and `LiveRunInfo`.
- `crates/lx-desktop/src/pages/agents/transcript.rs` exports `TranscriptView`.
- `crates/lx-desktop/src/styles.rs` contains style constants from Unit 7 (`BTN_OUTLINE_SM`, `BTN_PRIMARY_SM`, `INPUT_FIELD`, `TAB_ACTIVE`, `TAB_INACTIVE`, `PROPERTY_LABEL`, etc).
- Page modules are registered in `crates/lx-desktop/src/pages/mod.rs`.

## Paperclip Source Files to Reference

| Paperclip File | What to Extract |
|---|---|
| `reference/paperclip/ui/src/pages/Issues.tsx` | Issues page: delegates to `IssuesList` component, passes agents/projects/liveRuns data |
| `reference/paperclip/ui/src/components/IssuesList.tsx` lines 1-177 | View state type, filter/sort/group logic, status/priority ordering, quick filter presets, list vs board toggle |
| `reference/paperclip/ui/src/components/IssuesList.tsx` lines 178+ | `IssuesList` component: filter bar, group headers, issue rows, empty state, search |
| `reference/paperclip/ui/src/components/IssueRow.tsx` | Issue row with identifier, title, status icon, priority icon, assignee identity |
| `reference/paperclip/ui/src/components/KanbanBoard.tsx` | Kanban board: columns by status, draggable cards with priority/assignee, drag-and-drop status change |
| `reference/paperclip/ui/src/pages/IssueDetail.tsx` lines 198-600 | Issue detail page: queries, mutations, breadcrumbs, properties panel, comment thread, activity timeline |
| `reference/paperclip/ui/src/pages/IssueDetail.tsx` lines 600+ | Render: title editing, description editing, tabs (comments/activity), documents, workspace card, live runs |
| `reference/paperclip/ui/src/components/IssueProperties.tsx` | Property panel: status picker, priority picker, labels, assignee picker, project picker, parent link, dates |
| `reference/paperclip/ui/src/components/IssueDocumentsSection.tsx` | Document list with inline markdown editor, create/delete, autosave, conflict detection |
| `reference/paperclip/ui/src/components/IssueWorkspaceCard.tsx` | Workspace card: branch name, worktree path, mode display, copyable inline values |
| `reference/paperclip/ui/src/components/NewIssueDialog.tsx` | New issue dialog: title, description editor, status/priority/assignee pickers, adapter overrides, file staging |

## Step 1: Create Issue Data Types

Create `crates/lx-desktop/src/pages/issues/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub identifier: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_agent_id: Option<String>,
    pub assignee_user_id: Option<String>,
    pub project_id: Option<String>,
    pub parent_id: Option<String>,
    pub label_ids: Vec<String>,
    pub labels: Vec<IssueLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub request_depth: u32,
    pub company_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueLabel {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: String,
    pub body: String,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueDocument {
    pub key: String,
    pub title: Option<String>,
    pub body: String,
    pub format: String,
    pub latest_revision_id: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueWorkspace {
    pub id: String,
    pub mode: Option<String>,
    pub branch_name: Option<String>,
    pub worktree_path: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentRef {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectRef {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

pub const STATUS_ORDER: &[&str] = &[
    "in_progress", "todo", "backlog", "in_review", "blocked", "done", "cancelled",
];

pub const PRIORITY_ORDER: &[&str] = &["critical", "high", "medium", "low"];

pub const QUICK_FILTER_PRESETS: &[(&str, &[&str])] = &[
    ("All", &[]),
    ("Active", &["todo", "in_progress", "in_review", "blocked"]),
    ("Backlog", &["backlog"]),
    ("Done", &["done", "cancelled"]),
];

#[derive(Clone, Debug, PartialEq)]
pub enum IssueViewMode {
    List,
    Board,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IssueViewState {
    pub statuses: Vec<String>,
    pub priorities: Vec<String>,
    pub assignees: Vec<String>,
    pub sort_field: String,
    pub sort_dir: String,
    pub group_by: String,
    pub view_mode: IssueViewMode,
    pub search: String,
}

impl Default for IssueViewState {
    fn default() -> Self {
        Self {
            statuses: Vec::new(),
            priorities: Vec::new(),
            assignees: Vec::new(),
            sort_field: "updated".to_string(),
            sort_dir: "desc".to_string(),
            group_by: "none".to_string(),
            view_mode: IssueViewMode::List,
            search: String::new(),
        }
    }
}

pub fn status_label(status: &str) -> String {
    status
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn status_icon_class(status: &str) -> &'static str {
    match status {
        "todo" => "text-blue-500",
        "in_progress" => "text-yellow-500",
        "in_review" => "text-purple-500",
        "blocked" => "text-red-500",
        "done" => "text-green-500",
        "cancelled" => "text-neutral-400",
        "backlog" => "text-neutral-500",
        _ => "text-neutral-400",
    }
}

pub fn priority_icon_class(priority: &str) -> &'static str {
    match priority {
        "critical" => "text-red-600",
        "high" => "text-orange-500",
        "medium" => "text-yellow-500",
        "low" => "text-blue-400",
        _ => "text-neutral-400",
    }
}

pub fn filter_issues(issues: &[Issue], state: &IssueViewState) -> Vec<Issue> {
    let mut result: Vec<Issue> = issues
        .iter()
        .filter(|i| {
            if !state.statuses.is_empty() && !state.statuses.contains(&i.status) {
                return false;
            }
            if !state.priorities.is_empty() && !state.priorities.contains(&i.priority) {
                return false;
            }
            if !state.search.is_empty() {
                let q = state.search.to_lowercase();
                let title_match = i.title.to_lowercase().contains(&q);
                let id_match = i.identifier.as_ref().map(|id| id.to_lowercase().contains(&q)).unwrap_or(false);
                if !title_match && !id_match {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    let dir: i32 = if state.sort_dir == "asc" { 1 } else { -1 };
    result.sort_by(|a, b| {
        let cmp = match state.sort_field.as_str() {
            "status" => {
                let ai = STATUS_ORDER.iter().position(|s| *s == a.status).unwrap_or(99);
                let bi = STATUS_ORDER.iter().position(|s| *s == b.status).unwrap_or(99);
                ai.cmp(&bi)
            }
            "priority" => {
                let ai = PRIORITY_ORDER.iter().position(|s| *s == a.priority).unwrap_or(99);
                let bi = PRIORITY_ORDER.iter().position(|s| *s == b.priority).unwrap_or(99);
                ai.cmp(&bi)
            }
            "title" => a.title.cmp(&b.title),
            "created" => a.created_at.cmp(&b.created_at),
            _ => a.updated_at.cmp(&b.updated_at),
        };
        if dir > 0 { cmp } else { cmp.reverse() }
    });
    result
}
```

## Step 2: Create Issues List Page

Create `crates/lx-desktop/src/pages/issues/list.rs`:

The list view with filter bar, quick filters, search, and issue rows. Reference `IssuesList.tsx`.

```rust
use dioxus::prelude::*;
use super::types::*;
use super::kanban::KanbanBoardView;
use crate::pages::agents::list::StatusBadge;
use crate::styles::{BTN_OUTLINE_SM, INPUT_FIELD, TAB_ACTIVE, TAB_INACTIVE, FLEX_BETWEEN};

#[component]
pub fn IssuesList(
    issues: Vec<Issue>,
    agents: Vec<AgentRef>,
    on_select: EventHandler<String>,
    on_new_issue: EventHandler<()>,
    on_update: EventHandler<(String, String, String)>,
) -> Element {
    let mut view_state = use_signal(IssueViewState::default);
    let filtered = filter_issues(&issues, &view_state.read());

    rsx! {
        div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
            // Header
            div { class: FLEX_BETWEEN,
                h1 { class: "text-lg font-semibold text-[var(--on-surface)]", "Issues" }
                div { class: "flex items-center gap-2",
                    // View mode toggle
                    div { class: "flex items-center border border-[var(--outline-variant)]/30",
                        button {
                            class: if view_state.read().view_mode == IssueViewMode::List { "p-1.5 bg-[var(--surface-container-high)]" } else { "p-1.5 hover:bg-[var(--surface-container)]" },
                            onclick: move |_| view_state.write().view_mode = IssueViewMode::List,
                            span { class: "material-symbols-outlined text-sm", "list" }
                        }
                        button {
                            class: if view_state.read().view_mode == IssueViewMode::Board { "p-1.5 bg-[var(--surface-container-high)]" } else { "p-1.5 hover:bg-[var(--surface-container)]" },
                            onclick: move |_| view_state.write().view_mode = IssueViewMode::Board,
                            span { class: "material-symbols-outlined text-sm", "view_column" }
                        }
                    }
                    button {
                        class: BTN_OUTLINE_SM,
                        onclick: move |_| on_new_issue.call(()),
                        "+ New Issue"
                    }
                }
            }
            // Quick filters
            div { class: "flex gap-1",
                for (label, statuses) in QUICK_FILTER_PRESETS {
                    {
                        let statuses_vec: Vec<String> = statuses.iter().map(|s| s.to_string()).collect();
                        let is_active = view_state.read().statuses == statuses_vec;
                        rsx! {
                            button {
                                class: if is_active { TAB_ACTIVE } else { TAB_INACTIVE },
                                onclick: {
                                    let sv = statuses_vec.clone();
                                    move |_| view_state.write().statuses = sv.clone()
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
            // Search
            input {
                class: INPUT_FIELD,
                placeholder: "Search issues...",
                value: "{view_state.read().search}",
                oninput: move |evt| view_state.write().search = evt.value().to_string(),
            }
            // Count
            p { class: "text-xs text-[var(--outline)]",
                "{filtered.len()} issue{}", if filtered.len() != 1 { "s" } else { "" }
            }
            // Content
            match view_state.read().view_mode {
                IssueViewMode::List => rsx! {
                    IssueListView {
                        issues: filtered.clone(),
                        agents: agents.clone(),
                        on_select: on_select,
                    }
                },
                IssueViewMode::Board => rsx! {
                    KanbanBoardView {
                        issues: filtered.clone(),
                        agents: agents.clone(),
                        on_select: on_select,
                        on_status_change: move |(id, status): (String, String)| {
                            on_update.call((id, "status".to_string(), status));
                        },
                    }
                },
            }
        }
    }
}

#[component]
fn IssueListView(
    issues: Vec<Issue>,
    agents: Vec<AgentRef>,
    on_select: EventHandler<String>,
) -> Element {
    if issues.is_empty() {
        return rsx! {
            div { class: "flex-1 flex items-center justify-center py-8",
                p { class: "text-sm text-[var(--outline)]", "No issues match the current filters." }
            }
        };
    }

    rsx! {
        div { class: "border border-[var(--outline-variant)]/30 overflow-hidden",
            for issue in issues.iter() {
                IssueRow {
                    issue: issue.clone(),
                    agents: agents.clone(),
                    on_click: {
                        let id = issue.identifier.clone().unwrap_or_else(|| issue.id.clone());
                        move |_| on_select.call(id.clone())
                    },
                }
            }
        }
    }
}

#[component]
fn IssueRow(
    issue: Issue,
    agents: Vec<AgentRef>,
    on_click: EventHandler<()>,
) -> Element {
    let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
    let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| {
        agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone())
    });

    rsx! {
        button {
            class: "flex items-center gap-3 px-3 py-2.5 w-full text-left border-b border-[var(--outline-variant)]/15 hover:bg-[var(--surface-container)] transition-colors",
            onclick: move |_| on_click.call(()),
            // Status icon
            span { class: "material-symbols-outlined text-sm {status_icon_class(&issue.status)}",
                match issue.status.as_str() {
                    "done" => "check_circle",
                    "cancelled" => "cancel",
                    "blocked" => "block",
                    "in_progress" => "pending",
                    "in_review" => "rate_review",
                    _ => "circle",
                }
            }
            // Priority icon
            span { class: "material-symbols-outlined text-sm {priority_icon_class(&issue.priority)}",
                match issue.priority.as_str() {
                    "critical" => "priority_high",
                    "high" => "arrow_upward",
                    "low" => "arrow_downward",
                    _ => "remove",
                }
            }
            // Identifier
            span { class: "text-xs font-mono text-[var(--outline)] shrink-0 w-16",
                "{id_display}"
            }
            // Title
            span { class: "flex-1 text-sm text-[var(--on-surface)] truncate min-w-0",
                "{issue.title}"
            }
            // Assignee
            if let Some(name) = assignee_name {
                span { class: "text-xs text-[var(--outline)] shrink-0", "{name}" }
            }
            // Status badge
            StatusBadge { status: issue.status.clone() }
        }
    }
}
```

## Step 3: Create Kanban Board View

Create `crates/lx-desktop/src/pages/issues/kanban.rs`:

A simplified kanban board (no drag-and-drop in Dioxus; use click-to-move-status instead). Reference `KanbanBoard.tsx`.

```rust
use dioxus::prelude::*;
use super::types::*;

const BOARD_STATUSES: &[&str] = &[
    "backlog", "todo", "in_progress", "in_review", "blocked", "done", "cancelled",
];

#[component]
pub fn KanbanBoardView(
    issues: Vec<Issue>,
    agents: Vec<AgentRef>,
    on_select: EventHandler<String>,
    on_status_change: EventHandler<(String, String)>,
) -> Element {
    let columns: Vec<(&str, Vec<&Issue>)> = BOARD_STATUSES
        .iter()
        .map(|status| {
            let col_issues: Vec<&Issue> = issues.iter().filter(|i| &i.status == status).collect();
            (*status, col_issues)
        })
        .collect();

    rsx! {
        div { class: "flex gap-3 overflow-x-auto pb-4 -mx-2 px-2",
            for (status, col_issues) in columns.iter() {
                KanbanColumn {
                    status: status.to_string(),
                    issues: col_issues.iter().map(|i| (*i).clone()).collect(),
                    agents: agents.clone(),
                    on_select: on_select,
                }
            }
        }
    }
}

#[component]
fn KanbanColumn(
    status: String,
    issues: Vec<Issue>,
    agents: Vec<AgentRef>,
    on_select: EventHandler<String>,
) -> Element {
    let label = status_label(&status);
    let count = issues.len();

    rsx! {
        div { class: "flex flex-col min-w-[260px] w-[260px] shrink-0",
            div { class: "flex items-center gap-2 px-2 py-2 mb-1",
                span { class: "material-symbols-outlined text-sm {status_icon_class(&status)}",
                    "circle"
                }
                span { class: "text-xs font-semibold uppercase tracking-wide text-[var(--outline)]",
                    "{label}"
                }
                span { class: "text-xs text-[var(--outline)]/60 ml-auto tabular-nums",
                    "{count}"
                }
            }
            div { class: "flex-1 min-h-[120px] rounded-md p-1 space-y-1 bg-[var(--surface-container)]/20",
                for issue in issues.iter() {
                    KanbanCard {
                        issue: issue.clone(),
                        agents: agents.clone(),
                        on_click: {
                            let id = issue.identifier.clone().unwrap_or_else(|| issue.id.clone());
                            move |_| on_select.call(id.clone())
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn KanbanCard(
    issue: Issue,
    agents: Vec<AgentRef>,
    on_click: EventHandler<()>,
) -> Element {
    let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
    let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| {
        agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone())
    });

    rsx! {
        button {
            class: "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left hover:shadow-sm transition-shadow",
            onclick: move |_| on_click.call(()),
            div { class: "flex items-start gap-1.5 mb-1.5",
                span { class: "text-xs text-[var(--outline)] font-mono shrink-0",
                    "{id_display}"
                }
            }
            p { class: "text-sm leading-snug text-[var(--on-surface)] line-clamp-2 mb-2",
                "{issue.title}"
            }
            div { class: "flex items-center gap-2",
                span { class: "material-symbols-outlined text-xs {priority_icon_class(&issue.priority)}",
                    match issue.priority.as_str() {
                        "critical" => "priority_high",
                        "high" => "arrow_upward",
                        "low" => "arrow_downward",
                        _ => "remove",
                    }
                }
                if let Some(name) = assignee_name {
                    span { class: "text-xs text-[var(--outline)]", "{name}" }
                }
            }
        }
    }
}
```

## Step 4: Create Issue Detail Page

Create `crates/lx-desktop/src/pages/issues/detail.rs`:

The issue detail page with title, description, properties sidebar, comment thread, documents, and live run widget. Reference `IssueDetail.tsx`.

```rust
use dioxus::prelude::*;
use super::types::*;
use super::properties::IssuePropertiesPanel;
use super::comments::CommentThread;
use super::documents::DocumentsSection;
use super::workspace_card::WorkspaceCard;
use crate::pages::agents::list::StatusBadge;
use crate::styles::{BTN_OUTLINE_SM, INPUT_FIELD};

#[component]
pub fn IssueDetailPage(
    issue: Issue,
    comments: Vec<IssueComment>,
    documents: Vec<IssueDocument>,
    workspace: Option<IssueWorkspace>,
    agents: Vec<AgentRef>,
    on_back: EventHandler<()>,
    on_update: EventHandler<(String, String)>,
    on_add_comment: EventHandler<String>,
) -> Element {
    let mut editing_title = use_signal(|| false);
    let mut draft_title = use_signal(|| issue.title.clone());

    let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);

    rsx! {
        div { class: "flex flex-col h-full overflow-auto",
            // Header
            div { class: "flex items-center gap-2 px-4 py-3 border-b border-[var(--outline-variant)]/30",
                button {
                    class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)]",
                    onclick: move |_| on_back.call(()),
                    "< Issues"
                }
                span { class: "text-xs font-mono text-[var(--outline)]", "{id_display}" }
                StatusBadge { status: issue.status.clone() }
            }
            // Main content
            div { class: "flex flex-1 min-h-0",
                // Left: issue body
                div { class: "flex-1 p-4 overflow-auto space-y-6",
                    // Title
                    if *editing_title.read() {
                        input {
                            class: "text-xl font-semibold w-full bg-transparent outline-none text-[var(--on-surface)] border-b border-[var(--primary)]",
                            value: "{draft_title}",
                            oninput: move |evt| draft_title.set(evt.value().to_string()),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter {
                                    on_update.call(("title".to_string(), draft_title.read().clone()));
                                    editing_title.set(false);
                                }
                            },
                        }
                    } else {
                        h1 {
                            class: "text-xl font-semibold text-[var(--on-surface)] cursor-pointer hover:text-[var(--primary)] transition-colors",
                            onclick: move |_| editing_title.set(true),
                            "{issue.title}"
                        }
                    }
                    // Description
                    if let Some(desc) = &issue.description {
                        div { class: "text-sm text-[var(--on-surface-variant)] whitespace-pre-wrap",
                            "{desc}"
                        }
                    }
                    // Workspace card
                    if let Some(ws) = &workspace {
                        WorkspaceCard { workspace: ws.clone() }
                    }
                    // Documents
                    if !documents.is_empty() {
                        DocumentsSection { documents: documents.clone() }
                    }
                    // Comments
                    CommentThread {
                        comments: comments.clone(),
                        agents: agents.clone(),
                        on_add: on_add_comment,
                    }
                }
                // Right: properties panel
                div { class: "w-64 shrink-0 border-l border-[var(--outline-variant)]/30 p-4 overflow-auto",
                    IssuePropertiesPanel {
                        issue: issue.clone(),
                        agents: agents.clone(),
                        on_update: on_update,
                    }
                }
            }
        }
    }
}
```

## Step 5: Create Issue Properties Panel

Create `crates/lx-desktop/src/pages/issues/properties.rs`:

Property rows for status, priority, assignee, project, dates. Reference `IssueProperties.tsx`.

```rust
use dioxus::prelude::*;
use super::types::*;
use crate::styles::PROPERTY_LABEL;

#[component]
pub fn IssuePropertiesPanel(
    issue: Issue,
    agents: Vec<AgentRef>,
    on_update: EventHandler<(String, String)>,
) -> Element {
    let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| {
        agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone())
    });

    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-1",
                PropertyRow { label: "Status",
                    StatusPicker {
                        current: issue.status.clone(),
                        on_change: move |s: String| on_update.call(("status".to_string(), s)),
                    }
                }
                PropertyRow { label: "Priority",
                    PriorityPicker {
                        current: issue.priority.clone(),
                        on_change: move |p: String| on_update.call(("priority".to_string(), p)),
                    }
                }
                PropertyRow { label: "Assignee",
                    AssigneePicker {
                        current_agent_id: issue.assignee_agent_id.clone(),
                        agents: agents.clone(),
                        on_change: move |id: String| on_update.call(("assignee_agent_id".to_string(), id)),
                    }
                }
                if !issue.labels.is_empty() {
                    PropertyRow { label: "Labels",
                        div { class: "flex flex-wrap gap-1",
                            for label in issue.labels.iter() {
                                span {
                                    class: "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium border",
                                    style: "border-color: {label.color}; background: {label.color}22;",
                                    "{label.name}"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "border-t border-[var(--outline-variant)]/30 pt-4 space-y-1",
                if let Some(name) = assignee_name {
                    PropertyRow { label: "Assigned to",
                        span { class: "text-sm text-[var(--on-surface)]", "{name}" }
                    }
                }
                PropertyRow { label: "Created",
                    span { class: "text-sm text-[var(--on-surface)]", "{issue.created_at}" }
                }
                PropertyRow { label: "Updated",
                    span { class: "text-sm text-[var(--on-surface)]", "{issue.updated_at}" }
                }
                if issue.request_depth > 0 {
                    PropertyRow { label: "Depth",
                        span { class: "text-sm font-mono text-[var(--on-surface)]",
                            "{issue.request_depth}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PropertyRow(label: &'static str, children: Element) -> Element {
    rsx! {
        div { class: "flex items-center gap-3 py-1.5",
            span { class: PROPERTY_LABEL, "{label}" }
            div { class: "flex items-center gap-1.5 min-w-0 flex-1", {children} }
        }
    }
}

#[component]
fn StatusPicker(current: String, on_change: EventHandler<String>) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "relative",
            button {
                class: "inline-flex items-center gap-1.5 cursor-pointer hover:bg-[var(--surface-container)] rounded px-1 py-0.5 transition-colors",
                onclick: move |_| open.set(!*open.read()),
                span { class: "material-symbols-outlined text-sm {status_icon_class(&current)}",
                    "circle"
                }
                span { class: "text-sm text-[var(--on-surface)]", "{status_label(&current)}" }
            }
            if *open.read() {
                div { class: "absolute left-0 top-full mt-1 z-50 w-40 border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-lg rounded p-1",
                    for status in STATUS_ORDER {
                        button {
                            class: "flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-[var(--surface-container-high)]",
                            onclick: {
                                let s = status.to_string();
                                move |_| {
                                    on_change.call(s.clone());
                                    open.set(false);
                                }
                            },
                            span { class: "material-symbols-outlined text-xs {status_icon_class(status)}",
                                "circle"
                            }
                            "{status_label(status)}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PriorityPicker(current: String, on_change: EventHandler<String>) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "relative",
            button {
                class: "inline-flex items-center gap-1.5 cursor-pointer hover:bg-[var(--surface-container)] rounded px-1 py-0.5 transition-colors",
                onclick: move |_| open.set(!*open.read()),
                span { class: "text-sm text-[var(--on-surface)]", "{current}" }
            }
            if *open.read() {
                div { class: "absolute left-0 top-full mt-1 z-50 w-36 border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-lg rounded p-1",
                    for priority in PRIORITY_ORDER {
                        button {
                            class: "flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-[var(--surface-container-high)]",
                            onclick: {
                                let p = priority.to_string();
                                move |_| {
                                    on_change.call(p.clone());
                                    open.set(false);
                                }
                            },
                            "{priority}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AssigneePicker(
    current_agent_id: Option<String>,
    agents: Vec<AgentRef>,
    on_change: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let current_name = current_agent_id.as_ref().and_then(|id| {
        agents.iter().find(|a| &a.id == id).map(|a| a.name.clone())
    }).unwrap_or_else(|| "Unassigned".to_string());

    rsx! {
        div { class: "relative",
            button {
                class: "inline-flex items-center gap-1.5 cursor-pointer hover:bg-[var(--surface-container)] rounded px-1 py-0.5 transition-colors",
                onclick: move |_| open.set(!*open.read()),
                span { class: "text-sm text-[var(--on-surface)]", "{current_name}" }
            }
            if *open.read() {
                div { class: "absolute left-0 top-full mt-1 z-50 w-44 border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-lg rounded p-1",
                    button {
                        class: "flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-[var(--surface-container-high)]",
                        onclick: move |_| {
                            on_change.call(String::new());
                            open.set(false);
                        },
                        "Unassigned"
                    }
                    for agent in agents.iter() {
                        button {
                            class: "flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-[var(--surface-container-high)]",
                            onclick: {
                                let id = agent.id.clone();
                                move |_| {
                                    on_change.call(id.clone());
                                    open.set(false);
                                }
                            },
                            "{agent.name}"
                        }
                    }
                }
            }
        }
    }
}
```

## Step 6: Create Comment Thread

Create `crates/lx-desktop/src/pages/issues/comments.rs`:

Comment list with add-comment form. Reference `CommentThread.tsx` from `IssueDetail.tsx`.

```rust
use dioxus::prelude::*;
use super::types::{AgentRef, IssueComment};
use crate::styles::BTN_PRIMARY_SM;

#[component]
pub fn CommentThread(
    comments: Vec<IssueComment>,
    agents: Vec<AgentRef>,
    on_add: EventHandler<String>,
) -> Element {
    let mut draft = use_signal(String::new);

    rsx! {
        div { class: "space-y-4",
            h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Comments" }
            if comments.is_empty() {
                p { class: "text-sm text-[var(--outline)]", "No comments yet." }
            }
            for comment in comments.iter() {
                CommentBubble {
                    comment: comment.clone(),
                    agents: agents.clone(),
                }
            }
            // Add comment form
            div { class: "space-y-2",
                textarea {
                    class: "w-full rounded border border-[var(--outline-variant)] px-3 py-2 bg-transparent outline-none text-sm min-h-[80px] resize-y placeholder:text-[var(--outline)]/40",
                    placeholder: "Write a comment...",
                    value: "{draft}",
                    oninput: move |evt| draft.set(evt.value().to_string()),
                }
                div { class: "flex justify-end",
                    button {
                        class: BTN_PRIMARY_SM,
                        disabled: draft.read().trim().is_empty(),
                        onclick: move |_| {
                            let body = draft.read().trim().to_string();
                            if !body.is_empty() {
                                on_add.call(body);
                                draft.set(String::new());
                            }
                        },
                        "Comment"
                    }
                }
            }
        }
    }
}

#[component]
fn CommentBubble(comment: IssueComment, agents: Vec<AgentRef>) -> Element {
    let author = comment.author_agent_id.as_ref().and_then(|aid| {
        agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone())
    }).unwrap_or_else(|| {
        if comment.author_user_id.is_some() {
            "User".to_string()
        } else {
            "System".to_string()
        }
    });

    rsx! {
        div { class: "border border-[var(--outline-variant)]/20 rounded-lg p-3 space-y-1",
            div { class: "flex items-center justify-between",
                span { class: "text-xs font-medium text-[var(--on-surface)]", "{author}" }
                span { class: "text-xs text-[var(--outline)]", "{comment.created_at}" }
            }
            div { class: "text-sm text-[var(--on-surface-variant)] whitespace-pre-wrap",
                "{comment.body}"
            }
        }
    }
}
```

## Step 7: Create Documents Section

Create `crates/lx-desktop/src/pages/issues/documents.rs`:

Collapsible document list with markdown body display. Reference `IssueDocumentsSection.tsx`.

```rust
use dioxus::prelude::*;
use super::types::IssueDocument;

#[component]
pub fn DocumentsSection(documents: Vec<IssueDocument>) -> Element {
    rsx! {
        div { class: "space-y-3",
            h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Documents" }
            div { class: "space-y-2",
                for doc in documents.iter() {
                    DocumentCard { document: doc.clone() }
                }
            }
        }
    }
}

#[component]
fn DocumentCard(document: IssueDocument) -> Element {
    let mut expanded = use_signal(|| false);
    let title = document.title.as_deref().unwrap_or(&document.key);
    let has_body = !document.body.is_empty();

    rsx! {
        div { class: "border border-[var(--outline-variant)]/20 rounded-lg overflow-hidden",
            button {
                class: "flex items-center gap-2 w-full px-3 py-2.5 text-left hover:bg-[var(--surface-container)] transition-colors",
                onclick: move |_| {
                    if has_body {
                        expanded.set(!*expanded.read());
                    }
                },
                span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                    "description"
                }
                span { class: "flex-1 text-sm font-medium text-[var(--on-surface)]",
                    "{title}"
                }
                if has_body {
                    span { class: "material-symbols-outlined text-xs text-[var(--outline)]",
                        if *expanded.read() { "expand_less" } else { "expand_more" }
                    }
                }
                if let Some(updated) = &document.updated_at {
                    span { class: "text-xs text-[var(--outline)]", "{updated}" }
                }
            }
            if *expanded.read() && has_body {
                div { class: "px-3 py-3 border-t border-[var(--outline-variant)]/15 text-sm text-[var(--on-surface-variant)] whitespace-pre-wrap",
                    "{document.body}"
                }
            }
        }
    }
}
```

## Step 8: Create Workspace Card

Create `crates/lx-desktop/src/pages/issues/workspace_card.rs`:

Workspace info display with branch, path, mode. Reference `IssueWorkspaceCard.tsx`.

```rust
use dioxus::prelude::*;
use super::types::IssueWorkspace;
use crate::styles::PROPERTY_LABEL;

#[component]
pub fn WorkspaceCard(workspace: IssueWorkspace) -> Element {
    let mode_label = match workspace.mode.as_deref() {
        Some("isolated_workspace") => "Isolated workspace",
        Some("operator_branch") => "Operator branch",
        Some("cloud_sandbox") => "Cloud sandbox",
        Some("adapter_managed") => "Adapter managed",
        _ => "Workspace",
    };

    rsx! {
        div { class: "border border-[var(--outline-variant)]/20 rounded-lg p-4 space-y-3",
            div { class: "flex items-center gap-2",
                span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                    "folder_open"
                }
                span { class: "text-sm font-medium text-[var(--on-surface)]",
                    "{mode_label}"
                }
            }
            if let Some(branch) = &workspace.branch_name {
                div { class: "flex items-center gap-3 py-1",
                    span { class: PROPERTY_LABEL, "Branch" }
                    span { class: "text-sm font-mono text-[var(--on-surface)] break-all",
                        "{branch}"
                    }
                }
            }
            if let Some(path) = &workspace.worktree_path {
                div { class: "flex items-center gap-3 py-1",
                    span { class: PROPERTY_LABEL, "Path" }
                    span { class: "text-sm font-mono text-[var(--on-surface)] break-all",
                        "{path}"
                    }
                }
            }
        }
    }
}
```

## Step 9: Create New Issue Dialog

Create `crates/lx-desktop/src/pages/issues/new_issue.rs`:

Dialog for creating a new issue with title, description, status, priority, and assignee. Reference `NewIssueDialog.tsx`.

```rust
use dioxus::prelude::*;
use super::types::{AgentRef, PRIORITY_ORDER, STATUS_ORDER};
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};

#[derive(Clone, Debug)]
pub struct NewIssuePayload {
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assignee_agent_id: Option<String>,
}

#[component]
pub fn NewIssueDialog(
    open: bool,
    agents: Vec<AgentRef>,
    on_close: EventHandler<()>,
    on_create: EventHandler<NewIssuePayload>,
) -> Element {
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut status = use_signal(|| "todo".to_string());
    let mut priority = use_signal(|| "medium".to_string());
    let mut assignee = use_signal(|| Option::<String>::None);

    if !open {
        return rsx! {};
    }

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg w-full max-w-lg overflow-hidden",
                onclick: move |evt| evt.stop_propagation(),
                // Header
                div { class: "flex items-center justify-between px-4 py-2.5 border-b border-[var(--outline-variant)]",
                    span { class: "text-sm text-[var(--outline)]", "New Issue" }
                    button {
                        class: "text-[var(--outline)] hover:text-[var(--on-surface)] text-lg",
                        onclick: move |_| on_close.call(()),
                        "x"
                    }
                }
                div { class: "p-4 space-y-4",
                    input {
                        class: "w-full text-lg font-semibold bg-transparent outline-none text-[var(--on-surface)] placeholder:text-[var(--outline)]/40",
                        placeholder: "Issue title",
                        value: "{title}",
                        oninput: move |evt| title.set(evt.value().to_string()),
                    }
                    textarea {
                        class: "w-full rounded border border-[var(--outline-variant)] px-3 py-2 bg-transparent outline-none text-sm min-h-[100px] resize-y placeholder:text-[var(--outline)]/40",
                        placeholder: "Description (optional)",
                        value: "{description}",
                        oninput: move |evt| description.set(evt.value().to_string()),
                    }
                    div { class: "grid grid-cols-3 gap-3",
                        div {
                            label { class: "text-xs text-[var(--outline)] block mb-1", "Status" }
                            select {
                                class: INPUT_FIELD,
                                value: "{status}",
                                onchange: move |evt| status.set(evt.value().to_string()),
                                for s in STATUS_ORDER {
                                    option { value: *s, "{s}" }
                                }
                            }
                        }
                        div {
                            label { class: "text-xs text-[var(--outline)] block mb-1", "Priority" }
                            select {
                                class: INPUT_FIELD,
                                value: "{priority}",
                                onchange: move |evt| priority.set(evt.value().to_string()),
                                for p in PRIORITY_ORDER {
                                    option { value: *p, "{p}" }
                                }
                            }
                        }
                        div {
                            label { class: "text-xs text-[var(--outline)] block mb-1", "Assignee" }
                            select {
                                class: INPUT_FIELD,
                                value: assignee.read().as_deref().unwrap_or(""),
                                onchange: move |evt| {
                                    let v = evt.value().to_string();
                                    assignee.set(if v.is_empty() { None } else { Some(v) });
                                },
                                option { value: "", "Unassigned" }
                                for agent in agents.iter() {
                                    option { value: "{agent.id}", "{agent.name}" }
                                }
                            }
                        }
                    }
                }
                // Footer
                div { class: "border-t border-[var(--outline-variant)] px-4 py-3 flex justify-end gap-2",
                    button {
                        class: BTN_OUTLINE_SM,
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: BTN_PRIMARY_SM,
                        disabled: title.read().trim().is_empty(),
                        onclick: {
                            let title = title.clone();
                            let description = description.clone();
                            let status = status.clone();
                            let priority = priority.clone();
                            let assignee = assignee.clone();
                            move |_| {
                                on_create.call(NewIssuePayload {
                                    title: title.read().trim().to_string(),
                                    description: description.read().trim().to_string(),
                                    status: status.read().clone(),
                                    priority: priority.read().clone(),
                                    assignee_agent_id: assignee.read().clone(),
                                });
                            }
                        },
                        "Create Issue"
                    }
                }
            }
        }
    }
}
```

## Step 10: Create Module Root and Wire Routes

Create `crates/lx-desktop/src/pages/issues/mod.rs`:

```rust
mod comments;
mod detail;
mod documents;
mod kanban;
mod list;
mod new_issue;
mod properties;
pub mod types;
mod workspace_card;

use dioxus::prelude::*;
use self::detail::IssueDetailPage;
use self::list::IssuesList;
use self::new_issue::{NewIssueDialog, NewIssuePayload};
use self::types::*;

#[component]
pub fn Issues() -> Element {
    let mut selected_issue_id = use_signal(|| Option::<String>::None);
    let mut show_new_dialog = use_signal(|| false);
    let issues: Vec<Issue> = Vec::new();
    let agents: Vec<AgentRef> = Vec::new();

    rsx! {
        match selected_issue_id.read().as_ref() {
            Some(_id) => rsx! {
                IssueDetailPage {
                    issue: Issue {
                        id: String::new(),
                        identifier: None,
                        title: "Loading...".to_string(),
                        description: None,
                        status: "todo".to_string(),
                        priority: "medium".to_string(),
                        assignee_agent_id: None,
                        assignee_user_id: None,
                        project_id: None,
                        parent_id: None,
                        label_ids: Vec::new(),
                        labels: Vec::new(),
                        created_at: String::new(),
                        updated_at: String::new(),
                        started_at: None,
                        completed_at: None,
                        created_by_agent_id: None,
                        created_by_user_id: None,
                        request_depth: 0,
                        company_id: None,
                    },
                    comments: Vec::new(),
                    documents: Vec::new(),
                    workspace: None,
                    agents: agents.clone(),
                    on_back: move |_| selected_issue_id.set(None),
                    on_update: move |_: (String, String)| {},
                    on_add_comment: move |_: String| {},
                }
            },
            None => rsx! {
                IssuesList {
                    issues,
                    agents: agents.clone(),
                    on_select: move |id: String| selected_issue_id.set(Some(id)),
                    on_new_issue: move |_| show_new_dialog.set(true),
                    on_update: move |_: (String, String, String)| {},
                }
            },
        }
        NewIssueDialog {
            open: *show_new_dialog.read(),
            agents: agents.clone(),
            on_close: move |_| show_new_dialog.set(false),
            on_create: move |_payload: NewIssuePayload| {
                show_new_dialog.set(false);
            },
        }
    }
}
```

The `pub mod issues;` declaration already exists in `pages/mod.rs` from Unit 3. No changes to `pages/mod.rs` or `routes.rs` are needed -- the Rust module system automatically resolves the directory module (`issues/mod.rs`) in place of the former single-file stub (`issues.rs`).

## Definition of Done

1. Nine new files exist under `crates/lx-desktop/src/pages/issues/`: `types.rs`, `list.rs`, `kanban.rs`, `detail.rs`, `properties.rs`, `comments.rs`, `documents.rs`, `workspace_card.rs`, `new_issue.rs`.
2. `mod.rs` exists and wires all submodules together into the `Issues` page component.
3. `pages/mod.rs` contains `pub mod issues;` (already present from Unit 3).
4. `routes.rs` compiles with existing route variants (already defined by Unit 3).
5. No file exceeds 300 lines.
6. `just diagnose` passes (no compiler errors, no clippy warnings).
7. The Issues page renders a list view with filter tabs, search, list/board toggle, and issue rows with status/priority/assignee.
8. The kanban board view renders columns by status with issue cards.
9. Clicking an issue transitions to the detail page with title, description, properties sidebar, comments, and documents.
10. The New Issue dialog opens with title/description/status/priority/assignee fields.
11. The properties panel has interactive status, priority, and assignee pickers.
