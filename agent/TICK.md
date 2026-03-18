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

Session 61 (2026-03-18). **78/78 tests pass.** `just diagnose` clean (0 errors, 7 pre-existing warnings).
Complete core, full agent system, 44 stdlib modules, 12 agent extensions, ~100 stdlib .rs files.
Last session: Bug fixes — list spread bp, Agent body uppercase, 7 file splits. Parser foundation solid.

## This Tick

**Feature consolidation audit** — Tier 0, Session 62 from `agent/PRIORITIES.md`. This is a
collaborative design session with the user. The feature surface (44 stdlib modules, 12 agent
extensions) was built across 61 sessions — many features likely overlap or share mechanics.

Goals:
1. Audit the full stdlib + agent extension surface with the user
2. Identify generic primitives that cover the same ground with fewer composable building blocks
3. Find overlapping modules that should be merged (reconcile/vote, routing mechanisms, retry/circuit)
4. Plan restructuring — layer specific instances on top of generic ones for DRYness

Do NOT proceed to Tier 2 features until this consolidation is done. Wait for user direction.

If the user wants to skip consolidation and build features instead, proceed to Tier 2 item 10:
`introspect.system` live observation (`spec/agents-introspect-live.md`).

## Read These Files

1. `agent/PRIORITIES.md` — the work queue, to understand context
2. `agent/INVENTORY.md` — full feature surface to audit
3. `agent/HEALTH.md` — current assessment

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
