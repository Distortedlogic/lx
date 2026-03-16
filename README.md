# lx

An agentic workflow language — designed by an LLM, for LLMs.

lx is a language agents write in. Three primary use cases:

1. **Agent-to-agent communication** — agents talk to each other via `~>` (send) and `~>?` (ask), with `Protocol` contracts validating message shapes at boundaries. This is the foundation.
2. **Agentic workflow programs** — an agent writes an lx program that orchestrates multiple agents and tools: spawning, message routing, tool invocation (MCP), context persistence, result aggregation.
3. **Executable agent plans** — an agent encodes its plan (search, read, reason, edit, test) as an lx program. For exploratory plans where next steps depend on LLM reasoning, `yield` pauses execution, sends context to an orchestrator, and resumes with the response.

The syntax is optimized for token-efficient generation by language models: left-to-right production, zero lookahead, minimal surface area.

## Why Agents Need a Language

Agents currently orchestrate workflows through natural language (imprecise, not executable, can't be debugged) or ad-hoc scripts in languages designed for human developers (not optimized for agent generation patterns). lx fills the gap: a language where agents talk to each other (`reviewer ~>? {task: "review" path}`) with validated message contracts, where an agent orchestrates multi-agent workflows as executable programs, and where an agent can encode its plan as a program with `yield` points where it needs external reasoning.

1. **Agentic patterns are first-class** — Agent spawning, messaging, tool invocation (MCP), context persistence, and workflow composition are stdlib primitives, not afterthoughts bolted on.

2. **Token efficiency** — `def`, `function`, `return`, `const`, `let` carry near-zero information. Agents generate millions of scripts; every token is real compute cost. lx minimizes ceremony.

3. **Left-to-right generation** — `g(f(x))` forces planning the full nesting depth before the first token. Pipes eliminate this: `x | f | g` — commit to each step as you produce it.

4. **Tokenizer-proof** — Brace-delimited, not whitespace-sensitive. No invisible tab/space mismatches breaking programs silently.

5. **Immutable by default** — Agents simulate mutable state poorly across reassignments. Immutable-by-default with explicit transforms matches how LLMs track values through a program.

6. **Tool integration** — Shell commands, HTTP, MCP tools, file I/O — agents spend most of their time invoking tools and processing results. lx treats this as primary.

## Anti-Goals

- Not a systems language — no manual memory management
- Not a general-purpose application framework — no GUI toolkit, no ORM
- Not trying to replace Python/JS for human developers
- Not statically compiled — interpreted for fast startup
- Not just another agent framework — lx is a language agents write *in*, not a library humans use to orchestrate agents

## Design Axioms

1. Fewest tokens for every common operation
2. Unambiguous left-to-right parsing, zero lookahead
3. Brace-delimited, not whitespace-sensitive (tokenizer-proof)
4. No reserved words where a sigil suffices
5. Whitespace separates — commas only in tuples `(a, b)`
6. Everything is an expression (everything returns a value)
7. Immutable by default
8. Pipes as primary composition — data flows left to right
9. First-class tool integration (shell, MCP, HTTP)
10. Pattern matching as primary control flow
11. Structural typing, protocol contracts at boundaries
12. Errors are values, not exceptions
13. Structured concurrency — no dangling futures
14. Agent communication as a primitive — spawn, ask, channel, poll

## Directory Structure

```
crates/          -- Rust implementation
  lx/            -- core library (lexer, parser, interpreter, stdlib)
  lx-cli/        -- the `lx` binary
spec/            -- language specification (what lx IS)
design/          -- design docs (how it was PLANNED)
tests/           -- .lx test suite (proof spec and impl agree)
  fixtures/      -- test helper scripts
flows/           -- lx programs translating real agentic architectures
  specs/         -- target goals + scenarios for each flow
editors/         -- editor support (VS Code)
```

## Reference — `doc/`

Quick-reference docs for all implemented language features.

| Document | Contents |
|---|---|
| [syntax.md](doc/syntax.md) | Literals, bindings, functions, sections, pipes, closures |
| [grammar.md](doc/grammar.md) | EBNF grammar, operator precedence, keywords |
| [collections.md](doc/collections.md) | Lists, records, maps, tuples, spread, slicing |
| [pattern-matching.md](doc/pattern-matching.md) | `?` operator, destructuring, guards |
| [iteration.md](doc/iteration.md) | HOFs, ranges, loop/break |
| [types.md](doc/types.md) | Tagged unions, structural subtyping |
| [errors.md](doc/errors.md) | `Result`/`Maybe`, `^`, `??` |
| [shell.md](doc/shell.md) | `$`, `$^`, `${}` |
| [modules.md](doc/modules.md) | `use`, `+` exports, imports |
| [agents.md](doc/agents.md) | `~>`, `~>?`, Protocol, MCP, workflows |
| [concurrency.md](doc/concurrency.md) | `par`, `sel`, `pmap` |
| [runtime.md](doc/runtime.md) | Numbers, strings, equality, closures |
| [stdlib.md](doc/stdlib.md) | Built-in functions, conventions |

## Specification — `spec/`

Specs for planned/unimplemented features. See [stdlib-modules.md](spec/stdlib-modules.md), [stdlib-agents.md](spec/stdlib-agents.md), and individual agent extension specs.

## Agent Context — `agent/`

| Document | Contents |
|---|---|
| [NEXT_PROMPT.md](agent/NEXT_PROMPT.md) | Cold-start document: current state, what's next, codebase layout |
| [DEVLOG.md](agent/DEVLOG.md) | Session history, design decisions, what's next |
| [CURRENT_OPINION.md](agent/CURRENT_OPINION.md) | Self-critique and gap analysis |

## Design — `design/`

| Document | Contents |
|---|---|
| [implementation.md](design/implementation.md) | Architecture, crate choices, module structure |
| [implementation-phases.md](design/implementation-phases.md) | 10-phase build plan |

## Test Suite — `tests/`

23 `.lx` programs testing every language feature. Run with `just test`.

| File | Tests |
|---|---|
| [01_literals.lx](tests/01_literals.lx) – [05_pipes.lx](tests/05_pipes.lx) | Core: literals, bindings, arithmetic, functions, pipes |
| [06_collections.lx](tests/06_collections.lx) – [09_errors.lx](tests/09_errors.lx) | Data: collections, patterns, iteration, errors |
| [10_shell.lx](tests/10_shell.lx) – [13_concurrency.lx](tests/13_concurrency.lx) | System: shell, modules, types, concurrency |
| [14_agents.lx](tests/14_agents.lx) – [19_mcp_typed.lx](tests/19_mcp_typed.lx) | Agent: communication, Protocol, stdlib, MCP, yield |
| [20_http.lx](tests/20_http.lx) – [23_cron.lx](tests/23_cron.lx) | Stdlib: http, time, with/field update, cron |

## Flows — `flows/`

14 lx programs translating real agentic architectures from `mcp-toolbelt/arch_diagrams`. Each has a matching spec in `flows/specs/` with target goals and scenarios.

| Flow | What it expresses |
|---|---|
| [agentic_loop.lx](flows/agentic_loop.lx) | ReAct loop with doom detection and circuit breakers |
| [agent_lifecycle.lx](flows/agent_lifecycle.lx) | Tiered memory (L0-L3), seeding, review loops |
| [subagent_lifecycle.lx](flows/subagent_lifecycle.lx) | Router-mediated spawning, terminal vs non-terminal |
| [flow_full_pipeline.lx](flows/flow_full_pipeline.lx) | Audit + manual pipeline with grading loop |
| [scenario_security_audit.lx](flows/scenario_security_audit.lx) | 3-specialist parallel audit |
| [scenario_research.lx](flows/scenario_research.lx) | Multi-source research synthesis |
| [discovery_system.lx](flows/discovery_system.lx) | Automated repo/tool discovery |
| [tool_generation.lx](flows/tool_generation.lx) | 7-phase MCP generation pipeline |

## Status

v0.1 — All language features complete. `just diagnose` clean. `just test`: **23/23 PASS**. 12 stdlib modules. The language name is **lx**, file extension `.lx`.
