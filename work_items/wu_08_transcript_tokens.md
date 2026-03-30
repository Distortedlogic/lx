# WU-08: Transcript token count display

## Dependencies: WU-07 must run first.

WU-08 modifies the same files as WU-07 (transcript.rs, transcript_blocks.rs). All line references below are based on the post-WU-07 state documented in WU-07's Post-Execution State section.

## Fixes
- Fix 1: Add `token_count: Option<u32>` field to `ActivityEvent` in lx-api types
- Fix 2: Add `token_count: Option<u32>` field to the `Tool` variant of `TranscriptBlock`
- Fix 3: Wire token_count through `event_to_block` conversion
- Fix 4: Display token count badge in the Tool block header row in transcript_blocks.rs
- Fix 5-18: Apply the same pattern across all tool-related event kinds (tool_call, tool_result, tool_error, tool_group items, command_group items, and the catch-all `k if k.contains("tool")` arm)

## Files Modified
- `crates/lx-api/src/types.rs` (49 lines)
- `crates/lx-desktop/src/pages/agents/transcript.rs` (~195 lines post-WU-07)
- `crates/lx-desktop/src/pages/agents/transcript_blocks.rs` (~180 lines post-WU-07)

## Preconditions
- `ActivityEvent` struct at line 2 of `crates/lx-api/src/types.rs` has fields: `timestamp: String`, `kind: String`, `message: String`
- Post-WU-07 state of transcript.rs:
  - `TranscriptBlock::Tool` variant at line 43 has fields: `name`, `input`, `result`, `is_error`, `status`, `ts`
  - `ToolItem` struct at line 25 has fields: `ts`, `name`, `input`, `result`, `is_error`, `status`
  - `event_to_block` function at line 54
- Post-WU-07 state of transcript_blocks.rs:
  - Import line 1: `use super::transcript::{TranscriptDensity, TranscriptMode, ToolStatus, TranscriptBlock, summarize_tool_input};`
  - `TranscriptBlockView` signature at line 7: `fn TranscriptBlockView(block: TranscriptBlock, mode: TranscriptMode, density: TranscriptDensity) -> Element`
  - Tool block header row rendered at lines ~61-71, containing icon, name, status_label, and expand button
- Token counts are only displayed on standalone Tool blocks and not within CommandGroup or ToolGroup items. The group renderers in transcript_groups.rs do not display per-item token counts; the `token_count` field on `ToolItem` is carried for data completeness but not rendered in group views.

## Steps

### Step 1: Add token_count to ActivityEvent in lx-api
- Open `crates/lx-api/src/types.rs`
- At line 5, after `pub message: String,`, add:

```rust
  #[serde(default)]
  pub token_count: Option<u32>,
```

- Why: The API event type needs to carry token count data from the backend. The `#[serde(default)]` attribute ensures deserialization succeeds when the field is absent from incoming JSON (older events or backends that don't yet send token counts).

### Step 2: Add token_count to TranscriptBlock::Tool variant
- Open `crates/lx-desktop/src/pages/agents/transcript.rs`
- At line 43 (post-WU-07), change the Tool variant from:

```rust
  Tool { name: String, input: String, result: Option<String>, is_error: bool, status: ToolStatus, ts: String },
```

to:

```rust
  Tool { name: String, input: String, result: Option<String>, is_error: bool, status: ToolStatus, ts: String, token_count: Option<u32> },
```

- Why: The transcript block needs to carry the token count for display

### Step 3: Add token_count to ToolItem struct
- Open `crates/lx-desktop/src/pages/agents/transcript.rs`
- At line 25 (post-WU-07), in the `ToolItem` struct, after `pub status: ToolStatus,` add:

```rust
  pub token_count: Option<u32>,
```

- Why: Tool items in command/tool groups also need token counts

### Step 4: Wire token_count through event_to_block for tool_call arm
- Open `crates/lx-desktop/src/pages/agents/transcript.rs`
- In `event_to_block` (starts at line 54 post-WU-07), the `"tool_call"` arm constructs `TranscriptBlock::Tool`. Add `token_count: event.token_count,` to the struct literal, after the `ts` field.

### Step 5: Wire token_count through event_to_block for tool_result arm
- The `"tool_result"` arm. Add `token_count: event.token_count,` after the `ts` field.

### Step 6: Wire token_count through event_to_block for tool_error arm
- The `"tool_error"` arm. Add `token_count: event.token_count,` after the `ts` field.

### Step 7: Wire token_count through event_to_block for command_group arm
- The `"command_group"` arm creates a `ToolItem`. Add `token_count: event.token_count,` to the ToolItem literal (after `status: ToolStatus::Running,`).

### Step 8: Wire token_count through event_to_block for tool_group arm
- The `"tool_group"` arm creates a `ToolItem`. Add `token_count: event.token_count,` to the ToolItem literal.

### Step 9: Wire token_count through event_to_block for catch-all tool arm
- The `k if k.contains("tool")` arm. Add `token_count: event.token_count,` to the `TranscriptBlock::Tool` literal.

### Step 10: Display token count in TranscriptBlockView Tool header
- Open `crates/lx-desktop/src/pages/agents/transcript_blocks.rs`
- In the Tool match arm (post-WU-07 this arm destructures with `mode` and `density` in scope), change `TranscriptBlock::Tool { name, input, result, is_error, status, .. }` to: `TranscriptBlock::Tool { name, input, result, is_error, status, token_count, .. }`
- After the status label span, before the expand button, add a conditional token count badge:

```rust
            if let Some(tokens) = token_count {
              span { class: "text-[10px] text-[var(--outline)] tabular-nums ml-1", "{tokens} tok" }
            }
```

- Why: Displays token count inline in the tool block header so users can see cost/size at a glance

### Step 11: Verify auto-open logic handles new field
- The auto-open-on-error pattern match in transcript_blocks.rs: `if let TranscriptBlock::Tool { is_error, .. } = &block`. The `..` already covers `token_count`, so no change needed here.

## File Size Check
- `crates/lx-api/src/types.rs`: was 49 lines, now ~51 lines (under 300)
- `transcript.rs`: was ~195 lines (post-WU-07), now ~203 lines (under 300)
- `transcript_blocks.rs`: was ~180 lines (post-WU-07), now ~185 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compile errors or warnings
- Confirm all `TranscriptBlock::Tool { .. }` construction sites include `token_count`
- Confirm all `ToolItem { .. }` construction sites include `token_count`
- When an ActivityEvent has `token_count: Some(1500)`, the Tool block header should display "1500 tok" after the status label
- When `token_count` is `None`, no badge should appear
