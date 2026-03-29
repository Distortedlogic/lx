# Unit 13: Inbox & Settings Pages

## Scope

Port the Inbox page (multi-tab, multi-category inbox with approval/failed-run/join-request rows) and expand the settings pages (company settings, instance settings with heartbeats/general/experimental sub-pages) from Paperclip React into Dioxus 0.7.3 components in lx-desktop.

## Paperclip Source Files

| Source | What it contains |
|--------|-----------------|
| `reference/paperclip/ui/src/pages/Inbox.tsx` (1290 lines) | Multi-tab inbox (mine/recent/all/unread), category filters, `FailedRunInboxRow`, `ApprovalInboxRow`, `JoinRequestInboxRow`, `Inbox` main component with approval/retry/archive mutations |
| `reference/paperclip/ui/src/pages/CompanySettings.tsx` (662 lines) | Company general settings (name, description, brand color, logo), hiring toggle, invite snippet generation, import/export links, archive/danger zone |
| `reference/paperclip/ui/src/pages/InstanceSettings.tsx` (283 lines) | Scheduler heartbeats list grouped by company, enable/disable toggle per agent, disable-all button |
| `reference/paperclip/ui/src/pages/InstanceGeneralSettings.tsx` (104 lines) | Single toggle: censor username in logs |
| `reference/paperclip/ui/src/pages/InstanceExperimentalSettings.tsx` (139 lines) | Two toggles: isolated workspaces, auto-restart dev server |

## Target Directory Structure

```
crates/lx-desktop/src/
  pages/
    inbox/
      mod.rs          (new — Inbox page component, tab bar, category filter)
      inbox_rows.rs   (new — FailedRunRow, ApprovalRow, JoinRequestRow)
      inbox_state.rs  (new — InboxTab enum, InboxCategoryFilter enum, state types)
    settings/
      mod.rs              (existing — update to add sub-page routing)
      env_vars.rs          (existing — no changes)
      quotas.rs            (existing — no changes)
      state.rs             (existing — no changes)
      task_priority.rs     (existing — no changes)
      company_settings.rs  (new — company general/appearance/hiring/invites/danger)
      instance_settings.rs (new — heartbeats list page)
      instance_general.rs  (new — general toggle settings)
      instance_experimental.rs (new — experimental toggle settings)
  pages/mod.rs           (existing — already has inbox module from Unit 3)
```

## Preconditions

- **Unit 3 is complete:** Unit 3 created stubs `pages/inbox.rs`, `pages/company_settings.rs`, and `pages/instance_settings.rs`. This unit replaces them with real modules. Delete the `pages/inbox.rs` stub and create `pages/inbox/mod.rs` as a directory module. Replace `pages/company_settings.rs` and `pages/instance_settings.rs` with real implementations (or convert to directory modules if needed). The `routes.rs` Route enum already has `Inbox {}`, `CompanySettings {}`, and `InstanceSettings {}` variants importing from the correct module paths -- no changes to `routes.rs` are needed.
- `crates/lx-desktop/src/pages/settings/mod.rs` exists with `Settings` component
- `crates/lx-desktop/src/pages/settings/state.rs` exists with `SettingsState` and `SettingsData`
- `crates/lx-desktop/src/pages/mod.rs` exists listing page modules
- Dioxus 0.7.3 with `Router`, `#[component]`, `rsx!`, `use_signal`, `use_context`

## Tasks

### Task 1: Create `crates/lx-desktop/src/pages/inbox/inbox_state.rs`

Define enums and state types used by the inbox page.

```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InboxTab {
    Mine,
    Recent,
    All,
    Unread,
}

impl InboxTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Mine => "Mine",
            Self::Recent => "Recent",
            Self::All => "All",
            Self::Unread => "Unread",
        }
    }

    pub fn all() -> &'static [InboxTab] {
        &[Self::Mine, Self::Recent, Self::All, Self::Unread]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InboxCategoryFilter {
    Everything,
    IssuesTouched,
    JoinRequests,
    Approvals,
    FailedRuns,
    Alerts,
}

impl InboxCategoryFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Everything => "Everything",
            Self::IssuesTouched => "Issues I Touched",
            Self::JoinRequests => "Join Requests",
            Self::Approvals => "Approvals",
            Self::FailedRuns => "Failed Runs",
            Self::Alerts => "Alerts",
        }
    }

    pub fn all() -> &'static [InboxCategoryFilter] {
        &[
            Self::Everything,
            Self::IssuesTouched,
            Self::JoinRequests,
            Self::Approvals,
            Self::FailedRuns,
            Self::Alerts,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Approved => write!(f, "approved"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxApprovalItem {
    pub id: String,
    pub approval_type: String,
    pub status: ApprovalStatus,
    pub requester_name: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxFailedRun {
    pub id: String,
    pub agent_id: String,
    pub agent_name: Option<String>,
    pub error_message: String,
    pub status: String,
    pub created_at: String,
    pub issue_id: Option<String>,
    pub issue_title: Option<String>,
    pub issue_identifier: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxJoinRequest {
    pub id: String,
    pub request_type: String,
    pub agent_name: Option<String>,
    pub adapter_type: Option<String>,
    pub request_ip: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxIssueItem {
    pub id: String,
    pub identifier: Option<String>,
    pub title: String,
    pub status: String,
    pub is_unread: bool,
    pub updated_at: String,
}
```

### Task 2: Create `crates/lx-desktop/src/pages/inbox/inbox_rows.rs`

Port the three inbox row components from `Inbox.tsx` lines 101-503.

```rust
use dioxus::prelude::*;
use super::inbox_state::{
    InboxApprovalItem, InboxFailedRun, InboxJoinRequest,
};

#[component]
pub fn FailedRunRow(
    run: InboxFailedRun,
    on_dismiss: EventHandler<String>,
    on_retry: EventHandler<String>,
    is_retrying: bool,
) -> Element {
    let display_error = run.error_message.clone();
    let issue_display = if let (Some(ref ident), Some(ref title)) =
        (&run.issue_identifier, &run.issue_title)
    {
        rsx! {
            span { class: "font-mono text-[var(--outline)] mr-1.5", "{ident}" }
            "{title}"
        }
    } else {
        let label = match &run.agent_name {
            Some(name) => format!("Failed run - {name}"),
            None => "Failed run".to_string(),
        };
        rsx! { "{label}" }
    };

    let run_id = run.id.clone();
    let run_id2 = run.id.clone();
    rsx! {
        div { class: "group border-b border-[var(--outline-variant)] px-2 py-2.5 last:border-b-0",
            div { class: "flex items-start gap-2",
                div { class: "mt-0.5 shrink-0 rounded-md bg-red-500/20 p-1.5",
                    span { class: "material-symbols-outlined text-red-500 text-base",
                        "cancel"
                    }
                }
                div { class: "min-w-0 flex-1",
                    div { class: "text-sm font-medium truncate", {issue_display} }
                    div { class: "mt-1 flex flex-wrap items-center gap-x-2 text-xs text-[var(--outline)]",
                        span { class: "px-1.5 py-0.5 rounded border border-[var(--outline-variant)] text-[10px]",
                            "{run.status}"
                        }
                        if let Some(ref name) = run.agent_name {
                            span { "{name}" }
                        }
                        span { class: "truncate max-w-[300px]", "{display_error}" }
                        span { "{run.created_at}" }
                    }
                }
                div { class: "flex shrink-0 items-center gap-2",
                    button {
                        class: "border border-[var(--outline-variant)] rounded px-2.5 py-1 text-xs hover:bg-[var(--surface-container)]",
                        disabled: is_retrying,
                        onclick: move |_| on_retry.call(run_id.clone()),
                        if is_retrying { "Retrying..." } else { "Retry" }
                    }
                    button {
                        class: "rounded-md p-1 text-[var(--outline)] hover:bg-[var(--surface-container)] hover:text-[var(--on-surface)]",
                        onclick: move |_| on_dismiss.call(run_id2.clone()),
                        span { class: "material-symbols-outlined text-base", "close" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ApprovalRow(
    approval: InboxApprovalItem,
    on_approve: EventHandler<String>,
    on_reject: EventHandler<String>,
    is_pending: bool,
) -> Element {
    let id1 = approval.id.clone();
    let id2 = approval.id.clone();
    let show_buttons = approval.status == super::inbox_state::ApprovalStatus::Pending;
    rsx! {
        div { class: "group border-b border-[var(--outline-variant)] px-2 py-2.5 last:border-b-0",
            div { class: "flex items-start gap-2",
                div { class: "mt-0.5 shrink-0 rounded-md bg-[var(--surface-container)] p-1.5",
                    span { class: "material-symbols-outlined text-[var(--outline)] text-base",
                        "approval"
                    }
                }
                div { class: "min-w-0 flex-1",
                    div { class: "text-sm font-medium truncate",
                        "{approval.approval_type}"
                    }
                    div { class: "mt-1 flex flex-wrap items-center gap-x-2 text-xs text-[var(--outline)]",
                        span { class: "capitalize", "{approval.status}" }
                        if let Some(ref name) = approval.requester_name {
                            span { "requested by {name}" }
                        }
                        span { "updated {approval.updated_at}" }
                    }
                }
                if show_buttons {
                    div { class: "flex shrink-0 items-center gap-2",
                        button {
                            class: "bg-green-700 text-white rounded px-3 py-1 text-xs hover:bg-green-600",
                            disabled: is_pending,
                            onclick: move |_| on_approve.call(id1.clone()),
                            "Approve"
                        }
                        button {
                            class: "bg-red-600 text-white rounded px-3 py-1 text-xs hover:bg-red-500",
                            disabled: is_pending,
                            onclick: move |_| on_reject.call(id2.clone()),
                            "Reject"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn JoinRequestRow(
    join_request: InboxJoinRequest,
    on_approve: EventHandler<String>,
    on_reject: EventHandler<String>,
    is_pending: bool,
) -> Element {
    let label = if join_request.request_type == "human" {
        "Human join request".to_string()
    } else {
        match &join_request.agent_name {
            Some(name) => format!("Agent join request: {name}"),
            None => "Agent join request".to_string(),
        }
    };
    let id1 = join_request.id.clone();
    let id2 = join_request.id.clone();
    rsx! {
        div { class: "group border-b border-[var(--outline-variant)] px-2 py-2.5 last:border-b-0",
            div { class: "flex items-start gap-2",
                div { class: "mt-0.5 shrink-0 rounded-md bg-[var(--surface-container)] p-1.5",
                    span { class: "material-symbols-outlined text-[var(--outline)] text-base",
                        "person_add"
                    }
                }
                div { class: "min-w-0 flex-1",
                    div { class: "text-sm font-medium truncate", "{label}" }
                    div { class: "mt-1 flex flex-wrap items-center gap-x-2 text-xs text-[var(--outline)]",
                        span { "requested {join_request.created_at} from IP {join_request.request_ip}" }
                        if let Some(ref adapter) = join_request.adapter_type {
                            span { "adapter: {adapter}" }
                        }
                    }
                }
                div { class: "flex shrink-0 items-center gap-2",
                    button {
                        class: "bg-green-700 text-white rounded px-3 py-1 text-xs hover:bg-green-600",
                        disabled: is_pending,
                        onclick: move |_| on_approve.call(id1.clone()),
                        "Approve"
                    }
                    button {
                        class: "bg-red-600 text-white rounded px-3 py-1 text-xs hover:bg-red-500",
                        disabled: is_pending,
                        onclick: move |_| on_reject.call(id2.clone()),
                        "Reject"
                    }
                }
            }
        }
    }
}
```

### Task 3: Create `crates/lx-desktop/src/pages/inbox/mod.rs`

Port `Inbox()` from `Inbox.tsx` lines 505-1290. This is the main inbox page with tab bar, category filter dropdown, and sections rendering the row components.

Reference: `Inbox.tsx` exports `Inbox` which uses `PageTabBar` for mine/recent/all/unread tabs, a category `Select` dropdown for the "all" tab, and renders sections for work items (issues + approvals + failed runs + join requests merged and sorted).

```rust
mod inbox_rows;
mod inbox_state;

use dioxus::prelude::*;
use self::inbox_rows::{ApprovalRow, FailedRunRow, JoinRequestRow};
use self::inbox_state::{
    InboxApprovalItem, InboxCategoryFilter, InboxFailedRun,
    InboxJoinRequest, InboxIssueItem, InboxTab,
};

#[component]
fn InboxTabBar(active: InboxTab, on_change: EventHandler<InboxTab>) -> Element {
    rsx! {
        div { class: "flex border-b border-[var(--outline-variant)]",
            for tab in InboxTab::all() {
                {
                    let t = *tab;
                    let is_active = t == active;
                    let cls = if is_active {
                        "px-4 py-2 text-xs font-semibold uppercase tracking-wider border-b-2 border-[var(--primary)] text-[var(--on-surface)]"
                    } else {
                        "px-4 py-2 text-xs uppercase tracking-wider text-[var(--outline)] hover:text-[var(--on-surface)] cursor-pointer"
                    };
                    rsx! {
                        button {
                            class: cls,
                            onclick: move |_| on_change.call(t),
                            "{tab.label()}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CategoryFilterSelect(
    value: InboxCategoryFilter,
    on_change: EventHandler<InboxCategoryFilter>,
) -> Element {
    rsx! {
        select {
            class: "bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded px-2 py-1 text-xs text-[var(--on-surface)]",
            value: "{value.label()}",
            onchange: move |evt| {
                let selected = match evt.value().as_str() {
                    "Everything" => InboxCategoryFilter::Everything,
                    "Issues I Touched" => InboxCategoryFilter::IssuesTouched,
                    "Join Requests" => InboxCategoryFilter::JoinRequests,
                    "Approvals" => InboxCategoryFilter::Approvals,
                    "Failed Runs" => InboxCategoryFilter::FailedRuns,
                    "Alerts" => InboxCategoryFilter::Alerts,
                    _ => InboxCategoryFilter::Everything,
                };
                on_change.call(selected);
            },
            for filter in InboxCategoryFilter::all() {
                option { value: "{filter.label()}", "{filter.label()}" }
            }
        }
    }
}

#[component]
pub fn Inbox() -> Element {
    let mut active_tab = use_signal(|| InboxTab::Mine);
    let mut category_filter = use_signal(|| InboxCategoryFilter::Everything);
    let mut action_error: Signal<Option<String>> = use_signal(|| None);

    let demo_approvals: Vec<InboxApprovalItem> = vec![];
    let demo_failed_runs: Vec<InboxFailedRun> = vec![];
    let demo_join_requests: Vec<InboxJoinRequest> = vec![];
    let demo_issues: Vec<InboxIssueItem> = vec![];

    let tab = active_tab();
    let filter = category_filter();

    let show_approvals = filter == InboxCategoryFilter::Everything
        || filter == InboxCategoryFilter::Approvals;
    let show_failed_runs = filter == InboxCategoryFilter::Everything
        || filter == InboxCategoryFilter::FailedRuns;
    let show_join_requests = filter == InboxCategoryFilter::Everything
        || filter == InboxCategoryFilter::JoinRequests;
    let show_issues = filter == InboxCategoryFilter::Everything
        || filter == InboxCategoryFilter::IssuesTouched;

    let is_empty = demo_approvals.is_empty()
        && demo_failed_runs.is_empty()
        && demo_join_requests.is_empty()
        && demo_issues.is_empty();

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center gap-2 px-4 py-3",
                span { class: "material-symbols-outlined text-[var(--outline)]",
                    "inbox"
                }
                h1 { class: "text-lg font-semibold text-[var(--on-surface)]",
                    "Inbox"
                }
            }
            InboxTabBar {
                active: tab,
                on_change: move |t| active_tab.set(t),
            }
            if tab == InboxTab::All {
                div { class: "px-4 py-2 border-b border-[var(--outline-variant)]",
                    CategoryFilterSelect {
                        value: filter,
                        on_change: move |f| category_filter.set(f),
                    }
                }
            }
            if let Some(ref err) = action_error() {
                div { class: "mx-4 mt-2 rounded-md border border-red-500/40 bg-red-500/5 px-3 py-2 text-sm text-red-500",
                    "{err}"
                }
            }
            div { class: "flex-1 overflow-auto",
                if is_empty {
                    div { class: "flex flex-col items-center justify-center py-16 text-[var(--outline)]",
                        span { class: "material-symbols-outlined text-4xl mb-4",
                            "inbox"
                        }
                        p { class: "text-sm", "Your inbox is empty." }
                    }
                } else {
                    if show_issues && !demo_issues.is_empty() {
                        div { class: "border-b border-[var(--outline-variant)]",
                            div { class: "px-4 py-2 text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                                "Issues"
                            }
                            for issue in demo_issues.iter() {
                                div { class: "px-4 py-2 border-b border-[var(--outline-variant)]/30 text-sm",
                                    div { class: "flex items-center gap-2",
                                        if let Some(ref ident) = issue.identifier {
                                            span { class: "font-mono text-[var(--outline)]", "{ident}" }
                                        }
                                        span { class: "font-medium", "{issue.title}" }
                                        span { class: "ml-auto text-xs text-[var(--outline)]", "{issue.status}" }
                                    }
                                }
                            }
                        }
                    }
                    if show_approvals && !demo_approvals.is_empty() {
                        div { class: "border-b border-[var(--outline-variant)]",
                            div { class: "px-4 py-2 text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                                "Approvals"
                            }
                            for approval in demo_approvals.iter() {
                                ApprovalRow {
                                    key: "{approval.id}",
                                    approval: approval.clone(),
                                    on_approve: move |id: String| {
                                        let _ = &id;
                                    },
                                    on_reject: move |id: String| {
                                        let _ = &id;
                                    },
                                    is_pending: false,
                                }
                            }
                        }
                    }
                    if show_failed_runs && !demo_failed_runs.is_empty() {
                        div { class: "border-b border-[var(--outline-variant)]",
                            div { class: "px-4 py-2 text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                                "Failed Runs"
                            }
                            for run in demo_failed_runs.iter() {
                                FailedRunRow {
                                    key: "{run.id}",
                                    run: run.clone(),
                                    on_dismiss: move |id: String| {
                                        let _ = &id;
                                    },
                                    on_retry: move |id: String| {
                                        let _ = &id;
                                    },
                                    is_retrying: false,
                                }
                            }
                        }
                    }
                    if show_join_requests && !demo_join_requests.is_empty() {
                        div {
                            div { class: "px-4 py-2 text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                                "Join Requests"
                            }
                            for jr in demo_join_requests.iter() {
                                JoinRequestRow {
                                    key: "{jr.id}",
                                    join_request: jr.clone(),
                                    on_approve: move |id: String| {
                                        let _ = &id;
                                    },
                                    on_reject: move |id: String| {
                                        let _ = &id;
                                    },
                                    is_pending: false,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Task 4: Create `crates/lx-desktop/src/pages/settings/company_settings.rs`

Port `CompanySettings` from `CompanySettings.tsx`. Reference sections: General (name, description), Appearance (brand color), Hiring (board approval toggle), Invites (snippet generation), Import/Export links, Danger Zone (archive).

```rust
use dioxus::prelude::*;

#[component]
pub fn CompanySettings() -> Element {
    let mut company_name = use_signal(|| "Default Company".to_string());
    let mut description = use_signal(String::new);
    let mut brand_color = use_signal(|| "#6366f1".to_string());
    let mut require_approval = use_signal(|| false);
    let mut invite_snippet: Signal<Option<String>> = use_signal(|| None);
    let mut snippet_copied = use_signal(|| false);

    let general_dirty = true;

    rsx! {
        div { class: "max-w-2xl space-y-6 p-4 overflow-auto",
            div { class: "flex items-center gap-2",
                span { class: "material-symbols-outlined text-[var(--outline)]", "settings" }
                h1 { class: "text-lg font-semibold text-[var(--on-surface)]",
                    "Company Settings"
                }
            }

            // General
            div { class: "space-y-4",
                div { class: "text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                    "General"
                }
                div { class: "space-y-3 rounded-md border border-[var(--outline-variant)] px-4 py-4",
                    div { class: "space-y-1",
                        label { class: "text-xs font-medium text-[var(--on-surface)]",
                            "Company name"
                        }
                        input {
                            class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-2.5 py-1.5 text-sm outline-none text-[var(--on-surface)]",
                            r#type: "text",
                            value: "{company_name}",
                            oninput: move |evt| company_name.set(evt.value()),
                        }
                    }
                    div { class: "space-y-1",
                        label { class: "text-xs font-medium text-[var(--on-surface)]",
                            "Description"
                        }
                        input {
                            class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-2.5 py-1.5 text-sm outline-none text-[var(--on-surface)]",
                            r#type: "text",
                            value: "{description}",
                            placeholder: "Optional company description",
                            oninput: move |evt| description.set(evt.value()),
                        }
                    }
                }
            }

            // Appearance
            div { class: "space-y-4",
                div { class: "text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                    "Appearance"
                }
                div { class: "space-y-3 rounded-md border border-[var(--outline-variant)] px-4 py-4",
                    div { class: "space-y-1",
                        label { class: "text-xs font-medium text-[var(--on-surface)]",
                            "Brand color"
                        }
                        div { class: "flex items-center gap-2",
                            input {
                                r#type: "color",
                                value: "{brand_color}",
                                class: "h-8 w-8 cursor-pointer rounded border border-[var(--outline-variant)] bg-transparent p-0",
                                oninput: move |evt| brand_color.set(evt.value()),
                            }
                            input {
                                r#type: "text",
                                value: "{brand_color}",
                                class: "w-28 rounded-md border border-[var(--outline-variant)] bg-transparent px-2.5 py-1.5 text-sm font-mono outline-none text-[var(--on-surface)]",
                                oninput: move |evt| brand_color.set(evt.value()),
                            }
                        }
                    }
                }
            }

            // Save button
            if general_dirty {
                div { class: "flex items-center gap-2",
                    button {
                        class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-1.5 text-xs font-semibold",
                        "Save changes"
                    }
                }
            }

            // Hiring
            div { class: "space-y-4",
                div { class: "text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                    "Hiring"
                }
                div { class: "rounded-md border border-[var(--outline-variant)] px-4 py-3",
                    div { class: "flex items-center justify-between",
                        div {
                            span { class: "text-sm text-[var(--on-surface)]",
                                "Require board approval for new hires"
                            }
                        }
                        button {
                            class: if require_approval() {
                                "relative inline-flex h-5 w-9 items-center rounded-full bg-green-600"
                            } else {
                                "relative inline-flex h-5 w-9 items-center rounded-full bg-[var(--surface-container)]"
                            },
                            onclick: move |_| {
                                let current = require_approval();
                                require_approval.set(!current);
                            },
                            span {
                                class: if require_approval() {
                                    "inline-block h-3.5 w-3.5 rounded-full bg-white translate-x-4"
                                } else {
                                    "inline-block h-3.5 w-3.5 rounded-full bg-white translate-x-0.5"
                                },
                            }
                        }
                    }
                }
            }

            // Invites
            div { class: "space-y-4",
                div { class: "text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
                    "Invites"
                }
                div { class: "space-y-3 rounded-md border border-[var(--outline-variant)] px-4 py-4",
                    p { class: "text-xs text-[var(--outline)]",
                        "Generate an agent invite snippet."
                    }
                    button {
                        class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-1.5 text-xs font-semibold",
                        onclick: move |_| {
                            invite_snippet.set(Some("Invite snippet placeholder".to_string()));
                        },
                        "Generate Invite Prompt"
                    }
                    if let Some(ref snippet) = invite_snippet() {
                        div { class: "rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container)]/30 p-2",
                            textarea {
                                class: "h-48 w-full rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] px-2 py-1.5 font-mono text-xs outline-none text-[var(--on-surface)]",
                                readonly: true,
                                value: "{snippet}",
                            }
                        }
                    }
                }
            }

            // Danger Zone
            div { class: "space-y-4",
                div { class: "text-xs font-medium text-red-500 uppercase tracking-wide",
                    "Danger Zone"
                }
                div { class: "space-y-3 rounded-md border border-red-500/40 bg-red-500/5 px-4 py-4",
                    p { class: "text-sm text-[var(--outline)]",
                        "Archive this company to hide it from the sidebar."
                    }
                    button {
                        class: "bg-red-600 text-white rounded px-4 py-1.5 text-xs font-semibold hover:bg-red-500",
                        "Archive company"
                    }
                }
            }
        }
    }
}
```

### Task 5: Create `crates/lx-desktop/src/pages/settings/instance_settings.rs`

Port `InstanceSettings` from `InstanceSettings.tsx`. Shows scheduler heartbeats list grouped by company with enable/disable toggles.

```rust
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
struct HeartbeatAgent {
    id: String,
    agent_name: String,
    company_id: String,
    company_name: String,
    title: String,
    interval_sec: u32,
    scheduler_active: bool,
    heartbeat_enabled: bool,
    last_heartbeat_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct HeartbeatGroup {
    company_name: String,
    agents: Vec<HeartbeatAgent>,
}

fn group_agents(agents: &[HeartbeatAgent]) -> Vec<HeartbeatGroup> {
    let mut map: std::collections::BTreeMap<String, Vec<HeartbeatAgent>> =
        std::collections::BTreeMap::new();
    for agent in agents {
        map.entry(agent.company_name.clone())
            .or_default()
            .push(agent.clone());
    }
    map.into_iter()
        .map(|(company_name, agents)| HeartbeatGroup {
            company_name,
            agents,
        })
        .collect()
}

#[component]
pub fn InstanceHeartbeats() -> Element {
    let agents: Vec<HeartbeatAgent> = vec![];
    let grouped = group_agents(&agents);
    let active_count = agents.iter().filter(|a| a.scheduler_active).count();
    let disabled_count = agents.len() - active_count;
    let enabled_count = agents.iter().filter(|a| a.heartbeat_enabled).count();

    rsx! {
        div { class: "max-w-5xl space-y-6 p-4 overflow-auto",
            div { class: "space-y-2",
                div { class: "flex items-center gap-2",
                    span { class: "material-symbols-outlined text-[var(--outline)]", "settings" }
                    h1 { class: "text-lg font-semibold text-[var(--on-surface)]",
                        "Scheduler Heartbeats"
                    }
                }
                p { class: "text-sm text-[var(--outline)]",
                    "Agents with a timer heartbeat enabled across all companies."
                }
            }
            div { class: "flex items-center gap-4 text-sm text-[var(--outline)]",
                span {
                    span { class: "font-semibold text-[var(--on-surface)]", "{active_count}" }
                    " active"
                }
                span {
                    span { class: "font-semibold text-[var(--on-surface)]", "{disabled_count}" }
                    " disabled"
                }
                span {
                    span { class: "font-semibold text-[var(--on-surface)]", "{grouped.len()}" }
                    if grouped.len() == 1 { " company" } else { " companies" }
                }
                if enabled_count > 0 {
                    button {
                        class: "ml-auto bg-red-600 text-white rounded px-3 py-1 text-xs font-semibold",
                        "Disable All"
                    }
                }
            }
            if agents.is_empty() {
                div { class: "flex flex-col items-center justify-center py-16 text-[var(--outline)]",
                    span { class: "material-symbols-outlined text-4xl mb-4", "schedule" }
                    p { class: "text-sm", "No scheduler heartbeats." }
                }
            } else {
                for group in grouped.iter() {
                    div { class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)]",
                        div { class: "border-b px-3 py-2 text-xs font-semibold uppercase tracking-wide text-[var(--outline)]",
                            "{group.company_name}"
                        }
                        for agent in group.agents.iter() {
                            div { class: "flex items-center gap-3 px-3 py-2 text-sm border-b border-[var(--outline-variant)]/30 last:border-b-0",
                                span {
                                    class: if agent.scheduler_active {
                                        "shrink-0 text-[10px] px-1.5 py-0 rounded border border-[var(--primary)] text-[var(--primary)]"
                                    } else {
                                        "shrink-0 text-[10px] px-1.5 py-0 rounded border border-[var(--outline-variant)] text-[var(--outline)]"
                                    },
                                    if agent.scheduler_active { "On" } else { "Off" }
                                }
                                span { class: "font-medium truncate", "{agent.agent_name}" }
                                span { class: "text-[var(--outline)] truncate", "{agent.title}" }
                                span { class: "text-[var(--outline)] tabular-nums shrink-0",
                                    "{agent.interval_sec}s"
                                }
                                span { class: "text-[var(--outline)] truncate",
                                    if let Some(ref ts) = agent.last_heartbeat_at {
                                        "{ts}"
                                    } else {
                                        "never"
                                    }
                                }
                                button {
                                    class: "ml-auto text-xs px-2 py-1 rounded hover:bg-[var(--surface-container)]",
                                    if agent.heartbeat_enabled {
                                        "Disable Timer Heartbeat"
                                    } else {
                                        "Enable Timer Heartbeat"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Task 6: Create `crates/lx-desktop/src/pages/settings/instance_general.rs`

Port `InstanceGeneralSettings` from `InstanceGeneralSettings.tsx`. Single toggle for censor username in logs.

```rust
use dioxus::prelude::*;

#[component]
pub fn InstanceGeneral() -> Element {
    let mut censor_username = use_signal(|| false);

    rsx! {
        div { class: "max-w-4xl space-y-6 p-4 overflow-auto",
            div { class: "space-y-2",
                div { class: "flex items-center gap-2",
                    span { class: "material-symbols-outlined text-[var(--outline)]",
                        "tune"
                    }
                    h1 { class: "text-lg font-semibold text-[var(--on-surface)]",
                        "General"
                    }
                }
                p { class: "text-sm text-[var(--outline)]",
                    "Configure instance-wide defaults that affect how operator-visible logs are displayed."
                }
            }
            div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] p-5",
                div { class: "flex items-start justify-between gap-4",
                    div { class: "space-y-1.5",
                        h2 { class: "text-sm font-semibold text-[var(--on-surface)]",
                            "Censor username in logs"
                        }
                        p { class: "max-w-2xl text-sm text-[var(--outline)]",
                            "Hide the username segment in home-directory paths and similar operator-visible log output."
                        }
                    }
                    button {
                        class: if censor_username() {
                            "relative inline-flex h-5 w-9 items-center rounded-full bg-green-600 transition-colors"
                        } else {
                            "relative inline-flex h-5 w-9 items-center rounded-full bg-[var(--surface-container)] transition-colors"
                        },
                        onclick: move |_| {
                            let current = censor_username();
                            censor_username.set(!current);
                        },
                        span {
                            class: if censor_username() {
                                "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform translate-x-4"
                            } else {
                                "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform translate-x-0.5"
                            },
                        }
                    }
                }
            }
        }
    }
}
```

### Task 7: Create `crates/lx-desktop/src/pages/settings/instance_experimental.rs`

Port `InstanceExperimentalSettings` from `InstanceExperimentalSettings.tsx`. Two toggles: isolated workspaces, auto-restart dev server.

```rust
use dioxus::prelude::*;

#[component]
fn ToggleSection(
    title: String,
    description: String,
    enabled: bool,
    on_toggle: EventHandler<bool>,
) -> Element {
    rsx! {
        div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] p-5",
            div { class: "flex items-start justify-between gap-4",
                div { class: "space-y-1.5",
                    h2 { class: "text-sm font-semibold text-[var(--on-surface)]",
                        "{title}"
                    }
                    p { class: "max-w-2xl text-sm text-[var(--outline)]",
                        "{description}"
                    }
                }
                button {
                    class: if enabled {
                        "relative inline-flex h-5 w-9 items-center rounded-full bg-green-600 transition-colors"
                    } else {
                        "relative inline-flex h-5 w-9 items-center rounded-full bg-[var(--surface-container)] transition-colors"
                    },
                    onclick: move |_| on_toggle.call(!enabled),
                    span {
                        class: if enabled {
                            "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform translate-x-4"
                        } else {
                            "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform translate-x-0.5"
                        },
                    }
                }
            }
        }
    }
}

#[component]
pub fn InstanceExperimental() -> Element {
    let mut isolated_workspaces = use_signal(|| false);
    let mut auto_restart = use_signal(|| false);

    rsx! {
        div { class: "max-w-4xl space-y-6 p-4 overflow-auto",
            div { class: "space-y-2",
                div { class: "flex items-center gap-2",
                    span { class: "material-symbols-outlined text-[var(--outline)]",
                        "science"
                    }
                    h1 { class: "text-lg font-semibold text-[var(--on-surface)]",
                        "Experimental"
                    }
                }
                p { class: "text-sm text-[var(--outline)]",
                    "Opt into features that are still being evaluated before they become default behavior."
                }
            }
            ToggleSection {
                title: "Enable Isolated Workspaces",
                description: "Show execution workspace controls in project configuration and allow isolated workspace behavior for new and existing issue runs.",
                enabled: isolated_workspaces(),
                on_toggle: move |v| isolated_workspaces.set(v),
            }
            ToggleSection {
                title: "Auto-Restart Dev Server When Idle",
                description: "Wait for all queued and running local agent runs to finish, then restart the server automatically when backend changes make the current boot stale.",
                enabled: auto_restart(),
                on_toggle: move |v| auto_restart.set(v),
            }
        }
    }
}
```

### Task 8: Update `crates/lx-desktop/src/pages/settings/mod.rs`

Edit `settings/mod.rs` -- add `pub mod company_settings;` and `pub mod instance_settings;` to the existing module declarations. Also add the instance sub-page modules:

```rust
pub mod company_settings;
pub mod instance_experimental;
pub mod instance_general;
pub mod instance_settings;
```

Add these public re-exports after the existing use statements:
```rust
pub use self::company_settings::CompanySettings;
pub use self::instance_experimental::InstanceExperimental;
pub use self::instance_general::InstanceGeneral;
pub use self::instance_settings::InstanceHeartbeats;
```

The existing `Settings` component body remains unchanged. The new pages are routed separately.

### Task 9: Verify `crates/lx-desktop/src/pages/mod.rs`

The `pub mod inbox;`, `pub mod company_settings;`, and `pub mod instance_settings;` declarations already exist from Unit 3. No changes needed.

### Task 10: Note on routes

Unit 3 already has `Inbox`, `CompanySettings`, and `InstanceSettings` route variants with imports pointing at `crate::pages::inbox`, `crate::pages::company_settings`, and `crate::pages::instance_settings`. Creating the real modules at those paths replaces the stubs automatically. Do NOT modify `routes.rs` or `pages/mod.rs`.

## Definition of Done

1. `just diagnose` passes with zero warnings
2. All new files exist at the paths listed above
3. All new files are under 300 lines
4. `routes.rs` compiles with existing route variants (already defined by Unit 3)
5. `pages/mod.rs` already includes `pub mod inbox` (from Unit 3)
6. `pages/settings/mod.rs` includes the four new module declarations
7. The `Inbox` component renders with tab bar (Mine/Recent/All/Unread), category filter on All tab, empty state, and section scaffolding for issues/approvals/failed runs/join requests
8. `CompanySettings` renders with General, Appearance, Hiring, Invites, and Danger Zone sections
9. `InstanceHeartbeats` renders heartbeat agent list grouped by company
10. `InstanceGeneral` renders single censor-username toggle
11. `InstanceExperimental` renders two toggles (isolated workspaces, auto-restart)
