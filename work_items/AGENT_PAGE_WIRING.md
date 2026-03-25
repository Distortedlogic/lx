# Agent Page Wiring

## Goal

Replace all mocked data on the Agents page with live state from VoiceContext. Rename the page to AGENT_MANAGER. Collapse to a single agent card showing the voice transcript as live output. Wire agent card buttons. Remove VoiceBanner transcript display (it moves to the card).

## Why

- The Agents page is 95% decorative mockup — two hardcoded agent cards with static strings, fake uptime counter, fake resource stats
- The voice transcript currently renders inside VoiceBanner's compact bar area but belongs in the agent card's live output section where there is room for a proper conversation display
- Agent card buttons (INTERCEPT, TERMINATE, DEPLOY MISSION) have no event handlers
- The second agent card (AGENT_ZETA_0) represents no real agent and should be removed

## Depends on

`VOICE_CONTEXT_EXTRACTION` must be completed first. This work item assumes `VoiceContext` is provided at the Agents page level and all voice state is accessible via `use_context::<VoiceContext>()`.

## Files affected

| File | Change |
|------|--------|
| `crates/lx-desktop/src/pages/agents/mod.rs` | Rename title, replace two mocked cards with single `AgentCard {}`, reorganize layout, remove `AgentStatus` import |
| `crates/lx-desktop/src/pages/agents/agent_card.rs` | Rewrite: zero props, read VoiceContext, display transcript as live output, wire TERMINATE button |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | Remove transcript rendering section, keep only compact control bar |
| `crates/lx-desktop/src/voice_backend.rs` | Make `SESSION_ID` pub |

## Task List

### Task 1: Make SESSION_ID pub

In `crates/lx-desktop/src/voice_backend.rs`, change line 11 from:

`static SESSION_ID: LazyLock<String>`

to:

`pub static SESSION_ID: LazyLock<String>`

### Task 2: Strip transcript display from VoiceBanner

In `crates/lx-desktop/src/pages/agents/voice_banner.rs`, remove these lines from the RSX block:

1. Remove `let entries = ctx.transcript.read().clone();` (the clone used for rendering).

2. Remove the entire `if !entries.is_empty()` block (the scrollable div that renders transcript entries with "You:" / "Agent:" prefixes).

VoiceBanner after this change renders only: the status bar div (icon, status text, volume bars placeholder, push to talk button) and the hidden widget mount div. It is a compact single-row control bar.

### Task 3: Rewrite agent_card.rs

Replace the entire contents of `crates/lx-desktop/src/pages/agents/agent_card.rs`.

The new file has these imports:
```
use dioxus::prelude::*;
use super::voice_context::{VoiceContext, VoiceStatus};
use crate::terminal::status_badge::{BadgeVariant, StatusBadge};
```

Delete the `AgentStatus` enum entirely.

The `AgentCard` component takes zero props. It reads state via `let ctx = use_context::<VoiceContext>();`.

Derive these local variables at the top of the component:
- `let status = ctx.status();`
- `let is_active = status != VoiceStatus::Idle;`
- `let stage = ctx.pipeline_stage();`
- `let entries = ctx.transcript.read();`
- `let turn_count = entries.iter().filter(|e| e.is_user).count();`
- `let border_class = if is_active { "border border-[var(--primary)]/60" } else { "border border-[var(--primary)]/30" };`

The card renders a `div` with class `"bg-[var(--surface-container)] rounded-lg p-4 {border_class}"` containing:

1. **Header row** (`div` with `class: "flex items-center gap-3 mb-3"`):
   - Status dot: `span { class: "text-[var(--primary)]", "\u{25CF}" }`
   - Agent name: `span { class: "font-semibold uppercase text-sm tracking-wider text-[var(--on-surface)]", "VOICE_AGENT" }`
   - Status badge: Map voice status to badge — `VoiceStatus::Idle` → `BadgeVariant::Idle` with label `"IDLE"`, all others → `BadgeVariant::Active` with label from `status.to_string()`.

2. **Info row** (only render `if is_active`): A `div` with `class: "flex gap-4 text-xs mb-3"` containing two field groups:
   - Left field: label `"PIPELINE_STAGE"` in `text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1`, value `"{stage}"` in `text-[var(--on-surface-variant)] uppercase`.
   - Right field: label `"TURNS"`, value `"{turn_count}"`.

3. **Conversation section** (only render `if !entries.is_empty()`): Label div `"CONVERSATION"` in `text-[10px] uppercase tracking-wider text-[var(--outline)] mb-2`. Then a scrollable output div with `class: "bg-[var(--surface-container-low)] rounded p-3 font-mono text-xs max-h-64 overflow-y-auto"`. Iterate `entries.iter()`:
   - User entries: `p { class: "mb-0.5 text-[#64b5f6]", "\u{203A} YOU: {entry.text}" }`
   - Agent entries: `p { class: "mb-0.5 text-[var(--success)]", "\u{203A} AGENT: {entry.text}" }`

4. **TERMINATE button** (only render `if is_active`): A `div` with `class: "mt-3"` containing:
   ```
   button {
     class: "w-full border border-[var(--outline)] text-[var(--on-surface)] rounded py-2 text-xs uppercase tracking-wider hover:bg-[var(--surface-container-high)] transition-colors duration-150",
     onclick: move |_| {
       if let Some(w) = ctx.widget() {
         w.send_update(serde_json::json!({ "type": "stop_capture" }));
       }
     },
     "TERMINATE"
   }
   ```

The file needs `serde_json` for the json macro in the onclick handler.

### Task 4: Rewrite mod.rs page layout

Replace the contents of `crates/lx-desktop/src/pages/agents/mod.rs`.

New module declarations and imports:
```
mod agent_card;
mod mcp_panel;
mod voice_banner;
mod voice_context;

use dioxus::prelude::*;

use self::agent_card::AgentCard;
use self::mcp_panel::McpPanel;
use self::voice_banner::VoiceBanner;
use self::voice_context::VoiceContext;
```

Note: `AgentStatus` import is removed. It no longer exists.

The `Agents` component:
1. First line: `use_context_provider(VoiceContext::new);`
2. Then `let ctx = use_context::<VoiceContext>();`
3. Compute `let session_short = &crate::voice_backend::SESSION_ID[..8];`
4. Compute `let status_text = ctx.status().to_string();`

RSX layout (inside the existing outer div with `"flex flex-col h-full gap-4 p-4 overflow-auto"`):

1. **Header**: Same structure as current but with dynamic content:
   - Title: `"AGENT_MANAGER"` (replaces "MCP_MANAGER")
   - Subtitle: `"SESSION: {session_short}"` (replaces "ENVIRONMENT: PRODUCTION // ACCESS_LEVEL: ROOT")
   - Right side: `"STATUS: {status_text}"` (replaces "UPTIME: 1842:12:04")

2. **VoiceBanner**: `VoiceBanner {}` — compact control bar.

3. **AgentCard**: `AgentCard {}` — single full-width card. No wrapper div with `"flex gap-4"`. Just `AgentCard {}` directly.

4. **McpPanel**: `McpPanel {}` — unchanged.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

- **VoiceContext is accessed via `use_context::<VoiceContext>()`.** Never pass signals as props.
- **`ctx.widget()` returns `Option<TsWidgetHandle>`.** The `()` call on a Signal clones the inner value. Since `Option<TsWidgetHandle>` is `Copy`, this is efficient. Do NOT use `.read()` in onclick handlers — it returns a `Ref` which cannot be held across await points or closure boundaries.
- **Do not modify voice_context.rs.** That file was created in the previous work item.
- **Do not modify mcp_panel.rs.** MCP wiring happens in a separate work item.
- **Do not modify any TypeScript files.**
- **AgentCard takes zero props.** All state comes from context.
- **300 line file limit.** agent_card.rs should be under 100 lines. mod.rs should be under 50 lines.
- **`SESSION_ID` is `LazyLock<String>`.** It derefs to `String` which derefs to `str`. Slicing with `[..8]` is valid.

---

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AGENT_PAGE_WIRING.md" })
```

Then call `next_task` to begin.
