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

**MANDATORY: Justfile recipes only — NEVER raw cargo commands:**

- Every task that runs a build, check, test, format, lint, or clippy operation MUST use the corresponding `just` recipe (`just fmt`, `just test`, `just diagnose`, `just fix`).
- NEVER run `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`, or any other raw `cargo` command directly — not in task descriptions, not during execution, not as a "quick check." The `just` recipes wrap `cargo` with additional steps (e.g., `just fmt` runs `dx fmt`, `cargo fmt`, AND `eclint`). Running raw `cargo` commands skips these steps and produces incorrect results.
- This applies to ALL tasks in the task list — implementation tasks, format tasks, verification tasks, and any ad-hoc commands.

**After each implementation task — format and commit ONLY (EACH AS ITS OWN INDEPENDENT TASK):**

After every implementation task (or task group), append EXACTLY the following two tasks as separate, independent tasks. These MUST NOT be combined into a single task or a single command. Each task below is its own standalone task in the task list. The commands must be written verbatim in the task description and the task must explicitly instruct the executor to run the command exactly as written, verbatim, with no modifications.

1. **Format task** — Task description must state: "Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way."
2. **Commit task** — Task description must state: "Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m \"<descriptive message for the preceding implementation task>\"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way." (replace the message with an appropriate commit message for the work just completed). CRITICAL: The commit command must be a plain string — do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution in the commit message. Just a simple `-m "message"` flag with a plain string. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to the commit command. The commit message is ONLY the `-m "message"` flag — nothing else.

**PROHIBITED between implementation tasks:** Do NOT insert `just test`, `just diagnose`, `cargo test`, `cargo check`, `cargo clippy`, or ANY verification/compilation-check command between implementation tasks. The ONLY commands that appear between implementation tasks are `just fmt` and `git commit`. Testing and diagnostics are EXPENSIVE and MUST ONLY appear in the final verification section below. Violating this by sneaking in early verification is explicitly forbidden.

**Final verification tasks (EACH AS ITS OWN INDEPENDENT TASK):**

The final tasks in the task list must be the following verification and cleanup steps. These are the ONLY place in the entire task list where `just test` and `just diagnose` may appear. These MUST NOT be combined into a single task or a single command. Each one below is its own standalone task. The commands must be written verbatim in the task description and the task must explicitly instruct the executor to run the command exactly as written, verbatim, with no modifications.

1. **Run tests** — Task description must state: "Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass."
2. **Run diagnostics** — Task description must state: "Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings."
3. **Final format** — Task description must state: "Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators."
4. **Final commit** — Task description must state: "Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m \"fix: final verification cleanup\"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m \"message\"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata."
5. **Remove work item file** — Task description must state: "Run the following command verbatim, exactly as written, with no modifications: `rm <path_to_work_item_md> && git add -A && git commit -m \"chore: remove completed work item\"`. Replace `<path_to_work_item_md>` with the actual path to the work item markdown file that this task list was loaded from (e.g., `work_items/REFACTOR_SCORE_METRICS.md`). Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata."

**CRITICAL REMINDERS footer — MUST be embedded in every generated task list:**

Every task list written into a work item markdown file MUST end with the following section, copied verbatim after the last task:

```markdown
---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

These are rules you (the executor) are known to violate. Re-read before starting each task:

1. **NEVER run raw cargo commands.** No `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`. ALWAYS use `just fmt`, `just test`, `just diagnose`, `just fix`. The `just` recipes include additional steps that raw `cargo` skips.
2. **Between implementation tasks, ONLY run `just fmt` + `git commit`.** Do NOT run `just test`, `just diagnose`, or any compilation/verification command between implementation tasks. These are expensive and only belong in the FINAL VERIFICATION section at the end.
3. **Run commands VERBATIM.** Copy-paste the exact command from the task description. Do not modify, "improve," or substitute commands. Do NOT append `--trailer`, `Co-authored-by`, or any metadata to commit commands.
4. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
5. **`just fmt` is not `cargo fmt`.** `just fmt` runs `dx fmt` + `cargo fmt` + `eclint`. Running `cargo fmt` alone is wrong.
6. **Do NOT modify verbatim commands.** No piping (`| tail`, `| head`, `| grep`), no redirects (`2>&1`, `> /dev/null`), no subshells. Run the exact command string as written — nothing appended, nothing prepended.
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

## Phase 7: Audit Against Rust Quality Checks

Review the entire work item markdown file written in Phase 5 and verify that none of the proposed changes, new code, or task instructions would introduce violations of any check defined in `rules/rust-codebase-audit.md`. If any task or proposed change would violate a check, fix the work item file before proceeding.

## Phase 8: Dioxus Audit (Conditional)

If any tasks in the work item involve Dioxus-related code (components, hooks, signals, UI rendering, etc.), review the entire work item markdown file and verify that none of the proposed changes would introduce violations of any check defined in `rules/dioxus-audit.md`. If any task or proposed change would violate a check, fix the work item file before proceeding.

Skip this phase if no tasks involve Dioxus code.

## Phase 9: Embed Task Loading Instructions

Append a `## Task Loading Instructions` section to the work item markdown file (after the CRITICAL REMINDERS footer). This section tells a future executor in a separate, contextless session how to mechanically load and execute the task list. It does NOT repeat the task list — it references the `## Task List` section already in the file.

The embedded instructions must contain the following (adapt wording to fit the specific work item, but preserve all constraints):

1. **How to load:** Read each `### Task N:` entry from the `## Task List` section above. For each task, call `TaskCreate` with:
   - `subject`: The task heading text (after `### Task N:`) — copied VERBATIM, not paraphrased
   - `description`: The full body text under that heading — copied VERBATIM, not paraphrased, summarized, or reworded. Every sentence, every command, every instruction must be transferred exactly as written. Do NOT omit lines, rephrase instructions, drop the "verbatim" language from command instructions, or inject your own wording.
   - `activeForm`: A present-continuous form of the subject (e.g., "Replacing outline-none in CSS")
2. **Dependency ordering:** After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.
3. **Execution rules** (embed these verbatim):
   - Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done
   - Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa
   - Do not run any command not specified in the current task
   - Do not "pre-check" compilation between implementation tasks — the task list already has verification in the correct places
   - If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands.
   - Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section
   - When a task description says "Run the following command verbatim, exactly as written, with no modifications" — that phrase and the command after it must appear identically in the loaded task. Do not drop the "verbatim" instruction or rephrase the command.
   - Do NOT append shell operators to commands — no pipes (`|`), no redirects (`>`/`2>&1`), no subshells. The command in the task description is the complete command string.
