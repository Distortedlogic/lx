-- Memory: journal + gap tracker. Session history and remaining work for brain/.
-- Add a session entry every tick. Update remaining gaps when work is done.

## Current State (2026-03-18)

71/71 lx tests pass. `just diagnose` clean (7 pre-existing clippy warnings in lx crate — not brain-related). All brain files under 300 lines.

## Completed Work

### Session: 2026-03-18 (3) — Language improvements, brain code sweep

Applied 10 lx language fixes driven by brain code analysis. Swept brain code to use new features:

| # | Change | Impact on brain |
|---|--------|-----------------|
| 1 | `/` returns Float for Int/Int | Removed all 14 `to_float` calls across 8 files |
| 2 | Map/Agent field miss → None | Uniform `??` fallback works everywhere |
| 3 | Trait validation → Err values | Defensive code can now catch bad data |
| 4 | Record spread allows fn calls | `{..mk () ...}` works |
| 5 | Agent `uses`/`on` wired to runtime | Metadata accessible via Agent.on |
| 6 | `receive` keyword | Converted all 5 agent main() from 10-line boilerplate to 4-line receive blocks |
| 7 | `ai.prompt_json` | Lightweight structured AI output without named Traits |

### Session: 2026-03-18 (2) — Deepen features, close 15 gaps

Worked through high and medium priority gaps from the initial audit. Changes across 16 files:

| # | Gap | Status | What was done |
|---|-----|--------|---------------|
| 1 | Trait field constraints | DONE | Added `where` clauses to 20+ fields across 14 traits (confidence 0-1, scores 0-100, tokens >= 0, durations >= 0, costs >= 0) |
| 2 | Trait composition | DONE | Created `Timestamped` base trait, composed into all 5 cognitive event traits via `{..Timestamped}` |
| 3 | MCP declarations | DONE | Added `MCP CognitiveTools` declaration with 9 typed tools in tools.lx; `available_tools` now derives from tool names |
| 4 | trace.improvement_rate/should_stop | DONE | Both refine loops (response + code) in quality.lx now use `with trace.create` sessions, `trace.should_stop` for diminishing returns, `trace.improvement_rate` for logging |
| 5 | pmap_n rate limiting | DONE | dispatcher.fan_out switched from `pmap` to `pmap_n 3` |
| 6 | introspect.strategy_shift | DONE | monitor.suggest_strategy now calls `introspect.strategy_shift` when suggestion != "continue" |
| 7 | agent.capabilities/advertise | DONE | All 4 specialist agents (analyst, critic, researcher, synthesizer) call `agent.advertise` at startup; dispatcher.route_to_worker queries `agent.capabilities` for dynamic routing with hardcoded fallback |
| 8 | Handoff + agent.as_context | DONE | cognitive_saga.lx uses `Handoff` trait and `agent.as_context` for perception→reasoning and execution→response transitions |
| 9 | std/user interactive gates | DONE | identity.gate_destructive now calls `user.confirm` before `agent.gate` |
| 10 | std/md structured responses | DONE | synthesizer.lx format/compose_parts use `md.parse`, `md.heading`, `md.paragraph`, `md.list`, `md.render` |
| 11 | std/git awareness | DONE | perception.lx adds `git_context` function that reads branch/modified files for code-related domains |
| 12 | agent.dialogue | DONE | dialogue.lx exports `open_dialogue`, `dialogue_turn`, `close_dialogue` wrappers around `agent.dialogue` system |
| 13 | std/plan replan callbacks | DONE | planner.lx execute_plan uses `plan.insert_after` for recoverable errors (record pattern + guard) and `plan.skip` for non-recoverable |
| 14 | Raw strings | DONE | Switched prompt system/instruction/constraint strings to backtick raw strings in reasoning.lx, perception.lx, quality.lx |
| 15 | Record patterns with guards | DONE | main.lx race_reasoning uses `{complexity: "complex"  ambiguity: a ..} & (a > 0.5)` guard; tool dispatch uses `{requires_tools: true ..}` record patterns |
| 16 | Currying / higher-order | DONE | tools.lx adds `make_tool_filter` and `make_executor` curried functions; `fs_tools`, `search_tools`, `web_tools` are partially applied |

### Session: 2026-03-18 — Initial audit and 10-point overhaul

Audited brain/ against agent/LANGUAGE.md. Found and closed these gaps:

| # | Priority | Status | What was done |
|---|----------|--------|---------------|
| 1 | Agent declarations | DONE | All 5 specialists converted to `Agent Name: Trait = { methods }` with +main ~>? loops |
| 2 | Pub/sub | DONE | CognitiveEvent union, dispatcher.setup_pubsub creates topics, orchestrator publishes events |
| 3 | Message middleware | DONE | dispatcher.setup_tracing wraps all agents with agent.intercept for trace recording |
| 4 | Human gates | DONE | identity.gate_destructive uses agent.gate; orchestrator calls it before saga |
| 5 | sel racing | DONE | main.lx:race_reasoning uses sel for complex inputs — thorough path vs timeout fallback |
| 6 | Saga compensations | DONE | New cognitive_saga.lx — undo steps clear context entries via std/ctx tracking |
| 7 | std/diag | DONE | orchestrator.visualize() emits mermaid diagram of cognitive pipeline |
| 8 | std/cron | DONE | run_continuous schedules memory consolidation (5min) and health checks (1min) |
| 9 | std/tasks | DONE | Orchestrator tracks cognitive_loop as task: create → start → submit → pass |
| 10 | Type annotations | DONE | All exported functions across 22 files have param + return type annotations |

### Additional changes in same session
- `with...as` for trace sessions (main.lx, orchestrator.lx) and memory stores (main.lx)
- 4 new specialist traits in traits.lx: Analyst, Critic, Researcher, Synthesizer
- CognitivePhase tagged union ADT in protocols.lx
- Sections used more idiomatically (.content, .success, etc.)
- cognitive_saga.lx split out from orchestrator.lx to keep both under 300 lines

## Known Remaining Gaps

### High priority (features exist in lx, brain doesn't use them)
- **agent.mock** — no test infrastructure for the brain (brain/tests/ doesn't exist)
- **std/agents/* stdlib agents** — router, planner, monitor, reviewer from std/agents/ are unused. Brain reinvents each with custom logic. Could compose with stdlib versions instead.

### Medium priority (used shallowly)
- **Slicing/indexing** — xs.1..3, xs.-1 not used anywhere in brain
- **Default parameters** — most functions don't use default params where they could
- **More record patterns** — only main.lx uses them so far; could spread to orchestrator, dispatcher, etc.
- **More currying** — only tools.lx has curried functions; other modules could benefit

### Low priority (nice to have)
- **Make lib modules into Agents** — perception, reasoning, etc. could be Agent declarations conforming to Perceiver, Reasoner traits
- **brain/tests/** — mock-based test suite for cognitive pipeline using agent.mock
- **agent.negotiate deeper use** — dispatcher.parallel_review uses it but orchestrator doesn't

## Architecture Notes

The brain has two entry points:
1. **main.lx** — Lightweight cognitive loop. Direct function calls to lib modules. Uses sel for racing, with...as for resources. Good for single-turn.
2. **orchestrator.lx** — Full pipeline. Saga-based, spawns specialist agents via dispatcher, pub/sub events, cron scheduling, task lifecycle tracking. Good for continuous sessions.

Both paths share the same lib modules (perception, reasoning, memory, tools, quality, etc.) and the same protocols/traits.

The specialist agents in agents/ are proper Agent declarations that can run either:
- **Locally** via `AgentName.method {input}` (used in main.lx path)
- **As subprocesses** via `agent.spawn` + `~>?` messaging (used in orchestrator path via dispatcher)
