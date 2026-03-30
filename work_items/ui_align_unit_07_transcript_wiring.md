# UNIT 7: Wire TranscriptView and RunsTab to ActivityLog Event Data

## Goal

Replace the hardcoded `vec![]` in `TranscriptView` and the empty `Vec::new()` in `AgentDetailShell`'s Runs tab with real data derived from `ActivityLog`. Transcript blocks map to event kinds. Runs are synthesized by grouping events that share an agent_id prefix pattern.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/pages/agents/transcript.rs` | Accept events as prop, convert to TranscriptBlock |
| `crates/lx-desktop/src/pages/agents/detail.rs` | Read ActivityLog, filter events, build HeartbeatRun list, pass to RunsTab and TranscriptView |

## Reference Files (read-only)

| File | Why |
|------|-----|
| `crates/lx-desktop/src/pages/agents/runs_tab.rs` | RunsTab component, takes `runs: Vec<HeartbeatRun>` |
| `crates/lx-desktop/src/pages/agents/run_types.rs` | HeartbeatRun struct definition |
| `crates/lx-desktop/src/pages/agents/live_run_widget.rs` | LiveRunWidget uses TranscriptView |
| `crates/lx-desktop/src/pages/agents/run_detail.rs` | RunDetailPanel uses TranscriptView |
| `crates/lx-desktop/src/contexts/activity_log.rs` | ActivityLog context, `ActivityEvent { timestamp, kind, message }` |
| `crates/lx-api/src/types.rs` | ActivityEvent definition |

---

## Current State

### `transcript.rs` lines 12-18
```rust
#[component]
pub fn TranscriptView(run_id: String) -> Element {
  let entries: Vec<TranscriptBlock> = vec![
    TranscriptBlock::Message { role: "system".into(), text: format!("Run {run_id} transcript"), ts: "00:00".into() },
    TranscriptBlock::Thinking { text: "Analyzing...".into(), ts: "00:01".into() },
    TranscriptBlock::ToolUse { name: "example".into(), input_summary: "...".into(), result: None, is_error: false, ts: "00:02".into() },
    TranscriptBlock::Event { label: "done".into(), text: "Complete".into(), tone: "success".into(), ts: "00:03".into() },
  ];
```

### `detail.rs` lines 88-89
```rust
          AgentDetailTab::Runs => rsx! {
            RunsTab { runs: Vec::new(), agent_route_id: agent.id.clone() }
```

---

## Step 1: Rewrite `TranscriptView` to accept events as a prop

In `crates/lx-desktop/src/pages/agents/transcript.rs`:

### Change 1a: Add import for ActivityEvent

Old text (lines 1-1):
```rust
use dioxus::prelude::*;
```

New text:
```rust
use dioxus::prelude::*;
use lx_api::types::ActivityEvent;
```

### Change 1b: Add mapping function and rewrite TranscriptView signature

Old text (lines 11-18):
```rust
#[component]
pub fn TranscriptView(run_id: String) -> Element {
  let entries: Vec<TranscriptBlock> = vec![
    TranscriptBlock::Message { role: "system".into(), text: format!("Run {run_id} transcript"), ts: "00:00".into() },
    TranscriptBlock::Thinking { text: "Analyzing...".into(), ts: "00:01".into() },
    TranscriptBlock::ToolUse { name: "example".into(), input_summary: "...".into(), result: None, is_error: false, ts: "00:02".into() },
    TranscriptBlock::Event { label: "done".into(), text: "Complete".into(), tone: "success".into(), ts: "00:03".into() },
  ];
```

New text:
```rust
fn event_to_block(event: &ActivityEvent) -> TranscriptBlock {
  match event.kind.as_str() {
    "log" | "agent_message" | "message" => TranscriptBlock::Message {
      role: "assistant".into(),
      text: event.message.clone(),
      ts: event.timestamp.clone(),
    },
    "thinking" => TranscriptBlock::Thinking {
      text: event.message.clone(),
      ts: event.timestamp.clone(),
    },
    k if k.contains("tool") => TranscriptBlock::ToolUse {
      name: event.kind.clone(),
      input_summary: event.message.clone(),
      result: None,
      is_error: false,
      ts: event.timestamp.clone(),
    },
    k if k.contains("error") => TranscriptBlock::Event {
      label: "error".into(),
      text: event.message.clone(),
      tone: "error".into(),
      ts: event.timestamp.clone(),
    },
    _ => TranscriptBlock::Event {
      label: event.kind.clone(),
      text: event.message.clone(),
      tone: "info".into(),
      ts: event.timestamp.clone(),
    },
  }
}

#[component]
pub fn TranscriptView(run_id: String, #[props(optional)] events: Option<Vec<ActivityEvent>>) -> Element {
  let entries: Vec<TranscriptBlock> = match events {
    Some(evts) => evts.iter().map(event_to_block).collect(),
    None => vec![],
  };
```

**Mapping rules:**
- `kind` is `"log"`, `"agent_message"`, or `"message"` -> `TranscriptBlock::Message` with role `"assistant"`
- `kind` is `"thinking"` -> `TranscriptBlock::Thinking`
- `kind` contains `"tool"` (e.g. `"tool_call"`, `"tool_result"`) -> `TranscriptBlock::ToolUse`
- `kind` contains `"error"` -> `TranscriptBlock::Event` with tone `"error"`
- All other kinds -> `TranscriptBlock::Event` with tone `"info"`

The `events` prop is `Option<Vec<ActivityEvent>>` so existing call sites (`run_detail.rs` line 37, `live_run_widget.rs` line 83) that pass only `run_id` continue to compile and render an empty transcript. Those files are unchanged in this unit.

---

## Step 2: Wire `detail.rs` to read ActivityLog and pass data to RunsTab

In `crates/lx-desktop/src/pages/agents/detail.rs`:

### Change 2a: Add imports

Old text (lines 1-10):
```rust
use super::budget_tab::BudgetTab;
use super::config_form::AgentConfigPanel;
use super::list::StatusBadge;
use super::overview::AgentOverview;
use super::run_types::{BudgetSummary, SkillSnapshot};
use super::runs_tab::RunsTab;
use super::skills_tab::SkillsTab;
use super::types::{AgentDetail as AgentDetailData, AgentDetailTab, role_label};
use crate::styles::{BTN_OUTLINE_SM, TAB_ACTIVE, TAB_INACTIVE};
use dioxus::prelude::*;
```

New text:
```rust
use super::budget_tab::BudgetTab;
use super::config_form::AgentConfigPanel;
use super::list::StatusBadge;
use super::overview::AgentOverview;
use super::run_types::{BudgetSummary, HeartbeatRun, SkillSnapshot};
use super::runs_tab::RunsTab;
use super::skills_tab::SkillsTab;
use super::types::{AgentDetail as AgentDetailData, AgentDetailTab, role_label};
use crate::contexts::activity_log::ActivityLog;
use crate::styles::{BTN_OUTLINE_SM, TAB_ACTIVE, TAB_INACTIVE};
use dioxus::prelude::*;
```

### Change 2b: Read ActivityLog and build runs inside AgentDetailShell

Old text (lines 21-22):
```rust
  let mut active_tab = use_signal(|| AgentDetailTab::Overview);

  let role_text = role_label(&agent.role);
```

New text:
```rust
  let mut active_tab = use_signal(|| AgentDetailTab::Overview);

  let log = use_context::<ActivityLog>();
  let all_events = log.events.read();

  let agent_events: Vec<_> = all_events
    .iter()
    .filter(|e| e.message.contains(&agent.name) || e.kind.contains("agent"))
    .cloned()
    .collect();

  let runs: Vec<HeartbeatRun> = {
    let mut run_list = Vec::new();
    for event in agent_events.iter() {
      if event.kind == "agent_start" || event.kind == "agent_running" {
        let status = if event.kind == "agent_running" { "running" } else { "queued" };
        let already = run_list.iter().any(|r: &HeartbeatRun| r.id == event.timestamp);
        if !already {
          run_list.push(HeartbeatRun {
            id: event.timestamp.clone(),
            agent_id: agent.id.clone(),
            company_id: String::new(),
            status: status.to_string(),
            invocation_source: "on_demand".to_string(),
            trigger_detail: None,
            started_at: Some(event.timestamp.clone()),
            finished_at: None,
            created_at: event.timestamp.clone(),
            error: None,
            error_code: None,
            usage_json: None,
            result_json: None,
            context_snapshot: None,
          });
        }
      }
    }
    run_list
  };

  let role_text = role_label(&agent.role);
```

### Change 2c: Pass runs to RunsTab

Old text (lines 88-89):
```rust
          AgentDetailTab::Runs => rsx! {
            RunsTab { runs: Vec::new(), agent_route_id: agent.id.clone() }
```

New text:
```rust
          AgentDetailTab::Runs => rsx! {
            RunsTab { runs: runs.clone(), agent_route_id: agent.id.clone() }
```

---

## Step 3: Verify no call-site breakage

The `TranscriptView` signature change adds `#[props(optional)] events: Option<Vec<ActivityEvent>>`. Existing call sites that pass only `run_id`:

- `crates/lx-desktop/src/pages/agents/run_detail.rs` line 37: `TranscriptView { run_id: run.id.clone() }` -- compiles, events defaults to None.
- `crates/lx-desktop/src/pages/agents/live_run_widget.rs` line 83: `TranscriptView { run_id: run.id.clone() }` -- compiles, events defaults to None.

No changes needed in those files.

---

## Verification

After all changes:
- `transcript.rs` is ~110 lines (under 300).
- `detail.rs` is ~175 lines (under 300).
- No code comments or docstrings.
- No `#[allow(...)]` macros.
- TranscriptView with no events prop renders the empty state message.
- TranscriptView with events prop renders real transcript blocks mapped from ActivityEvent kinds.
- RunsTab receives synthesized HeartbeatRun entries from agent-related events.
- Existing call sites in `run_detail.rs` and `live_run_widget.rs` continue to compile unchanged.
