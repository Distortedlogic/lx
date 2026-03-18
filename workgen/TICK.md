-- Tick: control register for workgen/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- The context files in workgen/ are your memory across sessions. They are literally how
-- you communicate with yourself across time. Treat them as a snapshot of your brain
-- state between ticks — keep them accurate, organized, and honest. A stale or sloppy
-- context file means the next you starts with a corrupted mental model.
-- Format: Identity → Siblings → State → Task → Reading → Context → Rules → End of Tick

## Identity

You are Claude, in `/home/entropybender/repos/lx/`. This is workgen — an lx program that
automates work-item generation from audit checklists. It lives inside the lx repo (the
language it's written in). CLAUDE.md (already loaded) has the project rules.

## Sibling Domains

Three independent tick-loop domains share this repo. Each has its own TICK.md.
Do not modify sibling files unless your task requires it.

| Domain | CONTINUE | Purpose | When to cross-read |
|--------|----------|---------|--------------------|
| **agent/** | `agent/TICK.md` | lx language — parser, interpreter, stdlib, tests | When you hit lx bugs or need syntax from FEATURES.md |
| **brain/** | `brain/TICK.md` | Claude's cognitive self-model written in lx | When you want patterns for agent.mock or complex lx usage |
| **workgen/** (you) | `workgen/TICK.md` | Work-item generation from audit checklists | — |

## State

Core pipeline works (`main.lx`, 237 lines). `run.lx` is the entry point (2026-03-18).
Test runner (`tests/run.lx`) has a **blocking runtime error**: `split: second arg must be
Str`. Run `lx run workgen/tests/run.lx` to see exact location. 71/71 lx suite pass.

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
5. `agent/FEATURES.md` — lx language guide (if you need syntax help)

## Context Files

| File | What it is | When to read |
|------|-----------|--------------|
| `workgen/REFERENCE.md` | File map, APIs, fixtures, how workgen works | First read when orienting in workgen/ |
| `workgen/main.lx` | Core pipeline — all functions, exports `main` and `run` | When modifying pipeline logic |
| `workgen/tests/spec.lx` | Satisfaction spec — scenarios, rubric, thresholds | When modifying test expectations |
| `workgen/tests/run.lx` | Test runner — executes scenarios, grades output | When debugging test failures |
| `workgen/main.mmd` | Mermaid architecture diagram of main.lx | When you need to understand control flow |
| `workgen/justfile` | Build recipes (audit, audit-test, etc.) | When running workgen commands |
| `agent/FEATURES.md` | Complete lx language guide | When you need lx syntax help |
| `agent/GOTCHAS.md` | Non-obvious lx behaviors | When something fails unexpectedly |

## Rules

- No code comments except `--` headers in lx flow files
- 300 line file limit
- Use `lx run` directly for workgen (not `cargo run`)
- `just diagnose`, `just test`, `just fmt` for lx crate checks
- Do not run commands with appended pipes or redirects

## End of Tick

When you finish (or run out of scope):

1. **Verify**: `just diagnose`, `just test`, `just fmt`. Test workgen: `lx run workgen/tests/run.lx`
2. **Handoff**: Rewrite THIS file for the next agent, following this exact structure:
   - **Identity** — keep as-is unless the domain description changed
   - **Sibling Domains** — keep as-is (stable)
   - **State** — update with current status
   - **This Tick** — set to next task
   - **Read These Files** — only files needed for that specific task
   - **Context Files** — keep as-is unless files were added/removed
   - **Rules** — keep as-is (from CLAUDE.md)
   - Stay under 80 lines. The next agent reads ONLY this file to start.
