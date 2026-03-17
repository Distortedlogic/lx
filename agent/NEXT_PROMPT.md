# Cold Start Prompt

Read this first when picking up lx work in a fresh agent.

## What This Is

lx is an agentic workflow language you (Claude) are designing and building. Three primary use cases:

1. **Agent-to-agent communication** — agents talk via `~>` (send) and `~>?` (ask). `Protocol` contracts validate message shapes. `Trait` declarations enforce behavioral contracts.
2. **Agentic workflow programs** — orchestrate agents and tools: spawning, message routing, MCP tool invocation, context persistence, result aggregation.
3. **Executable agent plans** — the plan IS an lx program. `yield` pauses for orchestrator input, `refine` loops grade/revise output quality.

**Identity:** lx is not a general scripting language. Every feature must serve one of these three use cases.

## Where We're At

Session 49+ (2026-03-17). **71/71 tests pass.** `just diagnose` clean (6 clippy warnings — large_enum_variant, type_complexity, result_large_err, 3x too_many_arguments). `lx` installed to `~/.cargo/bin/lx`.

Last sessions: `std/user` + `std/profile` + `Agent` declarations (Session 49). Flow testing infrastructure + `std/test` + `std/describe` + `describe`/`it` blocks (post-49).

Next priorities: Enforced `Trait` methods. See `agent/PRIORITIES.md` for the full queue.

The language has a complete core (functions, pipes, pattern matching, modules, type checker), a full agent system (protocols, traits, scoped resources, yield, refine, emit, Agent declarations), 39 stdlib modules, and 11 agent extensions. See `agent/INVENTORY.md` for the full list.

## File Map

| File | When to read |
|------|--------------|
| `agent/PRIORITIES.md` | To decide what to work on |
| `agent/INVENTORY.md` | To see what's already implemented |
| `agent/OPINION.md` | To understand design strengths and remaining gaps |
| `agent/ROADMAP.md` | To see all planned future features |
| `agent/DEVLOG.md` | To review design decisions, tech debt, session history |
| `agent/REFERENCE.md` | To look up codebase layout or how-to guides |
| `agent/GOTCHAS.md` | Permanent non-obvious behaviors that trip up implementation |
| `agent/WORKAROUNDS.md` | Temporary limitations from incomplete implementation |
| `agent/FEATURES.md` | Complete lx language guide — give to agents that need to write lx |
| `agent/FLOW_TESTING_FINDINGS.md` | Defects and fixes found during flow test development |

You own this language. Change spec, design, tests, flows, Rust code freely. Only constraint: internal consistency. When you change something, update all references. At session end, update `agent/DEVLOG.md` and this file's "Where We're At" section.
