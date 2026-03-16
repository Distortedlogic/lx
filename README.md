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
spec/            -- specs for planned/unimplemented features
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
| [NEXT_PROMPT.md](agent/NEXT_PROMPT.md) | Cold-start bootstrap: identity, current status, file map |
| [PRIORITIES.md](agent/PRIORITIES.md) | Ordered work queue with rationale |
| [INVENTORY.md](agent/INVENTORY.md) | Full list of implemented features |
| [OPINION.md](agent/OPINION.md) | Design self-critique: what works, what's wrong |
| [ROADMAP.md](agent/ROADMAP.md) | All planned future features |
| [DEVLOG.md](agent/DEVLOG.md) | Design decisions, tech debt, session history |
| [REFERENCE.md](agent/REFERENCE.md) | Codebase layout, how-to guides |

## Test Suite — `tests/`

65 test suites (64 `.lx` files + 11_modules dir) testing every language feature. Run with `just test`.

| File | Tests |
|---|---|
| 01–05 | Core: literals, bindings, arithmetic, functions, pipes |
| 06–09 | Data: collections, patterns, iteration, errors |
| 10–13 | System: shell, modules, types, concurrency |
| 14–19 | Agent: communication, Protocol, stdlib, MCP, yield |
| 20–25 | Stdlib: http, time, with, cron, type annotations, regex |
| 26–32 | AI, tasks, audit, circuit, knowledge, plan, introspect |
| 33–40 | Standard agents, memory, trace, monitor, reviewer, diag, saga |
| 41–45 | Refine, reconcile, trace_progress, dialogue, intercept |
| 46–54 | Handoff, capability, gate, supervise, ai_structured, mock, dispatch |
| 55–57 | Emit, protocol extensions, with_resource, trait |
| 58–63 | Pool, budget, prompt, context, agent_negotiate, agent_pubsub |
| 64 | Git: structured git access |

## Flows — `flows/`

13 lx programs translating real agentic architectures from `mcp-toolbelt/arch_diagrams`. Each has a matching spec in `flows/specs/` with target goals and scenarios. 10 reusable library modules in `flows/lib/`.

| Flow | What it expresses |
|---|---|
| agentic_loop | ReAct loop with doom detection and circuit breakers |
| agent_lifecycle | Tiered memory (L0-L3), seeding, review loops |
| full_pipeline | Audit + manual pipeline with grading loop |
| research | Multi-source research synthesis |
| perf_analysis | Performance analysis with specialist routing |
| project_setup | Task-based project scaffolding |
| discovery_system | Automated repo/tool discovery |
| tool_generation | 7-phase MCP generation pipeline |
| post_hoc_review | Post-hoc output review with memory |
| mcp_tool_audit | MCP tool audit with scoring |
| fine_tuning | Fine-tuning workflow with embeddings |
| software_diffusion | Software diffusion pipeline |

## Status

v0.1 — **65/65 tests pass.** `just diagnose` clean (2 pre-existing clippy warnings). 36 stdlib modules, 13 agent extensions, 6 standard agents. Complete core language, full agent system, structured git access. The language name is **lx**, file extension `.lx`.
