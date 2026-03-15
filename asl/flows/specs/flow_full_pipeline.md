# Full Pipeline

Two entry points (audit, manual), parallel subagents, grading loop, QC monitoring.

## Target Goal

Process work items through a complete pipeline: classify the task, dispatch to parallel subagents or sequential research+compose, grade the output against a 10-category rubric (threshold ≥95), iterate with feedback until passing, and monitor quality via random sampling. Audit entry point runs preset checks; manual entry point takes user prompts.

## Scenarios

### Scenario 1: Audit Entry — Code Quality

`run_pipeline("code-quality")`. GritQL auto-fixes (T0). T1-T3 checks find 12 issues. Per-file verification subagents confirm 9 of 12. Workflow manager produces report.

**Success:** Auto-fix handles trivial issues. Verification reduces false positives from 12 to 9 confirmed findings.

### Scenario 2: Manual — List Classification

User prompt: "review these 5 API endpoints for security issues." Classifier identifies this as a list of 5 items. Orchestrator spawns 5 parallel subagents, one per endpoint. Each returns findings. Results aggregated.

**Success:** All 5 subagents complete. Results ordered by severity. Total time ≈ max(individual times), not sum.

### Scenario 3: Manual — Single/Meta Task

User prompt: "refactor the error handling in src/interpreter/." Classifier identifies as single/meta task. Research agent investigates current patterns. Task breakdown produces 4 subtasks. Composer drafts the refactoring plan. Grader evaluates.

**Success:** Research → breakdown → compose pipeline produces a coherent plan. Grader evaluates it holistically.

### Scenario 4: Grading Loop — First Pass

Grader evaluates work across 10 categories. Score: 82/100. Two categories fail: "completeness" (missed 2 files) and "test coverage" (no new tests). Feedback sent to authoring agent. Agent revises: adds the 2 files and writes tests. Incremental re-grade on only the 2 failed categories. Score: 97. Pass.

**Success:** Grading loop converges in 2 iterations. Incremental re-grade avoids re-evaluating the 8 passing categories.

### Scenario 5: Grading Loop — Max Attempts

Agent produces work that scores 72. Revises to 78. Revises to 81. Revises to 83. Revises to 84. Five attempts exhausted, still below 95.

**Success:** Pipeline reports failure after 5 attempts with the best score achieved (84) and the remaining feedback. Does not loop forever.

### Scenario 6: QC Sampling Catches Suspicious Agent

QC monitor randomly samples a subagent's transcript. Detects: agent has read the same file 8 times in 10 turns. Flags as "stuck." QC interrupts the agent and redirects.

**Success:** QC catches the issue before the agent wastes its full turn budget. Intervention happens without stopping other parallel subagents.

### Scenario 7: Audit + Manual in Same Session

User runs an audit, reviews findings, then submits a manual prompt to fix the top finding. Both paths use the same workflow manager for task tracking.

**Success:** Audit results persist in context. Manual task references the audit finding. Workflow state is consistent across both paths.
