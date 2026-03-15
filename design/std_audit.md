# std/audit Design — PLANNED, NOT IMPLEMENTED

Structural response quality checks with configurable rubrics.

## Why Stdlib

Every agentic flow has a quality gate. The flow_full_pipeline has a 10-category grading loop. The agentic loop has verification. The post-hoc review extracts mistakes. The defense layers have guardrails. The pattern is universal: agent produces output → something checks it → pass or feedback for revision.

The structural checks (response too short, repetitive, self-contradicting, references nonexistent things) don't need an LLM. They're deterministic and fast. LLM-based evaluation (is this correct? is it thorough?) is a separate concern handled by an auditor agent that builds on top of these checks.

## API

```
use std/audit

rubric = audit.rubric [
  {name: "completeness"  weight: 25  check: (r) r.output | len > 100}
  {name: "no_hallucination"  weight: 25  check: (r) audit.files_exist r.referenced_files}
  {name: "not_lazy"  weight: 25  check: (r) !audit.is_hedging r.output}
  {name: "follows_task"  weight: 25  check: (r) audit.references_task r.output r.task}
]

result = audit.evaluate rubric {
  output: agent_response
  context: available_context
  task: task_description
}

result.score        -- Int (0-100, weighted)
result.passed       -- Bool (score >= threshold, default 95)
result.categories   -- [{name: Str  score: Int  passed: Bool}]
result.feedback     -- Str (combined feedback from failed categories)
result.failed       -- [Str] (names of failed categories)
```

## Built-in Checks

Structural checks that don't need an LLM:

```
audit.is_empty output               -- Bool: output is blank or whitespace-only
audit.is_too_short output min_len   -- Bool: under min_len characters
audit.is_repetitive output          -- Bool: >50% of sentences repeat
audit.is_hedging output             -- Bool: contains "I think", "maybe", "possibly", "I'm not sure"
audit.is_refusal output             -- Bool: contains "I can't", "I'm unable", "as an AI"
audit.references_task output task   -- Bool: output mentions key terms from task
audit.files_exist paths             -- Bool: all file paths in list actually exist on disk
audit.has_diff output               -- Bool: output contains actual changes, not just description
```

These are fast (regex + string matching + filesystem checks). They catch the obvious failures without burning tokens on LLM evaluation.

## Rubric Model

A rubric is a list of categories. Each category has:
- `name` — identifier for feedback
- `weight` — percentage of total score (all weights should sum to 100)
- `check` — function `(record) -> Bool` that returns true if the category passes

The evaluate function runs all checks, computes weighted score, determines pass/fail against threshold.

## Simple Mode

For flows that just need pass/fail without a rubric:

```
audit.quick_check {
  output: response
  min_length: 100
  no_hedging: true
  no_refusal: true
  task: task_description
}
```

Returns `{passed: Bool  reasons: [Str]}`. No scores, no weights — just a list of reasons it failed.

## Interaction with std/tasks

The audit module produces results. The task module consumes them for state transitions. They don't import each other. User code composes them:

```
use std/tasks
use std/audit

result = audit.evaluate rubric {output: draft  context: ctx  task: task}
result.passed ? {
  true -> tasks.pass store task_id
  false -> {
    tasks.fail store task_id {feedback: result.feedback}
    agent ~>? {action: "revise"  work: draft  feedback: result.feedback  focus: result.failed} ^
  }
}
```

## What std/audit Does NOT Do

- LLM-based evaluation (correctness, depth, creativity) — that's an auditor AGENT that calls an LLM and uses std/audit for structural pre-checks
- Semantic similarity (embedding-based dedup) — that's a future module or MCP tool
- Code execution / test running — that's shell integration (`$^cargo test`)

The boundary: std/audit handles what can be checked with string matching, regex, and filesystem access. Anything requiring an LLM is agent-level, not stdlib-level.

## Implementation

Pure functions operating on strings and records. No state, no persistence, no threads. The rubric is a list of records. Evaluate iterates the list, calls each check function via `call_value`, aggregates results.

Estimated: ~120 lines of Rust. One stdlib file.
