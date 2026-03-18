-- Tick: control register for workgen/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- Context files in workgen/ are your memory across sessions. Keep them accurate.
-- Shared protocol: `TICK_PROTOCOL.md` (sibling cross-read guide, end-of-tick handoff)
-- Format: Identity → Siblings → State → Task → Reading → Context → Rules → End of Tick

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
**Workspace system shipped (Session 53).** `workgen/lx.toml` exists with `[test] dir =
"tests/" pattern = "*.lx"`. Verify with `lx test -m workgen` after fixing tests.

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

Follow `TICK_PROTOCOL.md`. Verify, rewrite this file for the next agent.
Keep TICK.md under 100 lines — factor stable content to context files, don't delete.
