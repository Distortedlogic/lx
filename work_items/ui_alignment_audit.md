# UI Alignment Audit: lx-desktop vs Paperclip

## A. Visual Fidelity Gaps

### A.1 Color System Mismatch

Paperclip uses oklch-based CSS variables with a full light/dark dual-theme system. lx-desktop uses a single dark theme with raw hex CSS variables.

| Concern | Paperclip | lx-desktop | File |
|---------|-----------|------------|------|
| Color space | oklch (perceptually uniform) | Raw hex (#0e0e0e) | `tailwind.css:17-45` |
| Light mode | Full light theme via `:root` | None | `tailwind.css` (no `.dark` class toggle) |
| Variable naming | Semantic tokens (`--background`, `--foreground`, `--card`, `--muted`, `--accent`) | Material-style surface tokens (`--surface`, `--surface-container-*`, `--on-surface`) | `tailwind.css:17-45` |
| Chart colors | `--chart-1` through `--chart-5` for data vis | Only 3 chart vars (`--color-chart-axis`, `--color-chart-split`, `--color-chart-tooltip`) | `tailwind.css:42-44` |
| Sidebar colors | Dedicated `--sidebar-*` tokens | None | - |

**Impact:** The lx-desktop variable naming doesn't align with Tailwind's semantic token expectations. Components like `components/ui/button.rs:31-43` reference `bg-primary`, `text-primary-foreground`, `bg-accent`, `bg-destructive` etc. — these Tailwind semantic classes won't resolve unless the CSS variables match Tailwind's expected `--color-*` names, or the theme is configured to map them. The button component was copied verbatim from Paperclip's shadcn/ui but the theme variables it depends on don't exist in `tailwind.css`.

### A.2 Hardcoded Colors vs Theme Variables

Many lx-desktop components use raw Tailwind color classes instead of CSS variables, breaking theme consistency.

| File | Line | Hardcoded | Should Be |
|------|------|-----------|-----------|
| `layout/sidebar.rs` | 8 | `border-gray-700/50` | `border-[var(--outline-variant)]/30` |
| `layout/sidebar.rs` | 10 | `text-white` | `text-[var(--on-surface)]` |
| `layout/sidebar.rs` | 87 | `text-gray-500` | `text-[var(--outline)]` |
| `layout/sidebar.rs` | 99 | `bg-white/10 text-white` (active) | `bg-[var(--surface-container-high)] text-[var(--on-surface)]` |
| `layout/sidebar.rs` | 100 | `text-gray-400 hover:bg-white/5 hover:text-white` | `text-[var(--on-surface-variant)] hover:bg-[var(--surface-container)] hover:text-[var(--on-surface)]` |
| `components/comment_thread.rs` | 35 | `text-gray-400` | `text-[var(--outline)]` |
| `components/comment_thread.rs` | 39 | `border-gray-700` | `border-[var(--outline-variant)]/30` |
| `components/comment_thread.rs` | 46 | `text-gray-400` | `text-[var(--outline)]` |
| `components/comment_thread.rs` | 56 | `bg-gray-800 border-gray-600` | `bg-[var(--surface-container)] border-[var(--outline-variant)]` |
| `components/comment_thread.rs` | 63 | `bg-blue-600 hover:bg-blue-500` | `bg-[var(--primary)] text-[var(--on-primary)]` |
| `components/filter_bar.rs` | 19 | `bg-gray-700` | `bg-[var(--surface-container-high)]` |
| `components/filter_bar.rs` | 20 | `text-gray-400` | `text-[var(--outline)]` |
| `components/filter_bar.rs` | 23 | `hover:bg-gray-600` | `hover:bg-[var(--surface-bright)]` |
| `components/filter_bar.rs` | 33 | `text-gray-400 hover:text-white` | `text-[var(--outline)] hover:text-[var(--on-surface)]` |
| `pages/dashboard/mod.rs` | 71 | `text-gray-400` | `text-[var(--outline)]` |
| `pages/dashboard/mod.rs` | 74 | `border-gray-700 divide-y divide-gray-700` | `border-[var(--outline-variant)]/30 divide-y divide-[var(--outline-variant)]/30` |
| `pages/dashboard/mod.rs` | 79 | `text-gray-400` | `text-[var(--outline)]` |
| `pages/dashboard/mod.rs` | 84 | `text-gray-500` | `text-[var(--outline)]/60` |

### A.3 Border Radius

lx-desktop sets all radii to 0rem (`tailwind.css:8-14`), giving everything sharp corners. Paperclip uses Tailwind default radii (`rounded-md`, `rounded-lg`, `rounded-full`). This is an intentional design choice for lx-desktop's industrial aesthetic, but creates inconsistency: some components still use `rounded-lg`, `rounded-md`, `rounded-full` classes (e.g., `pages/org/chart.rs:152`, `pages/agents/config_form.rs:94`, `components/comment_thread.rs:39`) which resolve to `0rem` anyway. The `rounded-full` on status dots and toggle switches (`config_form.rs:109`, `styles.rs:3-6`) needs an explicit pixel-based override to remain circular — without it, status dots render as squares.

### A.4 Typography

| Concern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Font stack | System fonts (shadcn default) | Space Grotesk (display), Inter (body), JetBrains Mono (mono) — `tailwind.css:6-7` |
| Font loading | Bundled with build | `@theme` declaration only — no `@font-face` or CDN link visible |
| Text sizes | `text-sm` (14px) base | Same, but `PAGE_HEADING` uses `text-2xl` uppercase tracking-wider (`styles.rs:1`) |

Font loading is a gap — if the fonts aren't loaded via HTML `<link>` tags or `@font-face` rules in the Dioxus HTML template, they'll fall back to sans-serif.

### A.5 Missing Animations & Transitions

| Animation | Paperclip | lx-desktop |
|-----------|-----------|------------|
| Activity row entry | `@keyframes activity-row-enter` with opacity+background fade (980ms) | None — `pages/dashboard/mod.rs:76` has only `hover:bg-white/5 transition-colors` |
| Status dot pulse | `animate-pulse` on running agents | Not applied — `styles.rs:3` defines `STATUS_DOT_ACTIVE` without pulse |
| Drag overlay shadow | `shadow-lg ring-1 ring-primary/20` during kanban drag | No drag support at all (`pages/issues/kanban.rs`) |
| Toast entry/exit | Slide-in animation with opacity | No animation visible in `contexts/toast.rs` or `components/toast_viewport.rs` |
| Collapsible expand | Height transition | GoalTree chevron rotates (`pages/goals/tree.rs:58`) but no height animation on children |
| Dialog backdrop | Fade-in animation | Instant show/hide (`pages/issues/new_issue.rs:22`, `components/onboarding/wizard.rs:82`) |

### A.6 Icon System

Paperclip uses Lucide React icons (SVG components). lx-desktop uses Material Symbols Outlined (icon font via class name). The icon font approach works but:
- No guarantee the Material Symbols font is loaded (same font-loading gap)
- Icon names don't always match between systems (Lucide `CircleDot` vs Material `circle`, Lucide `SquarePen` vs no equivalent)
- The icon font renders at inconsistent sizes — some use `text-sm`, `text-base`, `text-xs`, `text-lg`, `text-xl` across files with no consistent sizing convention

---

## B. Component Completeness Gaps

### B.1 AgentConfigForm

| Feature | Paperclip (`components/AgentConfigForm.tsx`, ~550 lines) | lx-desktop (`pages/agents/config_form.rs`, 115 lines) |
|---------|-----------|------------|
| Adapter-specific fields | Different field sets per adapter (Claude, Codex, Gemini, etc.) with dedicated config-fields.tsx per adapter | Single model text input for all adapters |
| Section layout modes | `"inline"` (border-b dividers) and `"cards"` (bordered sections) | Cards only |
| Dirty tracking | Overlay pattern comparing identity, adapterType, adapterConfig, heartbeat, runtime sections independently | Single boolean `dirty` signal |
| Environment testing | "Test Environment" button with loading/error/success states | None |
| Runtime config | Max concurrent runs, auto-pause, timeout, resource limits | None |
| Instructions file | File picker for agent instructions path | None |
| Prompt template | Template editor section | None |
| Create vs Edit modes | Discriminated union: `mode: "create"` vs `mode: "edit"` with different prop shapes | Single mode (edit only) |
| Save handler | `onSave(patch)` wired to API mutation | `dirty.set(false)` — no actual save |
| Input class | Shared constant with `font-mono placeholder:text-muted-foreground/40` | Uses `INPUT_FIELD` constant (consistent) |

### B.2 RunTranscriptView

| Feature | Paperclip (`components/transcript/RunTranscriptView.tsx`, ~400 lines) | lx-desktop (`pages/agents/transcript.rs`, 111 lines) |
|---------|-----------|------------|
| Block types | message, thinking, tool, activity, command_group, tool_group, stderr_group, stdout, event | message, thinking, tool_use, event |
| Display modes | `"nice"` and `"raw"` modes | Single mode |
| Density | `"comfortable"` and `"compact"` | Single density |
| Streaming indicator | Animated indicator for in-progress messages | None |
| Collapse/expand | Collapsible thinking blocks, tool groups, stderr groups | None |
| Copy button | Per-message copy-to-clipboard | None |
| Markdown rendering | MarkdownBody for message content | Plain text `whitespace-pre-wrap` |
| Token counts | `formatTokens()` display | None |
| Tool references | Linked tool names | Plain text tool names |
| Limit prop | Configurable entry limit | None |
| Empty message | Customizable via prop | Hardcoded "No transcript data available." |
| Live data | Connected to `useLiveRunTranscripts` hook | Hardcoded demo data (`transcript.rs:13-18`) |

### B.3 KanbanBoard

| Feature | Paperclip (`components/KanbanBoard.tsx`, 150+ lines) | lx-desktop (`pages/issues/kanban.rs`, 101 lines) |
|---------|-----------|------------|
| Drag-and-drop | Full @dnd-kit integration: PointerSensor, sortable within columns, draggable between columns, DragOverlay | None — cards are clickable buttons only |
| Drop zone feedback | `bg-accent/40` on active drop, `bg-muted/20` inactive | Static `bg-[var(--surface-container)]/20` |
| Drag visual | `opacity-30` on source, `shadow-lg ring-1 ring-primary/20` on overlay | N/A |
| Live status | `liveIssueIds` set with pulsing indicator on active cards | None |
| Card hover | `hover:shadow-sm` with transition | `hover:shadow-sm transition-shadow` (present) |
| Status change | On drop between columns triggers `onUpdateIssue` | `on_status_change` handler exists in props but never called by UI |
| Column header | Status icon + label + count | Status icon + label + count (matches) |

### B.4 CommentThread

| Feature | Paperclip (`components/CommentThread.tsx`, ~200 lines) | lx-desktop (`components/comment_thread.rs`, 77 lines) |
|---------|-----------|------------|
| Timeline merging | Merges comments and runs into single timeline sorted by timestamp | Comments only, no run items |
| Draft persistence | localStorage-based with 800ms debounce, customizable draft key | None |
| Editor | MarkdownEditor with image upload, @mention autocomplete | Plain textarea |
| Reopen checkbox | Checkbox to reopen issue when commenting | None |
| Reassign dropdown | Agent/user reassignment on comment | None |
| Cmd+Enter submit | Keyboard shortcut in editor | None |
| Copy markdown | Per-comment copy button | None |
| Image upload | File attachment handler | None |
| Run items in timeline | Shows run status, agent link, cost info | None |
| Identity component | Full identity with avatar, agent icon | Simple Identity with name only |

### B.5 MarkdownEditor

| Paperclip (`components/MarkdownEditor.tsx`, ~350 lines) | lx-desktop |
|-----------|------------|
| MDXEditor with CodeMirror, headings, lists, links, quotes, tables, images, markdown shortcuts | **Does not exist** — only `components/markdown_body.rs` (read-only renderer) |
| @mention autocomplete with chip decoration | N/A |
| Image upload handler | N/A |
| Cmd+Enter submit | N/A |
| Code block syntax highlighting with 17+ language support | N/A |

This is the largest single component gap.

### B.6 ScheduleEditor

| Feature | Paperclip (`components/ScheduleEditor.tsx`, ~344 lines) | lx-desktop (`pages/routines/schedule_editor.rs`, 280 lines) |
|---------|-----------|------------|
| Presets | 7 presets (every_minute through custom) | 7 presets (matches) |
| Time pickers | Hour (0-23), Minute (0-55 in 5-min) | Hour (0-23), Minute (0-55 in 5-min) (matches) |
| Day of week picker | Checkbox-style selection | Button-style selection (visual difference but functional) |
| Human-readable description | `describeSchedule(cron)` → "Every day at 10:00 AM" | None — no schedule description display |
| Ordinal suffixes | "1st", "2nd", "3rd" for monthly dates | None — just raw numbers |
| Cron parsing | `parseCronToPreset(cron)` bidirectional | `parse_cron_to_preset` in `cron_utils.rs` (present) |

Closest to complete among all ported components.

### B.7 OrgChart

| Feature | Paperclip (`pages/OrgChart.tsx`, 150+ lines) | lx-desktop (`pages/org/chart.rs`, 175 lines) |
|---------|-----------|------------|
| Layout algorithm | `subtreeWidth()`, `layoutTree()`, `layoutForest()` | `chart_layout.rs` — same algorithm ported |
| Pan | Mouse drag pan | Mouse drag pan (implemented) |
| Zoom | +/- buttons | +/- buttons + Fit reset (implemented) |
| Node cards | 200x100px, agent name, title, status dot | Same dimensions, name, role, status dot |
| Status colors | cyan/green/yellow/red/gray matching Paperclip | Exact same hex colors (`chart.rs:28-35`) |
| SVG edges | Vertical orthogonal lines | Vertical orthogonal lines (matches) |
| Data source | REST API for org tree | Hardcoded sample data (`default_org_nodes()` at line 9) |

Layout and interaction matches well. Main gap is real data.

### B.8 GoalTree

| Feature | Paperclip (`components/GoalTree.tsx`, 118 lines) | lx-desktop (`pages/goals/tree.rs`, 84 lines) |
|---------|-----------|------------|
| Recursive nodes | GoalNode with collapse/expand | GoalNode with collapse/expand (matches) |
| Indentation | `paddingLeft: ${depth * 16 + 12}px` | `padding-left: {pad}px` where `pad = depth * 16 + 12` (matches) |
| Status badge | StatusBadge component | Inline text with `status_color()` (simpler) |
| Goal level display | `text-xs` level label | `text-[10px]` level label (close) |
| Hover state | `hover:bg-accent/50 transition-colors` | `hover:bg-white/5 transition-colors` (hardcoded color) |
| Link/callback | `goalLink` prop or `onSelect` callback | `Route::GoalDetail` link (always links) |

Structurally complete. Styling uses hardcoded colors.

### B.9 FilterBar

| Feature | Paperclip (`components/FilterBar.tsx`, 40 lines) | lx-desktop (`components/filter_bar.rs`, 40 lines) |
|---------|-----------|------------|
| Filter badge | `Badge variant="secondary"` | `span` with `bg-gray-700` (hardcoded, not using Badge component) |
| Clear button | `Button variant="ghost" size="sm"` | Plain `button` with inline classes |
| X icon | Lucide `X` SVG | Material `close` icon font |
| Layout | `flex items-center gap-2 flex-wrap` | Same (matches) |

Functionally equivalent but doesn't use its own UI primitives (Badge, Button).

### B.10 NewIssueDialog

| Feature | Paperclip (`components/NewIssueDialog.tsx`, ~400 lines) | lx-desktop (`pages/issues/new_issue.rs`, 116 lines) |
|---------|-----------|------------|
| Draft persistence | localStorage with debounce | None |
| File staging | Image, PDF, markdown, JSON, CSV, HTML uploads with drag-and-drop | None |
| Description editor | MarkdownEditor | Plain textarea |
| Model overrides | Codex/OpenCode thinking effort levels | None |
| Execution workspace | Isolated vs shared workspace selection | None |
| Document upload | Drag-and-drop with preview | None |
| Expand/minimize | Modal size toggle | None |
| Form fields | Title, description, status, priority, assignee, project, workspace, model override | Title, description, status, priority, assignee |
| Close icon | Lucide X in button | Plain "x" text character (`new_issue.rs:33`) |

### B.11 OnboardingWizard

| Feature | Paperclip (`pages/OnboardingWizard.tsx`, 150+ lines) | lx-desktop (`components/onboarding/wizard.rs`, 197 lines) |
|---------|-----------|------------|
| Steps | Company → Agent → Task → Project/Goal | Company → Agent → Task → Launch |
| Adapter selection | Full adapter type selector with model picker, search filtering | None — step_agent only has name input |
| Environment testing | Test button with loading/error states | None |
| Auto-grow textarea | Dynamic height textarea | None |
| Popover model picker | Searchable model selection | None |
| API key management | Anthropic key unset option | None |
| Auto-select company | Selects first company on success | Close wizard only |
| Cache invalidation | React Query invalidation on complete | None |

Structure matches but the content-rich steps (especially agent config) are simplified to basic text inputs.

---

## C. Interaction Pattern Gaps

### C.1 Keyboard Navigation

| Pattern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Cmd+K command palette | `useKeyboardShortcuts` hook dispatches event, CommandPalette catches it | `components/command_palette.rs` exists but no keyboard shortcut hook wiring visible |
| Cmd+Enter submit | MarkdownEditor `onSubmit` prop | Not implemented (no MarkdownEditor) |
| Arrow key list navigation | Used in command palette, select dropdowns | Not implemented |
| Escape to close | Dialog components handle Escape | Not implemented — dialogs close on backdrop click only |
| Enter to confirm | Inline editor commit | `IssueDetailPage` (`pages/issues/detail.rs:45-48`) handles Enter on title edit — partial |

### C.2 Drag-and-Drop

Kanban drag-and-drop is completely absent. Paperclip uses `@dnd-kit/core` + `@dnd-kit/sortable` with:
- PointerSensor (5px activation threshold)
- Per-column vertical list sorting
- Cross-column card moves
- DragOverlay for visual feedback
- `opacity-30` on dragged source

lx-desktop's `KanbanBoardView` (`pages/issues/kanban.rs`) accepts `on_status_change: EventHandler<(String, String)>` but never triggers it — there's no mechanism to change a card's column.

Sidebar project/agent reordering (drag-to-reorder in Paperclip via `SidebarProjects.tsx`) is also absent.

### C.3 Click-Outside-to-Close

Paperclip dialogs use portal-based rendering with click-outside detection. lx-desktop uses backdrop `onclick` (`new_issue.rs:24`, `wizard.rs:83`) with `stop_propagation` on the dialog body — this works for modals but:
- Popovers and dropdowns don't have click-outside behavior
- Command palette doesn't close on outside click
- Select/dropdown components rely on native `<select>` which handles its own close

### C.4 Toast Auto-Dismiss

`contexts/toast.rs` defines TTLs (Info: 4s, Success: 3.5s, Warn: 8s, Error: 10s) and stores `created_at` timestamps, but there's no timer that calls `dismiss()`. The `ToastState::push()` method inserts toasts and truncates to 5, but no `use_future` or `spawn` sets up the auto-dismiss tick. Toasts will accumulate until pushed off by newer ones.

Paperclip's `ToastContext` has an active timer that removes expired toasts.

### C.5 Optimistic Updates

Paperclip uses React Query `onMutate` for optimistic updates — the UI reflects changes immediately before the server confirms. lx-desktop has no API integration at all, so this is moot currently, but the architecture for it is absent.

### C.6 Loading and Error States

| Pattern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Page loading | Skeleton components (shimmer placeholders) | `SuspenseBoundary` fallback shows "Loading..." text (`layout/shell.rs:91-93`) |
| Error boundary | Per-page error display | Single `ErrorBoundary` in shell (`layout/shell.rs:82-88`) |
| API loading | Per-query loading spinners, Loader2 icon animation | N/A (no API calls) |
| Empty states | Contextual messages per entity type | `EmptyState` component exists but only used in Dashboard |
| Form validation | Inline error messages, disabled submit on invalid | Minimal — only `disabled: title.read().trim().is_empty()` on NewIssueDialog |

### C.7 Live Updates

Paperclip has WebSocket-based live updates with:
- Toast notifications with cooldown (max 3 per 10s window)
- Optimistic React Query cache updates
- Reconnection with 2000ms suppress delay
- Routes-aware context tracking

lx-desktop's `LiveUpdatesProvider` (`contexts/live_updates.rs`) connects to `ws://127.0.0.1:8080/ws/events` with exponential backoff reconnection and pushes events to `ActivityLog` — but:
- No toast notifications on events
- No query/state invalidation on events
- No cooldown/throttling
- No routes-aware tracking
- Hardcoded URL (should be configurable)

### C.8 Scroll Behavior

Paperclip has `ScrollToBottom` component for auto-scrolling transcripts and comment threads (respects user scroll position). lx-desktop has no equivalent — long transcript views or comment threads won't auto-scroll to latest.

### C.9 Form Patterns

| Pattern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Auto-grow textarea | `element.style.height = element.scrollHeight + "px"` | None — textareas have fixed `min-h` |
| Debounced save | 800ms debounce on draft saves | None |
| localStorage drafts | Issue, comment, routine drafts persisted | None |
| File drag-and-drop | NewIssueDialog, MarkdownEditor | None |
| Inline editing | ContentEditable-based InlineEditor | Basic input swap on click (`pages/issues/detail.rs:39-57`) |

---

## D. Wiring Translation Notes

### D.1 Agent Detail

**Paperclip wiring:** REST API fetches agent by ID, heartbeat runs via `heartbeatsApi`, adapter config from adapter registry, budget from `budgetsApi`, live run tracking via 5-15s polling.

**lx-desktop should wire to:**
- Agent spawn events from `EventStream` — when a `spawn_agent` event appears in the JSONL, populate the agent detail
- Mailbox traffic: parse `tell`/`ask`/`reply` events to show message queue depth, pending `ask` replies, message history
- Active tool calls: parse `tool_call`/`tool_result` events to show in-progress and completed tool invocations
- No heartbeat concept — agents run continuously. Show uptime since spawn, current state (running/waiting/blocked)
- No adapter config — agents in lx use the interpreter's configured LLM backend. Config form should show the agent's `.lx` source definition instead
- Budget: sum token usage from `tool_result` events that include `usage` fields

**File:** `pages/agents/detail.rs` — the tab structure (Overview, Config, Runs, Skills, Budget) should become (Overview, Messages, Tools, Source, Costs)

### D.2 Issues/Tasks

**Paperclip wiring:** `issuesApi.list()`, `issuesApi.create()`, etc. Issues have statuses (backlog/todo/in_progress/in_review/blocked/done/cancelled), priorities, assignees.

**lx-desktop should wire to:**
- Parse `task` blocks from the loaded `.lx` program's AST — each `task` declaration becomes an item
- Task status comes from the event stream: no events yet = `pending`, `task_start` event = `running`, `task_complete` = `done`, `task_error` = `error`
- No issue creation dialog needed — tasks are defined in `.lx` source. The "new issue" concept maps to creating a new task block in the program editor (future)
- Kanban columns should map to lx task states, not Paperclip issue statuses
- Assignee = the agent that the task's `run` block delegates to

**Files:** `pages/issues/mod.rs`, `pages/issues/list.rs`, `pages/issues/kanban.rs`, `pages/issues/detail.rs`

### D.3 Dashboard

**Paperclip wiring:** `dashboardApi.summary()` for metrics, `activityApi.list()` for timeline, `issuesApi.list()` for recent tasks, `heartbeatsApi.list()` for chart data.

**lx-desktop should wire to:**
- Metrics from EventStream: agent count (count unique agent IDs in spawn events), message throughput (tell/ask events per minute), tool calls (tool_call events count), errors (events with error tone)
- Activity feed: tail the JSONL event stream, newest first
- Charts: aggregate events by type over time windows from the event stream ring buffer
- "Active agents" panel: agents with spawn but no stop event

**File:** `pages/dashboard/mod.rs` — the current implementation already reads from `ActivityLog` context, which is fed by `LiveUpdatesProvider`. The gap is that `LiveUpdatesProvider` connects to a Paperclip-style WebSocket at `ws://127.0.0.1:8080/ws/events` instead of reading from the lx interpreter's EventStream.

### D.4 Activity

**Paperclip wiring:** `activityApi.list(companyId)` with pagination, filtering by type.

**lx-desktop should wire to:**
- `EventStream.xread()` — the lx interpreter exposes an event stream that can be tailed
- Each event has: timestamp, event_type, agent_id, payload
- Filter by event type (spawn, tell, ask, reply, tool_call, tool_result, error, log)
- No REST API — read directly from the in-memory ring buffer or JSONL file

**File:** `pages/activity.rs`, `contexts/live_updates.rs` — the live_updates provider should be rewritten to consume lx's EventStream rather than a WebSocket

### D.5 Runs/Transcripts

**Paperclip wiring:** `heartbeatsApi.runs(agentId)` fetches run history, `RunTranscriptView` renders parsed transcript blocks from the run's log.

**lx-desktop should wire to:**
- lx doesn't have "runs" in Paperclip's sense (heartbeat wake/execute/sleep cycles). Instead, show program execution traces
- A "run" in lx = one execution of a `.lx` program from start to finish
- Transcript blocks come from JSONL events: `log` events → message blocks, `tool_call`/`tool_result` → tool blocks, errors → event blocks
- The `TranscriptView` component (`pages/agents/transcript.rs`) has the right block types but needs to parse from JSONL instead of using hardcoded demo data

**File:** `pages/agents/transcript.rs:13-18` — replace hardcoded `vec![]` with JSONL event parsing

### D.6 Org Chart

**Paperclip wiring:** REST API for org tree (agent hierarchy with `reports_to` relationships).

**lx-desktop should wire to:**
- Parse agent-channel topology from the running program
- Nodes = agents defined in the `.lx` source
- Edges = channel subscriptions and direct tell/ask connections
- No hierarchy — lx agents don't have reporting lines. The graph is a communication topology, not an org chart
- Could also show flow composition: which flow contains which agents

**File:** `pages/org/chart.rs:9-14` — replace `default_org_nodes()` with runtime topology. The `OrgNode` type needs a rethink: instead of `reports_to: Option<String>`, use `connected_to: Vec<String>` with edge labels (channel name, tell/ask)

### D.7 Costs

**Paperclip wiring:** `costsApi.summary()`, `costsApi.byAgent()`, `costsApi.byProject()` — budget enforcement with hard stops.

**lx-desktop should wire to:**
- Parse token usage from `tool_result` events in the event stream that include `usage: { input_tokens, output_tokens }` fields
- Aggregate by agent, by flow, by tool
- No budget enforcement yet — display only
- Cost calculation: `(input_tokens * input_price + output_tokens * output_price)` using configured model pricing

**Files:** `pages/costs/overview.rs`, `pages/costs/provider_card.rs`

### D.8 Routines

**Paperclip wiring:** `routinesApi.list()`, `routinesApi.create()` — cron-scheduled recurring heartbeats with concurrency/catch-up policies.

**lx-desktop should wire to:**
- lx has a `cron` stdlib module — parse `use cron` declarations and `cron.schedule(expr, fn)` calls from the loaded `.lx` program
- Show cron expressions and their human-readable descriptions
- Run history from event stream (events triggered by cron callbacks)
- No concurrency/catch-up policies yet in lx's cron module

**File:** `pages/routines/schedule_editor.rs` — the editor UI is solid, just needs to read/write from `.lx` source definitions

---

## E. Prioritized Fix List

### Critical (blocks usability)

| # | File | What's Wrong | Fix | Size |
|---|------|-------------|-----|------|
| 1 | `tailwind.css` | UI primitive components (`components/ui/button.rs` etc.) reference Tailwind semantic tokens (`bg-primary`, `text-foreground`, `bg-accent`, etc.) that don't exist in the CSS variable definitions. Buttons, badges, cards likely render with missing/default colors. | Add Tailwind v4 `@theme` mappings that bridge lx-desktop's Material-style vars to Tailwind semantic names: `--color-primary: var(--primary)`, `--color-foreground: var(--on-surface)`, `--color-background: var(--surface)`, etc. | M |
| 2 | `tailwind.css:8-14` | All border radii = 0rem. Status dots (`styles.rs:3-6`), toggle switches (`config_form.rs:109`), avatar circles, filter badges all render as squares instead of circles. | Override `rounded-full` to use a fixed pixel value (`9999px`) or add explicit `border-radius` in the component styles for elements that must be circular. | S |
| 3 | `contexts/live_updates.rs:37` | Hardcoded `ws://127.0.0.1:8080/ws/events` — connects to a Paperclip server that doesn't exist in the lx stack. | Replace with lx EventStream consumer. Read from the interpreter's event ring buffer or watch the JSONL output file. This is the fundamental data source change. | L |
| 4 | `pages/agents/transcript.rs:13-18` | Hardcoded demo transcript data. `TranscriptView` never shows real data. | Wire to JSONL event stream parser. Parse events into `TranscriptBlock` variants. | M |

### High (visual/functional gaps)

| # | File | What's Wrong | Fix | Size |
|---|------|-------------|-----|------|
| 5 | Multiple files (see A.2) | ~20 instances of hardcoded gray-* Tailwind colors instead of CSS variables. Breaks theme consistency. | Replace all `gray-400/500/600/700/800` with corresponding `var(--outline)`, `var(--outline-variant)`, `var(--surface-container-*)` references. | M |
| 6 | `components/comment_thread.rs` | Plain textarea instead of MarkdownEditor. No draft persistence, no @mentions, no image upload, no Cmd+Enter. | Build a MarkdownEditor component for Dioxus. Start with basic markdown formatting toolbar + Cmd+Enter submit. Full MDXEditor equivalent is L, basic functional version is M. | L |
| 7 | `pages/issues/kanban.rs` | No drag-and-drop. Cards click but can't be moved between columns. | Implement pointer-event-based drag-and-drop in Dioxus: `onpointerdown`/`onpointermove`/`onpointerup` with visual overlay. The `on_status_change` handler already exists. | L |
| 8 | `contexts/toast.rs` | Toasts have TTLs defined but no auto-dismiss timer. Toasts accumulate forever. | Add a `use_future` in `ToastViewport` that ticks every 500ms and calls `dismiss()` on expired toasts (where `now - created_at > ttl_ms`). | S |
| 9 | `pages/org/chart.rs:9-14` | Hardcoded sample org data (Atlas, Nova, Orbit, Spark). | Wire to lx program's agent definitions. Parse agent and channel declarations from loaded `.lx` AST. | M |
| 10 | `pages/dashboard/mod.rs` | Activity list uses hardcoded gray colors. Charts show demo data from echarts JS. | Fix colors per A.2. Wire charts to aggregate event stream data. | M |

### Medium (polish and completeness)

| # | File | What's Wrong | Fix | Size |
|---|------|-------------|-----|------|
| 11 | `pages/agents/config_form.rs:78-80` | Save/Cancel buttons don't persist changes. `onclick: move \|_\| dirty.set(false)` just clears dirty flag. | Wire to actual state mutation or API. For lx, "saving" config means updating the agent's `.lx` source definition. | M |
| 12 | `pages/issues/new_issue.rs:33` | Close button is text "x" instead of icon. | Replace with `span { class: "material-symbols-outlined text-lg", "close" }` matching wizard.rs:94. | S |
| 13 | `components/filter_bar.rs:19` | Uses raw `span` with `bg-gray-700` instead of Badge component. | Use `crate::components::ui::badge::Badge` with variant secondary. | S |
| 14 | `layout/shell.rs:91-93` | Loading state is plain "Loading..." text. | Add a proper skeleton/spinner component matching Paperclip's skeleton placeholders. | S |
| 15 | `pages/agents/detail.rs:89` | Runs tab always shows `Vec::new()`. | Wire to event stream run data for the specific agent. | M |
| 16 | No file | Missing `ScrollToBottom` component — transcripts and comment threads don't auto-scroll. | Implement a container that tracks scroll position and auto-scrolls to bottom on new children, unless user has scrolled up. | M |
| 17 | `styles.rs:3` | `STATUS_DOT_ACTIVE` missing `animate-pulse` for running agents. Paperclip pulses running dots. | Add `animate-pulse` variant: `STATUS_DOT_RUNNING: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-cyan-400 animate-pulse"`. Also need `@keyframes pulse` in `tailwind.css` if not provided by Tailwind. | S |
| 18 | `components/onboarding/wizard.rs` | Onboarding step_agent only has name input — no adapter type, model selector, environment test. | For lx, onboarding should configure: interpreter backend, default model, MCP tool paths. Different from Paperclip but needs substance. | M |

### Low (nice-to-have)

| # | File | What's Wrong | Fix | Size |
|---|------|-------------|-----|------|
| 19 | `pages/routines/schedule_editor.rs` | Missing human-readable schedule description ("Every day at 10:00 AM"). | Port `describeSchedule()` from Paperclip's ScheduleEditor. Pure string formatting. | S |
| 20 | No file | No keyboard shortcut system. Cmd+K, Escape, arrow keys not wired. | Add `use_keyboard_shortcuts` hook using Dioxus `document::eval` or `onkeydown` on root. | M |
| 21 | No file | No dialog backdrop fade-in/out animation. | Add CSS `@keyframes fade-in` and apply to dialog backdrop divs. | S |
| 22 | No file | No activity row entry animation. | Port `@keyframes activity-row-enter` from Paperclip's index.css. | S |
| 23 | `pages/goals/tree.rs:48` | `hover:bg-white/5` instead of theme variable. | Change to `hover:bg-[var(--surface-container)]`. | S |
| 24 | Multiple | Several Paperclip features absent but not needed for lx: company brand color in sidebar, plugin slot outlets, SidebarProjects drag-reorder, SidebarAgents with live run counts. | Skip — these are Paperclip-specific features. lx-desktop should develop its own equivalents (e.g., flow list in sidebar, agent spawn counts). | - |
