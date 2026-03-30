# Unit 02: TranscriptView Block Type Expansion

## Goal

Expand `TranscriptView` from 4 block types to 9, matching Paperclip's `RunTranscriptView.tsx` block types, with collapsible tool/command/stderr sections, status indicators, and markdown rendering for messages.

## Preconditions

- No other units need to be complete first.
- `MarkdownBody` component exists at `crates/lx-desktop/src/components/markdown_body.rs` and renders HTML from markdown via `pulldown_cmark`.

## Files to Modify

- `crates/lx-desktop/src/pages/agents/transcript.rs` (currently 124 lines)

This will exceed 300 lines. Split as follows:

- **`transcript.rs`**: `TranscriptBlock` enum, `ToolStatus`, `ToolItem`, `StderrLine`, `event_to_block`, and `TranscriptView` component.
- **`transcript_blocks.rs`** (new): `TranscriptBlockView` component and all individual block rendering functions.
- **`mod.rs`**: Add `mod transcript_blocks;`.

## Context: Current State

The current `TranscriptBlock` enum has 4 variants:

```rust
pub enum TranscriptBlock {
  Message { role: String, text: String, ts: String },
  Thinking { text: String, ts: String },
  ToolUse { name: String, input_summary: String, result: Option<String>, is_error: bool, ts: String },
  Event { label: String, text: String, tone: String, ts: String },
}
```

## Context: Paperclip Block Types

Paperclip's `RunTranscriptView.tsx` defines 9 block types (lines 30-107):

1. `message` - role, text, streaming flag
2. `thinking` - text, streaming flag
3. `tool` - name, input, result, isError, status (running/completed/error)
4. `activity` - name, activityId, status (running/completed)
5. `command_group` - items array of {input, result, isError, status}
6. `tool_group` - items array of {name, input, result, isError, status}
7. `stderr_group` - lines array of {ts, text}
8. `stdout` - text
9. `event` - label, tone, text, detail

## Steps

### Step 1: Expand the TranscriptBlock enum

Replace the current `TranscriptBlock` enum in `transcript.rs` with:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum ToolStatus {
  Running,
  Completed,
  Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolItem {
  pub ts: String,
  pub name: String,
  pub input: String,
  pub result: Option<String>,
  pub is_error: bool,
  pub status: ToolStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StderrLine {
  pub ts: String,
  pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TranscriptBlock {
  Message { role: String, text: String, ts: String },
  Thinking { text: String, ts: String },
  Tool { name: String, input: String, result: Option<String>, is_error: bool, status: ToolStatus, ts: String },
  Activity { name: String, status: ToolStatus, ts: String },
  CommandGroup { items: Vec<ToolItem>, ts: String },
  ToolGroup { items: Vec<ToolItem>, ts: String },
  StderrGroup { lines: Vec<StderrLine>, ts: String },
  Stdout { text: String, ts: String },
  Event { label: String, text: String, detail: Option<String>, tone: String, ts: String },
}
```

### Step 2: Update event_to_block

Replace the `event_to_block` function. The `ActivityEvent` struct has `kind`, `message`, and `timestamp` fields. Map them:

```rust
fn event_to_block(event: &ActivityEvent) -> TranscriptBlock {
  match event.kind.as_str() {
    "log" | "agent_message" | "message" | "assistant" | "user" => {
      let role = if event.kind == "user" { "user" } else { "assistant" };
      TranscriptBlock::Message { role: role.into(), text: event.message.clone(), ts: event.timestamp.clone() }
    },
    "thinking" => TranscriptBlock::Thinking { text: event.message.clone(), ts: event.timestamp.clone() },
    "tool_call" => TranscriptBlock::Tool {
      name: event.kind.clone(),
      input: event.message.clone(),
      result: None,
      is_error: false,
      status: ToolStatus::Running,
      ts: event.timestamp.clone(),
    },
    "tool_result" => TranscriptBlock::Tool {
      name: "tool".into(),
      input: String::new(),
      result: Some(event.message.clone()),
      is_error: false,
      status: ToolStatus::Completed,
      ts: event.timestamp.clone(),
    },
    "stderr" => TranscriptBlock::StderrGroup {
      lines: vec![StderrLine { ts: event.timestamp.clone(), text: event.message.clone() }],
      ts: event.timestamp.clone(),
    },
    "stdout" => TranscriptBlock::Stdout { text: event.message.clone(), ts: event.timestamp.clone() },
    "activity" => TranscriptBlock::Activity {
      name: event.message.clone(),
      status: ToolStatus::Running,
      ts: event.timestamp.clone(),
    },
    k if k.contains("tool") => TranscriptBlock::Tool {
      name: event.kind.clone(),
      input: event.message.clone(),
      result: None,
      is_error: false,
      status: ToolStatus::Running,
      ts: event.timestamp.clone(),
    },
    k if k.contains("error") => TranscriptBlock::Event {
      label: "error".into(),
      text: event.message.clone(),
      detail: None,
      tone: "error".into(),
      ts: event.timestamp.clone(),
    },
    _ => TranscriptBlock::Event {
      label: event.kind.clone(),
      text: event.message.clone(),
      detail: None,
      tone: "info".into(),
      ts: event.timestamp.clone(),
    },
  }
}
```

### Step 3: Render TranscriptBlockView for each variant

Replace the `TranscriptBlockView` component. Below is the exact RSX for each variant, using lx-desktop's CSS variable system (not Paperclip's hardcoded Tailwind colors).

Add this import at the top of the file:
```rust
use crate::components::markdown_body::MarkdownBody;
```

#### Message block (existing, enhanced with MarkdownBody)
```rust
TranscriptBlock::Message { role, text, .. } => {
  let icon = if role == "assistant" { "smart_toy" } else { "person" };
  let bg = if role == "assistant" { "bg-[var(--surface-container)]" } else { "bg-[var(--surface-container-high)]" };
  rsx! {
    div { class: "flex gap-3 p-3 rounded-lg {bg}",
      span { class: "material-symbols-outlined text-sm text-[var(--outline)] shrink-0 mt-0.5", "{icon}" }
      div { class: "flex-1 min-w-0",
        MarkdownBody { content: text }
      }
    }
  }
}
```

#### Thinking block (unchanged)
```rust
TranscriptBlock::Thinking { text, .. } => {
  rsx! {
    div { class: "flex gap-3 p-3 rounded-lg bg-[var(--warning)]/5 border border-[var(--warning)]/10",
      span { class: "material-symbols-outlined text-sm text-[var(--warning)] shrink-0 mt-0.5", "psychology" }
      div { class: "flex-1 min-w-0 text-xs text-[var(--outline)] italic whitespace-pre-wrap", "{text}" }
    }
  }
}
```

#### Tool block (expanded with status + collapsible detail)

All signal hooks for collapsible sections are called unconditionally at the top of `TranscriptBlockView`, before the match. The Tool block reads from the pre-allocated signal.

Inside `TranscriptBlockView`, before the match on `block`, declare all collapsible signals unconditionally:

```rust
let mut tool_open = use_signal(|| false);
let mut cmd_group_open = use_signal(|| false);
let mut tool_group_open = use_signal(|| false);
let mut stderr_open = use_signal(|| false);
let mut stdout_open = use_signal(|| true);
```

Then set the initial value for the Tool block based on block data, after signals are created but before rendering:

```rust
if let TranscriptBlock::Tool { is_error, .. } = &block {
    if *is_error && !tool_open() {
        tool_open.set(true);
    }
}
```

Then in the match arm:

```rust
TranscriptBlock::Tool { name, input, result, is_error, status, .. } => {
  let status_label = match status {
    ToolStatus::Running => "Running",
    ToolStatus::Completed => "Completed",
    ToolStatus::Error => "Errored",
  };
  let status_color = match status {
    ToolStatus::Running => "text-[var(--tertiary)]",
    ToolStatus::Completed => "text-[var(--success)]",
    ToolStatus::Error => "text-[var(--error)]",
  };
  let icon = match status {
    ToolStatus::Running => "build",
    ToolStatus::Completed => "check_circle",
    ToolStatus::Error => "error",
  };
  let border = if is_error { "border-[var(--error)]/20 bg-[var(--error)]/[0.04]" } else { "border-[var(--outline-variant)]/20" };
  rsx! {
    div { class: "border {border} rounded-lg p-3 space-y-2",
      div { class: "flex items-center gap-2",
        span { class: "material-symbols-outlined text-sm {status_color}", "{icon}" }
        span { class: "text-[11px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]", "{name}" }
        span { class: "text-[10px] font-semibold uppercase tracking-wider {status_color}", "{status_label}" }
        button {
          class: "ml-auto text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
          onclick: move |_| tool_open.set(!tool_open()),
          span { class: "material-symbols-outlined text-sm",
            if tool_open() { "expand_more" } else { "chevron_right" }
          }
        }
      }
      if !input.is_empty() && !tool_open() {
        p { class: "text-xs text-[var(--outline)] font-mono truncate", "{input}" }
      }
      if tool_open() {
        div { class: "grid gap-3 lg:grid-cols-2",
          div {
            div { class: "mb-1 text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]", "Input" }
            pre { class: "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80",
              if input.is_empty() { "<empty>" } else { "{input}" }
            }
          }
          div {
            div { class: "mb-1 text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]", "Result" }
            pre {
              class: if is_error { "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--error)]" } else { "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80" },
              match result {
                Some(r) => rsx! { "{r}" },
                None => rsx! { "Waiting for result..." },
              }
            }
          }
        }
      }
    }
  }
}
```

#### Activity block (new)
```rust
TranscriptBlock::Activity { name, status, .. } => {
  rsx! {
    div { class: "flex items-start gap-2",
      match status {
        ToolStatus::Completed => rsx! {
          span { class: "material-symbols-outlined text-sm text-[var(--success)] shrink-0 mt-0.5", "check_circle" }
        },
        _ => rsx! {
          span { class: "relative mt-1 flex h-2.5 w-2.5 shrink-0",
            span { class: "absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--tertiary)] opacity-70" }
            span { class: "relative inline-flex h-2.5 w-2.5 rounded-full bg-[var(--tertiary)]" }
          }
        },
      }
      div { class: "break-words text-sm text-[var(--on-surface)]/80 leading-6", "{name}" }
    }
  }
}
```

#### CommandGroup block (new)

```rust
TranscriptBlock::CommandGroup { items, .. } => {
  let has_error = items.iter().any(|i| i.is_error);
  let is_running = items.iter().any(|i| matches!(i.status, ToolStatus::Running));
  let title = if is_running { "Executing command".to_string() } else if items.len() == 1 { "Executed command".to_string() } else { format!("Executed {} commands", items.len()) };
  rsx! {
    div { class: if has_error && cmd_group_open() { "rounded-lg border border-[var(--error)]/20 bg-[var(--error)]/[0.04] p-3" } else { "" },
      div {
        class: "flex items-center gap-2 cursor-pointer",
        onclick: move |_| cmd_group_open.set(!cmd_group_open()),
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]", "terminal" }
        span { class: "text-[11px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]/70", "{title}" }
        button {
          class: "ml-auto text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
          onclick: move |evt| { evt.stop_propagation(); cmd_group_open.set(!cmd_group_open()); },
          span { class: "material-symbols-outlined text-sm",
            if cmd_group_open() { "expand_more" } else { "chevron_right" }
          }
        }
      }
      if cmd_group_open() {
        div { class: "mt-3 space-y-3",
          for (idx , item) in items.iter().enumerate() {
            div { key: "{idx}", class: "space-y-2",
              div { class: "flex items-center gap-2",
                span { class: "material-symbols-outlined text-xs text-[var(--outline)]", "terminal" }
                span { class: "font-mono text-xs break-all", "{item.input}" }
              }
              if let Some(ref res) = item.result {
                pre {
                  class: if item.is_error { "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--error)]" } else { "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80" },
                  "{res}"
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

#### ToolGroup block (new)

```rust
TranscriptBlock::ToolGroup { items, .. } => {
  let has_error = items.iter().any(|i| i.is_error);
  let is_running = items.iter().any(|i| matches!(i.status, ToolStatus::Running));
  let title = if is_running { format!("Using {} tools", items.len()) } else { format!("Used {} tools ({} calls)", items.len(), items.len()) };
  rsx! {
    div { class: "rounded-lg border border-[var(--outline-variant)]/40 bg-[var(--surface-container)]/25",
      div {
        class: "flex items-center gap-2 px-3 py-2.5 cursor-pointer",
        onclick: move |_| tool_group_open.set(!tool_group_open()),
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]", "build" }
        span { class: "text-[11px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]/70", "{title}" }
        button {
          class: "ml-auto text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
          onclick: move |evt| { evt.stop_propagation(); tool_group_open.set(!tool_group_open()); },
          span { class: "material-symbols-outlined text-sm",
            if tool_group_open() { "expand_more" } else { "chevron_right" }
          }
        }
      }
      if tool_group_open() {
        div { class: "space-y-2 border-t border-[var(--outline-variant)]/30 px-3 py-3",
          for (idx , item) in items.iter().enumerate() {
            div { key: "{idx}", class: "space-y-1.5",
              div { class: "flex items-center gap-2",
                span { class: "material-symbols-outlined text-xs text-[var(--outline)]", "build" }
                span { class: "text-[10px] font-semibold uppercase tracking-wider text-[var(--on-surface-variant)]", "{item.name}" }
                span {
                  class: match item.status {
                    ToolStatus::Running => "text-[10px] font-semibold uppercase tracking-wider text-[var(--tertiary)]",
                    ToolStatus::Completed => "text-[10px] font-semibold uppercase tracking-wider text-[var(--success)]",
                    ToolStatus::Error => "text-[10px] font-semibold uppercase tracking-wider text-[var(--error)]",
                  },
                  match item.status {
                    ToolStatus::Running => "Running",
                    ToolStatus::Completed => "Completed",
                    ToolStatus::Error => "Errored",
                  }
                }
              }
              div { class: "grid gap-2 pl-7 grid-cols-1 lg:grid-cols-2",
                div {
                  div { class: "mb-0.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]", "Input" }
                  pre { class: "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80",
                    if item.input.is_empty() { "<empty>" } else { "{item.input}" }
                  }
                }
                if let Some(ref res) = item.result {
                  div {
                    div { class: "mb-0.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]", "Result" }
                    pre {
                      class: if item.is_error { "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--error)]" } else { "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80" },
                      "{res}"
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
}
```

#### StderrGroup block (new)

Uses the `stderr_open` signal declared at the top of `TranscriptBlockView`.

```rust
TranscriptBlock::StderrGroup { lines, .. } => {
  let count = lines.len();
  let noun = if count == 1 { "line" } else { "lines" };
  rsx! {
    div { class: "rounded-lg border border-[var(--warning)]/20 bg-[var(--warning)]/[0.06] p-2 text-[var(--warning)]",
      div {
        class: "flex items-center gap-2 cursor-pointer",
        onclick: move |_| stderr_open.set(!stderr_open()),
        span { class: "text-[10px] font-semibold uppercase tracking-wider", "{count} log {noun}" }
        span { class: "material-symbols-outlined text-sm",
          if stderr_open() { "expand_more" } else { "chevron_right" }
        }
      }
      if stderr_open() {
        pre { class: "mt-2 overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--warning)]/80 pl-5",
          for line in lines.iter() {
            "{line.text}\n"
          }
        }
      }
    }
  }
}
```

#### Stdout block (new)

Uses the `stdout_open` signal declared at the top of `TranscriptBlockView`.

```rust
TranscriptBlock::Stdout { text, .. } => {
  rsx! {
    div {
      div { class: "flex items-center gap-2",
        span { class: "text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]", "stdout" }
        button {
          class: "text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
          onclick: move |_| stdout_open.set(!stdout_open()),
          span { class: "material-symbols-outlined text-sm",
            if stdout_open() { "expand_more" } else { "chevron_right" }
          }
        }
      }
      if stdout_open() {
        pre { class: "mt-2 overflow-x-auto whitespace-pre-wrap break-words font-mono text-xs text-[var(--on-surface)]/80", "{text}" }
      }
    }
  }
}
```

#### Event block (enhanced with detail + tone styling)
```rust
TranscriptBlock::Event { label, text, detail, tone, .. } => {
  let wrapper_class = match tone.as_str() {
    "error" => "rounded-lg border border-[var(--error)]/20 bg-[var(--error)]/[0.06] p-3 text-[var(--error)]",
    "warn" => "text-[var(--warning)]",
    "info" => "text-[var(--tertiary)]",
    _ => "text-[var(--on-surface)]/75",
  };
  let icon = match tone.as_str() {
    "error" => "error",
    "warn" => "terminal",
    _ => "circle",
  };
  rsx! {
    div { class: "{wrapper_class}",
      div { class: "flex items-start gap-2",
        span { class: "material-symbols-outlined text-sm shrink-0 mt-0.5", "{icon}" }
        div { class: "min-w-0 flex-1",
          div { class: "whitespace-pre-wrap break-words text-xs",
            span { class: "text-[10px] font-semibold uppercase tracking-wider text-[var(--on-surface-variant)]/70", "{label}" }
            if !text.is_empty() {
              span { class: "ml-2", "{text}" }
            }
          }
          if let Some(ref d) = detail {
            pre { class: "mt-2 overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/75", "{d}" }
          }
        }
      }
    }
  }
}
```

### Step 4: File splitting

This will exceed 300 lines. Split as follows:

1. Create `crates/lx-desktop/src/pages/agents/transcript_blocks.rs`.
2. Move the `TranscriptBlockView` component and all individual block rendering into that file.
3. Keep the `TranscriptBlock` enum, `ToolStatus`, `ToolItem`, `StderrLine`, `event_to_block`, and `TranscriptView` in `transcript.rs`.
4. In `transcript_blocks.rs`, add `use super::*;` as the first line.
5. In `crates/lx-desktop/src/pages/agents/mod.rs`, add `mod transcript_blocks;`.

## Verification

1. Run `just diagnose` to confirm no compilation errors or warnings.
2. Confirm no file exceeds 300 lines.
3. Visual check: navigate to an agent's run transcript. Verify:
   - Message blocks render markdown (bold, code, links) via MarkdownBody.
   - Tool blocks show status label (Running/Completed/Errored) with color coding.
   - Tool blocks have a chevron that toggles Input/Result detail.
   - Error tool blocks have a red-tinted background.
   - Activity blocks show an animated ping dot when running, a green check when completed.
   - StderrGroup shows a collapsible amber-tinted block with line count.
   - Stdout shows a collapsible block labeled "stdout".
   - Event blocks with error tone get a red border/background wrapper.
