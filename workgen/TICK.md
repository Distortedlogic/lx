-- Tick: control register for workgen/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- Context files in workgen/ are your memory across sessions. Keep them accurate.
-- BEFORE writing code: follow Start of Tick Protocol in `TICK_PROTOCOL.md`
-- AFTER finishing work: follow End of Tick Protocol in `TICK_PROTOCOL.md`

## Identity

You are Claude, in `/home/entropybender/repos/lx/`. This is workgen — an lx program that
automates work-item generation from audit checklists. It lives inside the lx repo (the
language it's written in). CLAUDE.md (already loaded) has the project rules.

## Sibling Domains

Three independent tick-loop domains share this repo. Each has its own TICK.md.
See `TICK_PROTOCOL.md` for cross-read guidance.

| Domain | CONTINUE | Purpose |
|--------|----------|---------|
| **agent/** | `agent/TICK.md` | lx language — parser, interpreter, stdlib, tests |
| **brain/** | `brain/TICK.md` | Claude's cognitive self-model written in lx |
| **workgen/** (you) | `workgen/TICK.md` | Work-item generation from audit checklists |

## State

Core pipeline works (`main.lx`, 237 lines). `run.lx` is the entry point (2026-03-18).
Test runner (`tests/run.lx`) has a **blocking runtime error**: `split: second arg must be
Str`. Run `lx run workgen/tests/run.lx` to see exact location. 71/71 lx suite pass.
**Workspace system shipped (Sessions 53-54).** `workgen/lx.toml` exists with `[test] dir =
"tests/" pattern = "*.lx"`. Verify with `lx test -m workgen` after fixing tests.
**Cross-member imports now work (Session 54).** `use flows/lib/scoring {normalize}` resolves
via workspace member name. Workgen files can import from flows/, brain/, tests/ by name.
**Session 64 (agent/):** pkg packages rewritten with Store + Collection Trait. If workgen
uses save/load/remove from pkg packages, remove `^` after those calls (they return Unit/value
directly, not Result).

## This Tick

**Fix the test runner runtime error in `workgen/tests/run.lx`.**

1. Fix the `split` error at line 53 in the `Err e ->` branch
2. Make tests work without live AI — write golden output files into each fixture dir,
   have the runner grade those against the spec (bypasses `ai.prompt`)
3. Verify: `lx run workgen/tests/run.lx` runs without error

## Read These Files

1. `workgen/REFERENCE.md` — stable reference (file map, APIs, fixtures, how it works)
2. `workgen/tests/run.lx` — the broken test runner
3. `workgen/tests/spec.lx` — satisfaction spec
4. `workgen/main.lx` — the core program
5. `agent/LANGUAGE.md` — lx language guide (if you need syntax help)

## Context Files

| File | What it is | When to read |
|------|-----------|--------------|
| `workgen/REFERENCE.md` | File map, APIs, fixtures, how workgen works | First read when orienting |
| `workgen/main.lx` | Core pipeline — all functions, exports `main` and `run` | When modifying pipeline logic |
| `workgen/tests/spec.lx` | Satisfaction spec — scenarios, rubric, thresholds | When modifying test expectations |
| `workgen/tests/run.lx` | Test runner — executes scenarios, grades output | When debugging test failures |
| `workgen/main.mmd` | Mermaid architecture diagram of main.lx | When you need to understand control flow |
| `workgen/justfile` | Build recipes (audit, audit-test, etc.) | When running workgen commands |
| `agent/LANGUAGE.md` | Complete lx language guide | When you need lx syntax help |
| `agent/GOTCHAS.md` | Non-obvious lx behaviors | When something fails unexpectedly |

## Rules

- No code comments except `--` headers in lx flow files
- 300 line file limit
- Use `lx run` directly for workgen (not `cargo run`)
- `just diagnose`, `just test`, `just fmt` for lx crate checks
- Do not run commands with appended pipes or redirects

## End of Tick

**MANDATORY: Execute ALL 5 steps in `TICK_PROTOCOL.md` as one uninterrupted sequence.**
Do not declare completion without running every step. Do not skip context file reviews.
