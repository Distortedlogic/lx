# Satisfaction-Based Testing

Agentic flows are non-deterministic. The same input can produce legitimately different outputs. A testing framework for lx must score output quality on a spectrum, not assert binary equality. The framework is written in lx itself — dogfooding the language for meta-level reasoning.

## Problem

`assert` is binary: pass or fail. This works for deterministic language features (`assert (1 + 1 == 2)`) but fails for agentic workflows where:

- An LLM-driven agent produces different phrasing each run
- Multiple valid approaches exist for the same task
- Quality is multi-dimensional (relevance, completeness, format, safety)
- A single run may score 0.6 and the next 0.9 — both are "working"
- You need statistical confidence, not one-shot verification

The current `lx test` runs assert-based `.lx` files. These remain valid for testing deterministic language features. Satisfaction testing is an additional layer for agentic flows.

## Core Concepts

### Spec

A `spec` declares what a flow should do, how to score it, and what threshold constitutes "passing."

### Scenario

A `scenario` is a specific input + rubric pair within a spec. Each scenario can be run multiple times for statistical confidence.

### Grader

A grader function takes `(output, scenario)` and returns a record of dimension scores (0.0–1.0 each). Graders use `std/audit` functions, custom logic, or LLM-as-judge.

### Satisfaction score

Weighted combination of dimension scores. Compared against threshold to determine pass/fail.

## `std/test`

```
use std/test

spec = test.spec "code review agent" {
  flow: "./src/review.lx"
  grader: (output scenario) {
    relevance: audit.references_task output scenario.task
    completeness: audit.rubric output scenario.rubric
    format: audit.has_diff output
    safety: !audit.is_refusal output
  }
  threshold: 0.75
  weights: {relevance: 0.4  completeness: 0.4  format: 0.1  safety: 0.1}
}

test.scenario spec "simple bug fix" {
  input: {task: "fix null check"  file: "src/auth.rs"}
  rubric: ["identifies the null dereference" "proposes a guard" "doesn't break callers"]
  runs: 3
}

test.scenario spec "refactor request" {
  input: {task: "extract method"  file: "src/handler.rs"  target: "process_request"}
  rubric: ["extracts coherent method" "preserves behavior" "improves readability"]
  runs: 5
}

results = test.run spec ^
test.report results
```

### `test.spec`

```
test.spec name opts -> Spec
```

| Field       | Type   | Required | Description                                    |
| ----------- | ------ | -------- | ---------------------------------------------- |
| `flow`      | Str    | yes      | Path to the .lx flow under test                |
| `grader`    | Fn     | yes      | `(output scenario) -> {dimension: Float ...}`  |
| `threshold` | Float  | no       | Pass threshold 0.0–1.0 (default: 0.75)        |
| `weights`   | Record | no       | Dimension weights (default: equal weight)      |
| `setup`     | Fn     | no       | `(scenario) -> ()` run before each scenario    |
| `teardown`  | Fn     | no       | `(scenario) -> ()` run after each scenario     |
| `timeout`   | Int    | no       | Per-run timeout in seconds (default: 300)      |

### `test.scenario`

```
test.scenario spec name opts -> Scenario
```

| Field    | Type     | Required | Description                                   |
| -------- | -------- | -------- | --------------------------------------------- |
| `input`  | Record   | yes      | Input passed to the flow                      |
| `rubric` | [Str]    | no       | Expected behaviors for the grader to check    |
| `runs`   | Int      | no       | Number of times to run (default: from spec or lx.toml) |
| `expect` | Record   | no       | Hard constraints (must-have fields/values)    |
| `tags`   | [Str]    | no       | Tags for filtering (`lx test --tag smoke`)    |

### `test.run`

```
test.run spec -> Result TestResults TestErr
```

Executes all scenarios, runs each `runs` times, scores via grader, aggregates.

Returns:

```
TestResults = {
  spec: Str
  passed: Bool
  score: Float
  scenarios: [{
    name: Str
    passed: Bool
    score: Float
    runs: [{
      scores: Record
      weighted: Float
      output: Any
      elapsed_ms: Int
    }]
  }]
}
```

### `test.report`

```
test.report results -> ()
```

Pretty-prints results:

```
code review agent
  simple bug fix ............ 0.82 PASS (3 runs, mean 0.82, min 0.71, max 0.91)
    relevance:    0.90 (0.85–0.95)
    completeness: 0.78 (0.60–0.90)
    format:       0.80 (0.80–0.80)
    safety:       1.00 (1.00–1.00)
  refactor request .......... 0.69 FAIL (5 runs, mean 0.69, min 0.55, max 0.80)
    relevance:    0.85 (0.70–0.95)
    completeness: 0.52 (0.40–0.70)
    format:       0.70 (0.60–0.80)
    safety:       1.00 (1.00–1.00)

Overall: 0.76 — 1/2 scenarios passed (threshold: 0.75)
```

### `test.run_scenario`

```
test.run_scenario spec scenario -> Result ScenarioResult TestErr
```

Run a single scenario (for debugging).

## Grader Patterns

### Using `std/audit`

```
grader: (output scenario) {
  not_empty: !audit.is_empty output
  not_hedging: !audit.is_hedging output
  references_task: audit.references_task output scenario.task
  rubric_score: audit.rubric output scenario.rubric
  has_code: audit.has_diff output || (output | to_str | contains? "```")
}
```

### LLM-as-judge

```
grader: (output scenario) {
  llm_score: ai.prompt_structured {
    prompt: "Score this output 0-1 on: {scenario.rubric | join ", "}\nOutput: {output | to_str}"
    schema: Protocol JudgeScore = {score: Float where score >= 0.0 && score <= 1.0}
  } ^ | (.score)
  format: audit.has_diff output
}
```

### Composite grader

```
grader: (output scenario) {
  auto: audit.evaluate output scenario.rubric
  llm: ai_judge output scenario
  safety: safety_check output
}
```

## CLI Integration

### `lx test`

When `lx test` finds `.lx` files with `test.spec` calls, it runs satisfaction-based testing. Files with only `assert` statements run in classic binary mode.

```
lx test                          -- run all tests
lx test --tag smoke              -- run scenarios tagged "smoke"
lx test --scenario "simple bug"  -- run specific scenario
lx test --threshold 0.90         -- override threshold
lx test --runs 10                -- override run count
lx test --json                   -- structured output
```

### `lx.toml` integration

```toml
[test]
threshold = 0.75
runs = 3
```

These defaults apply to all specs unless overridden per-spec or per-scenario.

## Implementation

`std/test` is a new stdlib module. The key insight is that it's mostly lx code calling existing stdlib modules (`std/audit`, `std/ai`, `std/trace`). The Rust implementation provides:

1. `test.spec` — constructs a Spec record
2. `test.scenario` — attaches a Scenario to a Spec
3. `test.run` — orchestrates execution: for each scenario, for each run, invoke the flow, capture output, call grader, compute weighted score, aggregate
4. `test.report` — formatted output to emit backend
5. `test.run_scenario` — single-scenario execution

### Flow invocation

The flow under test is invoked via the interpreter with a fresh environment. The `input` record is passed as the flow's argument (available via `env.args` or as module-level bindings).

### Score aggregation

Per-run: weighted mean of dimension scores.
Per-scenario: mean of per-run scores.
Per-spec: mean of per-scenario scores.

### Dependencies

- `std/audit` (rubric scoring, content checks)
- `std/time` (elapsed measurement)
- `std/json` (structured output)
- `RuntimeCtx` backends (for flow execution)

## Cross-References

- Binary testing: [toolchain.md](toolchain.md) (Test Runner section)
- Audit functions: [stdlib-agents.md](stdlib-agents.md) (`std/audit`)
- Package manifest: [package-manifest.md](package-manifest.md) (`[test]` section)
- Refine loop: `refine` expression — grading within a flow vs testing the flow itself
- AI scoring: `ai.prompt_structured` — LLM-as-judge pattern
