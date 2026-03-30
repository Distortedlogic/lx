# WU-07: Transcript mode/density toggle and tool input summarization

## Fixes
- Fix 1: Add TranscriptMode (Nice/Raw) and TranscriptDensity (Comfortable/Compact) enums and propagate as props
- Fix 2: Add summarize_tool_input() function that truncates long tool inputs in Nice mode
- Fix 3: Add a toolbar row above the transcript with toggle buttons for mode and density

## Files Modified
- `crates/lx-desktop/src/pages/agents/transcript.rs` (142 lines)
- `crates/lx-desktop/src/pages/agents/transcript_blocks.rs` (171 lines)

## Preconditions
- `TranscriptBlock` enum defined at line 30 of transcript.rs with variants: Message, Thinking, Tool, Activity, CommandGroup, ToolGroup, StderrGroup, Stdout, Event
- `TranscriptView` component defined at line 116 of transcript.rs, accepts `run_id: String` and optional `events: Option<Vec<ActivityEvent>>`
- `TranscriptBlockView` component defined at line 7 of transcript_blocks.rs, accepts `block: TranscriptBlock`
- The `Tool` variant of `TranscriptBlock` has fields: `name`, `input`, `result`, `is_error`, `status`, `ts` (line 33)
- `ScrollToBottom` component imported and used at line 133 of transcript.rs
- transcript_blocks.rs imports `MarkdownBody` from `crate::components::markdown_body` (line 3)

## Steps

### Step 1: Add TranscriptMode and TranscriptDensity enums to transcript.rs
- Open `crates/lx-desktop/src/pages/agents/transcript.rs`
- At line 6, before the `ToolStatus` enum, add two new enums:

```rust
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum TranscriptMode {
  Nice,
  Raw,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum TranscriptDensity {
  Comfortable,
  Compact,
}
```

- Why: These enums control how transcript content is displayed — Nice mode summarizes tool inputs, Raw shows everything verbatim; Compact reduces spacing

### Step 2: Add summarize_tool_input() function to transcript.rs
- Open `crates/lx-desktop/src/pages/agents/transcript.rs`
- After the `event_to_block` function (after line 113), add:

```rust
pub fn summarize_tool_input(input: &str, max_len: usize) -> String {
  let trimmed = input.trim();
  if trimmed.len() <= max_len {
    return trimmed.to_string();
  }
  let mut end = max_len;
  while !trimmed.is_char_boundary(end) {
    end -= 1;
  }
  format!("{}...", &trimmed[..end])
}
```

- Why: In Nice mode, long tool inputs (file contents, large JSON) should be truncated to keep the transcript scannable

### Step 3: Add toolbar and mode/density signals to TranscriptView component
- Open `crates/lx-desktop/src/pages/agents/transcript.rs`
- Replace the `TranscriptView` component (lines 115-141) with a version that:
  1. Adds `use_signal` for `mode` (default `TranscriptMode::Nice`) and `density` (default `TranscriptDensity::Comfortable`)
  2. Renders a toolbar div above the ScrollToBottom with two toggle button groups
  3. Passes `mode` and `density` as props to each `TranscriptBlockView`

The toolbar should be a `div` with `class: "flex items-center gap-2 mb-2"` containing:
- A mode toggle group: two buttons "Nice" and "Raw", styled with active state using `bg-[var(--primary)] text-[var(--on-primary)]` for the selected one and `bg-[var(--surface-container)] text-[var(--on-surface-variant)]` for the other
- A density toggle group: two buttons "Comfortable" and "Compact" with the same active/inactive styling
- Each button: `class: "text-xs px-2 py-1 rounded transition-colors"` plus the active/inactive class

The updated component:

```rust
#[component]
pub fn TranscriptView(run_id: String, #[props(optional)] events: Option<Vec<ActivityEvent>>) -> Element {
  let mut mode = use_signal(|| TranscriptMode::Nice);
  let mut density = use_signal(|| TranscriptDensity::Comfortable);

  let entries: Vec<TranscriptBlock> = match events {
    Some(evts) => evts.iter().map(event_to_block).collect(),
    None => vec![],
  };

  if entries.is_empty() {
    return rsx! {
      div { class: "border border-[var(--outline-variant)]/30 rounded-lg p-4",
        p { class: "text-sm text-[var(--outline)] text-center",
          "No transcript data available."
        }
      }
    };
  }

  let active_btn = "text-xs px-2 py-1 rounded transition-colors bg-[var(--primary)] text-[var(--on-primary)]";
  let inactive_btn = "text-xs px-2 py-1 rounded transition-colors bg-[var(--surface-container)] text-[var(--on-surface-variant)] hover:bg-[var(--surface-container-high)]";
  let cur_mode = mode();
  let cur_density = density();

  rsx! {
    div { class: "flex items-center gap-3 mb-2",
      div { class: "flex gap-0.5 rounded-md bg-[var(--surface-container)]/50 p-0.5",
        button {
          class: if cur_mode == TranscriptMode::Nice { active_btn } else { inactive_btn },
          onclick: move |_| mode.set(TranscriptMode::Nice),
          "Nice"
        }
        button {
          class: if cur_mode == TranscriptMode::Raw { active_btn } else { inactive_btn },
          onclick: move |_| mode.set(TranscriptMode::Raw),
          "Raw"
        }
      }
      div { class: "flex gap-0.5 rounded-md bg-[var(--surface-container)]/50 p-0.5",
        button {
          class: if cur_density == TranscriptDensity::Comfortable { active_btn } else { inactive_btn },
          onclick: move |_| density.set(TranscriptDensity::Comfortable),
          "Comfortable"
        }
        button {
          class: if cur_density == TranscriptDensity::Compact { active_btn } else { inactive_btn },
          onclick: move |_| density.set(TranscriptDensity::Compact),
          "Compact"
        }
      }
    }
    ScrollToBottom { class: "max-h-[60vh]".to_string(),
      div { class: if cur_density == TranscriptDensity::Compact { "space-y-1" } else { "space-y-2" },
        for entry in entries.iter() {
          transcript_blocks::TranscriptBlockView { block: entry.clone(), mode: cur_mode, density: cur_density }
        }
      }
    }
  }
}
```

- Why: Users need UI controls to toggle between Nice/Raw display and Comfortable/Compact spacing

### Step 4: Update TranscriptBlockView to accept mode and density props
- Open `crates/lx-desktop/src/pages/agents/transcript_blocks.rs`
- At line 1, add import for the new types: change `use super::transcript::{ToolStatus, TranscriptBlock};` to `use super::transcript::{TranscriptDensity, TranscriptMode, ToolStatus, TranscriptBlock, summarize_tool_input};`
- At line 7, change the component signature from `pub fn TranscriptBlockView(block: TranscriptBlock) -> Element` to `pub fn TranscriptBlockView(block: TranscriptBlock, mode: TranscriptMode, density: TranscriptDensity) -> Element`
- Why: The block renderer needs to know the current mode and density to alter its output

### Step 5: Apply mode to Tool block input display in transcript_blocks.rs
- Open `crates/lx-desktop/src/pages/agents/transcript_blocks.rs`
- In the `TranscriptBlock::Tool` match arm (line 42), find the collapsed input preview at line 73-75:

```rust
          if !input.is_empty() && !tool_open() {
            p { class: "text-xs text-[var(--outline)] font-mono truncate", "{input}" }
          }
```

Replace with:

```rust
          if !input.is_empty() && !tool_open() {
            p { class: "text-xs text-[var(--outline)] font-mono truncate",
              {if mode == TranscriptMode::Nice { summarize_tool_input(&input, 120) } else { input.clone() }}
            }
          }
```

- Why: In Nice mode, the collapsed preview should show a summarized version of the input

### Step 6: Apply density to spacing in transcript_blocks.rs
- Open `crates/lx-desktop/src/pages/agents/transcript_blocks.rs`
- In the `TranscriptBlock::Tool` match arm, find the outer div at line 60:

```rust
        div { class: "border {border} rounded-lg p-3 space-y-2 animate-transcript-enter",
```

Replace with:

```rust
        div { class: if density == TranscriptDensity::Compact { "border {border} rounded p-2 space-y-1 animate-transcript-enter" } else { "border {border} rounded-lg p-3 space-y-2 animate-transcript-enter" },
```

- In the `TranscriptBlock::Message` match arm (line 26), find:

```rust
        div { class: "flex gap-3 p-3 rounded-lg {bg} animate-transcript-enter",
```

Replace with:

```rust
        div { class: if density == TranscriptDensity::Compact { "flex gap-3 p-2 rounded {bg} animate-transcript-enter" } else { "flex gap-3 p-3 rounded-lg {bg} animate-transcript-enter" },
```

- Why: Compact density reduces padding and spacing throughout

## File Size Check
- `transcript.rs`: was 142 lines, now ~195 lines (under 300)
- `transcript_blocks.rs`: was 171 lines, now ~180 lines (under 300)

## Post-Execution State

After WU-07 completes, the following signatures and file sizes are in effect. WU-08 depends on this state.

- `transcript.rs`: ~195 lines
  - `TranscriptMode` enum at line 6 (Nice, Raw)
  - `TranscriptDensity` enum at line 12 (Comfortable, Compact)
  - `ToolStatus` enum at line 18
  - `ToolItem` struct at line 25 with fields: `ts`, `name`, `input`, `result`, `is_error`, `status`
  - `StderrLine` struct at line 35
  - `TranscriptBlock` enum at line 41 — `Tool` variant has fields: `name`, `input`, `result`, `is_error`, `status`, `ts`
  - `event_to_block` at line 54
  - `summarize_tool_input` at line 125
  - `TranscriptView` component at line 133 with signature: `fn TranscriptView(run_id: String, #[props(optional)] events: Option<Vec<ActivityEvent>>) -> Element`
- `transcript_blocks.rs`: ~180 lines
  - Import line 1: `use super::transcript::{TranscriptDensity, TranscriptMode, ToolStatus, TranscriptBlock, summarize_tool_input};`
  - `TranscriptBlockView` component at line 7 with signature: `fn TranscriptBlockView(block: TranscriptBlock, mode: TranscriptMode, density: TranscriptDensity) -> Element`

## Verification
- Run `just diagnose` to confirm no compile errors or warnings
- Visually: The transcript view should show a toolbar with Nice/Raw and Comfortable/Compact toggles
- In Nice mode, long tool inputs in collapsed state should be truncated to 120 chars with "..."
- In Compact mode, spacing between blocks and internal padding should be visibly reduced
- In Raw mode, full input text should display without truncation
