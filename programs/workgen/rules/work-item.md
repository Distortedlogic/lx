# Work Item Details Creation Process

This rule describes the multi-phase process for taking a work item from a brief description to a fully fleshed out, implementation-ready specification with an embedded task list.

## Phase 1: Deep Codebase Investigation

Be full, complete, robust, deep, thorough, comprehensive, and rigorous. Exhaustively search the codebase to understand every file, struct, function, trait, and code path relevant to the work item. Do not stop at surface-level understanding.

- Use `codebase-retrieval` to find all code related to the work item's domain
- Use `view` to read every file identified as relevant
- Map out all data structures, types, and relationships involved
- Trace all code paths that will be affected by the change
- Identify every file that will need changes
- Identify cross-cutting concerns (anything the change touches beyond the primary target — callers, consumers, tests, serialization, etc.)
- Cover ALL sub-domains and aspects the user specifies — if the work item touches multiple systems or concepts, investigate each one fully
- Evaluate whether existing data structures and patterns match known optimal conventions for the domain — flag any that don't

Do NOT proceed to Phase 2 until you have read every relevant file and can name every type, field, function signature, and call site involved.

## Phase 2: Findings Enumeration

Present findings to the user as a numbered list. Each finding must:

- Name a specific concrete problem or issue that needs addressing
- State the root cause (not "looks wrong" — name the structural problem)
- Provide exactly one fix (the best-practice, highly standard, most idiomatic long-term solution)
- Cover gotchas, edge cases, and interactions with other findings
- Include data structure or convention mismatches identified in Phase 1

Rules for findings:

- Report ONLY problems — do not list things that are correct or working
- Verbal only — no code snippets
- Do not be overly verbose but remain clear, no ambiguity
- Do NOT provide multiple options — provide ONLY the single best solution
- Do all investigating upfront to provide the absolute best long-term solution without deferring work to later
- If a finding belongs in a separate work item (tangential to the primary goal), flag it as such

## Phase 3: Work Item Description Composition

Compose the full work item description with these sections:

1. **Goal** — One paragraph stating what changes and why, incorporating all decisions from discussion
2. **Why** — Bullet list of concrete problems this solves, with measurable impact where applicable
3. **What changes** — Detailed enumeration of every type change, new field, new function, behavioral change, organized by component
4. **How it works** — Explain the mechanics of the change for any non-obvious aspects — how different code paths are affected, how components interact after the change
5. **Additional sections as needed** — Any topic that needs dedicated explanation based on the specific domain of the work item
6. **Files affected** — Every file that needs changes with a brief note on what changes in each
7. **Task List** — Embedded executable task list (see Phase 4)

Remove any backward compatibility concerns unless the user explicitly states production constraints. Keep the description as simple as possible.

**No code snippets** — Describe all changes verbally. Do not embed code blocks or multi-line inline code. Reference identifiers (type names, field names, function names) as plain text. A single-expression snippet is acceptable only when a verbal description would be genuinely ambiguous; multi-line code blocks are never acceptable anywhere in the work item file.

## Phase 4: Task List Creation

Embed a task list at the bottom of the work item description in the internal task list format. The task list must be full, complete, robust, deep, thorough, comprehensive, and rigorous. It must follow these rules:

**Structure:**

- Use Claude Code's native TaskCreate format. Each task has:
  - **subject**: Brief imperative title (e.g., "Update StrategyConfig struct fields")
  - **description**: Detailed implementation instructions (exact files, changes, verification)
  - **activeForm**: Present continuous form shown in spinner while in progress (e.g., "Updating StrategyConfig struct fields")
- Tasks support dependency ordering via `blockedBy` relationships to enforce execution order
- Statuses: `pending` → `in_progress` → `completed`
- One root-level task group per logical unit of work, with subtasks for granular steps
- Subtasks under a parent when a task involves more than one distinct code change or more than one file edit
- No task description may exceed 10 lines — split into subtasks if it does
- Tasks should be optimally structured where the nested task groups and the granularity of the task details are optimal both in size and scope

**Ordering:**

- Tasks ordered based on the impact of the code changes for a task on prior and subsequent tasks
- Foundation changes first (dependencies, type changes), then core logic, then consumers, then tests
- Each task's code changes should not break compilation when applied in sequence

**Content per task:**

- Exact files to edit
- Exact changes to make (field names, function names, type changes)
- No ambiguity — write it so the executor cannot misinterpret what to do
- No conversational context — only implementation instructions
- Ensure there are no caveats, gotchas, points the executor might get stuck trying to implement, or places the executor might attempt to diverge or tangent based on the task details
- Task details should be robust, properly formatted, and without any conversational context not related to doing the task
- **No code snippets in tasks** — Describe what to change verbally, not with code blocks. The executor can read the source files referenced in the task. Reference identifiers (type names, field names, function names) inline as plain text. A single-expression snippet is acceptable only when the verbal description would be genuinely ambiguous; multi-line code blocks are never acceptable in task descriptions.

**MANDATORY: Use workflow MCP tools for all non-implementation operations:**

All formatting, committing, verification, and cleanup operations MUST be performed by calling the workflow MCP tools. The workflow MCP server (registered as `workflow` in `.mcp.json`) provides these tools:

- `mcp__workflow__complete_task` — no params — completes the current in-progress task. Runs fmt, commit, and diagnostics automatically. On the last task, also runs tests and cleans up the work item file.
- `mcp__workflow__next_task` — no params — returns the next actionable task and marks it in progress.
- `mcp__workflow__load_work_item` — params: `{ path: "<relative path>" }` — loads a work item from a markdown file. Returns context and first task.
- `mcp__workflow__start_work_item` — params: `{ brief: "<description>" }` — begins a new work item creation flow.
- `mcp__workflow__advance_phase` — params: `{ content: "<phase output>" }` — submits phase output and advances to the next phase.
- `mcp__workflow__verify` — no params — runs the full test suite and diagnostics.
- `mcp__workflow__get_guide` — params: `{ name: "<guide name>" }` — returns the content of a named guide.

**Tasks are implementation-only:** Task lists contain only implementation tasks. Do not add commit, verify, format, or cleanup tasks — the MCP handles these operations automatically via `complete_task`. After each implementation task, call `complete_task` which formats, commits, and runs diagnostics. On the last task, `complete_task` also runs the full test suite and removes the work item file.

**CRITICAL REMINDERS footer — MUST be embedded in every generated task list:**

Every task list written into a work item markdown file MUST end with the following section, copied verbatim after the last task:

```markdown
---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
```

**Quality gates:**

- Write tasks so they cannot be marked complete without actually doing the work
- Write tasks so the executor cannot lie about completion — each task should have a verifiable outcome
- Include explicit verification steps where applicable

## Phase 5: Write to File

Write the full output of Phase 3 (work item description) and Phase 4 (task list) into a single markdown file in the `work_items/` directory at the project root. Create the directory if it does not exist. The file must follow the naming convention:

`work_items/{DESCRIPTION_HERE}.md`

Where `{DESCRIPTION_HERE}` is a short, UPPER_SNAKE_CASE summary of the work item (e.g., `work_items/REFACTOR_SCORE_METRICS.md`, `work_items/ADD_REGIME_DETECTION.md`).

Do not redisplay the full content in chat — just confirm the file was written and state the filename.

## Phase 6: Subagent Verification Loop

After writing the file, spawn a verification subagent to rigorously audit the work item file against the codebase and the rules in this document. This is an iterative process — the main agent and subagent go back and forth until the work item meets quality standards.

**Subagent prompt construction:**

Spawn the subagent using the Agent tool with `subagent_type: "general-purpose"` and the following prompt:

```
Read the verification agent instructions at `subagents/work-item-verifier.md`.
Then read the work item creation rules at `rules/work-item.md` (Phases 1-5).
Then read the audit rules at `rules/rust-audit.md` and `rules/primitives-audit.md`.
Then read the work item file at `<path_to_work_item_md>`.
Then systematically execute every check in the verification checklist against
the work item file and the actual codebase. Output your results in the
specified format with failures, warnings, and a grade.
```

Replace `<path_to_work_item_md>` with the actual path written in Phase 5.

**Iteration protocol:**

1. Spawn the verification subagent
2. Read the subagent's report
3. If the grade is **95 or above** — verification passes. Proceed to Phase 7.
4. If the grade is **below 95**:
   a. Fix every failure and address every warning the subagent reported by editing the work item file directly
   b. Re-spawn the verification subagent with the same prompt (it re-reads the updated file)
   c. Repeat from step 2
5. There is no iteration cap — keep iterating until the grade exceeds 95

**Rules for the main agent during iteration:**

- Fix ALL reported failures before re-spawning the subagent — do not fix only some and hope the grade improves enough
- Do not argue with the subagent's findings — fix them. If a finding is genuinely wrong (the subagent misread the codebase), verify by reading the relevant code yourself, then fix the subagent prompt to include the clarification and re-spawn
- Do not inflate the grade by weakening the work item — fixes must improve correctness, not remove content to avoid checks
- Each iteration should make meaningful progress — if the same failures persist across 2 consecutive iterations, re-investigate the root cause before re-spawning

