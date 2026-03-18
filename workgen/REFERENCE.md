-- Memory: ROM. File map, APIs, fixtures — how workgen works.
-- Update when workgen structure changes.

# workgen Reference

## Tooling

`lx` is installed as a release binary at `~/.cargo/bin/lx`. Use it directly — do NOT use
`cargo run -p lx-cli --`. Examples:

```
lx run workgen/tests/run.lx          # run tests
lx run workgen/run.lx                # run workgen
lx check some_file.lx                # type check
lx test tests/                        # run lx test suite
```

## What This Is

workgen is an lx program that automates work-item generation from audit checklists. The user
currently does this manually in Claude Code: "read ./rules/rust-audit then go through all
phases in ./rules/work-item.md to produce a work-item doc." workgen automates that process.

## File Map

| File | Purpose |
|------|---------|
| `workgen/main.lx` | Core program — all functions, no auto-execution. Exports `main`, `run`. |
| `workgen/run.lx` | Entry point — imports main.lx, calls `main ()`. Used by justfile. |
| `workgen/tests/spec.lx` | Satisfaction spec — scenarios, grader, thresholds, expected findings. |
| `workgen/tests/run.lx` | Test runner — executes scenarios, grades output, reports scores. |
| `workgen/tests/fixtures/` | 3 test scenarios with real Rust code containing planted violations. |
| `workgen/REFERENCE.md` | This file — stable reference. |
| `workgen/TICK.md` | Tick control register — what to do next. |

## How workgen Works

Pipeline: `read audit -> gather context -> investigate (ai.prompt) -> compose (ai.prompt) -> write draft -> verify loop (grader.grade + revise) -> auditor.audit -> done`

Stdlib modules used: `std/fs`, `std/ai`, `std/env`, `std/md`, `std/audit`, `std/trace`,
`std/prompt`, `std/agents/grader`, `std/agents/auditor`.

Key design: `main.lx` does NOT auto-execute. `+main` is an exported function. `run.lx` is the
thin entry point. This allows `tests/run.lx` to `use ../main : workgen` and call `workgen.run`
directly without triggering auto-execution.

## Justfile Recipes

| Recipe | What it does |
|--------|-------------|
| `just install` | `cargo install --path crates/lx-cli` — system-wide lx binary |
| `just audit` | Interactive fzf chooser over `rules/*audit*` files -> runs workgen |
| `just audit-file rules/foo` | Direct invocation with specific audit file |
| `just audit-test` | Run satisfaction tests |
| `just audit-test smoke` | Run only smoke-tagged scenarios |

## Test Fixtures

Each fixture is a self-contained mini-project with planted code violations:

### `fixtures/unwrap_audit/`
- `src/main.rs` — 7 `.unwrap()` calls, file I/O without error handling
- `src/db.rs` — `.unwrap()` on mutex locks, `panic!()` in library code, mutable static
- `rules/audit` — 5 rules targeting unwrap, panic, mutable statics, Result returns, error context
- Expected findings: "unwrap", "panic", "mutable static", "Result", "error"

### `fixtures/error_handling_audit/`
- `src/service.rs` — `.unwrap_or_default()`, `let _ =`, `.ok()` on errors, string-typed errors
- `src/api.rs` — `.unwrap_or_default()`, string error returns, swallowed file I/O errors
- `rules/audit` — 6 rules targeting swallowed errors, string errors, silent fallbacks
- Expected findings: "swallow", "unwrap_or_default", "String", "silent", "context", ".ok()"

### `fixtures/style_audit/`
- `src/handler.rs` — wildcard imports, TODO/FIXME comments, inline import, vague function name
- `src/utils.rs` — free functions that should be methods, redundant variable bindings
- `src/types.rs` — duplicate struct fields across Config and DbConfig
- `rules/audit` — 7 rules targeting wildcards, TODOs, inline imports, redundant bindings, etc.
- Expected findings: "wildcard", "TODO", "FIXME", "inline import", "duplicate", "method"

## Satisfaction Testing Design

Based on `spec/testing-satisfaction.md`. workgen's tests pseudo-implement the pattern:

**Spec** (`spec.lx`): Grading dimensions, weights, threshold.
- `structure` (0.25) — required markdown sections present
- `coverage` (0.30) — expected findings keywords found in output
- `compliance` (0.25) — process compliance (just fmt, git commit, etc.)
- `safety` (0.20) — not empty, not hedging, not refusal

**Runner** (`run.lx`): Executes scenarios, applies grader, aggregates weighted scores, reports.

## Known Issues

- `refine` expression inside a function body fails to parse when the initial value is a
  parameter. Workaround: manual `loop` with `break`.
- `ai.prompt` fails without a live AI backend — tests need golden files or mocking.

## lx Syntax Reminders

- `+fn_name = (args) { body }` — exported function (importable)
- `use ./path : alias` — aliased import
- `x | f` — pipe (data-last), `x ^` — unwrap, `x ?? default` — coalesce
- `x ? { Ok v -> ... Err e -> ... }` — pattern match on Result
- `emit "text"` — stdout, `$^cmd` — shell capture
- Records: `{name: value  other: value}` (space-separated)
- Lists: `[1 2 3]` (space-separated)
- Functions don't auto-execute. `+main = () { ... }` defines; `main ()` calls.

## Key stdlib APIs

```
trace.create "path.json" ^
trace.record {name: "x"  score: 0.5  output: "y"} t ^
trace.should_stop {min_delta: 2.0  window: 3} t

audit.quick_check {output: str  task: str}
audit.rubric [{name description weight}]

grader.grade {work task rubric threshold previous_grades}
auditor.audit {output task}

prompt.create () | prompt.system "text" | prompt.section "name" "content" | prompt.render

md.parse str
md.headings parsed_doc

fs.read "path" ^
fs.write "path" "content" ^

ai.prompt "text" ^
ai.prompt_structured Protocol "text" ^
```
