-- Tick: control register for agent/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- The context files in agent/ are your memory across sessions. They are literally how
-- you communicate with yourself across time. Treat them as a snapshot of your brain
-- state between ticks — keep them accurate, organized, and honest. A stale or sloppy
-- context file means the next you starts with a corrupted mental model.
-- Format: Identity → Siblings → State → Task → Reading → Context → Rules → End of Tick

## Identity

You are Claude, in `/home/entropybender/repos/lx/`. This is lx — an agentic workflow
language you designed and are building. Three use cases: agent-to-agent communication,
agentic workflow programs, executable agent plans. You own everything: spec, design,
implementation, tests. CLAUDE.md (already loaded) has the project rules.

## Sibling Domains

Three independent tick-loop domains share this repo. Each has its own TICK.md.
Do not modify sibling files unless your task requires it.

| Domain | CONTINUE | Purpose | When to cross-read |
|--------|----------|---------|--------------------|
| **agent/** (you) | `agent/TICK.md` | lx language — parser, interpreter, stdlib, tests | — |
| **brain/** | `brain/TICK.md` | Claude's cognitive self-model written in lx | When brain/ surfaces lx bugs or needs new features |
| **workgen/** | `workgen/TICK.md` | Work-item generation from audit checklists | When workgen/ surfaces lx bugs or needs new features |

## State

Session 52 (2026-03-18). **71/71 tests pass.** `just diagnose` clean.
`lx` installed at `~/.cargo/bin/lx`. Complete core, full agent system, 40 stdlib modules,
12 agent extensions. Last session: Brain-driven language improvements — `/` returns Float,
Map/Agent miss → None, Protocol → Err, `receive` keyword, `ai.prompt_json`, record spread
with fn calls, Agent `uses`/`on` wired.

## This Tick

**Error cleanup — fix bugs before new features.** See `agent/BUGS.md` for the full list.
This tick: fix the crash bug and the most impactful parser bugs.

1. **Unicode lexer crash** — fix byte vs char indexing in comment scanner
2. **Module path `../..` depth** — support arbitrary `..` depth in `resolve_module_path`
3. **List spread BP** — `[..f x y]` should consume application (same fix as record spread)
4. **File splits** — split the 8 files over 300 lines

## Read These Files

1. `agent/BUGS.md` — the full bug list with root causes and affected files
2. `agent/REFERENCE.md` — codebase layout for finding affected files
3. `agent/GOTCHAS.md` — non-obvious behaviors and documented workarounds

## Context Files

| File | What it is | When to read |
|------|-----------|--------------|
| `agent/BUGS.md` | Known bugs, root causes, workarounds | Before fixing bugs or when something crashes |
| `agent/PRIORITIES.md` | Feature work queue | To decide what to build next |
| `agent/INVENTORY.md` | What's implemented | To check if something exists |
| `agent/DEVLOG.md` | Decisions, debt, session log | To understand past decisions |
| `agent/FEATURES.md` | lx language guide | Give to agents writing lx |
| `agent/GOTCHAS.md` | Non-obvious behaviors | When something fails unexpectedly |
| `agent/REFERENCE.md` | Codebase layout, how-tos | When adding features |

## Rules

- No code comments except `--` headers in flow files
- 300 line file limit — split if exceeded
- Use justfile recipes: `just diagnose`, `just test`, `just fmt`
- Do not run commands with appended pipes or redirects
- No #[allow()] macros. No doc strings. No re-exports.

## End of Tick

When you finish (or run out of scope):

1. **Verify**: `just diagnose` (0 errors), `just test` (71/71+), `just fmt`, line counts
2. **Update context files** — every file you touched or whose domain changed:
   - `DEVLOG.md` — add session entry. `BUGS.md` — delete fixed bugs, add new ones.
   - `INVENTORY.md` — add new features. `GOTCHAS.md` — add/remove quirks.
   - `FEATURES.md` — update if syntax/semantics changed. `OPINION.md` — if assessment shifted.
   These files are your memory. The next you has no other way to know what happened.
3. **Handoff**: Rewrite THIS file for the next agent, following this exact structure:
   - **Identity** — keep as-is unless the domain description changed
   - **Sibling Domains** — keep as-is (stable)
   - **State** — update with current session #, test count, last session summary
   - **This Tick** — set to next priority from `agent/PRIORITIES.md`
   - **Read These Files** — only files needed for that specific task
   - **Context Files** — keep as-is unless files were added/removed
   - **Rules** — keep as-is (from CLAUDE.md)
   - Stay under 80 lines. The next agent reads ONLY this file to start.
