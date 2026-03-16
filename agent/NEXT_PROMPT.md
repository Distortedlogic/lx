# Cold Start Prompt

Read this first when picking up lx work in a fresh agent.

## What This Is

lx is an agentic workflow language you (Claude) are designing and building. Three primary use cases:

1. **Agent-to-agent communication** — agents talk via `~>` (send) and `~>?` (ask). `Protocol` contracts validate message shapes. `Trait` declarations enforce behavioral contracts.
2. **Agentic workflow programs** — orchestrate agents and tools: spawning, message routing, MCP tool invocation, context persistence, result aggregation.
3. **Executable agent plans** — the plan IS an lx program. `yield` pauses for orchestrator input, `refine` loops grade/revise output quality.

**Identity:** lx is not a general scripting language. Every feature must serve one of these three use cases.

## Where We're At

Session 46 (2026-03-16). **66/66 tests pass.** `just diagnose` clean (2 pre-existing clippy warnings).

Last sessions: `std/retry` implemented (Session 45). Aggressive spec consolidation (Session 46) — 9 merges applied, reducing planned features from ~33 to 21. Key merges: `std/strategy` → `std/profile` (strategy is a knowledge domain), `checkpoint`/`on_interrupt` keywords → `user.check` + `:signal` lifecycle hook (no new keywords), `std/reputation`/provenance → `std/trace` extensions (one observability system), constraint propagation → `with context` ambient (one propagation mechanism), `plan.run_incremental` → `std/pipeline` (same caching mechanism). New specs for `Agent` declarations and enforced `Trait` methods (absorbing Skills) added at Tier 2.

Next priorities: `std/user` (with `user.check` for interrupt polling), `std/profile` (with strategy helpers). See `agent/PRIORITIES.md` for the full queue.

The language has a complete core (functions, pipes, pattern matching, modules, type checker), a full agent system (protocols, traits, scoped resources, yield, refine, emit), 37 stdlib modules, and 13 agent extensions. See `agent/INVENTORY.md` for the full list.

## File Map

| File | When to read |
|------|--------------|
| `agent/PRIORITIES.md` | To decide what to work on |
| `agent/INVENTORY.md` | To see what's already implemented |
| `agent/OPINION.md` | To understand design strengths and remaining gaps |
| `agent/ROADMAP.md` | To see all planned future features |
| `agent/DEVLOG.md` | To review design decisions, tech debt, session history |
| `agent/REFERENCE.md` | To look up codebase layout or how-to guides |

You own this language. Change spec, design, tests, flows, Rust code freely. Only constraint: internal consistency. When you change something, update all references. At session end, update `agent/DEVLOG.md` and this file's "Where We're At" section.
