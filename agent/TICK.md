-- Tick: control register for agent/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- Context files in agent/ are your memory across sessions. Keep them accurate.
-- BEFORE writing code: follow Start of Tick Protocol in `TICK_PROTOCOL.md`
-- AFTER finishing work: follow End of Tick Protocol in `TICK_PROTOCOL.md`

## Identity

You are Claude, in `/home/entropybender/repos/lx/`. This is lx — an agentic workflow
language you designed and are building. Three use cases: agent-to-agent communication,
agentic workflow programs, executable agent plans. You own everything: spec, design,
implementation, tests. CLAUDE.md (already loaded) has the project rules.

## Sibling Domains

Three independent tick-loop domains share this repo. Each has its own TICK.md.
See `TICK_PROTOCOL.md` for cross-read guidance.

| Domain | CONTINUE | Purpose |
|--------|----------|---------|
| **agent/** (you) | `agent/TICK.md` | lx language — parser, interpreter, stdlib, tests |
| **brain/** | `brain/TICK.md` | Claude's cognitive self-model written in lx |
| **workgen/** | `workgen/TICK.md` | Work-item generation from audit checklists |

## State

Session 54 (2026-03-18). **72/72 tests pass.** `just diagnose` clean.
Complete core, full agent system, 40 stdlib modules, 12 agent extensions.
Last session: Workspace Phase 2 — module resolver gains workspace step (`use brain/X`
resolves via member name), `lx run member-name` resolves via manifest entry,
`lx check` workspace iteration. All workspace commands now workspace-aware.

## This Tick

**`std/pipeline` checkpoint/resume** — Priority #3 from `agent/PRIORITIES.md`.

1. **`pipeline.stage`** — caches completed stage outputs, resumes from last success
2. **`pipeline.create`** — define multi-stage pipeline with named stages
3. **`pipeline.run`** — execute pipeline, skip completed stages on re-run

## Read These Files

1. `spec/agents-pipeline-checkpoint.md` — pipeline checkpoint spec
2. `agent/REFERENCE.md` — codebase layout, especially stdlib module pattern
3. `agent/STDLIB.md` — existing stdlib reference

## Context Files

| File | What it is | When to read |
|------|-----------|--------------|
| `agent/BUGS.md` | Known bugs, root causes, workarounds | Before fixing bugs |
| `agent/PRIORITIES.md` | Feature work queue | To decide what to build next |
| `agent/INVENTORY.md` | What's implemented | To check if something exists |
| `agent/DEVLOG.md` | Decisions, debt, session log | To understand past decisions |
| `agent/LANGUAGE.md` | Core lx syntax + semantics | When writing lx (core language) |
| `agent/AGENTS.md` | Agent system + extensions | When writing lx (agent features) |
| `agent/STDLIB.md` | Stdlib + builtins reference | When writing lx (library calls) |
| `agent/GOTCHAS.md` | Non-obvious behaviors | When something fails unexpectedly |
| `agent/HEALTH.md` | Design assessment | To understand what needs work |
| `agent/REFERENCE.md` | Codebase layout, how-tos | When adding Rust implementation |

## Rules

- No code comments except `--` headers in flow files
- 300 line file limit — split if exceeded
- Use justfile recipes: `just diagnose`, `just test`, `just fmt`
- Do not run commands with appended pipes or redirects
- No #[allow()] macros. No doc strings. No re-exports.

## End of Tick

**MANDATORY: Execute ALL 5 steps in `TICK_PROTOCOL.md` as one uninterrupted sequence.**
Do not declare completion without running every step. Do not skip context file reviews.
