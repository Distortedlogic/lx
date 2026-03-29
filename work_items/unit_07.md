# Unit 7: Agent List & Agent Detail (Part 1 -- Structure & Config)

## Scope

Port the Paperclip Agents page, AgentDetail shell with tab navigation (Overview, Runs, Config, Skills, Budget), the Overview tab content, the config form, the new-agent dialog, and the agent icon picker into Dioxus 0.7.3 components under `crates/lx-desktop/src/pages/agents/`.

The existing voice-based agent page (`mod.rs`, `pane_area.rs`, `voice_banner.rs`, `voice_context.rs`, `voice_pipeline.rs`, `voice_porcupine.rs`) is replaced entirely. No backward compatibility needed.

## Preconditions

- **Unit 3 is complete:** The Route enum in routes.rs already has `Agents {}` and `AgentDetail { agent_id: String }` variants (as stubs). This unit replaces the stubs with real implementations by creating the real `pages::agents` module. Do NOT modify routes.rs.
- Dioxus 0.7.3 is the desktop framework in `crates/lx-desktop`.
- The route enum lives at `crates/lx-desktop/src/routes.rs`. Unit 3's route imports reference `crate::pages::agent_detail::AgentDetail` (a stub module); creating the real agent detail module replaces the stub.
- Page modules are registered in `crates/lx-desktop/src/pages/mod.rs`.
- CSS class constants live in `crates/lx-desktop/src/styles.rs`.
- The 300-line file limit applies to every file.

## Paperclip Source Files to Reference

| Paperclip File | What to Extract |
|---|---|
| `reference/paperclip/ui/src/pages/Agents.tsx` | Agent list page: filter tabs (all/active/paused/error), list view with EntityRow, org-tree view, live-run indicator, "New Agent" button |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 518-1053 | AgentDetail shell: header with icon picker + name + role, tab bar (Dashboard/Instructions/Skills/Configuration/Runs/Budget), action buttons (Run/Pause/Resume/Terminate), overflow menu |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 1055-1310 | AgentOverview: LatestRunCard, charts placeholder, recent issues list, CostsSection |
| `reference/paperclip/ui/src/pages/NewAgent.tsx` | New agent full-page form: name, title, role picker, reports-to picker, config form, skills selection, submit |
| `reference/paperclip/ui/src/components/AgentConfigForm.tsx` | Config form: adapter type selector, model picker, heartbeat toggle, env vars, working directory, timeout fields |
| `reference/paperclip/ui/src/components/AgentActionButtons.tsx` | RunButton and PauseResumeButton components |
| `reference/paperclip/ui/src/components/AgentProperties.tsx` | PropertyRow helper, agent property display (status, role, title, adapter, session, last heartbeat, reports-to, created) |
| `reference/paperclip/ui/src/components/AgentIconPicker.tsx` | AgentIcon display component, AgentIconPicker popover with search and icon grid |
| `reference/paperclip/ui/src/components/NewAgentDialog.tsx` | Dialog with two views: "Ask CEO" recommendation vs advanced adapter-type grid |

## Step 1: Delete Old Voice-Based Agent Files

Delete these files (they are fully replaced):

- `crates/lx-desktop/src/pages/agents/pane_area.rs`
- `crates/lx-desktop/src/pages/agents/voice_banner.rs`
- `crates/lx-desktop/src/pages/agents/voice_context.rs`
- `crates/lx-desktop/src/pages/agents/voice_pipeline.rs`
- `crates/lx-desktop/src/pages/agents/voice_porcupine.rs`

## Step 2: Add Style Constants

In `crates/lx-desktop/src/styles.rs`, add these constants (append, do not remove existing):

```rust
pub const STATUS_DOT_ACTIVE: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-green-500";
pub const STATUS_DOT_PAUSED: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-yellow-500";
pub const STATUS_DOT_ERROR: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-red-500";
pub const STATUS_DOT_DEFAULT: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-neutral-400";
pub const CARD_BORDER: &str = "border border-[var(--outline-variant)]/30";
pub const TAB_ACTIVE: &str = "text-sm font-medium text-[var(--on-surface)] border-b-2 border-[var(--primary)] pb-2 px-3";
pub const TAB_INACTIVE: &str = "text-sm text-[var(--outline)] pb-2 px-3 hover:text-[var(--on-surface)] transition-colors";
pub const PROPERTY_LABEL: &str = "text-xs text-[var(--outline)] shrink-0 w-20";
pub const PROPERTY_VALUE: &str = "text-sm text-[var(--on-surface)]";
pub const BTN_OUTLINE_SM: &str = "border border-[var(--outline-variant)] text-[var(--on-surface)] rounded px-3 py-1.5 text-xs hover:bg-[var(--surface-container-high)] transition-colors";
pub const BTN_PRIMARY_SM: &str = "bg-[var(--primary)] text-[var(--on-primary)] rounded px-3 py-1.5 text-xs font-semibold hover:brightness-110 transition-all";
pub const INPUT_FIELD: &str = "w-full rounded border border-[var(--outline-variant)] px-2.5 py-1.5 bg-transparent outline-none text-sm font-mono placeholder:text-[var(--outline)]/40";
```

## Step 3: Create Data Types Module

Create `crates/lx-desktop/src/pages/agents/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub role: String,
    pub title: Option<String>,
    pub status: String,
    pub adapter_type: String,
    pub icon: Option<String>,
    pub last_heartbeat_at: Option<String>,
    pub reports_to: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDetail {
    pub id: String,
    pub name: String,
    pub role: String,
    pub title: Option<String>,
    pub status: String,
    pub adapter_type: String,
    pub icon: Option<String>,
    pub last_heartbeat_at: Option<String>,
    pub reports_to: Option<String>,
    pub created_at: String,
    pub budget_monthly_cents: i64,
    pub spent_monthly_cents: i64,
    pub adapter_config: serde_json::Value,
    pub runtime_config: serde_json::Value,
    pub pause_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterTab {
    All,
    Active,
    Paused,
    Error,
}

impl FilterTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Paused => "Paused",
            Self::Error => "Error",
        }
    }

    pub fn matches(&self, status: &str) -> bool {
        match self {
            Self::All => status != "terminated",
            Self::Active => matches!(status, "active" | "running" | "idle"),
            Self::Paused => status == "paused",
            Self::Error => status == "error",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentDetailTab {
    Overview,
    Runs,
    Config,
    Skills,
    Budget,
}

impl AgentDetailTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Runs => "Runs",
            Self::Config => "Configuration",
            Self::Skills => "Skills",
            Self::Budget => "Budget",
        }
    }

    pub fn all() -> &'static [AgentDetailTab] {
        &[
            Self::Overview,
            Self::Runs,
            Self::Config,
            Self::Skills,
            Self::Budget,
        ]
    }
}

pub const ADAPTER_LABELS: &[(&str, &str)] = &[
    ("claude_local", "Claude"),
    ("codex_local", "Codex"),
    ("gemini_local", "Gemini"),
    ("opencode_local", "OpenCode"),
    ("cursor", "Cursor"),
    ("hermes_local", "Hermes"),
    ("openclaw_gateway", "OpenClaw Gateway"),
    ("process", "Process"),
    ("http", "HTTP"),
];

pub fn adapter_label(adapter_type: &str) -> &str {
    ADAPTER_LABELS
        .iter()
        .find(|(k, _)| *k == adapter_type)
        .map(|(_, v)| *v)
        .unwrap_or(adapter_type)
}

pub const ROLE_LABELS: &[(&str, &str)] = &[
    ("ceo", "CEO"),
    ("executive", "Executive"),
    ("manager", "Manager"),
    ("general", "General"),
    ("specialist", "Specialist"),
];

pub fn role_label(role: &str) -> &str {
    ROLE_LABELS
        .iter()
        .find(|(k, _)| *k == role)
        .map(|(_, v)| *v)
        .unwrap_or(role)
}

pub fn status_dot_class(status: &str) -> &'static str {
    match status {
        "active" | "running" | "idle" => "inline-flex h-2.5 w-2.5 rounded-full bg-green-500",
        "paused" => "inline-flex h-2.5 w-2.5 rounded-full bg-yellow-500",
        "error" => "inline-flex h-2.5 w-2.5 rounded-full bg-red-500",
        _ => "inline-flex h-2.5 w-2.5 rounded-full bg-neutral-400",
    }
}
```

## Step 4: Create Agent List Page

Create `crates/lx-desktop/src/pages/agents/list.rs`:

This component renders the agent list page with filter tabs and a flat list of agent cards. Reference `Agents.tsx` lines 66-311.

```rust
use dioxus::prelude::*;
use super::types::{AgentSummary, FilterTab, adapter_label, role_label, status_dot_class};
use crate::styles::{PAGE_HEADING, FLEX_BETWEEN, BTN_OUTLINE_SM, TAB_ACTIVE, TAB_INACTIVE};

#[component]
pub fn AgentList(
    agents: Vec<AgentSummary>,
    on_select: EventHandler<String>,
    on_new_agent: EventHandler<()>,
) -> Element {
    let mut active_tab = use_signal(|| FilterTab::All);
    let filtered: Vec<&AgentSummary> = agents
        .iter()
        .filter(|a| active_tab.read().matches(&a.status))
        .collect();

    rsx! {
        div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
            div { class: FLEX_BETWEEN,
                div { class: "flex gap-1",
                    for tab in [FilterTab::All, FilterTab::Active, FilterTab::Paused, FilterTab::Error] {
                        button {
                            class: if *active_tab.read() == tab { TAB_ACTIVE } else { TAB_INACTIVE },
                            onclick: {
                                let tab = tab.clone();
                                move |_| active_tab.set(tab.clone())
                            },
                            "{tab.label()}"
                        }
                    }
                }
                button {
                    class: BTN_OUTLINE_SM,
                    onclick: move |_| on_new_agent.call(()),
                    "+ New Agent"
                }
            }
            if filtered.is_empty() {
                div { class: "flex-1 flex items-center justify-center",
                    p { class: "text-sm text-[var(--outline)]",
                        "No agents match this filter."
                    }
                }
            }
            p { class: "text-xs text-[var(--outline)]",
                "{filtered.len()} agent{}", if filtered.len() != 1 { "s" } else { "" }
            }
            div { class: "border border-[var(--outline-variant)]/30 overflow-hidden",
                for agent in filtered.iter() {
                    AgentRow {
                        agent: (*agent).clone(),
                        on_click: {
                            let id = agent.id.clone();
                            move |_| on_select.call(id.clone())
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn AgentRow(agent: AgentSummary, on_click: EventHandler<()>) -> Element {
    let subtitle = {
        let role = role_label(&agent.role);
        match &agent.title {
            Some(t) => format!("{role} - {t}"),
            None => role.to_string(),
        }
    };
    let adapter = adapter_label(&agent.adapter_type);

    rsx! {
        button {
            class: "flex items-center gap-3 px-3 py-2.5 w-full text-left border-b border-[var(--outline-variant)]/15 hover:bg-[var(--surface-container)] transition-colors",
            onclick: move |_| on_click.call(()),
            span { class: "{status_dot_class(&agent.status)}" }
            div { class: "flex-1 min-w-0",
                span { class: "text-sm font-medium text-[var(--on-surface)]",
                    "{agent.name}"
                }
                span { class: "text-xs text-[var(--outline)] ml-2", "{subtitle}" }
            }
            span { class: "text-xs text-[var(--outline)] font-mono w-14 text-right",
                "{adapter}"
            }
            StatusBadge { status: agent.status.clone() }
        }
    }
}

#[component]
pub fn StatusBadge(status: String) -> Element {
    let (bg, text) = match status.as_str() {
        "active" | "running" | "idle" => ("bg-green-500/10 text-green-600", "Active"),
        "paused" => ("bg-yellow-500/10 text-yellow-600", "Paused"),
        "error" => ("bg-red-500/10 text-red-600", "Error"),
        "terminated" => ("bg-neutral-500/10 text-neutral-500", "Terminated"),
        "pending_approval" => ("bg-amber-500/10 text-amber-600", "Pending"),
        other => ("bg-neutral-500/10 text-neutral-400", other),
    };
    let label = text.to_string();
    rsx! {
        span { class: "inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium {bg}",
            "{label}"
        }
    }
}
```

## Step 5: Create Agent Detail Shell

Create `crates/lx-desktop/src/pages/agents/detail.rs`:

This is the AgentDetail shell with header, icon, tab bar, and action buttons. It delegates to tab content components. Reference `AgentDetail.tsx` lines 518-1053.

```rust
use dioxus::prelude::*;
use super::types::{AgentDetail as AgentDetailData, AgentDetailTab, role_label};
use super::overview::AgentOverview;
use super::config_form::AgentConfigPanel;
use super::list::StatusBadge;
use crate::styles::{BTN_OUTLINE_SM, TAB_ACTIVE, TAB_INACTIVE};

#[component]
pub fn AgentDetailShell(
    agent: AgentDetailData,
    on_back: EventHandler<()>,
    on_run: EventHandler<()>,
    on_pause: EventHandler<()>,
    on_resume: EventHandler<()>,
    on_terminate: EventHandler<()>,
) -> Element {
    let mut active_tab = use_signal(|| AgentDetailTab::Overview);

    let role_text = role_label(&agent.role);
    let subtitle = match &agent.title {
        Some(t) => format!("{role_text} - {t}"),
        None => role_text.to_string(),
    };
    let is_paused = agent.status == "paused";

    rsx! {
        div { class: "flex flex-col h-full p-4 overflow-auto gap-6",
            // Header
            div { class: "flex items-center justify-between gap-2",
                div { class: "flex items-center gap-3 min-w-0",
                    button {
                        class: "shrink-0 text-xs text-[var(--outline)] hover:text-[var(--on-surface)]",
                        onclick: move |_| on_back.call(()),
                        "< Back"
                    }
                    AgentIconDisplay { icon: agent.icon.clone() }
                    div { class: "min-w-0",
                        h2 { class: "text-2xl font-bold text-[var(--on-surface)] truncate",
                            "{agent.name}"
                        }
                        p { class: "text-sm text-[var(--outline)] truncate", "{subtitle}" }
                    }
                }
                div { class: "flex items-center gap-2 shrink-0",
                    button {
                        class: BTN_OUTLINE_SM,
                        onclick: move |_| on_run.call(()),
                        "Run"
                    }
                    if is_paused {
                        button {
                            class: BTN_OUTLINE_SM,
                            onclick: move |_| on_resume.call(()),
                            "Resume"
                        }
                    } else {
                        button {
                            class: BTN_OUTLINE_SM,
                            onclick: move |_| on_pause.call(()),
                            "Pause"
                        }
                    }
                    StatusBadge { status: agent.status.clone() }
                }
            }
            // Tab bar
            div { class: "flex gap-1 border-b border-[var(--outline-variant)]/30",
                for tab in AgentDetailTab::all() {
                    button {
                        class: if *active_tab.read() == *tab { TAB_ACTIVE } else { TAB_INACTIVE },
                        onclick: {
                            let tab = tab.clone();
                            move |_| active_tab.set(tab.clone())
                        },
                        "{tab.label()}"
                    }
                }
            }
            // Tab content
            match *active_tab.read() {
                AgentDetailTab::Overview => rsx! {
                    AgentOverview { agent: agent.clone() }
                },
                AgentDetailTab::Config => rsx! {
                    AgentConfigPanel { agent: agent.clone() }
                },
                AgentDetailTab::Runs => rsx! {
                    p { class: "text-sm text-[var(--outline)]", "Runs tab (Unit 8)" }
                },
                AgentDetailTab::Skills => rsx! {
                    p { class: "text-sm text-[var(--outline)]", "Skills tab (Unit 8)" }
                },
                AgentDetailTab::Budget => rsx! {
                    p { class: "text-sm text-[var(--outline)]", "Budget tab (Unit 8)" }
                },
            }
        }
    }
}

#[component]
fn AgentIconDisplay(icon: Option<String>) -> Element {
    let icon_char = icon.as_deref().unwrap_or("smart_toy");
    rsx! {
        div { class: "shrink-0 flex items-center justify-center h-12 w-12 rounded-lg bg-[var(--surface-container-high)]",
            span { class: "material-symbols-outlined text-xl", "{icon_char}" }
        }
    }
}
```

## Step 6: Create Agent Overview Tab

Create `crates/lx-desktop/src/pages/agents/overview.rs`:

Renders the Overview/Dashboard tab content. Reference `AgentDetail.tsx` lines 1055-1310 (AgentOverview, LatestRunCard, CostsSection, SummaryRow).

```rust
use dioxus::prelude::*;
use super::types::{AgentDetail, adapter_label, role_label};
use crate::styles::PROPERTY_LABEL;

#[component]
pub fn AgentOverview(agent: AgentDetail) -> Element {
    rsx! {
        div { class: "space-y-8",
            // Properties
            AgentPropertiesPanel { agent: agent.clone() }
            // Latest run placeholder
            div { class: "space-y-3",
                h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Latest Run" }
                p { class: "text-sm text-[var(--outline)]",
                    "No runs yet."
                }
            }
            // Costs section
            div { class: "space-y-3",
                h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Costs" }
                CostsGrid {
                    budget_cents: agent.budget_monthly_cents,
                    spent_cents: agent.spent_monthly_cents,
                }
            }
        }
    }
}

#[component]
fn AgentPropertiesPanel(agent: AgentDetail) -> Element {
    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-1",
                PropertyRow { label: "Status",
                    span { class: "text-sm text-[var(--on-surface)]", "{agent.status}" }
                }
                PropertyRow { label: "Role",
                    span { class: "text-sm text-[var(--on-surface)]",
                        "{role_label(&agent.role)}"
                    }
                }
                if let Some(title) = &agent.title {
                    PropertyRow { label: "Title",
                        span { class: "text-sm text-[var(--on-surface)]", "{title}" }
                    }
                }
                PropertyRow { label: "Adapter",
                    span { class: "text-sm font-mono text-[var(--on-surface)]",
                        "{adapter_label(&agent.adapter_type)}"
                    }
                }
                if let Some(hb) = &agent.last_heartbeat_at {
                    PropertyRow { label: "Heartbeat",
                        span { class: "text-sm text-[var(--on-surface)]", "{hb}" }
                    }
                }
                PropertyRow { label: "Created",
                    span { class: "text-sm text-[var(--on-surface)]", "{agent.created_at}" }
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
            div { class: "flex items-center gap-1.5 min-w-0", {children} }
        }
    }
}

#[component]
fn CostsGrid(budget_cents: i64, spent_cents: i64) -> Element {
    let budget_str = format_cents(budget_cents);
    let spent_str = format_cents(spent_cents);
    let pct = if budget_cents > 0 {
        format!("{}%", (spent_cents as f64 / budget_cents as f64 * 100.0) as i64)
    } else {
        "No cap".to_string()
    };

    rsx! {
        div { class: "border border-[var(--outline-variant)]/30 rounded-lg p-4",
            div { class: "grid grid-cols-2 gap-4",
                div {
                    span { class: "text-xs text-[var(--outline)] block", "Spent" }
                    span { class: "text-lg font-semibold text-[var(--on-surface)]",
                        "{spent_str}"
                    }
                    span { class: "text-xs text-[var(--outline)] block", "{pct} of limit" }
                }
                div {
                    span { class: "text-xs text-[var(--outline)] block", "Budget" }
                    span { class: "text-lg font-semibold text-[var(--on-surface)]",
                        "{budget_str}"
                    }
                }
            }
        }
    }
}

fn format_cents(cents: i64) -> String {
    if cents == 0 {
        return "Disabled".to_string();
    }
    format!("${:.2}", cents as f64 / 100.0)
}
```

## Step 7: Create Agent Config Form Panel

Create `crates/lx-desktop/src/pages/agents/config_form.rs`:

A simplified config form for editing agent configuration. Reference `AgentConfigForm.tsx` lines 170-400, `AgentActionButtons.tsx`.

```rust
use dioxus::prelude::*;
use super::types::{AgentDetail, ADAPTER_LABELS};
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};

#[component]
pub fn AgentConfigPanel(agent: AgentDetail) -> Element {
    let mut adapter_type = use_signal(|| agent.adapter_type.clone());
    let mut model = use_signal(|| {
        agent.adapter_config.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    });
    let mut heartbeat_enabled = use_signal(|| {
        agent.runtime_config.get("heartbeat")
            .and_then(|v| v.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });
    let mut interval_sec = use_signal(|| {
        agent.runtime_config.get("heartbeat")
            .and_then(|v| v.get("intervalSec"))
            .and_then(|v| v.as_u64())
            .unwrap_or(300) as u32
    });
    let mut dirty = use_signal(|| false);

    rsx! {
        div { class: "max-w-3xl space-y-6",
            // Adapter type
            ConfigSection { title: "Adapter",
                div { class: "space-y-3",
                    label { class: "text-xs text-[var(--outline)] block", "Adapter type" }
                    select {
                        class: INPUT_FIELD,
                        value: "{adapter_type}",
                        onchange: move |evt| {
                            adapter_type.set(evt.value().to_string());
                            dirty.set(true);
                        },
                        for (key, label) in ADAPTER_LABELS {
                            option { value: *key, "{label}" }
                        }
                    }
                    label { class: "text-xs text-[var(--outline)] block", "Model" }
                    input {
                        class: INPUT_FIELD,
                        value: "{model}",
                        placeholder: "e.g. claude-sonnet-4-20250514",
                        oninput: move |evt| {
                            model.set(evt.value().to_string());
                            dirty.set(true);
                        },
                    }
                }
            }
            // Heartbeat
            ConfigSection { title: "Heartbeat",
                div { class: "space-y-3",
                    div { class: "flex items-center justify-between",
                        span { class: "text-sm text-[var(--on-surface)]", "Enabled" }
                        ToggleSwitch {
                            checked: *heartbeat_enabled.read(),
                            on_toggle: move |v: bool| {
                                heartbeat_enabled.set(v);
                                dirty.set(true);
                            },
                        }
                    }
                    if *heartbeat_enabled.read() {
                        div {
                            label { class: "text-xs text-[var(--outline)] block mb-1",
                                "Interval (seconds)"
                            }
                            input {
                                class: INPUT_FIELD,
                                r#type: "number",
                                value: "{interval_sec}",
                                oninput: move |evt| {
                                    if let Ok(v) = evt.value().parse::<u32>() {
                                        interval_sec.set(v);
                                        dirty.set(true);
                                    }
                                },
                            }
                        }
                    }
                }
            }
            // Save / Cancel
            if *dirty.read() {
                div { class: "flex items-center justify-end gap-2 pt-4 border-t border-[var(--outline-variant)]/30",
                    button {
                        class: BTN_OUTLINE_SM,
                        onclick: move |_| dirty.set(false),
                        "Cancel"
                    }
                    button {
                        class: BTN_PRIMARY_SM,
                        onclick: move |_| dirty.set(false),
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
fn ConfigSection(title: &'static str, children: Element) -> Element {
    rsx! {
        div { class: "border border-[var(--outline-variant)]/30 rounded-lg",
            div { class: "px-4 py-3 border-b border-[var(--outline-variant)]/30",
                h3 { class: "text-sm font-medium text-[var(--on-surface)]", "{title}" }
            }
            div { class: "px-4 py-4", {children} }
        }
    }
}

#[component]
fn ToggleSwitch(checked: bool, on_toggle: EventHandler<bool>) -> Element {
    let bg = if checked { "bg-green-600" } else { "bg-[var(--outline-variant)]" };
    let translate = if checked { "translate-x-4" } else { "translate-x-0.5" };
    rsx! {
        button {
            class: "relative inline-flex h-5 w-9 items-center rounded-full transition-colors shrink-0 {bg}",
            onclick: move |_| on_toggle.call(!checked),
            span {
                class: "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform {translate}",
            }
        }
    }
}
```

## Step 8: Create New Agent Dialog

Create `crates/lx-desktop/src/pages/agents/new_agent.rs`:

A dialog component for creating a new agent. Reference `NewAgentDialog.tsx` and `NewAgent.tsx`.

```rust
use dioxus::prelude::*;
use super::types::ADAPTER_LABELS;
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};

#[component]
pub fn NewAgentDialog(
    open: bool,
    on_close: EventHandler<()>,
    on_create: EventHandler<NewAgentPayload>,
) -> Element {
    let mut name = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut role = use_signal(|| "general".to_string());
    let mut adapter_type = use_signal(|| "claude_local".to_string());
    let mut show_advanced = use_signal(|| false);

    if !open {
        return rsx! {};
    }

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg w-full max-w-md overflow-hidden",
                onclick: move |evt| evt.stop_propagation(),
                // Header
                div { class: "flex items-center justify-between px-4 py-2.5 border-b border-[var(--outline-variant)]",
                    span { class: "text-sm text-[var(--outline)]", "New Agent" }
                    button {
                        class: "text-[var(--outline)] hover:text-[var(--on-surface)] text-lg",
                        onclick: move |_| on_close.call(()),
                        "x"
                    }
                }
                div { class: "p-6 space-y-4",
                    if !*show_advanced.read() {
                        // Simple mode
                        div { class: "space-y-4",
                            input {
                                class: INPUT_FIELD,
                                placeholder: "Agent name",
                                value: "{name}",
                                oninput: move |evt| name.set(evt.value().to_string()),
                            }
                            input {
                                class: INPUT_FIELD,
                                placeholder: "Title (e.g. VP of Engineering)",
                                value: "{title}",
                                oninput: move |evt| title.set(evt.value().to_string()),
                            }
                            div {
                                label { class: "text-xs text-[var(--outline)] block mb-1", "Role" }
                                select {
                                    class: INPUT_FIELD,
                                    value: "{role}",
                                    onchange: move |evt| role.set(evt.value().to_string()),
                                    option { value: "ceo", "CEO" }
                                    option { value: "executive", "Executive" }
                                    option { value: "manager", "Manager" }
                                    option { value: "general", "General" }
                                    option { value: "specialist", "Specialist" }
                                }
                            }
                            div {
                                label { class: "text-xs text-[var(--outline)] block mb-1",
                                    "Adapter"
                                }
                                select {
                                    class: INPUT_FIELD,
                                    value: "{adapter_type}",
                                    onchange: move |evt| adapter_type.set(evt.value().to_string()),
                                    for (key, label) in ADAPTER_LABELS {
                                        option { value: *key, "{label}" }
                                    }
                                }
                            }
                        }
                        button {
                            class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)] underline",
                            onclick: move |_| show_advanced.set(true),
                            "Show advanced options"
                        }
                    } else {
                        button {
                            class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)]",
                            onclick: move |_| show_advanced.set(false),
                            "< Back"
                        }
                        p { class: "text-sm text-[var(--outline)]",
                            "Advanced configuration available after creation."
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
                        disabled: name.read().trim().is_empty(),
                        onclick: {
                            let name = name.clone();
                            let title = title.clone();
                            let role = role.clone();
                            let adapter_type = adapter_type.clone();
                            move |_| {
                                on_create.call(NewAgentPayload {
                                    name: name.read().trim().to_string(),
                                    title: {
                                        let t = title.read().trim().to_string();
                                        if t.is_empty() { None } else { Some(t) }
                                    },
                                    role: role.read().clone(),
                                    adapter_type: adapter_type.read().clone(),
                                });
                            }
                        },
                        "Create Agent"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewAgentPayload {
    pub name: String,
    pub title: Option<String>,
    pub role: String,
    pub adapter_type: String,
}
```

## Step 9: Create Icon Picker Component

Create `crates/lx-desktop/src/pages/agents/icon_picker.rs`:

A popover-style icon picker using Material Symbols icon names. Reference `AgentIconPicker.tsx`.

```rust
use dioxus::prelude::*;

const AGENT_ICONS: &[&str] = &[
    "smart_toy", "psychology", "engineering", "terminal", "code",
    "bug_report", "build", "science", "analytics", "security",
    "support_agent", "manage_accounts", "group", "school", "lightbulb",
    "auto_fix_high", "memory", "hub", "device_hub", "dns",
];

#[component]
pub fn AgentIconPicker(
    value: Option<String>,
    on_change: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let mut search = use_signal(String::new);
    let current = value.as_deref().unwrap_or("smart_toy");

    let filtered: Vec<&&str> = AGENT_ICONS
        .iter()
        .filter(|name| {
            let q = search.read();
            q.is_empty() || name.contains(q.as_str())
        })
        .collect();

    rsx! {
        div { class: "relative",
            button {
                class: "shrink-0 flex items-center justify-center h-12 w-12 rounded-lg bg-[var(--surface-container-high)] hover:bg-[var(--surface-container)] transition-colors",
                onclick: move |_| open.set(!*open.read()),
                span { class: "material-symbols-outlined text-xl", "{current}" }
            }
            if *open.read() {
                div { class: "absolute top-full left-0 mt-1 z-50 w-72 border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-lg rounded-lg p-3",
                    input {
                        class: "w-full rounded border border-[var(--outline-variant)] px-2 py-1.5 bg-transparent text-sm mb-2 outline-none placeholder:text-[var(--outline)]/40",
                        placeholder: "Search icons...",
                        value: "{search}",
                        oninput: move |evt| search.set(evt.value().to_string()),
                    }
                    div { class: "grid grid-cols-7 gap-1 max-h-48 overflow-y-auto",
                        for icon_name in filtered.iter() {
                            button {
                                class: {
                                    let selected = **icon_name == current;
                                    if selected {
                                        "flex items-center justify-center h-8 w-8 rounded bg-[var(--primary)]/20 ring-1 ring-[var(--primary)]"
                                    } else {
                                        "flex items-center justify-center h-8 w-8 rounded hover:bg-[var(--surface-container-high)] transition-colors"
                                    }
                                },
                                onclick: {
                                    let name = icon_name.to_string();
                                    move |_| {
                                        on_change.call(name.clone());
                                        open.set(false);
                                        search.set(String::new());
                                    }
                                },
                                span { class: "material-symbols-outlined text-base",
                                    "{icon_name}"
                                }
                            }
                        }
                        if filtered.is_empty() {
                            p { class: "col-span-7 text-xs text-[var(--outline)] text-center py-2",
                                "No icons match"
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## Step 10: Rewrite Module Root and Update Routes

Rewrite `crates/lx-desktop/src/pages/agents/mod.rs`:

```rust
mod config_form;
mod detail;
mod icon_picker;
mod list;
mod new_agent;
mod overview;
pub mod types;

use dioxus::prelude::*;
use self::detail::AgentDetailShell;
use self::list::AgentList;
use self::new_agent::{NewAgentDialog, NewAgentPayload};
use self::types::{AgentDetail, AgentSummary};

#[component]
pub fn Agents() -> Element {
    let mut selected_agent_id = use_signal(|| Option::<String>::None);
    let mut show_new_dialog = use_signal(|| false);
    let agents: Vec<AgentSummary> = Vec::new();

    let selected_detail: Option<AgentDetail> = selected_agent_id
        .read()
        .as_ref()
        .and_then(|_id| None);

    rsx! {
        match selected_detail {
            Some(agent) => rsx! {
                AgentDetailShell {
                    agent,
                    on_back: move |_| selected_agent_id.set(None),
                    on_run: move |_| {},
                    on_pause: move |_| {},
                    on_resume: move |_| {},
                    on_terminate: move |_| {},
                }
            },
            None => rsx! {
                AgentList {
                    agents,
                    on_select: move |id: String| selected_agent_id.set(Some(id)),
                    on_new_agent: move |_| show_new_dialog.set(true),
                }
            },
        }
        NewAgentDialog {
            open: *show_new_dialog.read(),
            on_close: move |_| show_new_dialog.set(false),
            on_create: move |_payload: NewAgentPayload| {
                show_new_dialog.set(false);
            },
        }
    }
}
```

No changes needed to `crates/lx-desktop/src/routes.rs` because the `Agents` component is already the `/` route and the import path (`crate::pages::agents::Agents`) remains valid.

No changes needed to `crates/lx-desktop/src/pages/mod.rs` because `pub mod agents;` already exists.

## Definition of Done

1. All old voice files are deleted: `pane_area.rs`, `voice_banner.rs`, `voice_context.rs`, `voice_pipeline.rs`, `voice_porcupine.rs`.
2. Seven new files exist under `crates/lx-desktop/src/pages/agents/`: `types.rs`, `list.rs`, `detail.rs`, `overview.rs`, `config_form.rs`, `new_agent.rs`, `icon_picker.rs`.
3. `mod.rs` is rewritten to import and compose the new modules.
4. `styles.rs` contains the new style constants.
5. No file exceeds 300 lines.
6. `just diagnose` passes (no compiler errors, no clippy warnings).
7. The app renders the agent list view at `/` with filter tabs, agent rows, and a "New Agent" button.
8. Clicking an agent row switches to the detail shell with tab navigation.
9. The New Agent dialog opens and accepts name/title/role/adapter inputs.
