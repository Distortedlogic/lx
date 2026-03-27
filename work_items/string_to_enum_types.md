# Goal

Replace stringly-typed fields with proper enums across four locations: `RunStatus.status`, `PendingPrompt.kind`, event filter options in `lx-mobile`, and backend names in `lx-cli` manifests. Each string field is currently matched against a fixed set of known values. Converting them to enums gives compile-time exhaustiveness checking and eliminates silent mismatches from typos.

# Task List

### Task 1: Define `RunState` enum and replace `RunStatus.status: String`

**Subject:** Replace `RunStatus.status` string field with `RunState` enum

**Description:**

In `crates/lx-api/src/types.rs`, add the following enum **above** the `RunStatus` struct. Find the exact string `pub struct RunStatus {` and insert this enum immediately before the `#[derive(...)]` line preceding it:

```rust
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    #[default]
    Idle,
    Running,
    Completed,
    Failed,
    Waiting,
}
```

Then change the `RunStatus` struct's `status` field. Find the exact string:

```rust
  pub status: String,
```

inside the `RunStatus` struct and replace with:

```rust
  pub status: RunState,
```

Then update the sole consumer in `crates/lx-mobile/src/pages/status.rs`. Find the exact string:

```rust
    let state = match status.status.as_str() {
      "running" => ExecutionState::Running,
      "completed" => ExecutionState::Done,
      "failed" => ExecutionState::Error,
      "waiting" => ExecutionState::Waiting,
      _ => ExecutionState::Idle,
    };
```

Replace with:

```rust
    let state = match status.status {
      RunState::Running => ExecutionState::Running,
      RunState::Completed => ExecutionState::Done,
      RunState::Failed => ExecutionState::Error,
      RunState::Waiting => ExecutionState::Waiting,
      RunState::Idle => ExecutionState::Idle,
    };
```

Add the import in `crates/lx-mobile/src/pages/status.rs`. Find the exact string:

```rust
use dioxus::prelude::*;
```

Add after it:

```rust
use lx_api::types::RunState;
```

No other files construct `RunStatus` directly — the only construction is `RunStatus::default()` in `crates/lx-api/src/run_api.rs`, which will use the `#[default]` `Idle` variant automatically.

---

### Task 2: Define `PromptKind` enum and replace `PendingPrompt.kind: String`

**Subject:** Replace `PendingPrompt.kind` string field with `PromptKind` enum

**Description:**

In `crates/lx-api/src/types.rs`, add the following enum **above** the `PendingPrompt` struct. Find the exact string `pub struct PendingPrompt {` and insert this enum immediately before the `#[derive(...)]` line preceding it:

```rust
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptKind {
    Confirm,
    Choose,
    Ask,
}
```

Then change the `PendingPrompt` struct's `kind` field. Find the exact string:

```rust
  pub kind: String,
```

inside the `PendingPrompt` struct and replace with:

```rust
  pub kind: PromptKind,
```

Then update the consumer in `crates/lx-mobile/src/pages/approvals.rs`. Find the exact string:

```rust
  match prompt.kind.as_str() {
```

Replace with:

```rust
  match prompt.kind {
```

Find the exact string `"confirm" => {` and replace with `PromptKind::Confirm => {`.

Find the exact string `"choose" => {` and replace with `PromptKind::Choose => {`.

Find the exact string `"ask" => {` and replace with `PromptKind::Ask => {`.

Find the exact string `_ => rsx! {},` (the wildcard arm in the `prompt.kind` match) and delete it entirely.

Add the import in `crates/lx-mobile/src/pages/approvals.rs`. Find the exact string:

```rust
use lx_api::types::{PendingPrompt, PromptResponse};
```

Replace with:

```rust
use lx_api::types::{PendingPrompt, PromptKind, PromptResponse};
```

No other files in `crates/` construct `PendingPrompt` directly — the struct is only created via serde deserialization.

---

### Task 3: Define `EventFilter` enum and replace string-based filter in events page

**Subject:** Replace string event filter with `EventFilter` enum

**Description:**

In `crates/lx-mobile/src/pages/events.rs`, replace the string-based filter system with an enum.

First, add the enum definition. Find the exact string `#[component]` (the first one in the file, on the `EventsPage` component) and insert the following **above** it:

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub enum EventFilter {
    #[default]
    All,
    Ai,
    Emit,
    Log,
    Shell,
    Messages,
    Agents,
    Progress,
    Errors,
}

impl EventFilter {
    const ALL: &[EventFilter] = &[
        EventFilter::All,
        EventFilter::Ai,
        EventFilter::Emit,
        EventFilter::Log,
        EventFilter::Shell,
        EventFilter::Messages,
        EventFilter::Agents,
        EventFilter::Progress,
        EventFilter::Errors,
    ];

    fn matches(&self, event_type: &str) -> bool {
        match self {
            EventFilter::All => true,
            EventFilter::Ai => event_type.starts_with("ai_"),
            EventFilter::Emit => event_type == "emit",
            EventFilter::Log => event_type == "log",
            EventFilter::Shell => event_type.starts_with("shell_"),
            EventFilter::Messages => event_type.starts_with("message_") || event_type == "user_prompt" || event_type == "user_response",
            EventFilter::Agents => event_type.starts_with("agent_"),
            EventFilter::Progress => event_type == "progress" || event_type == "program_started" || event_type == "program_finished" || event_type == "trace_span_recorded",
            EventFilter::Errors => event_type == "error",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            EventFilter::All => "all",
            EventFilter::Ai => "ai",
            EventFilter::Emit => "emit",
            EventFilter::Log => "log",
            EventFilter::Shell => "shell",
            EventFilter::Messages => "messages",
            EventFilter::Agents => "agents",
            EventFilter::Progress => "progress",
            EventFilter::Errors => "errors",
        }
    }
}
```

Then change the signal type. Find the exact string:

```rust
  let mut filter = use_signal(|| "all".to_string());
```

Replace with:

```rust
  let mut filter = use_signal(EventFilter::default);
```

Find the exact string:

```rust
  let current_filter = filter.read().clone();
  let visible: Vec<_> = events
    .read()
    .iter()
    .filter(|e| {
      if current_filter == "all" {
        return true;
      }
      event_type_matches(&e.kind, &current_filter)
    })
    .cloned()
    .collect();
```

Replace with:

```rust
  let current_filter = filter.read().clone();
  let visible: Vec<_> = events
    .read()
    .iter()
    .filter(|e| current_filter.matches(&e.kind))
    .cloned()
    .collect();
```

Find the exact string:

```rust
        for f in FILTER_OPTIONS {
          button {
            class: "px-2 py-1 text-xs rounded",
            class: if current_filter == *f { "bg-[var(--primary)] text-[var(--on-primary)]" } else { "bg-[var(--surface-container-high)] text-[var(--on-surface-variant)]" },
            onclick: {
                let f = f.to_string();
                move |_| filter.set(f.clone())
            },
            "{f}"
          }
        }
```

Replace with:

```rust
        for f in EventFilter::ALL {
          button {
            class: "px-2 py-1 text-xs rounded",
            class: if current_filter == *f { "bg-[var(--primary)] text-[var(--on-primary)]" } else { "bg-[var(--surface-container-high)] text-[var(--on-surface-variant)]" },
            onclick: {
                let f = f.clone();
                move |_| filter.set(f.clone())
            },
            "{f.label()}"
          }
        }
```

Find the exact string and delete it entirely:

```rust
const FILTER_OPTIONS: &[&str] = &["all", "ai", "emit", "log", "shell", "messages", "agents", "progress", "errors"];
```

Find the exact string and delete it entirely:

```rust
fn event_type_matches(event_type: &str, filter: &str) -> bool {
  match filter {
    "ai" => event_type.starts_with("ai_"),
    "emit" => event_type == "emit",
    "log" => event_type == "log",
    "shell" => event_type.starts_with("shell_"),
    "messages" => event_type.starts_with("message_") || event_type == "user_prompt" || event_type == "user_response",
    "agents" => event_type.starts_with("agent_"),
    "progress" => event_type == "progress" || event_type == "program_started" || event_type == "program_finished" || event_type == "trace_span_recorded",
    "errors" => event_type == "error",
    _ => true,
  }
}
```

Both `FILTER_OPTIONS` and `event_type_matches` are only referenced within this file — no other files use them.

---

### Task 4: Define backend enums and replace `BackendsSection` string fields

**Subject:** Replace `BackendsSection` string fields with typed backend enums

**Description:**

In `crates/lx-cli/src/manifest.rs`, replace the `BackendsSection` struct's `Option<String>` fields with typed enums. Find the exact string `pub struct BackendsSection {` and insert the following enums immediately before the `#[derive(Deserialize)]` line preceding it:

```rust
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EmitBackend {
    Noop,
    Stdout,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogBackend {
    Noop,
    Stderr,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LlmBackend {
    ClaudeCode,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HttpBackend {
    Reqwest,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum YieldBackend {
    StdinStdout,
}
```

Then replace the `BackendsSection` struct fields. Find the exact string:

```rust
  pub llm: Option<String>,
  pub http: Option<String>,
  pub emit: Option<String>,
  #[serde(rename = "yield")]
  pub yield_backend: Option<String>,
  pub log: Option<String>,
```

Replace with:

```rust
  pub llm: Option<LlmBackend>,
  pub http: Option<HttpBackend>,
  pub emit: Option<EmitBackend>,
  #[serde(rename = "yield")]
  pub yield_backend: Option<YieldBackend>,
  pub log: Option<LogBackend>,
```

Then update `crates/lx-cli/src/main.rs`. Find the exact string:

```rust
  if let Some(ref name) = backends.emit {
    match name.as_str() {
      "noop" => ctx.emit = Arc::new(NoopEmitBackend),
      "stdout" => {},
      other => eprintln!("warning: unknown emit backend '{other}'"),
    }
  }
  if let Some(ref name) = backends.log {
    match name.as_str() {
      "noop" => ctx.log = Arc::new(NoopLogBackend),
      "stderr" => {},
      other => eprintln!("warning: unknown log backend '{other}'"),
    }
  }
  if let Some(ref name) = backends.llm {
    match name.as_str() {
      "claude-code" => ctx.llm = Arc::new(llm_backend::ClaudeCodeLlmBackend),
      other => eprintln!("warning: unknown llm backend '{other}'"),
    }
  }
  if let Some(ref name) = backends.http {
    match name.as_str() {
      "reqwest" => {},
      other => eprintln!("warning: unknown http backend '{other}'"),
    }
  }
  if let Some(ref name) = backends.yield_backend {
    match name.as_str() {
      "stdin-stdout" => {},
      other => eprintln!("warning: unknown yield backend '{other}'"),
    }
  }
```

Replace with:

```rust
  if let Some(ref backend) = backends.emit {
    match backend {
      manifest::EmitBackend::Noop => ctx.emit = Arc::new(NoopEmitBackend),
      manifest::EmitBackend::Stdout => {},
    }
  }
  if let Some(ref backend) = backends.log {
    match backend {
      manifest::LogBackend::Noop => ctx.log = Arc::new(NoopLogBackend),
      manifest::LogBackend::Stderr => {},
    }
  }
  if let Some(ref backend) = backends.llm {
    match backend {
      manifest::LlmBackend::ClaudeCode => ctx.llm = Arc::new(llm_backend::ClaudeCodeLlmBackend),
    }
  }
  if let Some(ref backend) = backends.http {
    match backend {
      manifest::HttpBackend::Reqwest => {},
    }
  }
  if let Some(ref backend) = backends.yield_backend {
    match backend {
      manifest::YieldBackend::StdinStdout => {},
    }
  }
```

No import changes needed in `main.rs` — the `manifest` module is already imported via `mod manifest;` and all enum types are accessed via the `manifest::` prefix.

Existing `lx.toml` files using `[backends]` (found at `programs/workrunner/lx.toml`, `programs/brain/lx.toml`, `programs/workgen/lx.toml`) all set `llm = "claude-code"` which deserializes to `LlmBackend::ClaudeCode` via `#[serde(rename_all = "kebab-case")]` — no manifest file changes needed.

---

## Preconditions

All four tasks are independent and can be done in any order, but they are listed in a natural sequence. Each task modifies a disjoint set of lines. The only shared file is `crates/lx-api/src/types.rs` (Tasks 1 and 2 both add enums and modify structs there), so Tasks 1 and 2 should be done in order.

## Files Modified

| Task | File | Change |
|------|------|--------|
| 1 | `crates/lx-api/src/types.rs` | Add `RunState` enum, change `RunStatus.status` to `RunState` |
| 1 | `crates/lx-mobile/src/pages/status.rs` | Add `RunState` import, replace string match with enum match |
| 2 | `crates/lx-api/src/types.rs` | Add `PromptKind` enum, change `PendingPrompt.kind` to `PromptKind` |
| 2 | `crates/lx-mobile/src/pages/approvals.rs` | Add `PromptKind` import, replace string match with enum match, remove wildcard arm |
| 3 | `crates/lx-mobile/src/pages/events.rs` | Add `EventFilter` enum with `matches`/`label` methods, replace `Signal<String>` with `Signal<EventFilter>`, delete `FILTER_OPTIONS` and `event_type_matches` |
| 4 | `crates/lx-cli/src/manifest.rs` | Add `EmitBackend`, `LogBackend`, `LlmBackend`, `HttpBackend`, `YieldBackend` enums, change `BackendsSection` fields from `Option<String>` to typed `Option<EnumType>` |
| 4 | `crates/lx-cli/src/main.rs` | Replace string matches in `apply_manifest_backends` with enum matches |
