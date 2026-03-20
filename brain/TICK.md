-- Tick: control register for brain/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- Context files in brain/ are your memory across sessions. Keep them accurate.
-- BEFORE writing code: follow Start of Tick Protocol in `TICK_PROTOCOL.md`
-- AFTER finishing work: follow End of Tick Protocol in `TICK_PROTOCOL.md`

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
22 lx files: contracts.lx, traits.lx, main.lx, orchestrator.lx, 6 agents, 12 lib modules.
Two sessions closed 26 gaps. No test infrastructure yet (brain/tests/ doesn't exist).
**Workspace system shipped (Sessions 53-54).** `brain/lx.toml` exists but has no `[test]`
section — add one when you create brain/tests/ (see `tests/lx.toml` for example).
After adding tests: verify with `lx test -m brain`.
**Cross-member imports now work (Session 54).** `use flows/lib/scoring {normalize}` resolves
via workspace member name. Brain files can import from flows/, workgen/, tests/ by name.
**Session 62: imports updated.** `std/knowledge` → `pkg/knowledge`, `std/circuit` → `pkg/circuit`,
`std/prompt` → `pkg/prompt`, `std/tasks` → `pkg/tasks`, `std/trace` → `pkg/trace`.
Also `trace.filter` renamed to `trace.query`. All brain files already updated.
**Session 64 (agent/):** pkg packages rewritten with Store + Collection Trait. API change:
`save path ^` and `load path ^` no longer work (save/load return Unit, not Result). Remove `^`
from save/load calls. `remove key ^` also returns the value directly, not Result. Store is now
a first-class value type (`Store ()` constructor, dot-access methods). Agent is now a Trait
defined in `pkg/agent.lx` with real defaults (handle, run, think, describe, etc.) — the
`Agent` keyword auto-imports it. Trait with fields acts as data contract. Brain agents now inherit
Agent Trait defaults (init, handle, run, perceive, reason, act, reflect, think, describe,
ask, tell, use_tool, tools). See `agent/SESSION_64_HANDOFF.md` if needed.
**Session 71b (agent/):** Five structural fixes landed. (1) `par`/`sel`/`pmap`/`pmap_n` are now
truly parallel (OS threads). (2) `lx check` resolves imports — far fewer false positives.
(3) `+` exports no longer shadow builtins — `+filter` inside a module can use builtin `filter`.
Self-recursive exports need two-step: `f = ...; +f = f`. (4) `std/durable` ships: Temporal-style
workflow persistence. (5) `lx install`/`lx update` for git+path dependency management.
Brain files modernized: record shorthand, string interpolation, eliminated intermediates in 18 files.
**Session 80 (agent/):** PKG_API_MINIMIZATION completed. Changes affecting brain/:
(1) `std/ctx` deprecated — brain/orchestrator.lx and brain/lib/cognitive_saga.lx already migrated to `Store()` + dot methods.
(2) `brain/lib/context_mgr.lx`: `pressure` renamed to `pressure_pct`, `pressure_level` removed (use `win.pressure()` directly).
(3) `pkg/agents/guard.check_safety` merged into `guard.check` — pass `guard.safety_patterns` as 2nd arg. brain/agents/critic.lx already updated.
(4) `pkg/agents/monitor` introspection functions (self_assess, detect_doom_loop, etc.) moved to `pkg/core/introspect`.
(5) `Store.merge` added — merge another Store or Record into a Store.
(6) `std/agents/planner`/`router`/`reviewer` now emit deprecation warnings — use `pkg/ai/` equivalents.

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
7. `brain/contracts.lx` — data shapes flowing through the pipeline

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

**MANDATORY: Execute ALL 5 steps in `TICK_PROTOCOL.md` as one uninterrupted sequence.**
Do not declare completion without running every step. Do not skip context file reviews.
