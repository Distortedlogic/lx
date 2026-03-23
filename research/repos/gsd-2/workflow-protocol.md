# GSD-2: Workflow Protocol

## Hierarchical Work Decomposition

```
Milestone  →  a shippable version (4-10 slices)
  Slice    →  one demoable vertical capability (1-7 tasks)
    Task   →  one context-window-sized unit of work
```

**The Iron Rule:** A task MUST fit in one context window. If it can't, it's two tasks.

Milestone = "what ships." Slice = "what the user can demo after." Task = "what the LLM can do in one session."

## The Seven Phases

```
1. Discuss (Optional)  → Captures user decisions on gray areas  → M###-CONTEXT.md
2. Research (Optional)  → Scouts codebase/docs                  → M###-RESEARCH.md / S##-RESEARCH.md
3. Plan                 → Decomposes into tasks                  → S##-PLAN.md + T##-PLAN.md
4. Execute              → Do the work, mark progress             → Code changes
5. Verify               → Check must-haves actually met          → Static/command/behavioral/human checks
6. Summarize            → Record what happened for downstream    → T##-SUMMARY.md, S##-SUMMARY.md
7. Advance              → Mark done, update STATE.md             → Next unit
```

## Auto-Mode Phase Flow

```
Research → Plan → Execute (per task) → Complete → Reassess Roadmap → Next Slice
                                                                      ↓ (all slices done)
                                                              Validate Milestone → Complete
```

## File Formats

### M###-ROADMAP.md

```markdown
# M001: Title

**Vision:** One paragraph describing what this milestone delivers.

**Success Criteria:**
- Observable outcome 1
- Observable outcome 2

## Slices

- [ ] **S01: Slice Title** `risk:low` `depends:[]`
  > After this: what user can demo

- [x] **S02: Completed Slice** `risk:medium` `depends:[S01]`
  > After this: demo sentence

## Boundary Map

| Slice | Produces | Consumes |
|-------|----------|----------|
| S01 | auth types, JWT helpers | — |
| S02 | login UI, form validation | S01: auth types |
```

The boundary map forces interface thinking before implementation. Each slice declares what it produces and what it consumes from other slices.

### S##-PLAN.md

```markdown
# S01: Slice Title

**Goal:** What this slice achieves.
**Demo:** What the user can see/do when done.

## Must-Haves
- Observable outcome 1
- Observable outcome 2

## Tasks
- [ ] **T01: Task Title**
  Description of what to do.
  - Files: `src/lib/auth.ts`, `src/routes/login.ts`
  - Verify: `npm run test -- auth`

- [ ] **T02: Task Title**
  ...

## Files Likely Touched
- path/to/file.ts
```

### T##-PLAN.md

```markdown
# T01: Task Title

**Slice:** S01
**Milestone:** M001

## Goal
One sentence.

## Must-Haves

### Truths
Observable behaviors when done:
- "User can sign up with email and password"

### Artifacts
Files with real implementation (min line counts):
- `src/lib/auth.ts` — JWT helpers (min 30 lines)

### Key Links
Critical wiring:
- `login/route.ts` → `auth.ts` via import of `generateToken`

## Steps
1. First thing to do
2. Second thing
...

## Context
Relevant prior decisions or patterns to follow.
```

### T##-SUMMARY.md (YAML Frontmatter + Narrative)

```markdown
---
id: T01
parent: S01
milestone: M001
provides:
  - What this task built (~5 items)
requires:
  - slice: S00
    provides: What that prior slice built
affects: [S02, S03]
key_files:
  - path/to/important/file.ts
key_decisions:
  - "Decision made: reasoning"
patterns_established:
  - "Pattern name and where it lives"
drill_down_paths:
  - .gsd/milestones/M001/slices/S01/tasks/T01-PLAN.md
duration: 15min
verification_result: pass
completed_at: 2026-03-07T16:00:00Z
---

# T01: Task Title

**Substantive one-liner — NOT "task complete" but what actually shipped**

## What Happened
Concise prose narrative of what was built.

## Deviations
What differed from the plan and why (or "None").

## Files Created/Modified
- `path/to/file.ts` — What it does
```

### continue.md (Resume Protocol)

Written when about to lose context but task isn't done:

```markdown
---
milestone: M001
slice: S01
task: T02
step: 3
total_steps: 7
saved_at: 2026-03-07T15:30:00Z
---

## Completed Work
What's already done.

## Remaining Work
What steps remain, with enough detail to resume.

## Decisions Made
Key decisions and WHY (so next session doesn't re-debate).

## Context
The "vibe" — what you were thinking, what's tricky.

## Next Action
The EXACT first thing to do when resuming.
```

## Verification Ladder

Strongest tier you can reach, in order:

| Tier | Method | Example |
|------|--------|---------|
| **Static** | Files exist, exports present, wiring connected, not stubs | Check file size > min lines |
| **Command** | Tests pass, build succeeds, lint clean | `npm run test -- auth` |
| **Behavioral** | Browser flows work, API responses correct | Screenshot comparison |
| **Human** | Ask user when you genuinely can't verify yourself | UAT script |

## Verification Evidence Table

Each task records a verification evidence table:

```markdown
| Check | Command | Exit | Verdict | Duration |
|-------|---------|------|---------|----------|
| Unit tests | npm run test | 0 | pass | 4.2s |
| Type check | npm run typecheck | 0 | pass | 8.1s |
| Lint | npm run lint | 1 | fail | 2.3s |
```

## State Management

**STATE.md** is a derived cache (not authoritative). Rebuilt from sources of truth:
- `M###-ROADMAP.md` → which slices done
- `S##-PLAN.md` → which tasks exist
- `T##-SUMMARY.md` → what happened
- `S##-SUMMARY.md` and `M###-SUMMARY.md` → compressed outcomes

STATE.md contains: active milestone/slice/task, recent decisions (last 3-5), blockers, and next action.

## Summary Injection for Downstream Tasks

When planning/executing a task, load relevant prior context:
1. Check `depends:[]` in roadmap
2. Load summaries from those slices
3. Start with highest available level (milestone summary first)
4. Drill down only if you need specific detail
5. Stay within ~2500 tokens of injected summary context

Target per summary: ~5 provides, ~10 key_files, ~5 key_decisions, ~3 patterns_established.

## Context Artifacts

| Artifact | Purpose | Tracked |
|----------|---------|---------|
| PROJECT.md | Living doc — what the project is right now | Yes |
| DECISIONS.md | Append-only register of architectural decisions | Yes |
| REQUIREMENTS.md | Project requirements | Yes |
| QUEUE.md | Future milestone queue | Yes |
| KNOWLEDGE.md | Cross-session memory, lessons, rules | Yes |
| STATE.md | Quick-glance dashboard — always read first | No (derived) |
| CAPTURES.md | Pending thoughts for triage | No |

## Git Strategy

### Isolation Modes

| Mode | Mechanism | Location |
|------|-----------|----------|
| `worktree` (default) | Separate git worktree | `.gsd/worktrees/<MID>/` on `milestone/<MID>` branch |
| `branch` | Branch in project root | Project root on `milestone/<MID>` branch |
| `none` | Direct commits | Current branch, no isolation |

### Commit Strategy

- Sequential commits on one milestone branch
- Conventional format: `feat(S01/T01): description`
- Squash-merge to main as one clean commit per milestone
- Worktree torn down after merge
- Git bisect works; individual milestones are revertable

**Main looks like:**
```
feat(M001/S03): milestone and slice discuss commands
feat(M001/S02): extension scaffold and command routing
feat(M001/S01): file I/O foundation
```

**Branch looks like:**
```
milestone/M001:
  test(S01/T03): round-trip tests passing
  feat(S01/T03): file writer
  feat(S01/T02): markdown parser
  feat(S01/T01): core types
```

### Workflow Modes

| Setting | Solo | Team |
|---------|------|------|
| auto_push | true | false |
| merge_strategy | squash | squash |
| milestone IDs | simple (M001) | unique |
| push_branches | false | true |
| pre_merge_check | false | true |

### Self-Healing

- Detached HEAD fix
- Stale lock removal
- Orphaned worktree cleanup
- Native git operations (libgit2) for read-heavy dispatch hot path
