# GSD-2: Auto-Mode Engine

## Core Concept

Auto-mode is a **state machine driven by files on disk**. Each unit of work gets a fresh session via the stashed `ctx.newSession()` pattern.

```
Read disk state → Determine next unit → Build prompt → Create fresh session → Execute → Close out → Repeat
```

## Dispatch Rules (auto-dispatch.ts)

Replaces a 130-line if-else chain with a data-driven rule table evaluated in order. First match wins.

```typescript
interface DispatchRule {
  name: string;
  match: (ctx: DispatchContext) => Promise<DispatchAction | null>;
}

type DispatchAction =
  | { action: "dispatch"; unitType: string; unitId: string; prompt: string; pauseAfterDispatch?: boolean }
  | { action: "stop"; reason: string; level: "info" | "warning" | "error" }
  | { action: "skip" };
```

### Rules in Evaluation Order

| # | Rule | Condition | Action |
|---|------|-----------|--------|
| 1 | rewrite-docs | Override files pending, under max attempts | dispatch rewrite-docs |
| 2 | summarizing | Phase is "summarizing" | dispatch complete-slice |
| 3 | run-uat | Most recently completed slice needs UAT | dispatch run-uat |
| 4 | reassess-roadmap | Most recently completed slice needs reassessment | dispatch reassess-roadmap |
| 5 | needs-discussion | Phase is "needs-discussion" | stop (manual discussion needed) |
| 6 | pre-planning (no context) | Pre-planning, no context file | stop (run /gsd discuss first) |
| 7 | pre-planning (no research) | Pre-planning, no research, prefs don't skip | dispatch research-milestone |
| 8 | pre-planning (has research) | Pre-planning, has research | dispatch plan-milestone |
| 9 | planning (no research) | Planning, no slice research (unless S01 w/ milestone research) | dispatch research-slice |
| 10 | planning | Phase is planning | dispatch plan-slice |
| 11 | replanning-slice | Phase is replanning-slice | dispatch replan-slice |
| 12 | executing (missing plan) | Executing, task plan missing (issue #909 fix) | dispatch plan-slice |
| 13 | executing | Phase executing, activeTask present | dispatch execute-task |
| 14 | validating-milestone | Phase validating-milestone | dispatch validate-milestone |
| 15 | completing-milestone | Phase completing-milestone | dispatch complete-milestone |

Circuit breaker: max 3 rewrite-docs attempts to prevent infinite loops.

## Prompt Builders (auto-prompts.ts)

Pure async functions that load templates and inline file content. ~1248 lines.

### Builder Functions

| Function | Unit Type | Inlined Context |
|----------|-----------|-----------------|
| `buildResearchMilestonePrompt()` | research-milestone | PROJECT, DECISIONS, existing research |
| `buildPlanMilestonePrompt()` | plan-milestone | Research, DECISIONS, REQUIREMENTS |
| `buildResearchSlicePrompt()` | research-slice | Milestone roadmap, prior research |
| `buildPlanSlicePrompt()` | plan-slice | Research, dependency summaries, DECISIONS, REQUIREMENTS |
| `buildExecuteTaskPrompt()` | execute-task | Task plan, slice excerpt, prior summaries, continue.md |
| `buildCompleteSlicePrompt()` | complete-slice | All task summaries, slice plan, verification results |
| `buildCompleteMilestonePrompt()` | complete-milestone | All slice summaries, roadmap |
| `buildValidateMilestonePrompt()` | validate-milestone | All summaries, verification, issues |
| `buildReplanSlicePrompt()` | replan-slice | Validation results, original plan |
| `buildRunUatPrompt()` | run-uat | Slice summary, UAT script |
| `buildReassessRoadmapPrompt()` | reassess-roadmap | Completed slices, remaining roadmap |
| `buildRewriteDocsPrompt()` | rewrite-docs | Pending override files |

### Context Inlining Helpers

- `inlineFile()` → Content wrapped with source header, fallback message if missing
- `inlineFileOptional()` → Returns null if file missing
- `inlineFileSmart()` → For large files (>3KB), uses TF-IDF semantic chunking to include only task-relevant portions
- `inlineDependencySummaries()` → Full slice dependency summaries with distillation/truncation
- `inlineGsdRootFile()` → Well-known .gsd/ root files (PROJECT, DECISIONS, QUEUE, STATE, REQUIREMENTS, KNOWLEDGE)
- `inlineDecisionsFromDb()` → From DB with optional milestone scoping
- `inlineRequirementsFromDb()` → From DB with optional slice scoping
- `buildResumeSection()` → Resume state from continue files
- `buildCarryForwardSection()` → Prior task summaries in slice
- `extractSliceExecutionExcerpt()` → Goal/Demo/Verification from slice plan

### Prompt Template: execute-task.md (Key Sections)

1. Working directory notice (stay in it, no cd)
2. Executor contract (trust the task plan, don't re-research)
3. Overrides section (if any pending)
4. Resume state (from continue.md)
5. Carry-forward section (prior task summaries)
6. Inlined task plan (authoritative local execution contract)
7. Slice plan excerpt (goal/demo/verification)
8. Step execution instructions (narrate, load skills, build real thing, write tests)
9. Background process rule (never bare `&`)
10. Verification instructions + evidence table
11. Debugging discipline (form hypothesis first, change one variable, read completely)
12. Blocker discovery (set `blocker_discovered: true` if plan is invalid)
13. Decisions/Knowledge appending instructions
14. Task summary template loading
15. Task marking (must mark `[x]` in plan AND write summary)

### Prompt Template: plan-slice.md (Key Sections)

1. Inlined context (preloaded — don't re-read)
2. Dependency summaries with Forward Intelligence sections
3. Role definition (researcher already explored, you decompose)
4. Executor context constraints (token budgets, task count ranges)
5. Requirements coverage
6. Template loading + skill loading
7. Verification definition (test files with assertions)
8. Observability planning
9. Decomposition instructions
10. Self-audit checklist (completion semantics, requirement coverage, task completeness, dependency correctness, key links, scope sanity)
11. DECISIONS.md appending
12. Commit instruction

## Unit Lifecycle

### Pre-Dispatch

1. Pre-dispatch health gate (verify system health)
2. Crash recovery (detect & recover from prior crashes)
3. Hook restoration from disk
4. State derivation (read `.gsd/`)
5. Pre-dispatch hooks (modify prompt, skip, replace unit)
6. Model selection based on task complexity

### Dispatch

7. Create fresh session with clean context window
8. Inject focused prompt with pre-inlined artifacts

### Execution

9. LLM executes task with tool calls
10. Tool tracking (mark start/end, detect stale tools)
11. Context pressure monitor at 70% usage → wrap-up signal

### Post-Execution

12. Verification gate (run configured lint/test commands)
13. Auto-fix retries on failure (max 2 retries)
14. Post-unit hooks (generate docs, run tests, etc.)
15. Unit closeout (save summary, update milestones)
16. Git commit with task-derived message
17. Snapshot metrics (tokens, cost, duration)
18. Persist state to disk
19. Loop to step 1

## Recovery Mechanisms

### Crash Recovery

Lock file (`auto.lock`) tracks current unit. On restart:
1. Detect stale lock
2. Read surviving session file
3. Synthesize recovery briefing from every tool call that made it to disk
4. Resume with full context

### Provider Error Recovery

| Error Type | Strategy |
|-----------|----------|
| Rate limit (429) | Auto-resume after 60s delay |
| Server error (500/503) | Auto-resume after 30s delay |
| Auth error | Pause for manual review |

### Stuck Detection

If the same unit dispatches twice:
1. Retry once with deep diagnostic prompt
2. If still fails, hard stop with exact file expected
3. `/gsd forensics` for structured root-cause analysis

### Timeout Supervision

| Type | Default | Behavior |
|------|---------|----------|
| Soft timeout | 20 min | Warns LLM to wrap up |
| Idle watchdog | 10 min | Detects stalls |
| Hard timeout | 30 min | Pauses auto mode |

## Budget Management

- Cost tracking per unit: tokens (input/output/cache read/write), USD, duration, tool calls, message counts
- Stored in `.gsd/metrics.json`
- Budget ceiling enforcement: `warn` (log), `pause` (stop auto), `halt` (refuse dispatch)
- Cost projections after 2+ slices: per-slice average × remaining
- Budget pressure auto-downgrades model tiers at 50%/75%/90% usage

## Fresh Session Per Unit

Every dispatch creates a new agent session with a clean context window containing only pre-inlined artifacts. This prevents quality degradation from context accumulation.

Key insight: the LLM starts oriented (files already in context) instead of spending tool calls reading files. Each session is optimally packed with relevant information.

## Idempotency & Durability

- `completed-units.json` — persistent set of finished units (survive context resets)
- Unit runtime records — JSON files tracking start/end, artifacts, decisions
- `continue.md` — resume from exact step if interrupted mid-task
- Atomic file writes via `atomicWriteSync()` prevent corruption
- Stale unit cleanup on startup
- Verification caching
