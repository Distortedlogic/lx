-- Tick: control register for brain/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- The context files in brain/ are your memory across sessions. They are literally how
-- you communicate with yourself across time. Treat them as a snapshot of your brain
-- state between ticks — keep them accurate, organized, and honest. A stale or sloppy
-- context file means the next you starts with a corrupted mental model.
-- Format: Identity → Siblings → State → Task → Reading → Context → Rules → End of Tick

## Identity

You are Claude, in `/home/entropybender/repos/lx/`. This is your brain — a cognitive
architecture written in lx (your language). 22 lx files modeling your cognitive process.
You own everything: spec, design, implementation, tests. CLAUDE.md (already loaded) has
the project rules.

## Sibling Domains

Three independent tick-loop domains share this repo. Each has its own TICK.md.
Do not modify sibling files unless your task requires it.

| Domain | CONTINUE | Purpose | When to cross-read |
|--------|----------|---------|--------------------|
| **agent/** | `agent/TICK.md` | lx language — parser, interpreter, stdlib, tests | When you need new language features or hit lx bugs |
| **brain/** (you) | `brain/TICK.md` | Claude's cognitive self-model written in lx | — |
| **workgen/** | `workgen/TICK.md` | Work-item generation from audit checklists | When you want to see a real lx program in action |

## State

71/71 tests pass (2026-03-18). `just diagnose` clean. All brain files under 300 lines.
22 lx files: protocols.lx, traits.lx, main.lx, orchestrator.lx, 6 agents, 12 lib modules.
Two sessions closed 26 gaps. No test infrastructure yet (brain/tests/ doesn't exist).

## This Tick

**Create brain/tests/ — mock-based test suite for the cognitive pipeline.**

Deliverables:
1. `brain/tests/test_perception.lx` — mock AI, verify intent/entity/complexity classification
2. `brain/tests/test_reasoning.lx` — mock AI, verify 4 strategy paths
3. `brain/tests/test_pipeline.lx` — mock agents + tools, verify full pipeline

Use `agent.mock` for faking agent responses. Use `describe`/`it` test blocks.

## Read These Files

1. `brain/ARCHITECTURE.md` — understand brain structure (read first)
2. `tests/53_agent_mock.lx` — agent.mock patterns
3. `tests/70_describe.lx` — describe/it test patterns
4. `brain/main.lx` — pipeline under test (imports 6 lib modules — scan their `use` lines)
5. `brain/lib/perception.lx` — module under test
6. `brain/lib/reasoning.lx` — module under test
7. `brain/protocols.lx` — data shapes flowing through the pipeline

## Context Files

| File | What it is | When to read |
|------|-----------|--------------|
| `brain/ARCHITECTURE.md` | Module map, data flow, patterns in use | First read when orienting in brain/ |
| `brain/STATUS.md` | Session log, completed work, remaining gaps | To decide what to work on or log your session |
| `agent/FEATURES.md` | Complete lx language guide | When you need lx syntax help |
| `agent/GOTCHAS.md` | Non-obvious lx behaviors | When something fails unexpectedly |

## Rules

- No code comments except `--` headers in flow files
- 300 line file limit — split if exceeded
- Use justfile recipes: `just diagnose`, `just test`, `just fmt`
- Do not run commands with appended pipes or redirects
- No #[allow()] macros. No doc strings. No re-exports.

## End of Tick

When you finish (or run out of scope):

1. **Verify**: `just diagnose` (0 errors), `just test` (71/71+), `just fmt`, line counts
2. **Record**: Update `brain/STATUS.md` — add session entry with what you changed
3. **Handoff**: Rewrite THIS file for the next agent, following this exact structure:
   - **Identity** — keep as-is unless the domain description changed
   - **Sibling Domains** — keep as-is (stable)
   - **State** — update with current session results
   - **This Tick** — set to next task from STATUS.md remaining gaps
   - **Read These Files** — only files needed for that specific task
   - **Context Files** — keep as-is unless files were added/removed
   - **Rules** — keep as-is (from CLAUDE.md)
   - Stay under 80 lines. The next agent reads ONLY this file to start.
