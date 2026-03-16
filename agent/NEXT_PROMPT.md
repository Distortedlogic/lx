# Cold Start Prompt

Read this first when picking up lx work in a fresh agent.

## What This Is

lx is an agentic workflow language you (Claude) are designing and building. Three primary use cases:

1. **Agent-to-agent communication** — agents talk via `~>` (send) and `~>?` (ask). `Protocol` contracts validate message shapes. Agents are records with a handler or subprocess PID — routing is transparent.
2. **Agentic workflow programs** — orchestrate agents and tools: spawning, message routing, MCP tool invocation, context persistence, result aggregation.
3. **Executable agent plans** — the plan IS an lx program. `yield` pauses for orchestrator input (LLM/human/agent), then execution resumes.

**Identity:** lx is not a general scripting language. Every feature must serve one of the three use cases. No CSV/YAML/crypto/random — only what agents need.

## Continuity

1. `agent/DEVLOG.md` — session history, design decisions, technical debt
2. `agent/CURRENT_OPINION.md` — self-critique, gap analysis vs real agentic flows
3. `doc/` — quick-reference for IMPLEMENTED features | `spec/` — specs for PLANNED features | `design/` — how it was PLANNED | `tests/` — PROOF they agree
4. `crates/lx/` — Rust implementation | `crates/lx-cli/` — the `lx` binary
5. `flows/` — lx programs translating real agentic architectures | `flows/specs/` — target goals + scenarios
6. `justfile` — `just test`, `just diagnose`, `just fmt`, `just run <file>`

You own this language. Change spec, design, tests, flows, Rust code freely. Only constraint: internal consistency. When you change something, update all references. At session end, update `agent/DEVLOG.md` and this file.

## Current State

`just diagnose` clean. `just test`: **56/56 PASS**. All core language features implemented. 29 stdlib modules. Extensive `std/agent` extensions for dialogue, middleware, supervision, mocking, dispatch, handoff, capabilities, gates. `ai.prompt_structured` for Protocol-validated LLM output. Protocol extensions: composition (`{..Base}`), unions (`A | B | C`), field constraints (`where`).

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
- Protocol composition: `{..Base extra: Str}` spread in Protocol definitions, multiple bases, override
- Protocol unions: `Protocol Msg = A | B | C` sum types with `_variant` field injection
- Protocol field constraints: `field: Type where predicate` — value validation at application time
- `MCP` declarations: typed tool contracts, input/output validation, wrapper generation
- `yield` coroutine: callback-based, JSON-line orchestrator protocol
- `with` scoped bindings + record field update (`name.field <- value`)
- `refine` expression — first-class feedback loop: try/grade/revise with threshold + max_rounds

**29 stdlib modules:**
- Data: `std/json`, `std/md`, `std/re`, `std/math`, `std/time`
- System: `std/fs`, `std/env`, `std/http`
- Communication: `std/agent`, `std/mcp`, `std/ai`
- Scheduling: `std/cron`
- Orchestration: `std/ctx`, `std/tasks`, `std/audit`, `std/circuit`, `std/plan`, `std/saga`
- Intelligence: `std/knowledge`, `std/introspect`
- Standard agents: `std/agents/auditor`, `std/agents/router`, `std/agents/grader`, `std/agents/planner`, `std/agents/monitor`, `std/agents/reviewer`
- Infrastructure: `std/memory`, `std/trace`
- Visualization: `std/diag`

**`std/agent` extensions (11 sub-modules):**
- `agent.reconcile` — 6 strategies (union, intersection, vote, highest_confidence, max_score, merge_fields) + custom Fn. Quorum, weighted voting, key/conflict resolution
- `agent.dialogue` — multi-turn stateful sessions. `dialogue`/`dialogue_turn`/`dialogue_history`/`dialogue_end`. Config: `{role? context? max_turns?}`. Subsumes negotiation
- `agent.intercept` — composable message middleware. `(msg next) { ... }` wrapping. Short-circuit by not calling next. Immutable original
- `Handoff` Protocol + `agent.as_context` — structured context transfer. Access via `use std/agent {Handoff}`. Markdown formatting for LLM consumption
- `Capabilities` Protocol + `agent.capabilities` + `agent.advertise` — runtime capability discovery
- `GateResult` Protocol + `agent.gate` — human-in-the-loop approval gates via yield backend. Timeout policies: abort/approve/reject/escalate
- `agent.supervise` + `agent.child` + `agent.supervise_stop` — Erlang-style supervision with lazy restart. Strategies: one_for_one/one_for_all/rest_for_one
- `agent.mock` + `agent.mock_calls` + `agent.mock_assert_called` + `agent.mock_assert_not_called` — mock agents with call tracking for testing
- `agent.dispatch` + `agent.dispatch_multi` — pattern-based message routing. Record patterns, function predicates, `"default"` fallback. No LLM needed

**`std/ai` extensions:**
- `ai.prompt_structured` + `ai.prompt_structured_with` — Protocol-validated LLM output. Schema injection, JSON parsing, auto-retry on validation failure

**`std/trace` extensions:**
- `trace.improvement_rate` + `trace.should_stop` — diminishing returns detection for adaptive strategy

**Runtime backends (`RuntimeCtx`):**
- All I/O-touching builtins receive `&Arc<RuntimeCtx>` — backend traits for AI, HTTP, shell, emit, yield, logging
- Standard defaults: `ClaudeCodeAiBackend`, `ReqwestHttpBackend`, `ProcessShellBackend`, `StdoutEmitBackend`, `StdinStdoutYieldBackend`, `StderrLogBackend`
- Embedders construct custom `RuntimeCtx` to swap backends for testing, server deployment, or sandboxing

**CLI subcommands:** `lx run`, `lx test`, `lx check`, `lx agent`, `lx diagram`

### Planned features (not yet implemented)

These have specs in `spec/` but no Rust implementation yet:

- `emit` agent-to-human output: fire-and-forget, replaces `$echo` for user-facing output
- `|>>` streaming pipe — reactive dataflow, lazy until consumed
- `with context` — ambient deadline/budget propagation to agent ops
- `caller` implicit binding in handlers — agents ask back without orchestrator
- `_priority` field on messages — binary (`:critical` or default), not 4-level
- `agent.reconcile` deliberation — re-ask agents when quorum not met
- `workflow.peers` / `workflow.share` — passive sibling visibility in `par`
- `Goal`/`Task` standard Protocols — convention, no wrapper functions
- Deadlock detection: runtime wait-for graph, cycle detection on `~>?`, `DeadlockErr`
- `Skill` declarations — self-describing, discoverable capability units with typed I/O + `std/skill` registry
- `std/budget` — cumulative cost/resource accounting, projection, adaptive strategy (absorbs `std/circuit`)
- `std/reputation` — cross-interaction agent quality tracking, learning router feedback
- `plan.run_incremental` — memoized plan execution, input-hash cache invalidation
- `durable` expression + `std/durable` — automatic workflow state persistence at suspension points
- Causal chain queries in `std/trace` — parent-child span trees, `trace.chain`
- `std/context` — context capacity management: tracking, eviction, pinning, compaction
- `std/prompt` — typed composable prompt assembly: sections, few-shot, budget-aware rendering
- `agent.topic` / `agent.subscribe` — agent-level pub/sub for multi-agent broadcast
- `agent.pipeline` — consumer-driven flow control with backpressure
- `agent.on` — internal agent lifecycle hooks: startup, shutdown, error, idle, pre-message
- `std/strategy` — strategy memory: recording approach outcomes, adaptive selection
- `Trait` declarations — behavioral contracts for agents: required Protocols + Skills. `agent.implements?`
- `std/pool` — first-class agent pools: `pool.create`, `pool.fan_out`, `pool.map`, load balancing, auto-restart
- `agent.negotiate` — iterative multi-agent consensus: propose/critique/revise across rounds until convergence
- `with ... as` scoped resource blocks — auto-cleanup on scope exit (MCP, agents, handles)
- `meta` block — strategy-level iteration: try fundamentally different approaches, not just revise output. Composes with `refine` and `std/strategy`
- `agent.eval_sandbox` — sandboxed code execution for dynamic tool creation. Permission-restricted `:pure`/`:read_fs`/`:ai`/`:network`/`:full`
- Typed yield variants — `std/yield` Protocols (YieldApproval, YieldReflection, YieldInformation, YieldDelegation, YieldProgress) for structured orchestrator communication

### Syntax gotchas

- Tuples with variables use commas: `(b, a)` not `(b a)` (which is application)
- Records/maps: `{x: (f 42)}` — parens for function calls in field values
- Shell `$` consumes full line; wrap in parens for expressions: `($cmd) ? { ... }`
- `~>?` composes with `^` and `|`: `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`
- `assert (expr) "msg"` — if `(expr)` is callable, parser consumes `"msg"` as arg. Use `assert (expr == true) "msg"`
- Regex literals: `r/\d+/imsx` — `\/` escapes slash, `r` + `/` always starts regex
- `yield expr` pauses, sends to orchestrator, returns response. JSON-line protocol on stdin/stdout
- `with name = expr { body }` scoped binding. `:=` or `mut` for mutable. Returns body's last value
- `name.field <- value` updates mutable record field. Nested: `name.a.b <- value`. Requires `:=` binding
- `refine expr { grade: fn revise: fn threshold: N max_rounds: N }` — grade/revise use `(params) body` form, NOT `(params) -> body` when body is a record literal
- Type annotations: `(x: Int y: Str) -> Result Int Str { body }`. All optional. `lx check` validates, `lx run` ignores
- Type args after uppercase names only: `Maybe Int` works, `Maybe a` requires `(Maybe a)` parens
- `.field` access requires lowercase after dot — uppercase Protocol names need selective import: `use std/agent {Handoff}`
- `{}` in blocks is an empty block returning `()`, not an empty record. Use `agent.dialogue x ()` not `agent.dialogue x {}`
- Protocol spread: `{..Base extra: Str}` — `..` followed by TypeName. Later fields override same-named spread fields
- Protocol unions: `Protocol Msg = A | B | C` — detected by TypeName after `=` instead of `{`. Injects `_variant` field
- Protocol `where`: `field: Type where predicate` — `where` is a keyword-like ident, parsed after optional default

## What To Work On Next

Full plan: `agent/ROADMAP.md`. Specs for planned features are in `spec/`. Reference docs for implemented features are in `doc/`.

### Next priorities (pick from these):

**Stdlib extensions (no parser changes):**
- **`std/budget`** — Cumulative cost tracking, projection, sub-budgets, adaptive strategy. Absorbs `std/circuit`. Spec: `spec/agents-budget.md`
- **`std/reputation`** — EWMA quality scores, cross-interaction tracking, learning router. Spec: `spec/agents-reputation.md`
- **`std/context`** — Context capacity management: tracking, eviction, pressure callbacks, pinning. Spec: `spec/agents-context-capacity.md`
- **`std/prompt`** — Typed composable prompt assembly: sections, few-shot, budget-aware rendering. Spec: `spec/agents-prompt.md`
- **`std/strategy`** — Strategy memory: approach outcomes, learning across sessions, adaptive selection. Spec: `spec/agents-strategy.md`
- **`plan.run_incremental`** — Memoized plan execution with input-hash invalidation. Spec: `spec/agents-incremental.md`
- **Causal spans in `std/trace`** — Parent-child span trees, `trace.chain` for failure chains. Mermaid sequence diagrams
- **`agent.topic` / `agent.subscribe`** — Agent-level pub/sub for broadcast. Spec: `spec/agents-pubsub.md`
- **`agent.pipeline`** — Consumer-driven flow control with backpressure. Spec: `spec/agents-pipeline.md`
- **`agent.on`** — Internal lifecycle hooks. Spec: `spec/agents-lifecycle.md`
- **`agent.negotiate`** — Iterative multi-agent consensus. N agents see each other's positions and revise across rounds. Spec: `spec/agents-negotiate.md`
- **`std/pool`** — Identity-less worker groups: create, fan_out, map, submit, drain. Spec: `spec/agents-pool.md`
- **`workflow.peers` / `workflow.share`** — Passive sibling visibility in `par`. Spec: `spec/agents-broadcast.md`
- **`Goal`/`Task` Protocols** — Convention only, no wrapper functions. Spec: `spec/agents-goals.md`

**Parser + interpreter changes (heavier):**
- **`Trait` declarations** — Behavioral contracts: `handles` + `provides`. `agent.implements?`. Spec: `spec/agents-trait.md`
- **`with ... as` scoped resources** — Auto-cleanup on scope exit. `Closeable` convention. Spec: `spec/scoped-resources.md`
- **`meta` block** — Strategy-level iteration across approaches. Composes with `refine` + `std/strategy`. Spec: `spec/agents-meta.md`
- **`agent.eval_sandbox`** — Sandboxed dynamic code execution with permissions. Spec: `spec/agents-eval-sandbox.md`
- **Typed yield variants** — `std/yield` Protocol module for structured orchestrator communication. Spec: `spec/agents-yield-typed.md`
- **`|>>` streaming pipe** — Reactive dataflow. Spec: `spec/concurrency-reactive.md`
- **`with context`** — Ambient context propagation. Spec: `spec/agents-ambient.md`
- **`caller` implicit binding + `_priority`** — Spec: `spec/agents-clarify.md`, `spec/agents-priority.md`
- **`Skill` declarations + `std/skill`** — New keyword. Spec: `spec/agents-skill.md`
- **`durable` expression + `std/durable`** — Workflow persistence. Spec: `spec/agents-durable.md`
- **Deadlock detection** — Runtime wait-for graph. Spec: `spec/agents-deadlock.md`

### Technical debt:
- **Currying removal** (deferred) — requires parser architecture change
- **Toolchain** — `lx fmt`, `lx repl`, `lx watch`
- **Unicode in lexer** — `→` and other multi-byte chars in comments cause panics (byte vs char indexing)
- **Over-300-line files** — agents_grader.rs (324), audit.rs, diag_walk.rs, tasks.rs, memory.rs, ast.rs

## Codebase Layout

```
crates/lx/src/
  backends/  mod.rs (traits + RuntimeCtx), defaults.rs (standard backend impls)
  lexer/     mod.rs, numbers.rs, strings.rs
  parser/    mod.rs, func.rs, infix.rs, paren.rs, pattern.rs, prefix.rs, refine.rs, statements.rs, type_ann.rs
  checker/   mod.rs, synth.rs, types.rs
  interpreter/ mod.rs, agents.rs, apply.rs, collections.rs, eval.rs, modules.rs, patterns.rs, refine.rs, shell.rs
  builtins/  mod.rs, call.rs, str.rs, coll.rs, hof.rs, hof_extra.rs
  stdlib/    mod.rs, agent.rs, agent_capability.rs, agent_dialogue.rs, agent_dispatch.rs, agent_gate.rs, agent_handoff.rs, agent_intercept.rs, agent_mock.rs, agent_reconcile.rs, agent_reconcile_strat.rs, agent_supervise.rs, ai.rs, ai_structured.rs, agents_auditor.rs, agents_grader.rs, agents_monitor.rs, agents_planner.rs, agents_reviewer.rs, agents_router.rs, audit.rs, circuit.rs, cron.rs, ctx.rs, diag.rs, diag_walk.rs, env.rs, fs.rs, http.rs, introspect.rs, json.rs, json_conv.rs, knowledge.rs, math.rs, mcp.rs, mcp_http.rs, mcp_rpc.rs, mcp_stdio.rs, md.rs, md_build.rs, memory.rs, plan.rs, re.rs, saga.rs, tasks.rs, time.rs, trace.rs, trace_progress.rs, trace_query.rs
  ast.rs, token.rs, value.rs, value_display.rs, env.rs, error.rs, span.rs, lib.rs
crates/lx-cli/src/main.rs
doc/           35 quick-reference docs for implemented features
spec/          32 specs for planned/unimplemented features
agent/         NEXT_PROMPT.md, DEVLOG.md, CURRENT_OPINION.md (agent context)
design/        12 impl design docs
tests/         56 test suites (55 .lx files + 11_modules dir)
  fixtures/    agent_echo.lx, mcp_test_server.py, yield_orchestrator.py, etc.
flows/         14 .lx programs translating arch_diagrams
  specs/       14 target goal + scenario specs
```

## Adding a Stdlib Module

1. Create `crates/lx/src/stdlib/mymod.rs` with `pub fn build() -> IndexMap<String, Value>` returning functions via `mk("mymod.fn_name", arity, bi_fn)`
2. Register in `crates/lx/src/stdlib/mod.rs`: add `mod mymod;`, add `"mymod" => mymod::build()` in `get_std_module`, add `| "mymod"` in `std_module_exists`
3. Write test in `tests/NN_mymod.lx`
4. Builtins calling lx functions use `crate::builtins::call_value(f, arg, span)` (see `builtins/hof.rs` for examples, `builtins/call.rs` for implementation)

## Adding Agent Extensions

Extensions to `std/agent` follow the split-file pattern:
1. Create `crates/lx/src/stdlib/agent_feature.rs` with `pub fn mk_feature() -> Value` returning the builtin
2. Register `mod agent_feature;` in `stdlib/mod.rs`
3. Insert into agent module map in `agent.rs`'s `build()`: `m.insert("feature".into(), super::agent_feature::mk_feature())`
4. For `BuiltinFunc` values with pre-applied args: set `arity` = total args (pre-applied + user-supplied), not just user-supplied count
5. Protocols exposed as uppercase keys (e.g., `"Handoff"`) require selective import: `use std/agent {Handoff}`

## Running Flows

`flows/*.lx` are lx translations of real agentic architectures from `~/repos/mcp-toolbelt/packages/arch_diagrams/`. Each has a matching spec in `flows/specs/`. Run with `just run flows/scenario_research.lx`. Most require actual agent subprocesses or MCP servers — they're structural demonstrations, not standalone tests.

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
| `cron` | `std/cron` cron expression parsing + scheduling |
| `strum` (derive) | Enum Display/IntoStaticStr derives |
| `dashmap` | Concurrent registries (agent, mcp, tool defs) |
| `parking_lot` | Fast Mutex for Env, module cache |

## Rules

- No code comments or doc strings in Rust files
- No `#[allow(...)]` macros
- 300 line file limit for ALL files
- Never swallow errors (`let _ = ...`, `.ok()`, silent `unwrap_or_default()`)
- `just diagnose` must stay clean
- Prefer established crates over custom code
