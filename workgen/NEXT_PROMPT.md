# workgen — Cold Start Prompt

Read this first when picking up workgen work in a fresh agent.

## Tooling

`lx` is installed as a release binary at `/home/entropybender/.cargo/bin/lx`. Use it directly — do NOT use `cargo run -p lx-cli --`. Examples:

```
lx run workgen/tests/run.lx          # run tests
lx run workgen/run.lx                # run workgen
lx check some_file.lx                # type check
lx test tests/                        # run lx test suite
```

The justfile recipes still use `cargo run` for consistency with the rest of the repo, but for ad-hoc execution during development, use `lx` directly — it's the release build and faster.

## What This Is

workgen is an lx program that automates work-item generation from audit checklists. The user currently does this manually in Claude Code: "read ./rules/rust-audit then go through all phases in ./rules/work-item.md to produce a work-item doc." workgen automates that entire multi-phase process.

It lives at `workgen/` in the lx repo (the language it's written in). Read `agent/NEXT_PROMPT.md` for lx language context and `agent/FEATURES.md` for the full language reference.

## File Map

| File | Purpose |
|------|---------|
| `workgen/main.lx` | Core program — all functions, no auto-execution. Exports `main`, `run`. |
| `workgen/run.lx` | Entry point — imports main.lx, calls `main ()`. Used by justfile. |
| `workgen/tests/spec.lx` | Satisfaction spec — scenarios, grader, thresholds, expected findings. |
| `workgen/tests/run.lx` | Test runner — executes scenarios, grades output, reports scores. |
| `workgen/tests/fixtures/` | 3 test scenarios with real Rust code containing planted violations. |

## How workgen Works

Pipeline: `read audit → gather context → investigate (ai.prompt) → compose (ai.prompt) → write draft → verify loop (grader.grade + revise) → auditor.audit → done`

Stdlib modules used:
- `std/fs` — read/write/mkdir
- `std/ai` — `ai.prompt` for LLM calls (investigation, composition, revision)
- `std/env` — `AUDIT_FILE` and `RULES_FILE` env vars
- `std/md` — `md.parse`, `md.headings` for structural document validation
- `std/audit` — `audit.rubric` (rubric validation), `audit.quick_check` (structural pre-check)
- `std/trace` — session tracing, `trace.record` with score field, `trace.should_stop` for diminishing returns
- `std/prompt` — `create`, `system`, `section`, `instruction`, `constraint`, `render`, `estimate`
- `std/agents/grader` — `grader.grade` with `rubric`, `threshold`, `previous_grades` (incremental re-grading)
- `std/agents/auditor` — `auditor.audit` (final LLM-backed quality gate)

Key design: `main.lx` does NOT auto-execute. `+main` is an exported function. `run.lx` is the thin entry point that calls `main ()`. This allows `tests/run.lx` to `use ../main : workgen` and call `workgen.run` directly without triggering auto-execution.

## Justfile Recipes

| Recipe | What it does |
|--------|-------------|
| `just install` | `cargo install --path crates/lx-cli` — system-wide lx binary |
| `just audit` | Interactive fzf chooser over `rules/*audit*` files → runs workgen |
| `just audit-file rules/foo` | Direct invocation with specific audit file |
| `just audit-test` | Run satisfaction tests |
| `just audit-test smoke` | Run only smoke-tagged scenarios |

## Current Task: Fix the Test Runner

The test runner (`workgen/tests/run.lx`) has runtime errors that need debugging and fixing. Here's the state:

### What works
- The runner starts, loads scenarios, emits header output
- Module imports resolve correctly (`use ./spec`, `use ../main : workgen`)
- `trace.create` API is correct (takes file path string, not record)
- `trace.record` API is correct (data-last: `trace.record {fields} session ^`)
- `trace.should_stop` API is correct (`trace.should_stop {min_delta window} session`)

### What's broken

**Runtime error in test runner:**
```
type error: split: second arg must be Str
  ╭─[workgen/tests/run.lx:53:22]
 52 │       emit "    ERROR: {e}"
 53 │       {name: scenario.name  passed: false  score: 0.0}
                       ──────────┬─────────
                                 ╰── split: second arg must be Str
```

The error occurs when constructing a record literal `{name: scenario.name ...}` inside the `Err e ->` branch of `run_scenario`. The `scenario.name` access is being parsed or evaluated incorrectly — possibly `name` is being interpreted as the builtin string function `name` rather than the record field, or there's a scoping issue with `name:` as a record field key colliding with something.

This is the **blocking issue**. The error happens on the first scenario because `workgen.run` returns Err (likely because `ai.prompt` fails without a live AI backend in test mode). The Err branch tries to construct a result record and crashes.

**Likely root cause:** The `name:` field key in the record literal `{name: scenario.name ...}` may conflict with a builtin or the `name` binding in scope. Try renaming the field or using a different construction approach.

**Secondary concern:** `ai.prompt` will fail in test mode (no live Claude backend). The test runner needs to either:
1. Mock the AI backend somehow, or
2. Test against pre-generated workgen output (golden files), or
3. Run workgen for real (expensive, requires live AI), or
4. Test only the structural/grading parts independently without running the full pipeline

Option 4 is probably best for initial testing: write golden output files (expected work-item documents) into each fixture directory, then have the test runner grade THOSE against the spec, bypassing the `ai.prompt` calls entirely. This tests the grading/spec logic without needing a live AI. Later, a `--live` flag can run the full pipeline.

### Parser bug discovered (not blocking, document for reference)

`refine` expression inside a function body fails to parse when the initial value is a parameter:
```lx
f = (content) {
  refine content { grade: ... revise: ... threshold: 95 max_rounds: 3 }
}
-- parse error: expected LBrace, found Semi
```

Workaround in `main.lx`: the verify loop uses a manual `loop` with `break` instead of `refine`. This works correctly. The bug is in the parser — `refine` works fine at top level but fails when its initial expression is a closure-captured variable inside a block.

## Test Fixtures

Each fixture is a self-contained mini-project with planted code violations:

### `fixtures/unwrap_audit/`
- `src/main.rs` — 7 `.unwrap()` calls, file I/O without error handling
- `src/db.rs` — `.unwrap()` on mutex locks, `panic!()` in library code, mutable static
- `rules/audit` — 5 rules targeting unwrap, panic, mutable statics, Result returns, error context
- Expected findings: "unwrap", "panic", "mutable static", "Result", "error"

### `fixtures/error_handling_audit/`
- `src/service.rs` — `.unwrap_or_default()`, `let _ =`, `.ok()` on errors, string-typed errors, silent empty fallbacks
- `src/api.rs` — `.unwrap_or_default()`, string error returns, swallowed file I/O errors
- `rules/audit` — 6 rules targeting swallowed errors, string errors, silent fallbacks, missing context
- Expected findings: "swallow", "unwrap_or_default", "String", "silent", "context", ".ok()"

### `fixtures/style_audit/`
- `src/handler.rs` — wildcard imports, TODO/FIXME comments, inline import in function body, vague function name
- `src/utils.rs` — free functions that should be methods, redundant variable bindings
- `src/types.rs` — duplicate struct fields across Config and DbConfig
- `rules/audit` — 7 rules targeting wildcards, TODOs, inline imports, redundant bindings, duplicate fields, methods, vague names
- Expected findings: "wildcard", "TODO", "FIXME", "inline import", "duplicate", "method", "do_the_thing"

## Satisfaction Testing Design

Based on `spec/testing-satisfaction.md` (the spec for `std/test` which isn't implemented yet). workgen's tests pseudo-implement the pattern:

**Spec** (`spec.lx`): Defines grading dimensions, weights, threshold.
- `structure` (0.25) — required markdown sections present (Goal, Why, Task List, etc.)
- `coverage` (0.30) — expected findings keywords found in output
- `compliance` (0.25) — process compliance (just fmt, git commit, just test, Loading instructions)
- `safety` (0.20) — not empty, not hedging, not refusal

**Runner** (`run.lx`): Executes scenarios, applies grader, aggregates weighted scores, reports.

**Grader** (in `spec.lx`): Uses `std/md` for section detection, `std/audit` for safety checks, string matching for coverage and compliance.

## lx Syntax Reminders

- `+fn_name = (args) { body }` — exported function (importable)
- `fn_name = (args) { body }` — private function
- `use ./path` — relative import, resolves from file's directory
- `use ./path : alias` — aliased import
- `x | f` — pipe (data-last)
- `x ^` — unwrap Ok or propagate Err
- `x ?? default` — coalesce Err/None to default
- `x ? { Ok v -> ... Err e -> ... }` — pattern match on Result
- `emit "text"` — stdout output
- `$^cmd` — shell capture (stdout as string)
- `$cmd` — shell run (returns Result with .out .err .code)
- Records: `{name: value  other: value}` (space-separated, no commas)
- Lists: `[1 2 3]` (space-separated)
- `x | lines` — split string into lines list
- `x | split "/" | last` — split and take last
- Functions don't auto-execute. `+main = () { ... }` just defines a function. You need `main ()` at the bottom to run it, or a separate entry point file.
- Module `use` evaluates the file — any top-level `main ()` call in an imported module WILL execute during import. Keep entry points in separate files.

## Key stdlib APIs (correct signatures from tests)

```
trace.create "path.json" ^                              -- returns trace handle
trace.record {name: "x"  score: 0.5  output: "y"} t ^  -- data-last
trace.should_stop {min_delta: 2.0  window: 3} t         -- returns Bool
trace.improvement_rate 3 t                               -- returns {trend samples ...}

audit.quick_check {output: str  task: str}               -- returns {passed reasons}
audit.rubric [{name description weight}]                 -- validates rubric structure
audit.is_empty str                                       -- Bool
audit.is_hedging str                                     -- Bool
audit.is_refusal str                                     -- Bool

grader.grade {work task rubric threshold previous_grades} -- returns {score passed categories feedback failed}
auditor.audit {output task}                               -- returns {score passed categories feedback failed}

prompt.create ()                                          -- returns handle
prompt.system "text" handle                               -- data-last, returns handle
prompt.section "name" "content" handle                    -- data-last
prompt.instruction "text" handle                          -- data-last
prompt.constraint "text" handle                           -- data-last
prompt.render handle                                      -- returns Str
prompt.estimate handle                                    -- returns Int (approx tokens)

md.parse str                                              -- returns parsed doc
md.headings parsed_doc                                    -- returns [Str]

fs.read "path" ^                                          -- returns Str
fs.write "path" "content" ^                               -- returns ()
fs.mkdir "path" ^                                         -- returns ()

ai.prompt "text" ^                                        -- returns Str (LLM response)
ai.prompt_structured Protocol "text" ^                    -- returns record matching Protocol
```
