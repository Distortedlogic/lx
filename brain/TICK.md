-- Tick: control register for brain/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- Context files in brain/ are your memory across sessions. Keep them accurate.
-- Shared protocol: `TICK_PROTOCOL.md` (sibling cross-read guide, end-of-tick handoff)
-- Format: Identity → Siblings → State → Task → Reading → Context → Rules → End of Tick

## Identity

You are Claude, in `/home/entropybender/repos/lx/`. This is your brain — a cognitive
architecture written in lx (your language). 22 lx files modeling your cognitive process.
You own everything: spec, design, implementation, tests. CLAUDE.md (already loaded) has
the project rules.

## Sibling Domains

Three independent tick-loop domains share this repo. Each has its own TICK.md.
See `TICK_PROTOCOL.md` for cross-read guidance.

| Domain | CONTINUE | Purpose |
|--------|----------|---------|
| **agent/** | `agent/TICK.md` | lx language — parser, interpreter, stdlib, tests |
| **brain/** (you) | `brain/TICK.md` | Claude's cognitive self-model written in lx |
| **workgen/** | `workgen/TICK.md` | Work-item generation from audit checklists |

## State

71/71 tests pass (2026-03-18). `just diagnose` clean. All brain files under 300 lines.
22 lx files: protocols.lx, traits.lx, main.lx, orchestrator.lx, 6 agents, 12 lib modules.
Two sessions closed 26 gaps. No test infrastructure yet (brain/tests/ doesn't exist).
**Workspace system shipped (Session 53).** `brain/lx.toml` exists but has no `[test]`
section — add one when you create brain/tests/ (see `tests/lx.toml` for example).
After adding tests: verify with `lx test -m brain`.

## This Tick

**Create brain/tests/ — mock-based test suite for the cognitive pipeline.**

Deliverables:
1. `brain/tests/test_perception.lx` — mock AI, verify intent/entity/complexity classification
2. `brain/tests/test_reasoning.lx` — mock AI, verify 4 strategy paths
3. `brain/tests/test_pipeline.lx` — mock agents + tools, verify full pipeline
4. Add `[test]` section to `brain/lx.toml` pointing at your new test dir + pattern

Use `agent.mock` for faking agent responses. Use `describe`/`it` test blocks.

## Read These Files

1. `brain/ARCHITECTURE.md` — understand brain structure (read first)
2. `brain/lx.toml` + `tests/lx.toml` — workspace manifests (add `[test]` to brain's)
3. `tests/53_agent_mock.lx` — agent.mock patterns
4. `tests/70_describe.lx` — describe/it test patterns
5. `brain/main.lx` — pipeline under test (imports 6 lib modules — scan their `use` lines)
6. `brain/lib/perception.lx` + `brain/lib/reasoning.lx` — modules under test
7. `brain/protocols.lx` — data shapes flowing through the pipeline

## Context Files

| File | What it is | When to read |
|------|-----------|--------------|
| `brain/ARCHITECTURE.md` | Module map, data flow, patterns in use | First read when orienting in brain/ |
| `brain/STATUS.md` | Session log, completed work, remaining gaps | To decide what to work on or log your session |
| `agent/LANGUAGE.md` | Complete lx language guide | When you need lx syntax help |
| `agent/GOTCHAS.md` | Non-obvious lx behaviors | When something fails unexpectedly |

## Rules

- No code comments except `--` headers in flow files
- 300 line file limit — split if exceeded
- Use justfile recipes: `just diagnose`, `just test`, `just fmt`
- Do not run commands with appended pipes or redirects
- No #[allow()] macros. No doc strings. No re-exports.

## End of Tick

Follow `TICK_PROTOCOL.md`. Verify, update `brain/STATUS.md`, rewrite this file for the
next agent. Keep TICK.md under 100 lines — factor stable content to context files, don't delete.
