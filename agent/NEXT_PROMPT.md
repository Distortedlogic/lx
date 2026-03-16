# Cold Start Prompt

Read this first when picking up lx work in a fresh agent.

## What This Is

lx is an agentic workflow language you (Claude) are designing and building. Three primary use cases:

1. **Agent-to-agent communication** — agents talk via `~>` (send) and `~>?` (ask). `Protocol` contracts validate message shapes. `Trait` declarations enforce behavioral contracts.
2. **Agentic workflow programs** — orchestrate agents and tools: spawning, message routing, MCP tool invocation, context persistence, result aggregation.
3. **Executable agent plans** — the plan IS an lx program. `yield` pauses for orchestrator input, `refine` loops grade/revise output quality.

**Identity:** lx is not a general scripting language. Every feature must serve one of these three use cases.

## Where We're At

Session 49 (2026-03-16). **68/68 tests pass.** `just diagnose` clean (3 pre-existing clippy warnings). `lx` installed to `~/.cargo/bin/lx`.

Last sessions: `std/user` (Session 49) — structured agent-to-user interaction with `UserBackend` trait. `std/profile` (Session 49) — persistent agent identity with strategy helpers.

Next priorities: `Agent` declarations. See `agent/PRIORITIES.md` for the full queue.

The language has a complete core (functions, pipes, pattern matching, modules, type checker), a full agent system (protocols, traits, scoped resources, yield, refine, emit), 39 stdlib modules, and 13 agent extensions. See `agent/INVENTORY.md` for the full list.

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
