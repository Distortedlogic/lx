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

`just diagnose` clean. `just test`: **24/24 PASS**. All language features complete.

### What's implemented

- Arithmetic, bindings, strings, interpolation, collections (lists, records, maps, tuples), pattern matching
- Functions, closures, currying, default params, pipes, sections, slicing, named args
- Type definitions with tagged values and pattern matching
- Concurrency: `par`, `sel`, `pmap`, `pmap_n`, `timeout` (sequential impl)
- Shell: `$cmd`, `$^cmd`, `${...}` with interpolation
- Error handling: `^` propagation, `??` coalescing, `(?? default)` sections
- Modules: `use ./path`, aliasing, selective imports, `+` exports
- Agent communication: `~>` send, `~>?` ask — infix operators, subprocess-transparent
- Message contracts: `Protocol Name = {field: Type}` with runtime validation
- `MCP` declarations: typed tool contracts, input/output validation, wrapper generation
- `yield` coroutine: callback-based, JSON-line orchestrator protocol
- `with` scoped bindings + record field update (`name.field <- value`)
- 12 stdlib modules: `std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/md`, `std/agent`, `std/mcp`, `std/http`, `std/time`, `std/cron`
- MCP HTTP streaming transport, `lx agent` subcommand
- Type annotations: `(x: Int y: Str) -> Result Int Str { ... }` on params, return types, bindings
- Type checker: `lx check` — bidirectional inference, unification, structural subtyping

### Syntax gotchas

- Tuples with variables use commas: `(b, a)` not `(b a)` (which is application)
- Records/maps: `{x: (f 42)}` — parens for function calls in field values
- Shell `$` consumes full line; wrap in parens for expressions: `($cmd) ? { ... }`
- `~>?` composes with `^` and `|`: `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`
- `assert (expr) "msg"` — if `(expr)` is callable, parser consumes `"msg"` as arg. Use `assert (expr == true) "msg"`
- `std/re` uses string patterns: `re.is_match "\\d+" text`
- `yield expr` pauses, sends to orchestrator, returns response. JSON-line protocol on stdin/stdout
- `with name = expr { body }` scoped binding. `:=` or `mut` for mutable. Returns body's last value
- `name.field <- value` updates mutable record field. Nested: `name.a.b <- value`. Requires `:=` binding
- Type annotations: `(x: Int y: Str) -> Result Int Str { body }`. Param types, return type, error type all optional
- Type args after uppercase names only: `Maybe Int` works, `Maybe a` requires `(Maybe a)` parens for lowercase type vars
- `lx check file.lx` — runs type checker without executing. `lx run` skips checking

## What To Work On Next

### Language priorities:

1. ~~**Type annotations + type checker**~~ — DONE (Session 29). `(x: Int) -> Str ^ Err` on params/returns/bindings. `lx check` runs bidirectional inference + unification. `lx run` still dynamic.
2. **Regex literals** — bring back `r/\d+/`. String patterns with double escaping is hostile to LLM generation. Lexer already had this; re-add.

### Stdlib roadmap (full plan: `design/stdlib_roadmap.md`):

3. **`std/ai`** — LLM integration. Generic interface, Claude Code CLI backend. `ai.prompt` (simple) + `ai.prompt_with` (full: system/model/tools/schema/budget/resume). Foundation for all standard agents — auditor/grader/router all depend on this.
4. **`std/tasks`** — task state machine, subtasks, auto-persist. Design doc: `design/std_tasks.md`.
5. **`std/audit`** — structural quality checks. Design doc: `design/std_audit.md`.
6. **`std/agents/auditor`** — LLM quality gate. Uses std/audit as pre-filter, std/ai for judgment.
7. **`std/agents/router`** — prompt → specialist classification. Uses std/ai.
8. **`std/agents/grader`** — rubric scoring, incremental re-grade. Uses std/ai.
9. **`std/agents/planner`** — task decomposition into ordered subtasks. Uses std/ai.
10. **`std/circuit`** — circuit breakers (turn/time/token limits, action repetition).
11. **`std/memory`** — tiered L0-L3 memory with confidence, promotion/demotion.
12. **`std/trace`** — trace collection, scoring, dataset export.
13. **`std/agents/monitor`** — QC sampling of running subagents.
14. **`std/agents/reviewer`** — post-hoc transcript review, learning extraction.
15. **`MCP Embeddings`** — typed interface to embedding services (similarity, retrieval).

Design docs: `design/standard_agents.md`, `design/stdlib_roadmap.md`

### Technical debt:

16. **Currying removal** (deferred) — requires parser architecture change
17. **Toolchain** (Phase 10) — `lx fmt`, `lx repl`, `lx check`, `lx watch`

### Real-flow coverage map:

| Flow Pattern | lx Status |
|---|---|
| Agent spawn + fanout | Covered (`pmap` + `~>?`) |
| Message validation | Covered (`Protocol`) |
| MCP tool invocation | Covered (`std/mcp` + `MCP` decls) |
| Context persistence | Covered (`std/ctx`) |
| Scheduled execution | Covered (`std/cron`) |
| Executable plans | Covered (`yield`) |
| Grading loops | Covered (`loop` + `~>?`) |
| Shell integration | Covered (`$`/`$^`/`${}`) |
| LLM integration | **GAP** → `std/ai` |
| End-to-end type safety | Covered (`lx check` + annotations) |
| Regex patterns | **GAP** → `r/pattern/` literals |
| Task tracking | **GAP** → `std/tasks` |
| Quality checks | **GAP** → `std/audit` + `std/agents/auditor` + `std/agents/grader` |
| Prompt routing | **GAP** → `std/agents/router` |
| Task decomposition | **GAP** → `std/agents/planner` |
| Circuit breakers | **GAP** → `std/circuit` |
| Tiered memory | **GAP** → `std/memory` |
| Observability | **GAP** → `std/trace` |
| Subagent QC | **GAP** → `std/agents/monitor` |
| Learning from experience | **GAP** → `std/agents/reviewer` |
| Embeddings/similarity | **GAP** → `MCP Embeddings` |

## Codebase Layout

```
crates/lx/src/
  lexer/     mod.rs, numbers.rs, strings.rs
  parser/    mod.rs, func.rs, infix.rs, paren.rs, pattern.rs, prefix.rs, statements.rs, type_ann.rs
  checker/   mod.rs, synth.rs, types.rs
  interpreter/ mod.rs, agents.rs, apply.rs, collections.rs, eval.rs, modules.rs, patterns.rs, shell.rs
  builtins/  mod.rs, call.rs, str.rs, coll.rs, hof.rs, hof_extra.rs
  stdlib/    mod.rs, json.rs, json_conv.rs, ctx.rs, math.rs, fs.rs, env.rs, re.rs, md.rs, md_build.rs, agent.rs, mcp.rs, mcp_rpc.rs, mcp_stdio.rs, mcp_http.rs, http.rs, time.rs, cron.rs
  ast.rs, token.rs, value.rs, value_display.rs, env.rs, error.rs, span.rs, lib.rs
crates/lx-cli/src/main.rs
spec/          23 language spec files
design/        11 impl design docs + DEVLOG + CURRENT_OPINION
tests/         23 .lx test files
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
| `regex` | `std/re` string pattern matching |
| `serde_json` (preserve_order) | JSON conversion, agent/MCP protocol |
| `pulldown-cmark` | `std/md` markdown parsing |
| `reqwest` (blocking, json) | `std/mcp` HTTP transport, `std/http` |
| `chrono` | `std/time` timestamp formatting/parsing |
| `strum` (derive) | Enum Display/IntoStaticStr derives |
| `dashmap` | Concurrent registries (agent, mcp, tool defs) |
| `parking_lot` | Fast Mutex for Env, module cache |

Custom code (~8300 lines: lexer, parser, interpreter, AST, builtins, stdlib) is language-specific — no crate replaces it. When adding new stdlib, use established crates.

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
