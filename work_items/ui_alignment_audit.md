# UI Alignment Audit: lx-desktop vs Paperclip

---

## A. Visual Fidelity Gaps

### A.1 Color System

| Gap | Paperclip | lx-desktop | Impact |
|-----|-----------|------------|--------|
| Theme class convention | `.dark` class toggle, `:root` = light defaults | `.dark` default, `.light` class for light mode | Tailwind `dark:` prefixes won't match. Not a problem since lx uses `var(--*)` everywhere, but means no Tailwind dark-mode utilities work. |
| Sidebar tokens | 6 dedicated `--sidebar-*` vars | None — sidebar uses generic surface tokens | Sidebar can't be themed independently |
| Color space | oklch (perceptually uniform) | Hex | Aesthetic only |

### A.2 Typography

| Concern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Page headings | `text-xl font-semibold` | `text-2xl font-bold uppercase tracking-wider` (`styles.rs:1`) |

Heading style is a deliberate lx design choice, not a bug.

### A.3 Animations & Transitions

| Gap | Paperclip | lx-desktop |
|-----|-----------|------------|
| Dialog close animation | `animate-out zoom-out-[0.97]` + `fade-out-0` on overlay | No close animation — unmounts instantly |
| Toast action links | `action: { label, href }` navigation toasts | No action link support |
| Streaming ping on tool blocks | Ping dot on in-progress tool/message blocks | Ping only on activity blocks (`transcript_blocks.rs:107-109`) |
| Kanban sort animation | @dnd-kit CSS transform animation during reorder | Instant repositioning |

### A.4 Icon Sizing

Inconsistent Material Symbols sizes across codebase:
- `kanban_card.rs:40`: `text-xs`
- `chart.rs:242`: `text-base`
- `drag_drop.rs:104`: `text-3xl`
- `wizard.rs:158-165`: `text-xl`
- `sidebar.rs`: `text-sm`

No convention established. Should be: `text-sm` default, `text-xs` inline, `text-lg`/`text-xl` hero.

---

## B. Component Completeness Gaps

### B.1 AgentConfigForm

Redesigned for lx (config panel not creation form). 4 sections: Source, Model & Backend, Tools, Channels.

| Gap | Detail | Size |
|-----|--------|------|
| No model search/popover | Plain text input (`config_form.rs:56-62`). Should use `Select { searchable: true }` | M |
| No env var / secrets section | Paperclip manages adapter secrets. lx equivalent = MCP server env vars | M |
| Source block read-only | `.lx` source as `<pre>` (`config_form.rs:22-23`). Should link to editor | S |

### B.2 RunTranscriptView

All 9 block types implemented. Collapsible groups, MarkdownBody, entry animations, ScrollToBottom.

| Gap | Paperclip reference | Size |
|-----|-------------------|------|
| No nice/raw mode toggle | `RunTranscriptView.tsx:19` — `mode?: "nice" \| "raw"` | S |
| No comfortable/compact density | `RunTranscriptView.tsx:20` — adjusts spacing + truncation thresholds (72 vs 120 chars) | S |
| No token counts | `formatTokens()` in tool block headers | S |
| No `summarizeToolInput()` | Extracts file paths, commands from tool args. lx shows raw input with CSS `truncate` | M |
| No `stripWrappedShell()` | Unwraps `bash -lc "..."` wrappers (`RunTranscriptView.tsx:160-180`) | S |
| No `parseStructuredToolResult()` | Parses `key: value` headers from tool results | S |
| No limit prop | `limit?: number` to cap entries | S |

### B.3 KanbanBoard

Within-column reorder, drag overlay, live issue pulsing all implemented.

| Gap | Detail | Size |
|-----|--------|------|
| No touch events | Mouse-only (`onmousedown`/`onmousemove`/`onmouseup`). No touch support | S |
| No sort animation | Instant repositioning vs @dnd-kit CSS transforms | S |
| Click fires after drag | `kanban_card.rs:36` — `onclick` fires even after drag. Should suppress if drag occurred | S |

### B.4 CommentThread

MarkdownEditor with file drop, 800ms debounced drafts, Cmd+Enter submit.

| Gap | Detail | Size |
|-----|--------|------|
| No timeline merging | Paperclip interleaves comments and run entries chronologically (`CommentThread.tsx:287-305`). lx context: interleave with event stream blocks | M |
| No per-comment copy button | Copy-to-clipboard per comment | S |
| No URL-based comment highlighting | `#comment-{id}` scroll + 3s highlight (`CommentThread.tsx:346-361`) | S |
| No reassignment dropdown | `InlineEntitySelector` to change assignee from thread | S |
| File drop only logs | `comment_thread.rs:91` logs files but doesn't upload/attach | M |
| @mentions not wired | `MarkdownEditor` accepts `mention_candidates` but `CommentThread` doesn't pass agent candidates | S |

### B.5 MarkdownEditor

Three modes, toolbar, cursor insertion, @mention popup, file drag-drop, auto-grow textarea.

| Gap | Paperclip reference | Size |
|-----|-------------------|------|
| No image upload handler | Inserts `upload://` placeholder links (`drag_drop.rs:90`) that don't resolve. Needs actual upload | M |
| No code syntax highlighting | Paperclip uses CodeMirror with 17+ language grammars. lx has plain `<pre>` via `pulldown_cmark` | L |
| Mention inserts plain text | `@Name` text (`markdown_editor.rs:93`) vs Paperclip's `[@Name](agent:ID#ICON)` rich link | S |
| No mention chip rendering | Paperclip renders mentions as styled chips. lx shows plain text | M |
| No bordered/borderless modes | Paperclip: `bordered?: boolean`. lx always bordered | S |

### B.6 ScheduleEditor — Complete

No gaps.

### B.7 OrgChart

Tree layout, pan/zoom, SVG edges, auto-fit, lateral edges with labels.

| Gap | Detail | Size |
|-----|--------|------|
| Stepped edge paths | L-shaped (`chart.rs:182`) vs Paperclip bezier curves | S |
| No adapter type in cards | Paperclip shows adapter label. lx shows name + role only | S |
| No click-to-navigate | Cards are non-interactive. Should link to agent detail | S |
| No minimap | Paperclip has optional minimap for large charts | M |

### B.8 GoalTree — Complete

No gaps.

### B.9 FilterBar — Complete

No gaps.

### B.10 NewIssueDialog

MarkdownEditor, custom Selects, localStorage draft, Cmd+Enter submit.

| Gap | Detail | Size |
|-----|--------|------|
| No file staging | Paperclip has drag-drop file attachments with MIME restrictions | M |
| No project selector | Paperclip assigns issue to project. lx equivalent: assign to flow | S |
| No focus trap | Dialog doesn't trap Tab | S |
| No draft debounce | Saves per render cycle. Should use 800ms debounce like `comment_thread.rs:19-30` | S |

### B.11 OnboardingWizard

4-step wizard, step tabs, Cmd+Enter, back/next, loading state.

| Gap | Detail | Size |
|-----|--------|------|
| No adapter radio cards | Paperclip shows 9 adapters as visual cards with badges. lx uses plain Select | M |
| No env testing | Paperclip tests adapter environment. lx has no test step | M |
| No API integration | Launch step shows preview but doesn't create anything | L |

---

## C. Interaction Pattern Gaps

### C.1 Keyboard Navigation

| Gap | Detail | Status |
|-----|--------|--------|
| Command palette arrow nav | `command_palette.rs` uses `Command` primitives but keyboard item selection unclear | Unverified |
| Focus trap in Select/Popover | Custom `Select` has keyboard nav but no Tab-focus trap | Partial |

### C.2 Click-Outside-to-Close

`Popover` (`popover.rs`) and `Tooltip` (`tooltip.rs`) lack click-outside-to-close. `Dialog`, `Select`, `DropdownMenu`, and `OnboardingWizard` all handle it correctly.

### C.3 Optimistic Updates — Architectural Gap

lx-desktop observes the event stream, doesn't write to it. No mutation layer exists yet. Expected — will be addressed when interpreter control API is built.

### C.4 Loading States

No per-component loading skeletons. `PageSkeleton` exists with 3 variants (`page_skeleton.rs`) but only used at app level, not within individual components.

### C.5 Form Patterns

| Gap | Detail |
|-----|--------|
| Draft debounce inconsistent | `comment_thread.rs` has 800ms debounce. `new_issue.rs` saves per-render |
| File drag-drop partial | Wired in MarkdownEditor + CommentThread. Not on NewIssueDialog |

---

## D. Wiring Translation Notes

### D.1 Agent Detail

Instead of Paperclip's REST API (`agentsApi.get`, `heartbeatsApi.runs`, `adapterModels`, `secretsApi`):

- **Agent metadata:** Parse from `.lx` AST (name, model, tools, channels). Static, source-defined.
- **Agent status:** `spawn_agent`/`stop_agent` events from `ActivityLog` (`chart_helpers.rs:13-25` already does this).
- **Messages:** `tell`/`ask`/`reply` events. Pending `ask` without `reply` = "awaiting response."
- **Tool calls:** `tool_call`/`tool_result` events. Active = unmatched `tool_call`.
- **Transcript:** `event_to_block()` in `transcript.rs:42-112` already maps event kinds to block variants.
- **Costs:** Sum `token_usage` from `tool_result` events, aggregate by agent.

Tab rename: Overview → Status, Runs → Transcript, Config → Source, Skills → Tools, Budget → Costs.

### D.2 Issues/Tasks

Instead of Paperclip's `issuesApi` CRUD:

- **Task definitions:** Parse `task` blocks from `.lx` AST. Name, agent assignment (from `run` block).
- **Task status:** Events `task_start` → in_progress, `task_complete` → done, `task_error` → blocked.
- **Kanban columns** = task lifecycle states.
- **Assignee** = agent in task's `run` block.
- **No creation dialog** for source-defined tasks. NewIssueDialog could be repurposed for ad-hoc runtime tasks via control API.

### D.3 Dashboard

Already wired to `ActivityLog`. Metrics computed: total/agent/tool/error events, activity buckets, event breakdown.

**Gap:** No live subscription. Dashboard updates on re-render but no push. Needs `use_future` polling loop or signal subscription on `ActivityLog.events`.

### D.4 Activity

`live_updates.rs` tails `$LX_EVENT_STREAM_PATH/events.jsonl`, dispatches to `ActivityLog`. `activity.rs` reads and filters.

**Gap:** File-based tailing has no push notification. Needs subscription for real-time updates.

### D.5 Runs/Transcripts

`event_to_block()` in `transcript.rs:42-112` maps lx events to transcript blocks. Handles `agent_start`/`agent_spawn` → Activity, `tool_call` → Tool(Running), `tool_result` → Tool(Complete/Error), `tell`/`ask`/`reply` → Message, `log` → Stdout, `error` → Event(Error).

Most complete wiring. Feeding real event stream data is the remaining step.

### D.6 Org Chart → Agent-Channel Topology

`chart_helpers.rs:8-58` parses topology from ActivityLog:
- `agent_start`/`agent_running`/`agent_spawn` → nodes
- `agent_reports_to` → vertical edges
- `tell`/`ask`/message → lateral communication edges (dashed, labeled)

Shows communication patterns, not hierarchy. Correct for lx's flat composition model.

**Enhancement:** Parse channel subscriptions from `.lx` AST for static topology (who *can* communicate) alongside dynamic (who *did* communicate).

### D.7 Costs

`pages/costs/` has Overview/Budgets/Providers tabs, MetricBox cards, BudgetCard, ProviderCard. All using mock data (`overview.rs:10-57`).

**Wiring needed:** Parse `token_usage` from `tool_result` events. Aggregate by agent/tool/model.

### D.8 Routines

ScheduleEditor UI complete. `pages/routines/` has list, detail, types.

**Wiring needed:** Parse `use cron` / `cron.schedule()` from `.lx` source.

---

## E. Prioritized Fix List

### High

| # | Files | Gap | Fix | Size |
|---|-------|-----|-----|------|
| 1 | `transcript.rs`, `transcript_blocks.rs` | No mode/density toggle, no `summarizeToolInput()` | Add mode/density props. Extract file paths from tool args, strip shell wrappers. Add `limit` prop | M |
| 2 | `config_form.rs:56-62` | Plain text input for model ID | `Select { searchable: true }` with known model list | S |
| 3 | `comment_thread.rs:91` | File drop logs but doesn't upload | Save to temp dir, insert real path instead of `upload://` | M |
| 4 | `comment_thread.rs` | @mentions not wired | Pass agent list from context as `mention_candidates` | S |
| 5 | `dialog.rs` | No close animation | `animate-dialog-content-out` keyframe, delay unmount 200ms | S |

### Medium

| # | Files | Gap | Fix | Size |
|---|-------|-----|-----|------|
| 6 | `new_issue.rs` | No draft debounce | Copy pattern from `comment_thread.rs:19-30` | S |
| 7 | `tailwind.css` | No `--sidebar-*` tokens | Add 6 sidebar vars mapped to surface-container variants | S |
| 8 | Multiple | Inconsistent icon sizes | Sweep: `text-sm` default, `text-xs` inline, `text-lg` hero | S |
| 9 | `chart.rs:182` | Stepped L-shape edges | Bezier: `M x1 y1 C x1 mid x2 mid x2 y2` | S |
| 10 | `chart.rs` | Cards don't navigate | Add `onclick`/`Link` to agent detail route | S |
| 11 | `markdown_editor.rs:93` | Mention inserts plain `@Name` | Insert `[@Name](agent:ID)` structured link | S |
| 12 | `kanban_card.rs:36` | Click fires after drag | Track `was_dragging`, suppress click if drag within 200ms | S |
| 13 | `toast.rs` | No action link on toasts | Add `action: Option<ToastAction>` with label + href | S |
| 14 | Multiple | Per-component skeletons unused | Wire `PageSkeleton` variants into pages during async loads | M |

### Low

| # | Files | Gap | Fix | Size |
|---|-------|-----|-----|------|
| 15 | `chart.rs` | No adapter type in org cards | Add `adapter` field to `OrgNode`, render below role | S |
| 16 | `new_issue.rs` | No file staging | `ondragover`/`ondrop` for attachment staging | M |
| 17 | `popover.rs`, `tooltip.rs` | No click-outside-to-close | Add `fixed inset-0` overlay | S |
| 18 | `onboarding/step_agent.rs` | Plain Select for adapter | Visual radio cards with icons | M |
| 19 | `onboarding/wizard.rs` | No env testing step | Adapter test button for API key / CLI validation | M |
| 20 | `kanban.rs` | No touch events | Add `ontouchstart`/`ontouchmove`/`ontouchend` | S |
