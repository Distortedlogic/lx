# Unit 3: Context Providers & Route Expansion

## Scope

Port 7 context providers from Paperclip (React) to Dioxus 0.7.3 signal-based contexts in `lx-desktop`, and expand `routes.rs` to include all page routes that mirror the Paperclip App.tsx routing structure. The existing `contexts/activity_log.rs` and `contexts/status_bar.rs` are kept unchanged. Stub page modules are created at their real module paths for routes whose pages do not yet exist. Later units replace each stub file with the real implementation -- no unit after Unit 3 needs to modify `routes.rs`.

## Preconditions

- `src/contexts/mod.rs` exists with `pub mod activity_log;` and `pub mod status_bar;`
- `src/routes.rs` exists with the current 5-route enum
- `src/pages/mod.rs` exists with modules: `accounts`, `activity`, `agents`, `settings`, `tools`
- `src/layout/shell.rs` exists and provides `Shell` as a layout component
- Units 1 and 2 are NOT required -- contexts and routes are independent of UI components

## File Inventory

All paths relative to `/home/entropybender/repos/lx/crates/lx-desktop/src/`.

| Action | File |
|--------|------|
| CREATE | `contexts/theme.rs` |
| CREATE | `contexts/toast.rs` |
| CREATE | `contexts/dialog.rs` |
| CREATE | `contexts/panel.rs` |
| CREATE | `contexts/sidebar.rs` |
| CREATE | `contexts/breadcrumb.rs` |
| CREATE | `contexts/company.rs` |
| MODIFY | `contexts/mod.rs` |
| CREATE | `pages/dashboard.rs` |
| CREATE | `pages/projects.rs` |
| CREATE | `pages/issues.rs` |
| CREATE | `pages/goals.rs` |
| CREATE | `pages/approvals.rs` |
| CREATE | `pages/costs.rs` |
| CREATE | `pages/inbox.rs` |
| CREATE | `pages/routines.rs` |
| CREATE | `pages/org.rs` |
| CREATE | `pages/company_settings.rs` |
| CREATE | `pages/instance_settings.rs` |
| CREATE | `pages/onboarding.rs` |
| CREATE | `pages/plugins.rs` |
| CREATE | `pages/companies.rs` |
| CREATE | `pages/company_export.rs` |
| CREATE | `pages/company_import.rs` |
| CREATE | `pages/company_skills.rs` |
| CREATE | `pages/not_found.rs` |
| CREATE | `pages/agent_detail.rs` |
| MODIFY | `pages/mod.rs` |
| MODIFY | `routes.rs` |
| MODIFY | `layout/shell.rs` |

---

## Part A: Context Providers

### Step 1: Port `contexts/theme.rs`

**Source:** `reference/paperclip/ui/src/context/ThemeContext.tsx`

File: `src/contexts/theme.rs`

#### Enum: `Theme`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}
```

#### Struct: `ThemeState`

```rust
#[derive(Clone, Copy)]
pub struct ThemeState {
    pub theme: Signal<Theme>,
}
```

#### Impl: `ThemeState`

```rust
impl ThemeState {
    pub fn provide() -> Self {
        let state = Self { theme: Signal::new(Theme::Dark) };
        use_context_provider(|| state);
        state
    }

    pub fn current(&self) -> Theme {
        *self.theme.read()
    }

    pub fn set(&self, theme: Theme) {
        let mut sig = self.theme;
        sig.set(theme);
    }

    pub fn toggle(&self) {
        let mut sig = self.theme;
        let next = match *sig.read() {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
        sig.set(next);
    }

    pub fn is_dark(&self) -> bool {
        *self.theme.read() == Theme::Dark
    }
}
```

Follow the pattern established by `ActivityLog::provide()` in `contexts/activity_log.rs`: construct state with signals, call `use_context_provider`, return the state.

### Step 2: Port `contexts/toast.rs`

**Source:** `reference/paperclip/ui/src/context/ToastContext.tsx`

File: `src/contexts/toast.rs`

#### Enum: `ToastTone`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ToastTone {
    #[default]
    Info,
    Success,
    Warn,
    Error,
}
```

#### Struct: `ToastAction`

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ToastAction {
    pub label: String,
    pub href: String,
}
```

#### Struct: `ToastInput`

```rust
#[derive(Clone, Debug)]
pub struct ToastInput {
    pub title: String,
    pub body: Option<String>,
    pub tone: ToastTone,
    pub ttl_ms: Option<u64>,
    pub action: Option<ToastAction>,
}
```

#### Struct: `ToastItem`

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ToastItem {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub tone: ToastTone,
    pub ttl_ms: u64,
    pub action: Option<ToastAction>,
    pub created_at: u64,
}
```

#### Constants (matching Paperclip)

```rust
const MAX_TOASTS: usize = 5;
```

Default TTLs by tone:
- `Info`: 4000
- `Success`: 3500
- `Warn`: 8000
- `Error`: 10000

#### Struct: `ToastState`

```rust
#[derive(Clone, Copy)]
pub struct ToastState {
    pub toasts: Signal<Vec<ToastItem>>,
}
```

#### Impl: `ToastState`

```rust
impl ToastState {
    pub fn provide() -> Self {
        let state = Self { toasts: Signal::new(Vec::new()) };
        use_context_provider(|| state);
        state
    }

    pub fn push(&self, input: ToastInput) -> String {
        let tone = input.tone;
        let ttl_ms = input.ttl_ms.unwrap_or_else(|| default_ttl(tone));
        let id = format!("toast_{}_{}", timestamp_ms(), random_suffix());
        let item = ToastItem {
            id: id.clone(),
            title: input.title,
            body: input.body,
            tone,
            ttl_ms,
            action: input.action,
            created_at: timestamp_ms(),
        };
        let mut toasts = self.toasts;
        let mut list = toasts.write();
        list.insert(0, item);
        list.truncate(MAX_TOASTS);
        id
    }

    pub fn dismiss(&self, id: &str) {
        let mut toasts = self.toasts;
        toasts.write().retain(|t| t.id != id);
    }

    pub fn clear(&self) {
        let mut toasts = self.toasts;
        toasts.write().clear();
    }
}
```

Helper functions `timestamp_ms() -> u64` (uses `SystemTime::now().duration_since(UNIX_EPOCH)`) and `random_suffix() -> String` (uses `uuid::Uuid::new_v4().to_string()[..8]`). The `uuid` crate is already a dependency of `lx-desktop`.

`default_ttl(tone: ToastTone) -> u64` matches the constants above.

### Step 3: Port `contexts/dialog.rs`

**Source:** `reference/paperclip/ui/src/context/DialogContext.tsx`

File: `src/contexts/dialog.rs`

#### Struct: `NewIssueDefaults`

```rust
#[derive(Clone, Debug, Default)]
pub struct NewIssueDefaults {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub project_id: Option<String>,
    pub assignee_agent_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}
```

#### Struct: `DialogState`

```rust
#[derive(Clone, Copy)]
pub struct DialogState {
    pub new_issue_open: Signal<bool>,
    pub new_issue_defaults: Signal<NewIssueDefaults>,
    pub new_project_open: Signal<bool>,
    pub new_agent_open: Signal<bool>,
    pub onboarding_open: Signal<bool>,
}
```

#### Impl: `DialogState`

```rust
impl DialogState {
    pub fn provide() -> Self {
        let state = Self {
            new_issue_open: Signal::new(false),
            new_issue_defaults: Signal::new(NewIssueDefaults::default()),
            new_project_open: Signal::new(false),
            new_agent_open: Signal::new(false),
            onboarding_open: Signal::new(false),
        };
        use_context_provider(|| state);
        state
    }

    pub fn open_new_issue(&self, defaults: NewIssueDefaults) {
        let mut d = self.new_issue_defaults;
        d.set(defaults);
        let mut o = self.new_issue_open;
        o.set(true);
    }

    pub fn close_new_issue(&self) {
        let mut o = self.new_issue_open;
        o.set(false);
        let mut d = self.new_issue_defaults;
        d.set(NewIssueDefaults::default());
    }

    pub fn open_new_project(&self) {
        let mut o = self.new_project_open;
        o.set(true);
    }

    pub fn close_new_project(&self) {
        let mut o = self.new_project_open;
        o.set(false);
    }

    pub fn open_new_agent(&self) {
        let mut o = self.new_agent_open;
        o.set(true);
    }

    pub fn close_new_agent(&self) {
        let mut o = self.new_agent_open;
        o.set(false);
    }

    pub fn open_onboarding(&self) {
        let mut o = self.onboarding_open;
        o.set(true);
    }

    pub fn close_onboarding(&self) {
        let mut o = self.onboarding_open;
        o.set(false);
    }
}
```

Note: the Paperclip `NewGoalDefaults` and `OnboardingOptions` are omitted. The Paperclip source has them but they can be added trivially later if the corresponding pages are ported. This is not a deferral -- they are absent from the lx scope because lx has no goals or onboarding wizard pages.

### Step 4: Port `contexts/panel.rs`

**Source:** `reference/paperclip/ui/src/context/PanelContext.tsx`

File: `src/contexts/panel.rs`

#### Struct: `PanelState`

```rust
#[derive(Clone, Copy)]
pub struct PanelState {
    pub visible: Signal<bool>,
    pub content_id: Signal<Option<String>>,
}
```

The React version stores a `ReactNode` as panel content. In Dioxus, we store an `Option<String>` content identifier; the layout component decides what to render based on the identifier.

#### Impl: `PanelState`

```rust
impl PanelState {
    pub fn provide() -> Self {
        let state = Self {
            visible: Signal::new(true),
            content_id: Signal::new(None),
        };
        use_context_provider(|| state);
        state
    }

    pub fn open(&self, id: String) {
        let mut c = self.content_id;
        c.set(Some(id));
    }

    pub fn close(&self) {
        let mut c = self.content_id;
        c.set(None);
    }

    pub fn set_visible(&self, v: bool) {
        let mut vis = self.visible;
        vis.set(v);
    }

    pub fn toggle_visible(&self) {
        let mut vis = self.visible;
        let current = *vis.read();
        vis.set(!current);
    }

    pub fn is_visible(&self) -> bool {
        *self.visible.read()
    }

    pub fn has_content(&self) -> bool {
        self.content_id.read().is_some()
    }
}
```

### Step 5: Port `contexts/sidebar.rs`

**Source:** `reference/paperclip/ui/src/context/SidebarContext.tsx`

File: `src/contexts/sidebar.rs`

#### Struct: `SidebarState`

```rust
#[derive(Clone, Copy)]
pub struct SidebarState {
    pub open: Signal<bool>,
}
```

The Paperclip version tracks `isMobile` via a `matchMedia` listener. In the Dioxus desktop app, there is no mobile breakpoint to track -- the sidebar is always present. The `open` signal controls whether the sidebar is expanded or collapsed.

#### Impl: `SidebarState`

```rust
impl SidebarState {
    pub fn provide() -> Self {
        let state = Self { open: Signal::new(true) };
        use_context_provider(|| state);
        state
    }

    pub fn is_open(&self) -> bool {
        *self.open.read()
    }

    pub fn set_open(&self, v: bool) {
        let mut o = self.open;
        o.set(v);
    }

    pub fn toggle(&self) {
        let mut o = self.open;
        let current = *o.read();
        o.set(!current);
    }
}
```

### Step 6: Port `contexts/breadcrumb.rs`

**Source:** `reference/paperclip/ui/src/context/BreadcrumbContext.tsx`

File: `src/contexts/breadcrumb.rs`

#### Struct: `BreadcrumbEntry`

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct BreadcrumbEntry {
    pub label: String,
    pub href: Option<String>,
}
```

#### Struct: `BreadcrumbState`

```rust
#[derive(Clone, Copy)]
pub struct BreadcrumbState {
    pub crumbs: Signal<Vec<BreadcrumbEntry>>,
}
```

#### Impl: `BreadcrumbState`

```rust
impl BreadcrumbState {
    pub fn provide() -> Self {
        let state = Self { crumbs: Signal::new(Vec::new()) };
        use_context_provider(|| state);
        state
    }

    pub fn set(&self, entries: Vec<BreadcrumbEntry>) {
        let mut c = self.crumbs;
        c.set(entries);
    }

    pub fn clear(&self) {
        let mut c = self.crumbs;
        c.set(Vec::new());
    }

    pub fn entries(&self) -> Vec<BreadcrumbEntry> {
        self.crumbs.read().clone()
    }
}
```

### Step 7: Port `contexts/company.rs`

**Source:** `reference/paperclip/ui/src/context/CompanyContext.tsx`

File: `src/contexts/company.rs`

The Paperclip CompanyContext is deeply tied to react-query and the Paperclip REST API. In `lx-desktop`, there is no Paperclip API. This context provides a local-only company selection mechanism. Uses local signal state. Backend integration is handled by Unit 17's API layer.

#### Struct: `Company`

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub issue_prefix: String,
}
```

#### Struct: `CompanyState`

```rust
#[derive(Clone, Copy)]
pub struct CompanyState {
    pub companies: Signal<Vec<Company>>,
    pub selected_id: Signal<Option<String>>,
}
```

#### Impl: `CompanyState`

```rust
impl CompanyState {
    pub fn provide() -> Self {
        let state = Self {
            companies: Signal::new(Vec::new()),
            selected_id: Signal::new(None),
        };
        use_context_provider(|| state);
        state
    }

    pub fn selected(&self) -> Option<Company> {
        let id = self.selected_id.read().clone();
        let companies = self.companies.read();
        id.and_then(|id| companies.iter().find(|c| c.id == id).cloned())
    }

    pub fn select(&self, company_id: String) {
        let mut s = self.selected_id;
        s.set(Some(company_id));
    }

    pub fn set_companies(&self, list: Vec<Company>) {
        let mut c = self.companies;
        c.set(list);
    }

    pub fn has_companies(&self) -> bool {
        !self.companies.read().is_empty()
    }
}
```

### Step 8: Update `contexts/mod.rs`

Replace the current content with:

```rust
pub mod activity_log;
pub mod breadcrumb;
pub mod company;
pub mod dialog;
pub mod panel;
pub mod sidebar;
pub mod status_bar;
pub mod theme;
pub mod toast;
```

### Step 9: Wire contexts into Shell

**File:** `src/layout/shell.rs`

In the `Shell` component function body, after the existing `ActivityLog::provide()` call, add provider calls for all new contexts:

```rust
let _theme = crate::contexts::theme::ThemeState::provide();
let _toast = crate::contexts::toast::ToastState::provide();
let _dialog = crate::contexts::dialog::DialogState::provide();
let _panel = crate::contexts::panel::PanelState::provide();
let _sidebar_ctx = crate::contexts::sidebar::SidebarState::provide();
let _breadcrumb = crate::contexts::breadcrumb::BreadcrumbState::provide();
let _company = crate::contexts::company::CompanyState::provide();
```

Insert these lines immediately after the line `let _activity_log = ActivityLog::provide();` (line 37 of current shell.rs).

---

## Part B: Route Expansion

### Step 10: Create stub page modules at their real module paths

Instead of a single `pages/stubs.rs`, create individual stub files at the real module paths that later units will replace. Each stub is a single-file module exporting a placeholder component. When a page unit runs, it either replaces that file or converts it to a directory module (deleting the `.rs` file and creating a `mod.rs` in its place).

Create each of the following files with the pattern shown:

**File:** `src/pages/dashboard.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Dashboard (stub)" } }
}

#[component]
pub fn DashboardAlt() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Dashboard (stub)" } }
}
```

**File:** `src/pages/projects.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Projects() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Projects (stub)" } }
}

#[component]
pub fn ProjectDetail(project_id: String) -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Project {project_id} (stub)" } }
}
```

**File:** `src/pages/issues.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Issues() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Issues (stub)" } }
}

#[component]
pub fn IssueDetail(issue_id: String) -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Issue {issue_id} (stub)" } }
}
```

**File:** `src/pages/goals.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Goals() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Goals (stub)" } }
}

#[component]
pub fn GoalDetail(goal_id: String) -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Goal {goal_id} (stub)" } }
}
```

**File:** `src/pages/approvals.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Approvals() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Approvals (stub)" } }
}

#[component]
pub fn ApprovalDetail(approval_id: String) -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Approval {approval_id} (stub)" } }
}
```

**File:** `src/pages/costs.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Costs() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Costs (stub)" } }
}
```

**File:** `src/pages/inbox.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Inbox() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Inbox (stub)" } }
}
```

**File:** `src/pages/routines.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Routines() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Routines (stub)" } }
}

#[component]
pub fn RoutineDetail(routine_id: String) -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Routine {routine_id} (stub)" } }
}
```

**File:** `src/pages/org.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn OrgChart() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Org Chart (stub)" } }
}
```

**File:** `src/pages/company_settings.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn CompanySettings() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Company Settings (stub)" } }
}
```

**File:** `src/pages/instance_settings.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn InstanceSettings() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Instance Settings (stub)" } }
}
```

**File:** `src/pages/onboarding.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn Onboarding() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Onboarding (stub)" } }
}
```

**File:** `src/pages/plugins.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn PluginManager() -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Plugin Manager (stub)" } }
}

#[component]
pub fn PluginPage(plugin_id: String) -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Plugin {plugin_id} (stub)" } }
}

#[component]
pub fn PluginSettingsPage(plugin_id: String) -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Plugin Settings {plugin_id} (stub)" } }
}
```

**File:** `src/pages/not_found.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    rsx! { div { class: "p-4 text-sm text-destructive", "404 -- Page not found" } }
}
```

**File:** `src/pages/agent_detail.rs`
```rust
use dioxus::prelude::*;

#[component]
pub fn AgentDetail(agent_id: String) -> Element {
    rsx! { div { class: "p-4 text-sm text-muted-foreground", "Agent {agent_id} (stub)" } }
}
```

These are temporary scaffolding files. When a page unit creates the real implementation, it deletes the stub file and either replaces it with a new single-file module or creates a directory module at that path.

### Step 11: Update `pages/mod.rs`

Add all stub page modules to the module declarations:

```rust
pub mod accounts;
pub mod activity;
pub mod agent_detail;
pub mod agents;
pub mod approvals;
pub mod companies;
pub mod company_export;
pub mod company_import;
pub mod company_settings;
pub mod company_skills;
pub mod costs;
pub mod dashboard;
pub mod goals;
pub mod inbox;
pub mod instance_settings;
pub mod issues;
pub mod not_found;
pub mod onboarding;
pub mod org;
pub mod plugins;
pub mod projects;
pub mod routines;
pub mod settings;
pub mod tools;
```

### Step 12: Rewrite `routes.rs`

**Source:** `reference/paperclip/ui/src/App.tsx` (the `boardRoutes()` function and top-level Routes)

File: `src/routes.rs`

Replace the entire file content. The new Route enum maps Paperclip's route tree to Dioxus Router's derive-based routing.

Dioxus Router uses the `#[derive(Routable)]` macro with `#[route(...)]` and `#[layout(...)]` attributes. All routes are nested under the `Shell` layout.

The mapping from Paperclip routes to lx-desktop routes:

| Paperclip Route | lx-desktop Route | Component |
|----------------|------------------|-----------|
| `/` (index) | `/` | `Dashboard` (stub) |
| `/dashboard` | `/dashboard` | `Dashboard` (stub) |
| `/agents` | `/agents` | `Agents` (existing) |
| `/agents/:agentId` | `/agents/:agent_id` | `Agents` (existing, detail view) |
| `/projects` | `/projects` | `Projects` (stub) |
| `/projects/:projectId` | `/projects/:project_id` | `ProjectDetail` (stub) |
| `/issues` | `/issues` | `Issues` (stub) |
| `/issues/:issueId` | `/issues/:issue_id` | `IssueDetail` (stub) |
| `/goals` | `/goals` | `Goals` (stub) |
| `/goals/:goalId` | `/goals/:goal_id` | `GoalDetail` (stub) |
| `/approvals` | `/approvals` | `Approvals` (stub) |
| `/approvals/:approvalId` | `/approvals/:approval_id` | `ApprovalDetail` (stub) |
| `/routines` | `/routines` | `Routines` (stub) |
| `/routines/:routineId` | `/routines/:routine_id` | `RoutineDetail` (stub) |
| `/costs` | `/costs` | `Costs` (stub) |
| `/activity` | `/activity` | `Activity` (existing) |
| `/inbox` | `/inbox` | `Inbox` (stub) |
| `/org` | `/org` | `OrgChart` (stub) |
| `/tools` | `/tools` | `Tools` (existing) |
| `/settings` | `/settings` | `Settings` (existing) |
| `/accounts` | `/accounts` | `Accounts` (existing) |
| `/company/settings` | `/company/settings` | `CompanySettings` (stub) |
| `/instance/settings` | `/instance/settings` | `InstanceSettings` (stub) |
| `/companies` | `/companies` | `Companies` (stub) |
| `/company/export` | `/company/export` | `CompanyExport` (stub) |
| `/company/import` | `/company/import` | `CompanyImport` (stub) |
| `/skills` | `/skills` | `CompanySkills` (stub) |
| `/onboarding` | `/onboarding` | `Onboarding` (stub) |
| `/plugins` | `/plugins` | `PluginManager` (stub) |
| `/plugins/:pluginId` | `/plugins/:plugin_id` | `PluginPage` (stub) |
| `/plugins/:pluginId/settings` | `/plugins/:plugin_id/settings` | `PluginSettingsPage` (stub) |

The new `routes.rs` content:

```rust
use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::accounts::Accounts;
use crate::pages::activity::Activity;
use crate::pages::agent_detail::AgentDetail;
use crate::pages::agents::Agents;
use crate::pages::approvals::{ApprovalDetail, Approvals};
use crate::pages::companies::Companies;
use crate::pages::company_export::CompanyExport;
use crate::pages::company_import::CompanyImport;
use crate::pages::company_settings::CompanySettings;
use crate::pages::company_skills::CompanySkills;
use crate::pages::costs::Costs;
use crate::pages::dashboard::{Dashboard, DashboardAlt};
use crate::pages::goals::{GoalDetail, Goals};
use crate::pages::inbox::Inbox;
use crate::pages::instance_settings::InstanceSettings;
use crate::pages::issues::{IssueDetail, Issues};
use crate::pages::not_found::NotFound;
use crate::pages::onboarding::Onboarding;
use crate::pages::org::OrgChart;
use crate::pages::plugins::{PluginManager, PluginPage, PluginSettingsPage};
use crate::pages::projects::{ProjectDetail, Projects};
use crate::pages::routines::{RoutineDetail, Routines};
use crate::pages::settings::Settings;
use crate::pages::tools::Tools;

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Dashboard {},
        #[route("/dashboard")]
        DashboardAlt {},
        #[route("/agents")]
        Agents {},
        #[route("/agents/:agent_id")]
        AgentDetail { agent_id: String },
        #[route("/projects")]
        Projects {},
        #[route("/projects/:project_id")]
        ProjectDetail { project_id: String },
        #[route("/issues")]
        Issues {},
        #[route("/issues/:issue_id")]
        IssueDetail { issue_id: String },
        #[route("/goals")]
        Goals {},
        #[route("/goals/:goal_id")]
        GoalDetail { goal_id: String },
        #[route("/approvals")]
        Approvals {},
        #[route("/approvals/:approval_id")]
        ApprovalDetail { approval_id: String },
        #[route("/routines")]
        Routines {},
        #[route("/routines/:routine_id")]
        RoutineDetail { routine_id: String },
        #[route("/costs")]
        Costs {},
        #[route("/activity")]
        Activity {},
        #[route("/inbox")]
        Inbox {},
        #[route("/org")]
        OrgChart {},
        #[route("/tools")]
        Tools {},
        #[route("/settings")]
        Settings {},
        #[route("/accounts")]
        Accounts {},
        #[route("/company/settings")]
        CompanySettings {},
        #[route("/instance/settings")]
        InstanceSettings {},
        #[route("/companies")]
        Companies {},
        #[route("/company/export")]
        CompanyExport {},
        #[route("/company/import")]
        CompanyImport {},
        #[route("/skills")]
        CompanySkills {},
        #[route("/onboarding")]
        Onboarding {},
        #[route("/plugins")]
        PluginManager {},
        #[route("/plugins/:plugin_id")]
        PluginPage { plugin_id: String },
        #[route("/plugins/:plugin_id/settings")]
        PluginSettingsPage { plugin_id: String },
        #[route("/:..segments")]
        NotFound { segments: Vec<String> },
}
```

**Important naming constraint:** Dioxus Router requires that each enum variant name matches a component function name. All stub components are defined in Step 10's individual stub module files at their real module paths.

**Conflict resolution:** The existing `Agents` page component is at `pages::agents::Agents`. The Dioxus router variant `Agents {}` at `/agents` correctly maps to this. The `AgentDetail` variant at `/agents/:agent_id` maps to `pages::agent_detail::AgentDetail`. There is no naming collision because `Agents` and `AgentDetail` are different names.

**This unit creates the canonical Route enum. No other unit modifies routes.rs.** All subsequent page units (6-18) replace stub module files with real implementations. Because `routes.rs` imports from the real module paths (e.g., `crate::pages::dashboard::Dashboard`), and those paths stay the same when a stub is replaced by a real module, no unit needs to touch `routes.rs` at all.

### Step 13: Verify imports compile

After all modifications, the import chain must be valid:
- `routes.rs` imports each component from its own page module (e.g., `Dashboard` from `pages::dashboard`, `Projects` from `pages::projects`, etc.)
- Each stub module exports the component(s) that the Route enum variant requires
- `Shell` import from `layout::shell` remains unchanged
- When a later unit replaces a stub module with a real implementation, the import path in `routes.rs` stays the same -- no modification to `routes.rs` is needed

---

## Definition of Done

1. `just diagnose` passes with zero errors and zero warnings for the `lx-desktop` crate
2. All 7 new context files exist at the specified paths under `src/contexts/`
3. `src/contexts/mod.rs` declares all 9 modules (2 existing + 7 new)
4. Each context struct has a `provide()` method that calls `use_context_provider`
5. `src/layout/shell.rs` calls `provide()` on all 7 new context types
6. Stub page modules exist at their real module paths (e.g., `src/pages/dashboard.rs`, `src/pages/projects.rs`, etc.) -- 19 stub files total
7. `src/pages/mod.rs` includes all stub module declarations
8. `src/routes.rs` contains the expanded `Route` enum with 32 variants (Dashboard, DashboardAlt, Agents, AgentDetail, Projects, ProjectDetail, Issues, IssueDetail, Goals, GoalDetail, Approvals, ApprovalDetail, Routines, RoutineDetail, Costs, Activity, Inbox, OrgChart, Tools, Settings, Accounts, CompanySettings, InstanceSettings, Onboarding, PluginManager, PluginPage, PluginSettingsPage, NotFound)
9. Every Route variant name matches an importable component function from its corresponding page module
10. No file exceeds 300 lines
11. No `#[allow(...)]` attributes are used
12. No doc comments or code comments are present
13. The existing `activity_log.rs` and `status_bar.rs` context files are unchanged
