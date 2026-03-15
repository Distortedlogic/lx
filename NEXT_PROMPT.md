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

`just diagnose` clean. `just test`: **42/42 PASS**. All core language features and stdlib modules implemented.

### What's implemented

**Core language:**
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
- `with` scoped bindings + record field update (`name.field <- value`)

**29 stdlib modules:**
- Data: `std/json`, `std/md`, `std/re`, `std/math`, `std/time`
- System: `std/fs`, `std/env`, `std/http`
- Communication: `std/agent`, `std/mcp`, `std/ai`
- Orchestration: `std/ctx`, `std/cron`, `std/tasks`, `std/audit`, `std/circuit`, `std/plan`, `std/saga`
- Intelligence: `std/knowledge`, `std/introspect`
- Standard agents: `std/agents/auditor`, `std/agents/router`, `std/agents/grader`, `std/agents/planner`, `std/agents/monitor`, `std/agents/reviewer`
- Infrastructure: `std/memory`, `std/trace`
- Visualization: `std/diag`

**Key stdlib details:**
- LLM integration (`std/ai`): `ai.prompt` (text → text) + `ai.prompt_with` (full options → result record). Backend: Claude Code CLI (`claude -p --output-format json`). `RuntimeCtx` refactor (`spec/runtime-backends.md`) will put this behind an `AiBackend` trait so embedders can swap it. Shared utilities: `ai::parse_llm_json`, `ai::extract_llm_text`, `ai::strip_json_fences` (used by all standard agents)
- Shared eval utilities: `audit::build_eval_result`, `audit::make_eval_category`, `audit::keyword_overlap`, `audit::check_empty/refusal/hedging/references_task` (used by auditor + grader)
- Task state machine (`std/tasks`): create/start/submit/audit/pass/fail/revise/complete, auto-persist, hierarchical subtasks
- Structural quality checks (`std/audit`): is_empty/is_hedging/is_refusal/has_diff/references_task + rubric evaluate + quick_check
- Circuit breakers (`std/circuit`): turn/time/action limits, repetition detection
- Shared knowledge (`std/knowledge`): file-backed, provenance metadata, query with filter functions, merge, expire
- Dynamic plans (`std/plan`): dependency-ordered execution, replan/insert_after/skip/abort mid-flight
- Agent introspection (`std/introspect`): identity, elapsed, turn count, action log, markers, stuck detection, strategy shift
- Multi-agent transactions (`std/saga`): `saga.run` executes steps in order with compensating undo on failure. `saga.run_with` adds options (timeout, max_retries, on_compensate callback). `saga.define`/`saga.execute` for reusable saga definitions with initial context. Supports dependency ordering.
- Program visualization (`std/diag`): AST walker extracts workflow graph (agents, messages, control flow), emits Mermaid flowchart. `lx diagram file.lx` CLI subcommand + `diag.extract`/`diag.to_mermaid` library API

**CLI subcommands:** `lx run`, `lx test`, `lx check`, `lx agent`, `lx diagram`

### Planned features (not yet implemented)

These have specs in `spec/` but no Rust implementation yet:

- `emit` agent-to-human output: fire-and-forget, replaces `$echo` for user-facing output
- `agent.dialogue` / `agent.dialogue_turn` — multi-turn sessions
- `agent.intercept` — message middleware (tracing, rate-limiting, transformation)
- `agent.handoff` / `agent.as_context` + `Handoff` Protocol — structured context transfer
- `|>>` streaming pipe — reactive dataflow, lazy until consumed
- `agent.supervise` — Erlang-style restart strategies (one_for_one/one_for_all/rest_for_one)
- `with context` — ambient deadline/budget propagation to agent ops
- `caller` implicit binding in handlers — agents ask back without orchestrator
- `agent.gate` — structured human-in-the-loop with timeout policies
- `agent.capabilities` — runtime capability discovery via `Capabilities` protocol
- `_priority` field on messages (`:critical`/`:high`/`:normal`/`:low`)
- `refine` expression — first-class try-grade-revise with threshold + max_rounds
- `consensus` expression — multi-agent voting with quorum policies + deliberation
- `introspect.progress` / `improvement_rate` / `should_stop` — gradient progress tracking
- `agent.reconcile` — structured merge of parallel results with strategies
- `workflow.peers` / `workflow.share` — passive sibling visibility in `par`
- `Goal`/`Task` standard protocols + `agent.send_goal`/`agent.send_task`
- Deadlock detection: runtime wait-for graph, cycle detection on `~>?`, `DeadlockErr`

### Syntax gotchas

- Tuples with variables use commas: `(b, a)` not `(b a)` (which is application)
- Records/maps: `{x: (f 42)}` — parens for function calls in field values
- Shell `$` consumes full line; wrap in parens for expressions: `($cmd) ? { ... }`
- `~>?` composes with `^` and `|`: `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`
- `assert (expr) "msg"` — if `(expr)` is callable, parser consumes `"msg"` as arg. Use `assert (expr == true) "msg"`
- Regex literals: `r/\d+/imsx` — `\/` escapes slash, `r` + `/` always starts regex (not an ident). `std/re` accepts both `r/pat/` and `"\\pat"` strings
- `yield expr` pauses, sends to orchestrator, returns response. JSON-line protocol on stdin/stdout
- `with name = expr { body }` scoped binding. `:=` or `mut` for mutable. Returns body's last value
- `name.field <- value` updates mutable record field. Nested: `name.a.b <- value`. Requires `:=` binding
- Type annotations: `(x: Int y: Str) -> Result Int Str { body }`. All optional. `lx check` validates, `lx run` ignores.
- Type args after uppercase names only: `Maybe Int` works, `Maybe a` requires `(Maybe a)` parens for lowercase type vars

## What To Work On Next

Full plan: `design/stdlib_roadmap.md`. Specs for all planned features are in `spec/`.

### Completed stdlib roadmap items (1-18):

All 18 stdlib items are done: std/ai, std/tasks, std/audit, std/circuit, std/knowledge, std/plan, std/introspect, std/agents/auditor, std/agents/router, std/agents/grader, std/agents/planner, std/memory, std/trace, std/agents/monitor, std/agents/reviewer, MCP Embeddings (typed decl), std/diag, std/saga.

### Next priorities (pick from these):
19. **`RuntimeCtx` backend refactor** — Put all I/O operations behind backend traits on a `RuntimeCtx` parameter passed to all builtins. Standard defaults: Claude Code CLI for AI, reqwest for HTTP, `std::process::Command` for shell, stdout for emit, stdin/stdout JSON-lines for yield, stderr for logging. Enables swapping backends for testing, server deployment, or sandboxing. Also implements `emit` AST node. Spec: `spec/runtime-backends.md`. Mechanical refactor touching all builtins + stdlib modules.
20. **`refine` expression** — New keyword. First-class feedback loop: try → grade → revise with threshold + max_rounds. Spec: `spec/agents-refine.md`. Requires parser + interpreter changes.
21. **`consensus` expression** — New keyword. Multi-agent voting with quorum policies. Spec: `spec/agents-consensus.md`. Requires parser + interpreter changes.
22. **`introspect.progress`** — Extension to `std/introspect`. Gradient progress tracking, improvement rate, adaptive stopping. Spec: `spec/agents-progress.md`.
23. **`agent.reconcile`** — Extension to `std/agent`. Structured merging of parallel results. Spec: `spec/agents-reconcile.md`.
24. **`agent.dialogue`** — Extension to `std/agent`. Multi-turn session management. Spec: `spec/agents-dialogue.md`.
25. **`agent.intercept`** — Extension to `std/agent`. Message middleware. Spec: `spec/agents-intercept.md`.
26. **`agent.handoff`** — Extension to `std/agent`. Structured context transfer. Spec: `spec/agents-handoff.md`.
27. **`|>>` streaming pipe** — New operator. Reactive dataflow. Spec: `spec/concurrency-reactive.md`. Requires parser + interpreter changes.
28. **`agent.supervise` + `agent.gate` + `agent.capabilities`** — Extensions to `std/agent`. Spec: `spec/agents-supervision.md`, `spec/agents-gates.md`, `spec/agents-capability.md`.
29. **`with context`** — Ambient context propagation. Spec: `spec/agents-ambient.md`. Requires parser + interpreter changes.
30. **`caller` implicit binding + `_priority` field** — Interpreter-level. Spec: `spec/agents-clarify.md`, `spec/agents-priority.md`.
31. **`workflow.peers` / `workflow.share`** — Sibling visibility in `par`. Spec: `spec/agents-broadcast.md`.
32. **Goal/Task protocols** — `agent.send_goal`/`agent.send_task`. Spec: `spec/agents-goals.md`.
33. **Deadlock detection** — Runtime wait-for graph. Spec: `spec/agents-deadlock.md`.

### Technical debt:

34. **Currying removal** (deferred) — requires parser architecture change
35. **Toolchain** — `lx fmt`, `lx repl`, `lx watch`
36. **Unicode in lexer** — `→` and other multi-byte chars in comments cause panics (byte vs char indexing)

## Codebase Layout

```
crates/lx/src/
  lexer/     mod.rs, numbers.rs, strings.rs
  parser/    mod.rs, func.rs, infix.rs, paren.rs, pattern.rs, prefix.rs, statements.rs, type_ann.rs
  checker/   mod.rs, synth.rs, types.rs
  interpreter/ mod.rs, agents.rs, apply.rs, collections.rs, eval.rs, modules.rs, patterns.rs, shell.rs
  builtins/  mod.rs, call.rs, str.rs, coll.rs, hof.rs, hof_extra.rs
  stdlib/    mod.rs, agents_auditor.rs, agents_grader.rs, agents_monitor.rs, agents_planner.rs, agents_reviewer.rs, agents_router.rs, ai.rs, audit.rs, circuit.rs, diag.rs, diag_walk.rs, introspect.rs, knowledge.rs, memory.rs, plan.rs, saga.rs, tasks.rs, trace.rs, json.rs, json_conv.rs, ctx.rs, math.rs, fs.rs, env.rs, re.rs, md.rs, md_build.rs, agent.rs, mcp.rs, mcp_rpc.rs, mcp_stdio.rs, mcp_http.rs, http.rs, time.rs, cron.rs
  ast.rs, token.rs, value.rs, value_display.rs, env.rs, error.rs, span.rs, lib.rs
crates/lx-cli/src/main.rs
spec/          48 language spec files
design/        11 impl design docs + DEVLOG + CURRENT_OPINION
tests/         42 .lx test files
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

Custom code (~11500 lines: lexer, parser, checker, interpreter, AST, builtins, stdlib) is language-specific — no crate replaces it. When adding new stdlib, use established crates.

## Adding a Stdlib Module

1. Create `crates/lx/src/stdlib/mymod.rs` with `pub fn build() -> IndexMap<String, Value>` returning functions via `mk("mymod.fn_name", arity, bi_fn)`
2. Register in `crates/lx/src/stdlib/mod.rs`: add `mod mymod;`, add `"mymod" => mymod::build()` in `get_std_module`, add `| "mymod"` in `std_module_exists`
3. Write test in `tests/NN_mymod.lx`
4. Builtins calling lx functions use `crate::builtins::call_value(f, arg, span)` (see `builtins/hof.rs` for examples, `builtins/call.rs` for implementation)

## Running Flows

`flows/*.lx` are lx translations of real agentic architectures from `~/repos/mcp-toolbelt/packages/arch_diagrams/`. Each has a matching spec in `flows/specs/` with target goals and test scenarios. Run with `just run flows/scenario_research.lx`. Most require actual agent subprocesses or MCP servers to be running — they're structural demonstrations, not standalone tests. Note: flow files with unicode characters (like `→`) in comments will panic due to a lexer byte-indexing bug (technical debt #35).

## Rules

- No code comments or doc strings in Rust files
- No `#[allow(...)]` macros
- 300 line file limit for ALL files
- Never swallow errors (`let _ = ...`, `.ok()`, silent `unwrap_or_default()`)
- `just diagnose` must stay clean
- Prefer established crates over custom code
