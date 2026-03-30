# UI Alignment Audit: lx-desktop vs Paperclip

## A. Visual Fidelity Gaps

### A.1 Color System Architecture

Paperclip uses oklch-based CSS variables with full light/dark theme toggling. lx-desktop uses a single dark theme with Material Design 3 surface tokens.

| Concern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Color space | oklch (perceptually uniform) | Hex CSS variables |
| Light mode | Full light theme via `:root`, dark via `.dark` | Dark only — no light mode |
| Variable naming | Tailwind semantic (`--background`, `--foreground`, `--card`, `--muted`, `--accent`) | Material surface tokens (`--surface`, `--surface-container-*`, `--on-surface`) |
| Chart colors | `--chart-1` through `--chart-5` | 3 chart vars (`--color-chart-axis`, `--color-chart-split`, `--color-chart-tooltip`) |
| Sidebar colors | Dedicated `--sidebar-*` tokens | None |

**Impact:** UI primitives (`components/ui/button.rs:31-43`) reference Tailwind semantic classes (`bg-primary`, `bg-accent`) that must map to Material tokens in `tailwind.css`. Incomplete `@theme` mappings mean some variant colors won't resolve.

### A.2 Border Radius

`tailwind.css:8-14` sets all Tailwind radii to `0rem` — lx-desktop's intentional industrial aesthetic. Most components handle this correctly, but `rounded-full` on status dots and toggle switches resolves to `0rem` too, rendering circles as squares unless overridden with explicit pixel values.

### A.3 Typography

| Concern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Font stack | System fonts (shadcn default) | Space Grotesk (display), Inter (body), JetBrains Mono (mono) |
| Font loading | Bundled with build | `app.rs:8-16` loads from Google Fonts CDN |
| Page headings | Standard `text-xl font-semibold` | `text-2xl font-bold uppercase tracking-wider` (`styles.rs:1`) |

### A.4 Animations & Transitions

| Animation | Paperclip | lx-desktop | Status |
|-----------|-----------|------------|--------|
| Activity row entry | `@keyframes activity-row-enter` (980ms opacity+bg fade) | `animate-activity-enter` on dashboard rows (`dashboard/mod.rs:114`) | Implemented |
| Status dot pulse | `animate-pulse` on running agents | `STATUS_DOT_RUNNING` with `animate-pulse` (`styles.rs:6`) | Implemented |
| Kanban drag feedback | `shadow-lg ring-1 ring-primary/20` overlay + `opacity-30` source | `opacity-30` source + `ring-1 ring-[var(--primary)]/40` on column | Partial — no DragOverlay |
| Toast entry/exit | Slide-in animation with opacity | `animate-fade-in` on dialog overlay | Partial — no exit animation |
| Collapsible expand | Height transition on content | Chevron rotates (`goals/tree.rs:58`) but no height animation | Missing |
| Dialog backdrop | `animate-in fade-in-0` + `zoom-in` content | Instant show/hide | Missing |
| Streaming indicator | Animated ping dot on in-progress transcript messages | None | Missing |
| Transcript entry | `animate-in fade-in slide-in-from-bottom-1 duration-300` | None | Missing |

### A.5 Icon System

Paperclip uses Lucide React SVGs. lx-desktop uses Material Symbols Outlined font (`app.rs:12`). Icon names don't always map 1:1. Material Symbols sizing is inconsistent — `text-sm` to `text-xl` used without convention.

---

## B. Component Completeness Gaps

### B.1 AgentConfigForm

**Paperclip:** `AgentConfigForm.tsx` (~2000 lines). Dual mode (create/edit), 8+ sections, adapter-specific fields, env var/secrets management, model popover with search, environment testing, dirty tracking with floating save bar, collapsible advanced settings.

**lx-desktop:** `pages/agents/config_form.rs` (152 lines). Edit-only, 3 fields (adapter type dropdown, model text input, heartbeat toggle/interval), basic dirty flag, no API integration.

| Missing in lx-desktop | Paperclip location |
|----------------------|-------------------|
| Create mode with separate prop shape | Lines 73-85 |
| Identity section (name, title, reports_to, capabilities) | Lines 481-556 |
| Prompt template editor (MarkdownEditor) | Lines 557-600 |
| Environment variable/secrets management | Lines 1248-1325 |
| Model popover with search and grouping | Lines 1044-1081 |
| "Test Environment" button with result display | Lines 634-644 |
| Floating save bar with dirty overlay | Lines 466-479 |
| Advanced run policy (max concurrent, cooldown, timeout) | Lines 937-979 |

### B.2 RunTranscriptView

**Paperclip:** `RunTranscriptView.tsx` (~1000 lines). 9 block types, nice/raw modes, comfortable/compact density, streaming indicator, collapsible groups, markdown rendering, token counts, accessibility.

**lx-desktop:** `pages/agents/transcript.rs` (124 lines). 4 block types (Message, Thinking, ToolUse, Event), single mode/density, plain text rendering, no collapse, no streaming, no accessibility.

| Missing block type | What it renders |
|-------------------|----------------|
| `command_group` | Stacked terminal icons, expandable command accordion with per-command status |
| `tool_group` | Grouped non-command tool calls with wrench icons, per-tool status badges |
| `stderr_group` | Collapsible amber-styled stderr lines with line count |
| `stdout` | Collapsible stdout with label |
| `activity` | Running/completed activity with animated ping dot |

Additional gaps: No expandable tool details, no status indicators per tool, no structured result parsing, no shell command unwrapping, no density-aware truncation.

### B.3 KanbanBoard

**Paperclip:** `KanbanBoard.tsx` (200+ lines). @dnd-kit with PointerSensor (5px activation), SortableContext, DragOverlay with shadow, `liveIssueIds` pulsing.

**lx-desktop:** `pages/issues/kanban.rs` (150 lines). Native HTML5 drag-and-drop, `opacity-30` on dragged card, `ring-1` on target column.

| Gap | Detail |
|-----|--------|
| No drag overlay | Paperclip renders floating card copy with `shadow-lg ring-1 ring-primary/20` |
| No activation threshold | HTML5 activates immediately; @dnd-kit uses 5px distance |
| No live issue indicators | Paperclip pulses `animate-pulse` on active cards |
| No smooth transform transitions | @dnd-kit animates positions; HTML5 is instant |

### B.4 CommentThread

**Paperclip:** `CommentThread.tsx` (300+ lines, 13 props). Timeline merging comments + runs, localStorage drafts with 800ms debounce, MarkdownEditor with @mentions and image upload, reassignment dropdown, URL-based comment highlighting, per-comment copy button.

**lx-desktop:** `components/comment_thread.rs` (79 lines, 2 props) and `pages/issues/comments.rs` (65 lines, 3 props). Basic textarea, simple submit, no drafts/mentions/images/runs/reassignment.

### B.5 MarkdownEditor

**Paperclip:** `MarkdownEditor.tsx` (350+ lines). MDXEditor with CodeMirror, 17+ language syntax highlighting, @mention autocomplete with chip decorations, drag-drop image upload, Cmd+Enter submit.

**lx-desktop:** `components/markdown_editor.rs` (150 lines). Three modes (Edit/Preview/Split), plain textarea, 5 toolbar buttons that append to end of value, Cmd+Enter submit, `MarkdownBody` preview via `pulldown_cmark`. Missing: cursor-position insertion, @mentions, image upload, code syntax highlighting.

### B.6 ScheduleEditor — Closest to complete. Same 7 presets, bidirectional cron parsing, `describe_schedule()` at top. Gaps: native selects vs styled dropdowns, no ordinal suffixes on monthly dates.

### B.7 OrgChart

Layout algorithm, pan/zoom, SVG edges all match. Gaps: no mouse wheel zoom (only +/- buttons), no auto-center on load, simpler card content (no agent icon or adapter type), no card hover states.

### B.8 GoalTree — Structurally complete. Same indentation, same collapse/expand. Minor gaps: inline `status_color()` instead of `StatusBadge` component; hardcoded link instead of flexible props.

### B.9 FilterBar — Functionally equivalent. Uses raw `span`/`button` instead of own `Badge`/`Button` UI primitives.

### B.10 NewIssueDialog

**Paperclip:** 400+ lines. MarkdownEditor, file staging with drag-drop, model overrides, execution workspace, draft persistence. **lx-desktop:** 116 lines. Plain input/textarea, native selects, no files/drafts/model overrides.

### B.11 OnboardingWizard

**Paperclip:** 1404 lines. 9 adapter radio cards with "Recommended" badges, model popover with search, env testing, auto-grow textarea, Cmd+Ctrl+Enter advance, ASCII art panel, full API integration.

**lx-desktop:** ~270 lines across wizard.rs + 4 step files. 4 fields on agent step (name, role, adapter dropdown, description), no model picker, no env testing, no animations, no API integration.

---

## C. Interaction Pattern Gaps

### C.1 Keyboard Navigation

| Pattern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Cmd+K command palette | `useKeyboardShortcuts` hook | `command_palette.rs` exists but no global hook wiring |
| Cmd+Enter submit | MarkdownEditor, OnboardingWizard | MarkdownEditor only (`markdown_editor.rs:84`) |
| Arrow key list navigation | Command palette, selects, mentions | Not implemented |
| Escape to close | Dialog components, mentions, popovers | Partial — `keyboard_shortcuts.rs` handles some |
| Tab focus management | Radix primitives provide focus trapping | No focus trapping |

### C.2 Click-Outside-to-Close — Paperclip uses Radix portals with automatic detection. lx-desktop uses backdrop `onclick` + `stop_propagation` — works for modals but popovers/dropdowns don't close on outside click.

### C.3 Optimistic Updates — Paperclip uses React Query `onMutate`. lx-desktop has no mutation layer — architectural gap for when wiring to lx's event stream.

### C.4 Loading and Error States

| Pattern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Page loading | `PageSkeleton` shimmer variants | `SuspenseBoundary` with `PageSkeleton` |
| Empty states | Contextual per entity type | `EmptyState` component, used in some pages |
| Form validation | Inline errors, disabled submit, `aria-invalid` | Minimal — only `disabled` on empty title |

### C.5 Scroll — Paperclip has `ScrollToBottom` for auto-scrolling transcripts/comments. lx-desktop has `scroll_to_bottom.rs` but it needs verification of usage.

### C.6 Form Patterns

| Pattern | Paperclip | lx-desktop |
|---------|-----------|------------|
| Auto-grow textarea | `scrollHeight`-based | Fixed `min-h` textareas |
| Debounced draft save | 800ms debounce to localStorage | None |
| File drag-and-drop | NewIssueDialog, MarkdownEditor | None |
| Toolbar cursor insertion | Inserts at cursor position | Appends to end of value |

### C.7 Select — Paperclip uses Radix Select with custom trigger/popup/groups. lx-desktop uses native `<select>` (`components/ui/select.rs:18-26`). No custom styling, search, or groups.

### C.8 Toast — Auto-dismiss with TTLs implemented. Still missing: deduplication (3.5s window), TTL clamping (1.5s-15s), entry/exit animations.

---

## D. Wiring Translation Notes

### D.1 Agent Detail
**Paperclip:** REST API for agent, heartbeat runs, adapter config, budget. **lx-desktop should wire to:** Agent spawn events from EventStream, mailbox traffic (tell/ask/reply), active tool calls, uptime since spawn. Config tab → show `.lx` source definition. Budget → sum token usage from tool_result events. Tabs should become: Overview, Messages, Tools, Source, Costs.

### D.2 Issues/Tasks
**Paperclip:** `issuesApi` CRUD. **lx-desktop:** Parse `task` blocks from `.lx` AST. Status from event stream (task_start/complete/error). No creation dialog — tasks defined in source. Kanban columns = lx task states. Assignee = agent in task's `run` block.

### D.3 Dashboard
**Paperclip:** REST APIs for metrics/activity/issues/heartbeats. **lx-desktop:** Metrics from EventStream (agent count, message throughput, tool calls, errors). Activity from JSONL tail. Charts from event aggregation. Already reads from `ActivityLog` context — gap is data source wiring.

### D.4 Activity
**Paperclip:** `activityApi.list()`. **lx-desktop:** `EventStream.xread()` — tail event stream directly. Filter by event type. `live_updates.rs` already uses file-based events via `LX_EVENT_STREAM_PATH` env var.

### D.5 Runs/Transcripts
**Paperclip:** `heartbeatsApi.runs()`. **lx-desktop:** No heartbeat runs. A "run" = one `.lx` program execution. Transcript blocks from JSONL. `transcript.rs` has correct block types, needs JSONL parsing.

### D.6 Org Chart
**Paperclip:** REST API org tree. **lx-desktop:** Agent-channel topology from running program. Communication topology, not hierarchy. `chart.rs` builds from ActivityLog — correct approach, needs `connected_to` edge labels.

### D.7 Costs
**Paperclip:** `costsApi` with budget enforcement. **lx-desktop:** Parse token usage from tool_result events. Aggregate by agent/flow/tool. Display only, no enforcement.

### D.8 Routines
**Paperclip:** `routinesApi` with cron heartbeats. **lx-desktop:** Parse `use cron` / `cron.schedule()` from `.lx` source. `schedule_editor.rs` UI is solid — needs `.lx` source read/write.

---

## E. Prioritized Fix List

### Critical

| # | File | What's Wrong | Fix | Size |
|---|------|-------------|-----|------|
| 1 | `tailwind.css` | UI primitives use Tailwind semantic tokens that may not fully resolve. Verify all `@theme` mappings. | Audit each class in `components/ui/*.rs`, ensure CSS variable mapping in `tailwind.css`. | M |
| 2 | `tailwind.css:8-14` | `rounded-full` = 0rem. Status dots, toggles, avatars render as squares. | Override `rounded-full` to `9999px` or add explicit `border-radius`. | S |
| 3 | `pages/agents/transcript.rs` | 4 block types vs 9. No collapse, streaming, status, markdown, density. | Add 5 block types, collapsible sections, `MarkdownBody` rendering. | L |

### High

| # | File | What's Wrong | Fix | Size |
|---|------|-------------|-----|------|
| 4 | `components/markdown_editor.rs` | Toolbar appends to end instead of cursor. No @mentions, no images, no syntax highlighting. | JS interop via `document::eval()` for cursor position. Basic @mention detection. | M |
| 5 | `components/comment_thread.rs` | No drafts, no @mentions, no images, no run timeline, no Cmd+Enter. | Add draft persistence, wire MarkdownEditor, add Cmd+Enter handler. | M |
| 6 | `pages/issues/kanban.rs` | No drag overlay, no activation threshold, no live indicators. | Add floating card clone during drag, 5px activation threshold. | M |
| 7 | `pages/agents/config_form.rs` | 3 fields vs 15+. For lx: should show agent `.lx` source, model config, tool declarations. | Redesign for lx — not a direct port. | L |
| 8 | `components/ui/select.rs` | Native `<select>`. No search, groups, custom rendering. | Build popover-based select using existing `Popover` component. | M |
| 9 | Keyboard system | No global shortcuts. Cmd+K, Escape, arrows not wired. | Expand `keyboard_shortcuts.rs` with registration-based system. | M |

### Medium

| # | File | What's Wrong | Fix | Size |
|---|------|-------------|-----|------|
| 10 | `pages/org/chart.rs` | No wheel zoom, no auto-center, no card hover states. | Add `onwheel` handler, fit-to-viewport on mount, `hover:shadow-md`. | M |
| 11 | `pages/issues/new_issue.rs` | Basic form, plain textarea, native selects, "x" text close button. | Add MarkdownEditor, Material Symbol close icon, draft persistence. | M |
| 12 | `components/onboarding/wizard.rs` | No model picker, no env testing, no keyboard shortcuts. | Redesign for lx: interpreter backend, default model, MCP tool paths. | M |
| 13 | `contexts/toast.rs` | Missing dedup (3.5s window), TTL clamping, entry/exit animations. | Add dedup map, clamp TTL, CSS slide-in/fade-out animations. | S |
| 14 | Dialog system | No backdrop animation, no focus trapping, limited Escape handling. | Add CSS keyframes, `onkeydown` Escape, focus trap via JS interop. | S |
| 15 | `components/filter_bar.rs` | Uses raw elements instead of `Badge`/`Button` UI primitives. | Replace with `Badge` variant secondary and `Button` variant ghost. | S |
| 16 | `components/scroll_to_bottom.rs` | Verify usage — transcripts/comments may not use it. | Wire into TranscriptView and CommentThread containers. | S |

### Low

| # | File | What's Wrong | Fix | Size |
|---|------|-------------|-----|------|
| 17 | Dialog/transcript | No entry animations on content. | Add CSS `slide-in-from-bottom` / `zoom-in` keyframes. | S |
| 18 | `pages/goals/tree.rs` | Inline `status_color()` instead of `StatusBadge` component. | Replace with `StatusBadge` for consistency. | S |
| 19 | Icon sizing | Inconsistent Material Symbols sizes across files. | Establish convention (base/sm/lg), sweep all files. | S |
| 20 | Multiple | Paperclip features not needed for lx: company brand colors, plugin slots, sidebar drag-reorder. | Skip — develop lx equivalents. | — |
