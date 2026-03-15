# Cold Start Prompt

Read this first when picking up lx work in a fresh agent.

## What This Is

lx is an agentic workflow language you (Claude) are designing and building. Three primary use cases:

1. **Agent-to-agent communication** — agents talk via `~>` (send) and `~>?` (ask). `Protocol` contracts validate message shapes. Agents are records with a handler or subprocess PID — routing is transparent.
2. **Agentic workflow programs** — orchestrate agents and tools: spawning, message routing, MCP tool invocation, context persistence, result aggregation.
3. **Executable agent plans** — the plan IS an lx program. `yield` pauses for orchestrator input (LLM/human/agent), then execution resumes.

**Identity:** lx is not a general scripting language. Every feature must serve one of the three use cases. No CSV/YAML/crypto/random — only what agents need.

## Continuity

1. `design/DEVLOG.md` — session history, design decisions, technical debt
2. `design/CURRENT_OPINION.md` — self-critique, gap analysis vs real agentic flows
3. `spec/` — what lx IS | `design/` — how it was PLANNED | `tests/` — PROOF they agree
4. `crates/lx/` — Rust implementation | `crates/lx-cli/` — the `lx` binary
5. `flows/` — lx programs translating real agentic architectures | `flows/specs/` — target goals + scenarios
6. `justfile` — `just test`, `just diagnose`, `just fmt`, `just run <file>`

You own this language. Change spec, design, tests, flows, Rust code freely. Only constraint: internal consistency. When you change something, update all references. At session end, update DEVLOG and this file.

## Current State

`just diagnose` clean. `just test`: **33/33 PASS**. All language features complete.

### What's implemented

- Arithmetic, bindings, strings, interpolation, regex literals (`r/\d+/flags`), collections (lists, records, maps, tuples), pattern matching
- Functions, closures, currying, default params, pipes, sections, slicing, named args
- Type definitions with tagged values and pattern matching
- Type annotations: `(x: Int y: Str) -> Result Int Str { ... }` on params, return types, bindings
- Type checker: `lx check` — bidirectional inference, unification, structural subtyping
- Concurrency: `par`, `sel`, `pmap`, `pmap_n`, `timeout` (sequential impl)
- Shell: `$cmd`, `$^cmd`, `${...}` with interpolation
- Error handling: `^` propagation, `??` coalescing, `(?? default)` sections
- Modules: `use ./path`, aliasing, selective imports, `+` exports
- Agent communication: `~>` send, `~>?` ask — infix operators, subprocess-transparent
- Message contracts: `Protocol Name = {field: Type}` with runtime validation
- `MCP` declarations: typed tool contracts, input/output validation, wrapper generation
- `yield` coroutine: callback-based, JSON-line orchestrator protocol
- `emit` agent-to-human output: fire-and-forget, callback-based, replaces `$echo` for user-facing output (planned)
- `with` scoped bindings + record field update (`name.field <- value`)
- 19 stdlib modules (12 original + 7 new):
  - Data: `std/json`, `std/md`, `std/re`, `std/math`, `std/time`
  - System: `std/fs`, `std/env`, `std/http`
  - Communication: `std/agent`, `std/mcp`, `std/ai`
  - Orchestration: `std/ctx`, `std/cron`, `std/tasks`, `std/audit`, `std/circuit`, `std/plan`
  - Intelligence: `std/knowledge`, `std/introspect`
- LLM integration: `ai.prompt` (text → text) + `ai.prompt_with` (full options → result record). Backend: `claude -p --output-format json`
- Task state machine: `std/tasks` — create/start/submit/audit/pass/fail/revise/complete, auto-persist, hierarchical subtasks
- Structural quality checks: `std/audit` — is_empty/is_hedging/is_refusal/has_diff/references_task + rubric evaluate + quick_check
- Circuit breakers: `std/circuit` — turn/time/action limits, repetition detection
- Shared knowledge: `std/knowledge` — file-backed, provenance metadata, query with filter functions, merge, expire
- Dynamic plans: `std/plan` — dependency-ordered execution, replan/insert_after/skip/abort mid-flight
- Agent introspection: `std/introspect` — identity, elapsed, turn count, action log, markers, stuck detection, strategy shift
- MCP HTTP streaming transport, `lx agent` and `lx check` subcommands
- Multi-turn dialogue: `agent.dialogue` / `agent.dialogue_turn` — session-based accumulated context (planned)
- Message interceptors: `agent.intercept` — middleware for `~>`/`~>?` (tracing, rate-limiting, transformation) (planned)
- Structured handoff: `agent.handoff` / `agent.as_context` + `Handoff` Protocol (planned)
- Reactive dataflow: `|>>` streaming pipe — items flow downstream as they complete, lazy until consumed (planned)
- Supervision trees: `agent.supervise` — Erlang-style restart strategies (one_for_one/one_for_all/rest_for_one) (planned)
- Ambient context: `with context deadline: N budget: M { }` — auto-propagation to agent ops (planned)
- Inline clarification: `caller` implicit binding in handlers — agents ask back without orchestrator (planned)
- Approval gates: `agent.gate` — structured human-in-the-loop with timeout policies (planned)
- Capability discovery: `Capabilities` protocol + `agent.capabilities` query (planned)
- Saga pattern: `std/saga` — multi-agent transactions with compensating actions (planned)
- Message priority: `_priority` field (`:critical`/`:high`/`:normal`/`:low`) on messages (planned)

### Syntax gotchas

- Tuples with variables use commas: `(b, a)` not `(b a)` (which is application)
- Records/maps: `{x: (f 42)}` — parens for function calls in field values
- Shell `$` consumes full line; wrap in parens for expressions: `($cmd) ? { ... }`
- `~>?` composes with `^` and `|`: `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`
- `assert (expr) "msg"` — if `(expr)` is callable, parser consumes `"msg"` as arg. Use `assert (expr == true) "msg"`
- Regex literals: `r/\d+/imsx` — `\/` escapes slash, `r` + `/` always starts regex (not an ident). `std/re` accepts both `r/pat/` and `"\\pat"` strings
- `yield expr` pauses, sends to orchestrator, returns response. JSON-line protocol on stdin/stdout
- `emit expr` sends to human/orchestrator, returns `()`, does not block. Strings to stdout, records JSON-encoded. Replaces `$echo` for user-facing output
- `with name = expr { body }` scoped binding. `:=` or `mut` for mutable. Returns body's last value
- `name.field <- value` updates mutable record field. Nested: `name.a.b <- value`. Requires `:=` binding
- Type annotations: `(x: Int y: Str) -> Result Int Str { body }`. All optional. `lx check` validates, `lx run` ignores.
- Type args after uppercase names only: `Maybe Int` works, `Maybe a` requires `(Maybe a)` parens for lowercase type vars

## What To Work On Next

### Stdlib roadmap (full plan: `design/stdlib_roadmap.md`):

1. ~~**`std/ai`**~~ — DONE. LLM integration via Claude CLI.
2. ~~**`std/tasks`**~~ — DONE. Task state machine with auto-persist.
3. ~~**`std/audit`**~~ — DONE. Structural quality checks + rubric evaluate.
4. ~~**`std/circuit`**~~ — DONE. Circuit breakers (turns/time/actions/repetition).
5. ~~**`std/knowledge`**~~ — DONE. File-backed shared discovery cache.
6. ~~**`std/plan`**~~ — DONE. Dynamic plan-as-data execution with revision.
7. ~~**`std/introspect`**~~ — DONE. Agent self-awareness + action log.
8. **`std/agents/auditor`** — LLM quality gate. Uses std/audit as pre-filter, std/ai for judgment.
9. **`std/agents/router`** — prompt → specialist classification. Uses std/ai.
10. **`std/agents/grader`** — rubric scoring, incremental re-grade. Uses std/ai.
11. **`std/agents/planner`** — task decomposition into ordered subtasks. Uses std/ai.
12. **`std/memory`** — tiered L0-L3 memory with confidence, promotion/demotion.
13. **`std/trace`** — trace collection, scoring, dataset export.
14. **`std/agents/monitor`** — QC sampling of running subagents.
15. **`std/agents/reviewer`** — post-hoc transcript review, learning extraction.
16. **`MCP Embeddings`** — typed interface to embedding services (similarity, retrieval).
17. **`std/diag`** — program visualization. `lx diagram` CLI subcommand + `std/diag` library. Extract workflow graph from lx source, emit Mermaid. Spec: `spec/stdlib-diag.md`.

Also planned as extensions to `std/agent` (not separate modules): `agent.dialogue` (multi-turn sessions), `agent.intercept` (message middleware), `agent.handoff` / `agent.as_context` (structured context transfer), `agent.supervise` (supervision trees), `agent.gate` (approval gates), `agent.capabilities` (runtime discovery). Specs: `spec/agents-dialogue.md`, `spec/agents-intercept.md`, `spec/agents-handoff.md`, `spec/agents-supervision.md`, `spec/agents-gates.md`, `spec/agents-capability.md`.

Also planned as new language features: `|>>` streaming pipe (`spec/concurrency-reactive.md`), `with context` ambient propagation (`spec/agents-ambient.md`), `caller` implicit binding (`spec/agents-clarify.md`), `_priority` message field (`spec/agents-priority.md`). New module: `std/saga` (`spec/agents-saga.md`).

Design docs: `design/standard_agents.md`, `design/stdlib_roadmap.md`

### Technical debt:

18. **Currying removal** (deferred) — requires parser architecture change
19. **Toolchain** — `lx fmt`, `lx repl`, `lx watch`

## Codebase Layout

```
crates/lx/src/
  lexer/     mod.rs, numbers.rs, strings.rs
  parser/    mod.rs, func.rs, infix.rs, paren.rs, pattern.rs, prefix.rs, statements.rs, type_ann.rs
  checker/   mod.rs, synth.rs, types.rs
  interpreter/ mod.rs, agents.rs, apply.rs, collections.rs, eval.rs, modules.rs, patterns.rs, shell.rs
  builtins/  mod.rs, call.rs, str.rs, coll.rs, hof.rs, hof_extra.rs
  stdlib/    mod.rs, ai.rs, audit.rs, circuit.rs, introspect.rs, knowledge.rs, plan.rs, tasks.rs, json.rs, json_conv.rs, ctx.rs, math.rs, fs.rs, env.rs, re.rs, md.rs, md_build.rs, agent.rs, mcp.rs, mcp_rpc.rs, mcp_stdio.rs, mcp_http.rs, http.rs, time.rs, cron.rs
  ast.rs, token.rs, value.rs, value_display.rs, env.rs, error.rs, span.rs, lib.rs
crates/lx-cli/src/main.rs
spec/          33 language spec files
design/        11 impl design docs + DEVLOG + CURRENT_OPINION
tests/         32 .lx test files
  fixtures/    agent_echo.lx, mcp_test_server.py, yield_orchestrator.py, etc.
flows/         14 .lx programs translating arch_diagrams
  specs/       14 target goal + scenario specs
```

## Dependencies (audited 2026-03-14)

| Crate | Purpose |
|-------|---------|
| `miette` + `thiserror` | Error diagnostics with source context |
| `clap` v4 derive | CLI argument parsing |
| `num-bigint` / `num-traits` / `num-integer` | Arbitrary-precision integers |
| `indexmap` | Ordered maps (records, maps) |
| `regex` | `r/pattern/` literals + `std/re` pattern matching |
| `serde_json` (preserve_order) | JSON conversion, agent/MCP protocol |
| `pulldown-cmark` | `std/md` markdown parsing |
| `reqwest` (blocking, json) | `std/mcp` HTTP transport, `std/http` |
| `chrono` | `std/time` timestamp formatting/parsing |
| `strum` (derive) | Enum Display/IntoStaticStr derives |
| `dashmap` | Concurrent registries (agent, mcp, tool defs) |
| `parking_lot` | Fast Mutex for Env, module cache |

Custom code (~10800 lines: lexer, parser, checker, interpreter, AST, builtins, stdlib) is language-specific — no crate replaces it. When adding new stdlib, use established crates.

## Adding a Stdlib Module

1. Create `crates/lx/src/stdlib/mymod.rs` with `pub fn build() -> IndexMap<String, Value>` returning functions via `mk("mymod.fn_name", arity, bi_fn)`
2. Register in `crates/lx/src/stdlib/mod.rs`: add `mod mymod;`, add `"mymod" => mymod::build()` in `get_std_module`, add `| "mymod"` in `std_module_exists`
3. Write test in `tests/NN_mymod.lx`
4. Builtins calling lx functions use `crate::builtins::call_value(f, arg, span)` (see `builtins/hof.rs` for examples, `builtins/call.rs` for implementation)

## Running Flows

`flows/*.lx` are lx translations of real agentic architectures from `~/repos/mcp-toolbelt/packages/arch_diagrams/`. Each has a matching spec in `flows/specs/` with target goals and test scenarios. Run with `just run flows/scenario_research.lx`. Most require actual agent subprocesses or MCP servers to be running — they're structural demonstrations, not standalone tests.

## Rules

- No code comments or doc strings in Rust files
- No `#[allow(...)]` macros
- 300 line file limit for ALL files
- Never swallow errors (`let _ = ...`, `.ok()`, silent `unwrap_or_default()`)
- `just diagnose` must stay clean
- Prefer established crates over custom code
