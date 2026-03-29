# Unit 8: Agent Detail (Part 2 -- Runs, Skills, Budget, Permissions)

## Scope

Complete the remaining AgentDetail tabs: Runs tab with run list and transcript viewer, Skills tab with skill management, Budget tab with budget policy card, and a Permissions section under Config. Also create a LiveRunWidget component for embedding in issue detail.

## Preconditions

- Unit 7 is complete: `crates/lx-desktop/src/pages/agents/` contains `types.rs`, `list.rs`, `detail.rs`, `overview.rs`, `config_form.rs`, `new_agent.rs`, `icon_picker.rs`, and `mod.rs`.
- `detail.rs` has placeholder text for Runs, Skills, and Budget tabs.
- `types.rs` contains `AgentDetail`, `AgentDetailTab`, and helper functions.
- `styles.rs` contains the style constants from Unit 7.

## Paperclip Source Files to Reference

| Paperclip File | What to Extract |
|---|---|
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 2809-2887 | `RunsTab`: sorted run list, side-by-side desktop layout (list + detail), mobile layout |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 2889-3010 | `RunDetail`: hydrated run view with metrics, status, cancel/retry/resume buttons, session info, transcript |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 100-107 | `runStatusIcons` mapping: succeeded/failed/running/queued/timed_out/cancelled |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 164-169 | `sourceLabels` mapping: timer/assignment/on_demand/automation |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 253-273 | `runMetrics` function: extract input/output/cached tokens and cost from run |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 2349-2808 | `AgentSkillsTab`: skill snapshot, optional/required/unmanaged skill rows, toggle checkboxes, autosave |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 1041-1053 | Budget tab: renders `BudgetPolicyCard` |
| `reference/paperclip/ui/src/pages/AgentDetail.tsx` lines 1506-1594 | Permissions section: canCreateAgents toggle, canAssignTasks toggle |
| `reference/paperclip/ui/src/components/BudgetPolicyCard.tsx` | Budget card: observed/budget display, progress bar, editable budget input, save |
| `reference/paperclip/ui/src/components/transcript/RunTranscriptView.tsx` | Transcript viewer: TranscriptBlock types, message/tool/thinking/command blocks, collapsible tool results |
| `reference/paperclip/ui/src/components/LiveRunWidget.tsx` | Live run widget: polls live runs for issue, renders transcript per run with cancel button |
| `reference/paperclip/ui/src/components/transcript/useLiveRunTranscripts.ts` | Live transcript polling logic |

## Step 1: Create Run Data Types in a New File

To keep files under 300 lines, create the run-related types in a new file `crates/lx-desktop/src/pages/agents/run_types.rs` (NOT appended to `types.rs`, which is already ~200 lines from Unit 7).

Create `crates/lx-desktop/src/pages/agents/run_types.rs` with these types:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatRun {
    pub id: String,
    pub agent_id: String,
    pub company_id: String,
    pub status: String,
    pub invocation_source: String,
    pub trigger_detail: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
    pub error: Option<String>,
    pub error_code: Option<String>,
    pub usage_json: Option<serde_json::Value>,
    pub result_json: Option<serde_json::Value>,
    pub context_snapshot: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillEntry {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub detail: Option<String>,
    pub required: bool,
    pub location_label: Option<String>,
    pub origin_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillSnapshot {
    pub entries: Vec<SkillEntry>,
    pub desired_skills: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BudgetSummary {
    pub amount: i64,
    pub observed_amount: i64,
    pub remaining_amount: i64,
    pub utilization_percent: f64,
    pub warn_percent: u32,
    pub hard_stop_enabled: bool,
    pub status: String,
    pub is_active: bool,
}

pub struct RunMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub cost_usd: f64,
    pub total_tokens: u64,
}

pub fn run_metrics(run: &HeartbeatRun) -> RunMetrics {
    let usage = run.usage_json.as_ref();
    let result = run.result_json.as_ref();

    fn get_u64(val: Option<&serde_json::Value>, keys: &[&str]) -> u64 {
        let Some(obj) = val.and_then(|v| v.as_object()) else { return 0 };
        for key in keys {
            if let Some(n) = obj.get(*key).and_then(|v| v.as_u64()) {
                return n;
            }
        }
        0
    }

    fn get_f64(val: Option<&serde_json::Value>, keys: &[&str]) -> f64 {
        let Some(obj) = val.and_then(|v| v.as_object()) else { return 0.0 };
        for key in keys {
            if let Some(n) = obj.get(*key).and_then(|v| v.as_f64()) {
                return n;
            }
        }
        0.0
    }

    let input = get_u64(usage, &["inputTokens", "input_tokens"]);
    let output = get_u64(usage, &["outputTokens", "output_tokens"]);
    let cached = get_u64(usage, &["cachedInputTokens", "cached_input_tokens", "cache_read_input_tokens"]);
    let cost = get_f64(usage, &["totalCostUsd", "total_cost_usd"])
        .max(get_f64(result, &["totalCostUsd", "total_cost_usd"]));

    RunMetrics {
        input_tokens: input,
        output_tokens: output,
        cached_tokens: cached,
        cost_usd: cost,
        total_tokens: input + output,
    }
}

pub fn source_label(source: &str) -> &str {
    match source {
        "timer" => "Timer",
        "assignment" => "Assignment",
        "on_demand" => "On-demand",
        "automation" => "Automation",
        other => other,
    }
}

pub fn run_status_class(status: &str) -> &'static str {
    match status {
        "succeeded" => "text-green-600",
        "failed" => "text-red-600",
        "running" => "text-cyan-600",
        "queued" => "text-yellow-600",
        "timed_out" => "text-orange-600",
        "cancelled" => "text-neutral-500",
        _ => "text-neutral-400",
    }
}

pub fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}
```

## Step 2: Create Runs Tab (split into two files for 300-line compliance)

Create `crates/lx-desktop/src/pages/agents/runs_tab.rs` (~150 lines) containing the run list and selection logic, and `crates/lx-desktop/src/pages/agents/run_detail.rs` (~170 lines) containing the run detail panel with metrics.

### runs_tab.rs

```rust
use dioxus::prelude::*;
use super::run_types::{HeartbeatRun, RunMetrics, run_metrics, run_status_class, source_label, format_tokens};
use super::list::StatusBadge;
use super::run_detail::RunDetailPanel;

#[component]
pub fn RunsTab(
    runs: Vec<HeartbeatRun>,
    agent_route_id: String,
) -> Element {
    let mut selected_run_id = use_signal(|| Option::<String>::None);

    if runs.is_empty() {
        return rsx! {
            p { class: "text-sm text-[var(--outline)]", "No runs yet." }
        };
    }

    let mut sorted = runs.clone();
    sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let effective_id = selected_run_id.read().clone().or_else(|| sorted.first().map(|r| r.id.clone()));
    let selected_run = effective_id.as_ref().and_then(|id| sorted.iter().find(|r| &r.id == id));

    rsx! {
        div { class: "flex gap-0",
            // Run list
            div { class: "shrink-0 border border-[var(--outline-variant)]/30 rounded-lg w-72 overflow-y-auto",
                style: "max-height: calc(100vh - 2rem);",
                for run in sorted.iter() {
                    RunListItem {
                        run: run.clone(),
                        is_selected: effective_id.as_ref() == Some(&run.id),
                        on_select: {
                            let id = run.id.clone();
                            move |_| selected_run_id.set(Some(id.clone()))
                        },
                    }
                }
            }
            // Run detail
            if let Some(run) = selected_run {
                div { class: "flex-1 min-w-0 pl-4",
                    RunDetailPanel { run: run.clone() }
                }
            }
        }
    }
}

#[component]
fn RunListItem(
    run: HeartbeatRun,
    is_selected: bool,
    on_select: EventHandler<()>,
) -> Element {
    let is_live = run.status == "running" || run.status == "queued";
    let short_id = &run.id[..8.min(run.id.len())];
    let bg = if is_selected { "bg-[var(--surface-container-high)]" } else { "" };

    rsx! {
        button {
            class: "flex items-center gap-2 w-full px-3 py-2.5 text-left border-b border-[var(--outline-variant)]/15 hover:bg-[var(--surface-container)] transition-colors {bg}",
            onclick: move |_| on_select.call(()),
            if is_live {
                span { class: "relative flex h-2 w-2 shrink-0",
                    span { class: "animate-pulse absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75" }
                    span { class: "relative inline-flex rounded-full h-2 w-2 bg-cyan-400" }
                }
            }
            div { class: "flex-1 min-w-0",
                span { class: "text-xs font-mono text-[var(--on-surface)]", "{short_id}" }
                span { class: "text-xs text-[var(--outline)] ml-2",
                    "{source_label(&run.invocation_source)}"
                }
            }
            StatusBadge { status: run.status.clone() }
        }
    }
}

```

### run_detail.rs

The `RunDetailPanel`, `RunMetricsGrid`, and `MetricCell` components go in `run_detail.rs`:

```rust
use dioxus::prelude::*;
use super::run_types::{HeartbeatRun, RunMetrics, run_metrics, run_status_class, source_label, format_tokens};
use super::list::StatusBadge;
use super::transcript::TranscriptView;

#[component]
pub fn RunDetailPanel(run: HeartbeatRun) -> Element {
    let metrics = run_metrics(&run);
    let short_id = &run.id[..8.min(run.id.len())];
    let is_live = run.status == "running" || run.status == "queued";

    rsx! {
        div { class: "space-y-6",
            // Header
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-2",
                    if is_live {
                        span { class: "relative flex h-2 w-2",
                            span { class: "animate-pulse absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75" }
                            span { class: "relative inline-flex rounded-full h-2 w-2 bg-cyan-400" }
                        }
                    }
                    span { class: "text-sm font-mono text-[var(--on-surface)]", "{short_id}" }
                    StatusBadge { status: run.status.clone() }
                    span { class: "text-xs text-[var(--outline)]",
                        "{source_label(&run.invocation_source)}"
                    }
                }
                span { class: "text-xs text-[var(--outline)]", "{run.created_at}" }
            }
            // Metrics
            RunMetricsGrid { metrics }
            // Error
            if let Some(err) = &run.error {
                div { class: "border border-red-500/20 bg-red-500/10 rounded-lg p-3",
                    p { class: "text-xs text-red-600", "{err}" }
                }
            }
            // Transcript placeholder
            TranscriptView { run_id: run.id.clone() }
        }
    }
}

#[component]
fn RunMetricsGrid(metrics: RunMetrics) -> Element {
    let cost_str = if metrics.cost_usd > 0.0 {
        format!("${:.4}", metrics.cost_usd)
    } else {
        "-".to_string()
    };
    rsx! {
        div { class: "border border-[var(--outline-variant)]/30 rounded-lg p-4",
            div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                MetricCell { label: "Input tokens", value: format_tokens(metrics.input_tokens) }
                MetricCell { label: "Output tokens", value: format_tokens(metrics.output_tokens) }
                MetricCell { label: "Cached tokens", value: format_tokens(metrics.cached_tokens) }
                MetricCell { label: "Cost", value: cost_str }
            }
        }
    }
}

#[component]
fn MetricCell(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            span { class: "text-xs text-[var(--outline)] block", "{label}" }
            span { class: "text-lg font-semibold text-[var(--on-surface)] tabular-nums",
                "{value}"
            }
        }
    }
}
```

## Step 3: Create Transcript View

Create `crates/lx-desktop/src/pages/agents/transcript.rs`:

A simplified transcript viewer component. Reference `RunTranscriptView.tsx`.

```rust
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum TranscriptBlock {
    Message { role: String, text: String, ts: String },
    Thinking { text: String, ts: String },
    ToolUse { name: String, input_summary: String, result: Option<String>, is_error: bool, ts: String },
    Event { label: String, text: String, tone: String, ts: String },
}

#[component]
pub fn TranscriptView(run_id: String) -> Element {
    let entries: Vec<TranscriptBlock> = Vec::new();

    if entries.is_empty() {
        return rsx! {
            div { class: "border border-[var(--outline-variant)]/30 rounded-lg p-4",
                p { class: "text-sm text-[var(--outline)] text-center",
                    "No transcript data available."
                }
            }
        };
    }

    rsx! {
        div { class: "space-y-2",
            for entry in entries.iter() {
                TranscriptBlockView { block: entry.clone() }
            }
        }
    }
}

#[component]
fn TranscriptBlockView(block: TranscriptBlock) -> Element {
    match block {
        TranscriptBlock::Message { role, text, .. } => {
            let icon = if role == "assistant" { "smart_toy" } else { "person" };
            let bg = if role == "assistant" {
                "bg-[var(--surface-container)]"
            } else {
                "bg-[var(--surface-container-high)]"
            };
            rsx! {
                div { class: "flex gap-3 p-3 rounded-lg {bg}",
                    span { class: "material-symbols-outlined text-sm text-[var(--outline)] shrink-0 mt-0.5",
                        "{icon}"
                    }
                    div { class: "flex-1 min-w-0 text-sm text-[var(--on-surface)] whitespace-pre-wrap break-words",
                        "{text}"
                    }
                }
            }
        }
        TranscriptBlock::Thinking { text, .. } => {
            rsx! {
                div { class: "flex gap-3 p-3 rounded-lg bg-amber-500/5 border border-amber-500/10",
                    span { class: "material-symbols-outlined text-sm text-amber-600 shrink-0 mt-0.5",
                        "psychology"
                    }
                    div { class: "flex-1 min-w-0 text-xs text-[var(--outline)] italic whitespace-pre-wrap",
                        "{text}"
                    }
                }
            }
        }
        TranscriptBlock::ToolUse { name, input_summary, result, is_error, .. } => {
            let border = if is_error { "border-red-500/20" } else { "border-[var(--outline-variant)]/20" };
            rsx! {
                div { class: "border {border} rounded-lg p-3 space-y-2",
                    div { class: "flex items-center gap-2",
                        span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                            "build"
                        }
                        span { class: "text-xs font-medium text-[var(--on-surface)]", "{name}" }
                    }
                    if !input_summary.is_empty() {
                        p { class: "text-xs text-[var(--outline)] font-mono truncate",
                            "{input_summary}"
                        }
                    }
                    if let Some(res) = result {
                        div { class: "text-xs font-mono whitespace-pre-wrap max-h-32 overflow-y-auto p-2 bg-[var(--surface-container)] rounded",
                            "{res}"
                        }
                    }
                }
            }
        }
        TranscriptBlock::Event { label, text, tone, .. } => {
            let color = match tone.as_str() {
                "error" => "text-red-600",
                "warn" => "text-amber-600",
                "info" => "text-cyan-600",
                _ => "text-[var(--outline)]",
            };
            rsx! {
                div { class: "flex items-center gap-2 py-1",
                    span { class: "text-[10px] font-semibold uppercase tracking-wider {color}",
                        "{label}"
                    }
                    span { class: "text-xs text-[var(--outline)]", "{text}" }
                }
            }
        }
    }
}
```

## Step 4: Create Skills Tab

Create `crates/lx-desktop/src/pages/agents/skills_tab.rs`:

Skill management tab with optional/required skill lists and toggles. Reference `AgentDetail.tsx` lines 2349-2808.

```rust
use dioxus::prelude::*;
use super::run_types::{SkillEntry, SkillSnapshot};

#[component]
pub fn SkillsTab(snapshot: SkillSnapshot) -> Element {
    let mut desired = use_signal(|| snapshot.desired_skills.clone());
    let required: Vec<&SkillEntry> = snapshot.entries.iter().filter(|e| e.required).collect();
    let optional: Vec<&SkillEntry> = snapshot.entries.iter().filter(|e| !e.required).collect();

    rsx! {
        div { class: "max-w-3xl space-y-6",
            // Required skills
            if !required.is_empty() {
                SkillSection {
                    title: "Required Skills",
                    description: "These skills are always enabled for this agent.",
                    skills: required.iter().map(|e| (*e).clone()).collect(),
                    desired_keys: desired.read().clone(),
                    read_only: true,
                    on_toggle: move |_: String| {},
                }
            }
            // Optional skills
            if !optional.is_empty() {
                SkillSection {
                    title: "Optional Skills",
                    description: "Toggle skills on or off for this agent.",
                    skills: optional.iter().map(|e| (*e).clone()).collect(),
                    desired_keys: desired.read().clone(),
                    read_only: false,
                    on_toggle: move |key: String| {
                        let mut current = desired.read().clone();
                        if current.contains(&key) {
                            current.retain(|k| k != &key);
                        } else {
                            current.push(key);
                        }
                        desired.set(current);
                    },
                }
            }
            if required.is_empty() && optional.is_empty() {
                p { class: "text-sm text-[var(--outline)]",
                    "No skills configured."
                }
            }
        }
    }
}

#[component]
fn SkillSection(
    title: &'static str,
    description: &'static str,
    skills: Vec<SkillEntry>,
    desired_keys: Vec<String>,
    read_only: bool,
    on_toggle: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "space-y-3",
            div {
                h3 { class: "text-sm font-medium text-[var(--on-surface)]", "{title}" }
                p { class: "text-xs text-[var(--outline)] mt-1", "{description}" }
            }
            div { class: "border border-[var(--outline-variant)]/30 rounded-lg divide-y divide-[var(--outline-variant)]/15",
                for skill in skills.iter() {
                    SkillRow {
                        skill: skill.clone(),
                        checked: skill.required || desired_keys.contains(&skill.key),
                        read_only: read_only || skill.required,
                        on_toggle: {
                            let key = skill.key.clone();
                            move |_| on_toggle.call(key.clone())
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn SkillRow(
    skill: SkillEntry,
    checked: bool,
    read_only: bool,
    on_toggle: EventHandler<()>,
) -> Element {
    let opacity = if read_only { "opacity-60" } else { "" };
    rsx! {
        button {
            class: "flex items-start gap-3 w-full px-4 py-3 text-left hover:bg-[var(--surface-container)] transition-colors {opacity}",
            disabled: read_only,
            onclick: move |_| on_toggle.call(()),
            div { class: "flex items-center justify-center h-4 w-4 shrink-0 mt-0.5 border border-[var(--outline-variant)] rounded-sm",
                class: if checked { "bg-[var(--primary)]" } else { "" },
                if checked {
                    span { class: "text-[10px] text-[var(--on-primary)] leading-none", "✓" }
                }
            }
            div { class: "flex-1 min-w-0",
                span { class: "text-sm font-medium text-[var(--on-surface)]", "{skill.name}" }
                if let Some(desc) = &skill.description {
                    p { class: "text-xs text-[var(--outline)] mt-0.5", "{desc}" }
                }
                if let Some(detail) = &skill.detail {
                    p { class: "text-xs text-[var(--outline)] font-mono mt-0.5", "{detail}" }
                }
            }
        }
    }
}
```

## Step 5: Create Budget Tab

Create `crates/lx-desktop/src/pages/agents/budget_tab.rs`:

Budget policy display and editing. Reference `BudgetPolicyCard.tsx`.

```rust
use dioxus::prelude::*;
use super::run_types::BudgetSummary;
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};

#[component]
pub fn BudgetTab(summary: BudgetSummary, on_save: EventHandler<i64>) -> Element {
    let mut draft_dollars = use_signal(|| format!("{:.2}", summary.amount as f64 / 100.0));
    let parsed = parse_dollar_input(&draft_dollars.read());
    let can_save = parsed.is_some() && parsed != Some(summary.amount);

    let progress = if summary.amount > 0 {
        (summary.utilization_percent as f64).min(100.0)
    } else {
        0.0
    };
    let status_tone = match summary.status.as_str() {
        "hard_stop" => "text-red-400 border-red-500/30 bg-red-500/10",
        "warning" => "text-amber-300 border-amber-500/30 bg-amber-500/10",
        _ => "text-emerald-300 border-emerald-500/30 bg-emerald-500/10",
    };

    rsx! {
        div { class: "max-w-3xl space-y-6",
            // Status badge
            div { class: "inline-flex items-center gap-2 border rounded-full px-3 py-1 text-xs font-medium {status_tone}",
                "{summary.status}"
            }
            // Observed vs Budget grid
            div { class: "grid gap-6 sm:grid-cols-2",
                div {
                    div { class: "text-[11px] uppercase tracking-widest text-[var(--outline)]",
                        "Observed"
                    }
                    div { class: "mt-2 text-xl font-semibold text-[var(--on-surface)] tabular-nums",
                        "{format_cents(summary.observed_amount)}"
                    }
                    div { class: "mt-1 text-xs text-[var(--outline)]",
                        if summary.amount > 0 {
                            "{summary.utilization_percent:.0}% of limit"
                        } else {
                            "No cap configured"
                        }
                    }
                }
                div {
                    div { class: "text-[11px] uppercase tracking-widest text-[var(--outline)]",
                        "Budget"
                    }
                    div { class: "mt-2 text-xl font-semibold text-[var(--on-surface)] tabular-nums",
                        if summary.amount > 0 {
                            "{format_cents(summary.amount)}"
                        } else {
                            "Disabled"
                        }
                    }
                    div { class: "mt-1 text-xs text-[var(--outline)]",
                        "Soft alert at {summary.warn_percent}%"
                    }
                }
            }
            // Progress bar
            if summary.amount > 0 {
                div { class: "w-full bg-[var(--surface-container-high)] rounded-full h-2",
                    div {
                        class: "h-2 rounded-full transition-all bg-[var(--primary)]",
                        style: "width: {progress}%",
                    }
                }
            }
            // Edit budget
            div { class: "space-y-3",
                h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Set Monthly Budget" }
                div { class: "flex items-center gap-2",
                    span { class: "text-sm text-[var(--outline)]", "$" }
                    input {
                        class: INPUT_FIELD,
                        r#type: "number",
                        step: "0.01",
                        min: "0",
                        placeholder: "0.00",
                        value: "{draft_dollars}",
                        oninput: move |evt| draft_dollars.set(evt.value().to_string()),
                    }
                    if can_save {
                        button {
                            class: BTN_PRIMARY_SM,
                            onclick: move |_| {
                                if let Some(cents) = parse_dollar_input(&draft_dollars.read()) {
                                    on_save.call(cents);
                                }
                            },
                            "Save"
                        }
                    }
                }
            }
        }
    }
}

fn format_cents(cents: i64) -> String {
    if cents == 0 {
        "$0.00".to_string()
    } else {
        format!("${:.2}", cents as f64 / 100.0)
    }
}

fn parse_dollar_input(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Some(0);
    }
    let parsed: f64 = trimmed.parse().ok()?;
    if parsed < 0.0 || !parsed.is_finite() {
        return None;
    }
    Some((parsed * 100.0).round() as i64)
}
```

## Step 6: Create Live Run Widget

Create `crates/lx-desktop/src/pages/agents/live_run_widget.rs`:

Embeddable widget showing live runs for an issue with inline transcript. Reference `LiveRunWidget.tsx`.

```rust
use dioxus::prelude::*;
use super::run_types::{source_label};
use super::list::StatusBadge;
use super::transcript::TranscriptView;

#[derive(Clone, Debug, PartialEq)]
pub struct LiveRunInfo {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub status: String,
    pub invocation_source: String,
    pub started_at: Option<String>,
    pub created_at: String,
}

#[component]
pub fn LiveRunWidget(
    runs: Vec<LiveRunInfo>,
    on_cancel: EventHandler<String>,
    on_open_run: EventHandler<(String, String)>,
) -> Element {
    if runs.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "overflow-hidden rounded-xl border border-cyan-500/25 bg-[var(--surface-container)]/80 shadow-lg",
            div { class: "border-b border-[var(--outline-variant)]/60 bg-cyan-500/[0.04] px-4 py-3",
                div { class: "text-xs font-semibold uppercase tracking-widest text-cyan-400",
                    "Live Runs"
                }
            }
            div { class: "divide-y divide-[var(--outline-variant)]/60",
                for run in runs.iter() {
                    LiveRunEntry {
                        run: run.clone(),
                        on_cancel: {
                            let id = run.id.clone();
                            move |_| on_cancel.call(id.clone())
                        },
                        on_open: {
                            let agent_id = run.agent_id.clone();
                            let run_id = run.id.clone();
                            move |_| on_open_run.call((agent_id.clone(), run_id.clone()))
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn LiveRunEntry(
    run: LiveRunInfo,
    on_cancel: EventHandler<()>,
    on_open: EventHandler<()>,
) -> Element {
    let is_active = run.status == "running" || run.status == "queued";
    let short_id = &run.id[..8.min(run.id.len())];

    rsx! {
        section { class: "px-4 py-4",
            div { class: "mb-3 flex items-start justify-between",
                div { class: "min-w-0",
                    span { class: "text-sm font-medium text-[var(--on-surface)]",
                        "{run.agent_name}"
                    }
                    div { class: "mt-2 flex items-center gap-2 text-xs text-[var(--outline)]",
                        span { class: "font-mono", "{short_id}" }
                        StatusBadge { status: run.status.clone() }
                        span { "{source_label(&run.invocation_source)}" }
                    }
                }
                div { class: "flex items-center gap-2",
                    if is_active {
                        button {
                            class: "inline-flex items-center gap-1 rounded-full border border-red-500/20 bg-red-500/[0.06] px-2.5 py-1 text-[11px] font-medium text-red-400 hover:bg-red-500/[0.12] transition-colors",
                            onclick: move |_| on_cancel.call(()),
                            "Stop"
                        }
                    }
                    button {
                        class: "inline-flex items-center gap-1 rounded-full border border-[var(--outline-variant)]/70 bg-[var(--surface-container)]/70 px-2.5 py-1 text-[11px] font-medium text-cyan-400 hover:border-cyan-500/30 transition-colors",
                        onclick: move |_| on_open.call(()),
                        "Open run"
                    }
                }
            }
            div { class: "max-h-80 overflow-y-auto pr-1",
                TranscriptView { run_id: run.id.clone() }
            }
        }
    }
}
```

## Step 7: Update Agent Detail Shell to Wire Tabs

Edit `crates/lx-desktop/src/pages/agents/detail.rs`:

Replace the three placeholder tab branches (`Runs`, `Skills`, `Budget`) with the actual components. Change these imports at the top:

Add imports:
```rust
use super::runs_tab::RunsTab;
use super::skills_tab::SkillsTab;
use super::budget_tab::BudgetTab;
use super::run_types::{HeartbeatRun, SkillSnapshot, BudgetSummary};
```

Replace the `match` arm for `AgentDetailTab::Runs`:
```rust
AgentDetailTab::Runs => rsx! {
    RunsTab {
        runs: Vec::new(),
        agent_route_id: agent.id.clone(),
    }
},
```

Replace the `match` arm for `AgentDetailTab::Skills`:
```rust
AgentDetailTab::Skills => rsx! {
    SkillsTab {
        snapshot: SkillSnapshot {
            entries: Vec::new(),
            desired_skills: Vec::new(),
        },
    }
},
```

Replace the `match` arm for `AgentDetailTab::Budget`:
```rust
AgentDetailTab::Budget => rsx! {
    BudgetTab {
        summary: BudgetSummary {
            amount: agent.budget_monthly_cents,
            observed_amount: agent.spent_monthly_cents,
            remaining_amount: (agent.budget_monthly_cents - agent.spent_monthly_cents).max(0),
            utilization_percent: if agent.budget_monthly_cents > 0 {
                agent.spent_monthly_cents as f64 / agent.budget_monthly_cents as f64 * 100.0
            } else { 0.0 },
            warn_percent: 80,
            hard_stop_enabled: true,
            status: "ok".to_string(),
            is_active: agent.budget_monthly_cents > 0,
        },
        on_save: move |_cents: i64| {},
    }
},
```

## Step 8: Update Module Root

Edit `crates/lx-desktop/src/pages/agents/mod.rs` to add the new modules:

Add these lines after the existing module declarations:
```rust
mod budget_tab;
mod live_run_widget;
mod run_detail;
pub mod run_types;
mod runs_tab;
mod skills_tab;
mod transcript;
```

## Definition of Done

1. Seven new files exist under `crates/lx-desktop/src/pages/agents/`: `run_types.rs`, `runs_tab.rs`, `run_detail.rs`, `transcript.rs`, `skills_tab.rs`, `budget_tab.rs`, `live_run_widget.rs`.
2. `run_types.rs` contains `HeartbeatRun`, `SkillEntry`, `SkillSnapshot`, `BudgetSummary`, `RunMetrics`, and helper functions (`run_metrics`, `source_label`, `run_status_class`, `format_tokens`). This keeps `types.rs` under 300 lines.
3. `detail.rs` imports and renders `RunsTab`, `SkillsTab`, and `BudgetTab` for their respective tab variants.
4. `mod.rs` declares all new modules.
5. No file exceeds 300 lines.
6. `just diagnose` passes (no compiler errors, no clippy warnings).
7. The Runs tab renders a side-by-side list + detail panel with metrics and transcript placeholder.
8. The Skills tab renders required and optional skill rows with checkboxes.
9. The Budget tab renders observed/budget stats, a progress bar, and an editable budget input with save button.
10. `LiveRunWidget` is a self-contained component that can be embedded in Issue Detail (Unit 9).
