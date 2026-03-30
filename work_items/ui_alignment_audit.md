# UI Alignment Audit: lx-desktop vs Paperclip (2026-03-30 revision)

This audit reflects the current state after 14 work item units were executed. Many gaps from the initial audit have been addressed. This revision identifies what was fixed, what remains, and what new gaps surfaced.

---

## A. Visual Fidelity Gaps

### A.1 Color System — Status: Partially Addressed

Paperclip uses oklch-based CSS variables with Tailwind semantic tokens (`--background`, `--foreground`, `--card`, `--muted`, `--accent`, `--sidebar-*`, `--chart-1` through `--chart-5`). lx-desktop maps Material Design 3 surface tokens to Tailwind semantic names in `src/tailwind.css:17-36`.

**Fixed since last audit:**
- `--radius-full` now correctly set to `9999px` (`tailwind.css:15`), so status dots, avatars, and toggle switches render as circles
- Hardcoded hex colors replaced with CSS variables (commits `3b5ac92c`, `d07abd4e`, `18339df9`)
- Theme mappings for `--color-background`, `--color-foreground`, `--color-card`, `--color-primary`, `--color-secondary`, `--color-muted`, `--color-accent`, `--color-destructive`, `--color-border`, `--color-input`, `--color-ring` all present

**Remaining gaps:**

| Gap | Paperclip | lx-desktop | Impact |
|-----|-----------|------------|--------|
| Light mode | Full light theme via `:root`, dark via `.dark` class toggle | Dark only — no `:root` light vars, no `.dark` toggle | Users in bright environments have no option |
| Sidebar tokens | `--sidebar-background`, `--sidebar-foreground`, `--sidebar-primary`, `--sidebar-accent`, `--sidebar-border`, `--sidebar-ring` | None — sidebar uses generic surface tokens | Sidebar cannot be styled independently |
| Chart colors | `--chart-1` through `--chart-5` (5 semantic chart vars) | `--color-chart-axis`, `--color-chart-split`, `--color-chart-tooltip` (3 infrastructure vars) | Multi-series charts can't differentiate series by color |
| Color space | oklch (perceptually uniform gradients) | Hex CSS variables | Minor — aesthetic difference only |

### A.2 Typography — Status: Unchanged

| Concern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Font stack | System fonts (shadcn default) | Space Grotesk (display), Inter (body), JetBrains Mono (mono) |
| Font loading | Bundled with build | Google Fonts CDN link in `app.rs:19-21` |
| Page headings | `text-xl font-semibold` | `text-2xl font-bold uppercase tracking-wider` via `styles.rs:1` |

**Risk:** Google Fonts CDN dependency means fonts fail offline. Desktop app should bundle fonts as static assets.

### A.3 Animations & Transitions — Status: Significantly Improved

**Fixed since last audit:**
- Toast entry/exit animations: `animate-toast-enter` and `animate-toast-exit` CSS keyframes in `tailwind.css:125-153`
- Dialog animations: `animate-dialog-overlay-in` (200ms fade) and `animate-dialog-content-in` (200ms scale+fade) in `tailwind.css:155-181`
- Transcript block entry: `animate-transcript-enter` (300ms fade+slide) in `tailwind.css:183-196`
- All transcript blocks use `animate-transcript-enter` class (`transcript_blocks.rs` throughout)

**Remaining gaps:**

| Animation | Paperclip | lx-desktop | Status |
|-----------|-----------|------------|--------|
| Collapsible height transition | Radix Collapsible animates content height | `CollapsibleContent` instant show/hide (`collapsible.rs:34-40`) | Missing |
| Streaming ping indicator | Animated ping dot during in-progress transcript blocks | Activity block has ping dot (`transcript_blocks.rs:107-109`), but individual tool/message blocks during streaming do not | Partial |
| Kanban card position transitions | @dnd-kit animates item repositioning during drag | Mouse-based drag has no position animation | Missing |

### A.4 Icon System — Status: Unchanged

Paperclip uses Lucide React SVGs (e.g., `<ChevronRight className="h-3 w-3">`). lx-desktop uses Material Symbols Outlined font (`app.rs:20` loads from Google Fonts). Icon sizing is inconsistent — `text-xs`, `text-sm`, `text-lg`, `text-xl` used without convention.

**Gap:** No standardized icon size convention. Most components use `text-sm` for icons but `transcript_blocks.rs` and `wizard.rs` use `text-xl`. Should establish `text-sm` (16px) as default, `text-xs` (12px) for inline, `text-lg`/`text-xl` for hero elements.

---

## B. Component Completeness Gaps

### B.1 AgentConfigForm — Status: Redesigned for lx

**Paperclip:** `AgentConfigForm.tsx` (~2000 lines). Dual create/edit, 8+ sections, adapter-specific fields, env var/secrets, model popover with search, environment testing, dirty tracking with floating save bar.

**lx-desktop:** `pages/agents/config_form.rs` (153 lines). Redesigned for lx's data model with 4 sections: Source Definition (read-only `.lx` source), Model & Backend (adapter dropdown + model text input), Tools (MCP tool list from source), Channels (channel subscriptions from source). Plus custom fields section. Dirty tracking with cancel/save bar.

**Assessment:** The redesign correctly reflects lx's agent model — agents are defined in `.lx` source, not configured via REST API. The config panel shows the source definition and lets you override model/adapter.

**Remaining gaps:**

| Gap | Detail | Size |
|-----|--------|------|
| No model search/popover | Paperclip has a searchable model picker with provider grouping. lx-desktop uses a plain text input (`config_form.rs:56-62`) | M |
| No env var / secrets section | Paperclip manages adapter environment variables. lx equivalent would be MCP server env vars | M |
| Source block not editable | Source shown as read-only `<pre>` (`config_form.rs:22-23`). Should support inline editing or link to editor | S |
| Copy button uses raw JS eval | `config_form.rs:31-34` — should use a shared clipboard utility | S |

### B.2 RunTranscriptView — Status: Significantly Improved

**Paperclip:** `RunTranscriptView.tsx` (~1000 lines). 9 block types, nice/raw modes, comfortable/compact density, streaming indicator, collapsible groups, markdown rendering, token counts.

**lx-desktop:** Now 3 files — `transcript.rs` (141 lines), `transcript_blocks.rs` (171 lines), `transcript_groups.rs` (138 lines). All 9 block types implemented: Message, Thinking, Tool, Activity, CommandGroup, ToolGroup, StderrGroup, Stdout, Event. Collapsible groups with chevron toggles. MarkdownBody rendering for message blocks. Per-tool status indicators. Entry animations.

**Remaining gaps:**

| Gap | Detail | Size |
|-----|--------|------|
| No nice/raw mode toggle | Paperclip has `TranscriptMode` toggle between "nice" (formatted) and "raw" (JSON) | S |
| No comfortable/compact density | Paperclip has `TranscriptDensity` that adjusts spacing and truncation thresholds | S |
| No token counts | Paperclip shows `formatTokens()` for tool results | S |
| No `summarizeToolInput()` logic | Paperclip extracts meaningful summaries from tool inputs (file paths, commands, queries) and truncates by density. lx shows raw input truncated with CSS `truncate` | M |
| No `stripWrappedShell()` | Paperclip unwraps `bash -lc "..."` wrappers from command inputs to show the actual command | S |
| No structured result parsing | Paperclip parses `key: value` headers from tool results (`parseStructuredToolResult`) | S |
| No `ScrollToBottom` wired | `transcript.rs:133` uses `ScrollToBottom` — this is actually wired. Verified. | Fixed |

### B.3 KanbanBoard — Status: Significantly Improved

**Paperclip:** `KanbanBoard.tsx` (200+ lines). @dnd-kit with PointerSensor (5px activation), SortableContext, DragOverlay with shadow, `liveIssueIds` pulsing animation.

**lx-desktop:** `pages/issues/kanban.rs` (251 lines). Pointer-based drag with 5px activation threshold (`kanban.rs:38-44`), floating drag overlay card with `shadow-lg ring-1 ring-[var(--primary)]/20` (`kanban.rs:227`), source card opacity-30 during drag, column highlight on drag-over.

**Remaining gaps:**

| Gap | Detail | Size |
|-----|--------|------|
| No `liveIssueIds` pulsing | Paperclip pulses `animate-pulse` on cards with active heartbeats | S |
| No within-column sorting | @dnd-kit SortableContext enables reordering within a column. lx-desktop only supports cross-column status change | M |
| No touch support | Pointer-based drag uses mouse events only (`onmousedown`/`onmousemove`/`onmouseup`) | S |

### B.4 CommentThread — Status: Significantly Improved

**Paperclip:** `CommentThread.tsx` (300+ lines, 13 props). Timeline merging comments + runs, localStorage drafts with 800ms debounce, MarkdownEditor with @mentions and image upload, reassignment dropdown, URL-based comment highlighting, per-comment copy button.

**lx-desktop:** `components/comment_thread.rs` (83 lines, 2 props). Uses `MarkdownEditor` for input with cursor-position toolbar. `dioxus_storage::use_persistent` for draft persistence. MarkdownBody for rendering comments. Cmd+Enter submit. Disabled-when-empty submit button.

**Remaining gaps:**

| Gap | Detail | Size |
|-----|--------|------|
| No @mentions | Paperclip has `MentionOption[]` with chip decorations and autocomplete | L |
| No image upload | Paperclip supports drag-drop image upload via `imageUploadHandler` | M |
| No timeline merging | Paperclip interleaves comments and run entries chronologically | M |
| No per-comment copy button | Paperclip has copy-to-clipboard on each comment | S |
| No URL-based comment highlighting | Paperclip scrolls to and highlights comments by hash fragment | S |
| No reassignment dropdown | Paperclip allows changing issue assignee from comment thread | S |
| Debounce timing | `dioxus_storage::use_persistent` saves on every keystroke. Paperclip debounces at 800ms | S |

### B.5 MarkdownEditor — Status: Significantly Improved

**Paperclip:** `MarkdownEditor.tsx` (350+ lines). MDXEditor with CodeMirror, 17+ language syntax highlighting, @mention autocomplete with chip decorations, drag-drop image upload, Cmd+Enter submit.

**lx-desktop:** `components/markdown_editor.rs` (201 lines). Three modes (Edit/Preview/Split), 5 toolbar buttons (bold, italic, code, link, heading), cursor-position insertion via JS interop (`insert_at_cursor` at line 107-147), Cmd+Enter submit, `MarkdownBody` preview via `pulldown_cmark`.

**Fixed since last audit:**
- Toolbar now inserts at cursor position instead of appending (`insert_at_cursor` function, `markdown_editor.rs:107-147`)
- Cursor repositioning after insertion (`markdown_editor.rs:142-146`)

**Remaining gaps:**

| Gap | Detail | Size |
|-----|--------|------|
| No @mention autocomplete | Paperclip's `MentionOption[]` with inline chip rendering | L |
| No image upload | Paperclip supports `imageUploadHandler` prop | M |
| No code syntax highlighting | Paperclip uses CodeMirror with 17+ language grammars | L |
| Single textarea ID | `id: "lx-md-editor"` (`markdown_editor.rs:79`) means only one editor per page works correctly | S |
| No auto-grow textarea | Fixed `min-h-[8rem] max-h-80` (`markdown_editor.rs:80`). Paperclip uses `scrollHeight`-based auto-grow | S |

### B.6 ScheduleEditor — Status: Complete

`pages/routines/schedule_editor.rs` (280 lines). Same 7 presets as Paperclip, bidirectional cron parsing via `cron_utils.rs`, `describe_schedule()` at top, custom Select components for all pickers. Day-of-week toggle buttons with active/inactive styling.

**Fixed since last audit:**
- Now uses custom `Select` component instead of native `<select>` elements

**Minor remaining gap:** No ordinal suffixes on monthly dates (Paperclip shows "1st", "2nd", "3rd").

### B.7 OrgChart — Status: Significantly Improved

`pages/org/chart.rs` (287 lines). Tree layout algorithm in `chart_layout.rs`, pan/zoom, SVG edges with stepped paths.

**Fixed since last audit:**
- Mouse wheel zoom (`onwheel` handler at `chart.rs:167-185`) with world-space-aware zoom pivoting
- Auto-center on load (`use_effect` at `chart.rs:105-124`) with fit-to-viewport calculation
- Card hover states (`hover:shadow-md hover:border-[var(--on-surface)]/20` at `chart.rs:266`)
- Fit button (`chart.rs:203-227`) resets to fit-to-viewport

**Remaining gaps:**

| Gap | Detail | Size |
|-----|--------|------|
| Hardcoded viewport dimensions | `800.0` x `600.0` used in fit calculations (`chart.rs:110-111, 208-209`). Should read actual container size | S |
| No agent icon or adapter type in card | Paperclip cards show agent icon + adapter. lx cards show status dot + name + role | S |
| No edge labels | `connected_to` edges from event stream don't show channel/message type labels | S |
| No minimap | Paperclip has optional minimap for large org charts | M |

### B.8 GoalTree — Status: Complete

`pages/goals/tree.rs` (72 lines). Same indentation, collapse/expand with chevron rotation transition.

**Fixed since last audit:**
- Now uses `StatusBadge` component instead of inline `status_color()` function (`tree.rs:1, 59`)

### B.9 FilterBar — Status: Complete

`components/filter_bar.rs` (45 lines). Now uses `Badge` with `BadgeVariant::Secondary` and `button_variant_class(ButtonVariant::Ghost, ButtonSize::Xs)` — matches Paperclip's `<Badge variant="secondary">` and `<Button variant="ghost" size="sm">` pattern exactly.

### B.10 NewIssueDialog — Status: Significantly Improved

`pages/issues/new_issue.rs` (167 lines). Material Symbol close icon. MarkdownEditor for description. Custom Select dropdowns for status, priority, assignee. localStorage draft persistence with load-on-open and clear-on-submit. Cmd+Enter submit.

**Fixed since last audit:**
- MarkdownEditor replaces plain textarea
- Custom Select components replace native `<select>`
- Draft persistence via localStorage
- Cmd+Enter keyboard shortcut

**Remaining gaps:**

| Gap | Detail | Size |
|-----|--------|------|
| No file staging | Paperclip has drag-drop file attachments with preview | M |
| No model override | Paperclip allows overriding agent model per-issue | S |
| No execution workspace selector | Paperclip-specific, not applicable to lx | N/A |
| No 800ms debounce on draft save | Draft saves on every `use_effect` render cycle. Paperclip debounces writes | S |
| No focus trap | Dialog doesn't trap Tab focus. Paperclip uses `DialogContent` with Radix focus trap | S |

### B.11 OnboardingWizard — Status: Redesigned for lx

`components/onboarding/wizard.rs` (229 lines) + 4 step files. 4-step wizard: Company, Agent, Task, Launch. Step tabs with icons. Keyboard navigation (Cmd+Enter to advance). Back/Next buttons. Loading state with spinner. Error display. Form reset on close.

**Fixed since last audit:**
- Redesigned with lx-appropriate steps
- Custom Select dropdowns for adapter/role
- Model ID text input with helper text
- Cmd+Enter keyboard shortcut for advancing steps
- Loading state with animated spinner

**Remaining gaps:**

| Gap | Detail | Size |
|-----|--------|------|
| No adapter radio cards | Paperclip shows 9 adapter options as visual radio cards with "Recommended" badges | M |
| No env testing | Paperclip tests adapter environment on agent step | M |
| No API integration | Launch step doesn't actually create anything — just shows preview | L |
| No auto-grow textarea | Agent description textarea has fixed height | S |
| No ASCII art / branding panel | Paperclip has `AsciiArtAnimation` side panel | S |

---

## C. Interaction Pattern Gaps

### C.1 Keyboard Navigation — Status: Improved

**Fixed since last audit:**
- Priority-based `ShortcutRegistry` with `ShortcutPriority` enum (Global/Page/Panel/Modal/Overlay) at `hooks/keyboard_shortcuts.rs`
- `use_shortcut` hook for declarative shortcut registration with automatic cleanup
- `key_match()` and `escape_match()` helper functions for common patterns
- Escape-to-close on `DialogContent` (`dialog.rs:41-46`)
- Arrow key navigation in custom `Select` component (`select.rs:134-178`)

**Remaining gaps:**

| Pattern | Paperclip | lx-desktop | Status |
|---------|-----------|------------|--------|
| Cmd+K command palette | `useKeyboardShortcuts` + `CommandPalette` | `command_palette.rs` exists but no evidence of global `use_shortcut` wiring | Unverified |
| Arrow key list navigation in command palette | Command palette uses arrow keys + enter to select | Need to verify `command_palette.rs` implementation | Unverified |
| Focus trapping in popovers/dropdowns | Radix primitives provide focus trapping | Custom `Select` has keyboard nav but no focus trap | Partial |
| Escape to close Select | Radix Select closes on Escape | Implemented at `select.rs:171-176` | Fixed |

### C.2 Click-Outside-to-Close — Status: Improved

**Fixed since last audit:**
- `DialogContent` uses fixed overlay with `onclick` to close (`dialog.rs:21-23`)
- Custom `Select` uses `fixed inset-0 z-40` overlay for click-outside detection (`select.rs:87-89`)
- OnboardingWizard uses overlay onclick with `stop_propagation` on content (`wizard.rs:93, 96`)

**Remaining gap:** Tooltip and Popover components (`popover.rs`, `tooltip.rs`) may not have click-outside handling. Need verification.

### C.3 Optimistic Updates — Status: Architectural Gap

Paperclip uses React Query `onMutate` for optimistic updates. lx-desktop has no mutation layer — it reads from `ActivityLog` (event stream) and has no write path to the lx interpreter yet. This is an expected architectural gap that will be addressed when the interpreter control API is built.

### C.4 Loading and Error States — Status: Adequate

- `PageSkeleton` used in app-level `SuspenseBoundary` fallback (`app.rs:33-36`)
- `EmptyState` component used on dashboard when no events (`dashboard/mod.rs:63-70`)
- Form validation: submit buttons disabled when required fields empty (NewIssueDialog, CommentThread, OnboardingWizard)
- Error boundary at app level (`app.rs:25-31`)

**Remaining gap:** No per-component loading skeletons (Paperclip uses contextual skeleton variants per entity type).

### C.5 Scroll — Status: Wired

`ScrollToBottom` (`scroll_to_bottom.rs`, 47 lines) uses `scrollIntoView` with smooth behavior. Used in `TranscriptView` (`transcript.rs:133`). Tracks user scroll position to avoid overriding manual scroll-up.

### C.6 Form Patterns

| Pattern | Paperclip | lx-desktop | Status |
|---------|-----------|------------|--------|
| Auto-grow textarea | `scrollHeight`-based resize | Fixed `min-h` / `max-h` | Missing |
| Debounced draft save | 800ms debounce to localStorage | `dioxus_storage::use_persistent` (every keystroke) or `use_effect` (every render) | Degraded |
| File drag-and-drop | NewIssueDialog, MarkdownEditor, CommentThread | None | Missing |
| Toolbar cursor insertion | Inserts at cursor via selection API | Implemented via JS interop (`markdown_editor.rs:107-147`) | Fixed |

### C.7 Select — Status: Complete

Custom `Select` component (`components/ui/select.rs`, 222 lines) with:
- Popover dropdown with click-outside-to-close overlay
- Searchable mode with filter input
- Arrow key navigation (up/down/enter/escape)
- Focus tracking on hover (`onmouseenter`)
- Selected item checkmark indicator
- Disabled item support
- "No results" empty state

### C.8 Toast — Status: Complete

`contexts/toast.rs` (117 lines) + `components/toast_viewport.rs` (108 lines):
- Deduplication with 3.5s window (`toast.rs:59-69`)
- TTL clamping between 1.5s-15s (`toast.rs:72`)
- Tone-based default TTLs (Info 4s, Success 3.5s, Warn 8s, Error 10s)
- Entry animation `animate-toast-enter` (slide from left)
- Exit animation `animate-toast-exit` (slide to left) with `dismissing` state
- Auto-dismiss via 250ms polling loop (`toast_viewport.rs:72-91`)
- Max 5 toasts

### C.9 Dialog — Status: Improved

`components/ui/dialog.rs` (151 lines):
- Overlay animation (`animate-dialog-overlay-in`)
- Content animation (`animate-dialog-content-in` — scale + fade)
- Escape to close (`dialog.rs:42-46`)
- Focus trap via JS interop — traps Tab/Shift+Tab within dialog focusable elements (`dialog.rs:47-73`)
- Auto-focus on mount (`dialog.rs:36-39`)
- SVG close button icon

---

## D. Wiring Translation Notes

### D.1 Agent Detail

**Paperclip:** REST API for agent CRUD, heartbeat runs, adapter config, budget policies, environment testing.

**lx-desktop should wire to:**
- Agent spawn events from `ActivityLog` (`ActivityEvent` with kind `agent_start`/`agent_running`/`agent_spawn`)
- Mailbox traffic: `tell`/`ask`/`reply` events from event stream
- Active tool calls: `tool_call` events without matching `tool_result`
- Uptime: derived from first `agent_spawn` event timestamp
- Config tab → `AgentConfigPanel` already shows `.lx` source definition, tools, channels (`config_form.rs`)
- Budget → sum token usage from `tool_result` events in event stream

**Current state:** `pages/agents/detail.rs` has tabs (Overview, Runs, Config, Skills, Budget). Config tab wired to `AgentConfigPanel`. Runs tab shows transcript. Data currently comes from mock/ActivityLog, not from live interpreter.

**Tab rename suggestion:** Overview, Messages, Tools, Source, Costs (to match lx concepts).

### D.2 Issues/Tasks

**Paperclip:** `issuesApi` CRUD with full lifecycle.

**lx-desktop should wire to:**
- Parse `task` blocks from `.lx` AST
- Status from event stream (`task_start`, `task_complete`, `task_error`)
- No creation dialog needed for lx tasks (they're defined in source), but the existing NewIssueDialog could be repurposed for ad-hoc tasks
- Kanban columns = lx task states
- Assignee = agent in task's `run` block

**Current state:** Issue types defined in `pages/issues/types.rs`. Kanban, list, detail views all work with `Issue` struct. Currently populated from ActivityLog, not from AST.

### D.3 Dashboard

**Paperclip:** REST APIs for company metrics, activity feed, issue counts, heartbeat status.

**lx-desktop should wire to:**
- `dashboard/mod.rs` already reads from `ActivityLog` context
- Metrics computed: total events, agent events, tool events, errors (`dashboard/mod.rs:26-29`)
- Activity buckets for sparkline chart (`dashboard/mod.rs:31-49`)
- Event breakdown by type (`dashboard/mod.rs:51-61`)
- Recent activity feed with `animate-activity-enter` (`dashboard/mod.rs:110-127`)

**Assessment:** Dashboard is already wired to the correct data source (ActivityLog/EventStream). Metrics are lx-appropriate. Main gap is live updating — needs `use_future` polling or subscription to EventStream changes.

### D.4 Activity

**Paperclip:** `activityApi.list()` REST endpoint.

**lx-desktop should wire to:**
- `EventStream.xread()` — tail event stream directly
- `contexts/live_updates.rs` already uses file-based events via `LX_EVENT_STREAM_PATH` env var
- `contexts/activity_log.rs` provides `ActivityLog` context with `Signal<Vec<ActivityEvent>>`

**Assessment:** Wiring infrastructure exists. The `live_updates.rs` context reads from the JSONL event stream file. Gap is subscription/push notification when new events arrive (currently requires manual refresh or polling).

### D.5 Runs/Transcripts

**Paperclip:** `heartbeatsApi.runs()` for heartbeat execution logs.

**lx-desktop should wire to:**
- A "run" = one `.lx` program execution session
- Transcript blocks from JSONL event stream
- `transcript.rs:42-112` (`event_to_block`) already maps `ActivityEvent` kinds to `TranscriptBlock` variants
- `transcript.rs:116` accepts optional `Vec<ActivityEvent>` — ready for event stream data

**Assessment:** Transcript is the most complete wiring. `event_to_block` correctly maps lx event types to transcript block types. The block rendering is thorough. Main gap is feeding real event data.

### D.6 Org Chart → Agent Topology

**Paperclip:** REST API for hierarchical org tree.

**lx-desktop should wire to:**
- `chart.rs:11-58` (`nodes_from_events`) already parses agent topology from ActivityLog events
- Recognizes `agent_start`/`agent_running`/`agent_spawn`, `agent_reports_to`, `tell`/`ask`/message events
- Builds `connected_to` edges from communication events

**Assessment:** This is correctly wired for lx's flat agent topology. Shows communication patterns rather than hierarchy. Edge labels for channel names would improve readability.

### D.7 Costs

**Paperclip:** `costsApi` with per-agent/project budget policies and hard stops.

**lx-desktop should wire to:**
- Parse token usage from `tool_result` events in event stream
- Aggregate by agent/flow/tool
- `pages/costs/` directory exists with `overview.rs`, `budget_card.rs`, `provider_card.rs`, `accounting_card.rs`
- Display only — no enforcement

**Assessment:** Cost UI components exist. Need to wire to actual token usage data from event stream.

### D.8 Routines

**Paperclip:** `routinesApi` with cron-scheduled heartbeats.

**lx-desktop should wire to:**
- Parse `use cron` / `cron.schedule()` from `.lx` source
- `ScheduleEditor` UI is complete and functional (`schedule_editor.rs`)
- `pages/routines/` has list, detail, and type definitions

**Assessment:** UI complete. Needs `.lx` source parsing for cron declarations.

---

## E. Prioritized Fix List

### Critical (blocks core UX)

| # | Files | What's Wrong | Fix | Size |
|---|-------|-------------|-----|------|
| 1 | `app.rs:19-21` | Google Fonts CDN dependency — fonts fail offline, violates desktop app expectations | Bundle Space Grotesk, Inter, JetBrains Mono, Material Symbols as static assets | M |
| 2 | `markdown_editor.rs:79` | Single hardcoded `id="lx-md-editor"` — multiple editors on same page (e.g., NewIssueDialog + CommentThread) collide on cursor position | Generate unique IDs per editor instance via `use_signal(|| uuid::Uuid::new_v4())` | S |

### High (noticeable quality gaps)

| # | Files | What's Wrong | Fix | Size |
|---|-------|-------------|-----|------|
| 3 | `transcript.rs`, `transcript_blocks.rs` | No nice/raw mode toggle, no density setting, no `summarizeToolInput()` for readable tool summaries | Add mode/density props and implement Paperclip's tool input summarization logic | M |
| 4 | `comment_thread.rs` | No @mention autocomplete | Build mention detection + dropdown popup. Can start with simple `@`-triggered agent name filter | L |
| 5 | `config_form.rs:56-62` | Plain text input for model selection | Build searchable model picker popover — reuse `Select` component with `searchable: true` | S |
| 6 | `kanban.rs` | No within-column reordering | Add item index tracking during drag to support reordering within same column | M |
| 7 | `chart.rs:110-111, 208-209` | Hardcoded 800x600 viewport | Use `onmounted` to read actual container dimensions | S |

### Medium (polish gaps)

| # | Files | What's Wrong | Fix | Size |
|---|-------|-------------|-----|------|
| 8 | `collapsible.rs:34-40` | No height transition animation — instant show/hide | Add CSS `max-height` transition or JS-based height animation | M |
| 9 | `markdown_editor.rs:80` | No auto-grow textarea | Add JS interop to set textarea height to `scrollHeight` on input | S |
| 10 | `comment_thread.rs`, `new_issue.rs` | Draft saves on every render cycle, not debounced | Add 800ms debounce timer before localStorage write | S |
| 11 | `tailwind.css` | No light mode theme vars | Add `:root` light mode variables and `.dark` class dark mode, with toggle in theme context | L |
| 12 | `tailwind.css` | Only 3 chart colors, Paperclip has 5 semantic chart colors | Add `--chart-1` through `--chart-5` variables for multi-series chart support | S |
| 13 | Icon sizing | Inconsistent Material Symbols sizes across files | Establish convention: `text-sm` default, `text-xs` inline, `text-lg` hero. Sweep all files | S |
| 14 | `onboarding/wizard.rs` | No API integration — Launch step doesn't create anything | Wire to lx interpreter init: create project dir, generate `.lx` file, start interpreter | L |

### Low (minor visual/behavioral differences)

| # | Files | What's Wrong | Fix | Size |
|---|-------|-------------|-----|------|
| 15 | `schedule_editor.rs` | No ordinal suffixes on monthly dates ("1" vs "1st") | Add ordinal suffix function for display labels | S |
| 16 | `chart.rs:266` | No agent icon or adapter type in org chart cards | Add icon field to `OrgNode`, render in card | S |
| 17 | `chart.rs` | No edge labels for communication channels | Add text along SVG path edges showing channel/message type | S |
| 18 | `transcript_blocks.rs` | No token count display on tool results | Parse token metadata from event stream, display in tool block header | S |
| 19 | `kanban.rs` | No `liveIssueIds` pulsing animation on active cards | Add `animate-pulse` class when agent is actively working on issue | S |
| 20 | Multiple | No file drag-and-drop upload on MarkdownEditor, NewIssueDialog, CommentThread | Add `ondragover`/`ondrop` handlers with file handling | M |
