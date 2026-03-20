You are implementing a work item for the lx language. Follow every task exactly as written. Use justfile recipes (just diagnose, just test, just fmt). No code comments. 300 line file limit. Follow CLAUDE.md rules.
Context Bootstrap:
# Context Bootstrap

-- Tick: control register for agent/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- Context files in agent/ are your memory across sessions. Keep them accurate.
-- BEFORE writing code: follow Start of Tick Protocol in `TICK_PROTOCOL.md`
-- AFTER finishing work: follow End of Tick Protocol in `TICK_PROTOCOL.md`

## Identity

You are Claude, in `/home/entropybender/repos/lx/`. This is lx — an agentic workflow
language you designed and are building. Three use cases: agent-to-agent communication,
agentic workflow programs, executable agent plans. You own everything: spec, design,
implementation, tests. CLAUDE.md (already loaded) has the project rules.

## Sibling Domains

Three independent tick-loop domains share this repo. Each has its own TICK.md.
See `TICK_PROTOCOL.md` for cross-read guidance.

| Domain | CONTINUE | Purpose |
|--------|----------|---------|
| **agent/** (you) | `agent/TICK.md` | lx language — parser, interpreter, stdlib, tests |
| **brain/** | `brain/TICK.md` | Claude's cognitive self-model written in lx |
| **workgen/** | `workgen/TICK.md` | Work-item generation from audit checklists |

## State

Session 82 (2026-03-20). **98/98 tests pass.** `just diagnose` clean (0 errors, 0 warnings).
42 Rust stdlib modules + 42 lx packages in `pkg/` (7 clusters). Async interpreter.
Shipped this session: TYPE_CHECKER_COMPLETION work item (14 tasks) — exhaustiveness checking,
mutable capture detection, import conflict detection, Trait field type validation, `--strict`
mode, all Expr variants explicitly handled (no Unknown fallback), pattern variable binding in
match arms, infinite type on reassignment fix, parse vs type error separation in workspace check.
Checker split into 7 files.

## This Tick

**Next priority from PRIORITIES.md: Tier 6 parser-heavy features. Pick from:**
- `|>>` streaming pipe (`spec/concurrency-reactive.md`)
- `caller` implicit binding (`spec/agents-clarify.md`)
- Deadlock detection (`spec/agents-deadlock.md`)

Or check `work_items/` for remaining work items:
- `work_items/REPO_PIPELINING.md`
- Various other work items (ls work_items/ to see full list)

Read the specs/work items and pick whichever is most impactful / tractable.

## Read These Files

1. `agent/PRIORITIES.md` — feature queue, context for what to build
2. `spec/concurrency-reactive.md` — streaming pipe spec
3. `spec/agents-clarify.md` — caller binding spec
4. `spec/agents-deadlock.md` — deadlock detection spec
5. `agent/INVENTORY.md` — what's implemented
6. `agent/REFERENCE.md` — codebase layout and how-tos
7. `agent/GOTCHAS.md` — parser traps (7 new gotchas added Session 80)

## Context Files

| File | What it is | When to read |
|------|-----------|--------------|
| `agent/BUGS.md` | Known bugs, root causes, workarounds | Before fixing bugs |
| `agent/PRIORITIES.md` | Feature work queue | To decide what to build next |
| `agent/INVENTORY.md` | What's implemented | To check if something exists |
| `agent/DEVLOG.md` | Decisions, debt, session log | To understand past decisions |
| `agent/LANGUAGE.md` | Core lx syntax + semantics | When writing lx (core language) |
| `agent/AGENTS.md` | Agent system + extensions | When writing lx (agent features) |
| `agent/STDLIB.md` | Stdlib + builtins reference | When writing lx (library calls) |
| `agent/GOTCHAS.md` | Non-obvious behaviors | When something fails unexpectedly |
| `agent/HEALTH.md` | Design assessment | To understand what needs work |
| `agent/REFERENCE.md` | Codebase layout, how-tos | When adding Rust implementation |

## Rules

- No code comments except `--` headers in flow files
- 300 line file limit — split if exceeded
- Use justfile recipes: `just diagnose`, `just test`, `just fmt`
- Do not run commands with appended pipes or redirects
- No #[allow()] macros. No doc strings. No re-exports.

## End of Tick

**MANDATORY: Execute ALL 5 steps in `TICK_PROTOCOL.md` as one uninterrupted sequence.**
Do not declare completion without running every step. Do not skip context file reviews.


# Inventory
-- Memory: capability register. Everything lx can do right now.
-- Add entries when features ship. Keep current — a stale inventory misleads the next you.

# Implemented Feature Inventory

## Core Language

- Arithmetic, bindings, strings, interpolation, regex literals (`r/\d+/flags`), collections (lists, records, maps, tuples), pattern matching
- Functions, closures, currying, default params, pipes, sections, slicing, named args
- Type definitions with tagged values and pattern matching
- Type annotations: `(x: Int y: Str) -> Result Int Str { ... }` on params, return types, bindings
- Type checker: `lx check` — bidirectional inference, unification, structural subtyping, import resolution (imported names bound as Unknown). Exhaustiveness checking for match on union types (warns on missing variants). Mutable capture detection in `par`/`sel` concurrent contexts. Import conflict detection (warns on duplicate names). Trait constructor field type validation. `--strict` mode (warnings as errors). All `Expr` variants explicitly handled (no Unknown fallback). Pattern variables bound in match arm scopes. Parse vs type error distinction in workspace check
- Concurrency: `par`, `sel`, `pmap`, `pmap_n`, `timeout` — async interpreter (`async fn eval` with `#[async_recursion(?Send)]`). `par` → `futures::join_all`, `sel` → `futures::select_all`, `pmap`/`pmap_n` → `join_all`. I/O operations yield naturally at `.await` points
- Shell: `$cmd`, `$^cmd`, `${...}` with interpolation
- Error handling: `^` propagation, `??` coalescing, `(?? default)` sections. Structured error tags: `Err Timeout "msg"` with pattern matching. Uniform `None` on miss for Record, Map, and Agent field access. `AgentErr` structured errors: 11 tagged variants (Timeout, RateLimited, BudgetExhausted, ContextOverflow, Incompetent, Upstream, PermissionDenied, TraitViolation, Unavailable, Cancelled, Internal) via `use std/agent {Timeout ...}`
- Arithmetic: `/` always returns Float (Python 3 semantics), `//` for integer division, mixed Int/Float auto-promotion
- Modules: `use ./path`, aliasing, selective imports, `+` exports (non-forward-declared — builtins not shadowed), workspace member resolution (`use brain/protocols`), dependency resolution (`use dep-name/module` via `.lx/deps/`)
- **`Class` keyword** — generic stateful objects with `self` method dispatch. `Class Name : [Traits] = { field: default; method = (params) { body } }`. Constructor: `Name {field_overrides}`. Interior mutability: `self.field <- val` mutates via global STORES (no reassign needed). Reference semantics: `a = b` shares same object. `Class Worker : [Agent] = { ... }` also works — explicitly adding Agent to traits list
- **`Store` as first-class Value** — `Value::Store { id }` with dot-access methods: set, get, keys, values, entries, remove, len, has, clear, filter, query, map, update, save, load, persist, reload. `Store ()` constructor. Reference semantics. Store cloning in Class constructors.
- **Trait default methods** — `Trait Name = { required: Sig -> Ret; default_method = (params) { body } }`. Default methods injected into conforming Class/Agent if not overridden.

## Agent System

- `Agent Name: TraitList = { methods }` — first-class agent declarations. `Agent` keyword auto-imports `pkg/agent {Agent}` Trait and auto-adds "Agent" to traits list. Runtime representation: `Value::Class { name, traits, defaults, methods }`. Agent Trait (`pkg/agent.lx`) provides defaults: init, perceive, reason, act, reflect, handle, run, think/think_with/think_structured, use_tool/tools, describe, ask/tell. Method access via `.`
- `receive { action -> handler }` — agent message loop sugar, desugars to yield/loop/match
- `~>` send, `~>?` ask, `~>>?` streaming ask — infix operators, subprocess-transparent. `~>>?` returns `Value::Stream` (mpsc channel-backed lazy sequence). Streams work with all HOFs (`map`, `filter`, `each`, `take`, `fold`, `flat_map`, etc.) and `collect`. Subprocess wire protocol: JSON-line `stream`/`stream_end`/`stream_error` types with background reader thread and cancellation
- `Trait Name = {field: Type}` — message contracts with runtime validation (returns `Err` on validation failure, catchable with `??`)
- Trait composition (`{..Base extra: Str}`), unions (`A | B | C` with `_variant`), field constraints (`where`)
- `Trait Name = { method: {input} -> output }` — agent behavioral contracts with default method implementations. Traits with non-empty `fields` act as Traits (callable as constructor, runtime validation). Behavioral Traits have empty `fields`.
- `agent.implements` — runtime trait checking for routing/filtering (works for Class/Agent, Object, Record). Checks traits list for "Agent" to distinguish Agents from plain Classes
- Two new builtins: `method_of(obj, name)` — returns a method by name or None; `methods_of(obj)` — returns list of method names
- `MCP` declarations — typed tool contracts, input/output validation, wrapper generation
- `with expr as name { body }` — scoped resources with auto-cleanup (LIFO close, cleanup on error)
- `yield` — callback-based coroutine, JSON-line orchestrator protocol. Typed yield variants via `std/yield`: 5 Traits (YieldApproval, YieldReflection, YieldInformation, YieldDelegation, YieldProgress) with auto-injected `kind` field. `use std/yield {YieldApproval ...}`. Backwards compatible — untyped `yield expr` unchanged
- `refine` — first-class feedback loop: try/grade/revise with threshold + max_rounds
- `emit` — agent-to-human fire-and-forget output via EmitBackend
- `with name = expr { body }` — scoped bindings + record field update (`name.field <- value`)
- `with context key: val { body }` — ambient context propagation. Scoped state flows through call chains without explicit parameter threading. `context.field` dot-access, `context.current ()` returns full context record, `context.get key` returns Some/None. Nesting merges with outer context; inner values override; outer restored on scope exit. `context` globally available (returns `{}` outside any scope)
- `meta task { strategies: [...] attempt: fn evaluate: fn select?: "sequential" on_switch?: fn }` — strategy-level iteration. Tries fundamentally different approaches. Returns `Ok {result strategy attempts}` on first viable, `Err {reason attempts best}` if all exhausted. Contextual keyword (not reserved — usable as identifier). Composes with `refine` (meta selects approach, refine optimizes within it)

## Stdlib (43 Rust modules + 6 standard agents + 11 lx packages)

- Data: `std/json`, `std/md`, `std/re`, `std/math`, `std/time`
- System: `std/fs`, `std/env`, `std/http`
- Git: `std/git` — 36 functions
- State primitive: `std/store` — backing implementation for `Value::Store`. `Store ()` constructor creates a first-class Store value with dot-access methods (set, get, keys, values, entries, remove, len, has, clear, filter, query, map, merge, update, save, load, persist, reload)
- Resilience: `std/retry`, `std/deadline`
- Communication: `std/agent`, `std/mcp`, `std/ai`
- Observation: `std/introspect` — system-wide live introspection: `system` (full snapshot), `agents` (agent list), `agent` (deep single-agent info), `messages` (in-flight), `bottleneck` (busiest agent). Aggregates from REGISTRY, SESSIONS, SUPERVISORS, TOPICS, ROUTE_TABLE
- Scheduling: `std/cron`
- Orchestration: `std/ctx` (deprecated — use Store), `std/audit`, `std/plan`, `std/saga`, `std/pipeline`, `std/taskgraph`, `std/workspace`
- Collaboration: `std/workspace` — concurrent multi-agent editing: `create`, `claim`, `claim_pattern`, `edit`, `append`, `release`, `snapshot`, `regions`, `conflicts`, `resolve`, `history`, `watch`. Line-based region claiming with overlap detection, auto-bound adjustment, regex pattern claiming, watcher callbacks
- Discovery: `std/registry` — cross-process agent discovery: `start`, `stop`, `connect`, `register`, `deregister`, `find`, `find_one`, `health`, `load`, `watch`. In-memory registry with trait/trait/domain filtering, 4 selection strategies (first, least_loaded, round_robin, random), health/load tracking, watcher callbacks for join/leave events
- Persistence: `std/durable` — Temporal-style workflow persistence: `workflow`, `run`, `step` (idempotent with caching), `sleep`, `signal`, `send_signal`, `status`, `list`. File-backed at `.lx/durable/`
- Yield types: `std/yield` — 5 Trait-only exports (YieldApproval, YieldReflection, YieldInformation, YieldDelegation, YieldProgress). Typed yield variants with `kind` field default for orchestrator dispatch. No functions, pure Trait definitions
- Cost management: `std/budget`
- Standard agents: `std/agents/auditor`, `std/agents/grader` (Rust-backed, use internal APIs). `std/agents/planner`, `std/agents/router`, `std/agents/reviewer` (deprecated — use pkg/ai/ equivalents). `std/agents/monitor` removed (use pkg/agents/guard)

## lx Packages (pkg/ workspace member — 42 packages in 7 clusters)

**pkg/core/** — foundational primitives:
- `circuit` — CircuitBreaker Class: turn/action/time/repetition trip conditions
- `collection` — Collection Trait: get, keys, values, remove, query, len, has, save, load defaults
- `contracts` — shared Trait definitions: ToolRequest, ToolResult, ActionPlan, ActionResult, AgentTask, AgentResult, ContextItem
- `introspect` — Inspector Class + 8 introspection functions: self_assess, detect_doom_loop, strategy_analysis, time_pressure, generate_status, should_pivot, narrate_thinking, suggest_pivot
- `pool` — Pool Class: worker pools with round-robin dispatch
- `prompt` — composable prompt assembly: create, system, section, instruction, constraint, render, ask, ask_with, ask_lines
- `connector` — Connector Trait: connect, disconnect, call, tools — generic interface for all external tool access
- `score` — composite scoring, tier classification, normalize, average

**pkg/connectors/** — external tool connectors:
- `mcp` — McpConnector Class: wraps std/mcp in Connector interface with session lifecycle
- `cli` — CliConnector Class: wraps shell execution in Connector interface with tool_defs-to-CLI-args mapping
- `catalog` — connector instances: gritql, forgejo, context_engine, langfuse, postgresql, uptime_kuma (MCP), gh, jq_tool, curl (CLI)

**pkg/ai/** — AI-powered operations:
- `ai_agent` — generic AI agent helpers: simple, with_fallback, structured, create, serve
- `agent_factory` — config-driven agent instantiation: from_config builds handlers from config records with type-dispatched handler construction
- `perception` — input classification: perceive, classify_intent, extract_entities, assess_complexity, detect_domain
- `planner` — AI task decomposition: plan, quick_plan (replaces std/agents/planner)
- `quality` — grading + refinement: rubrics, grade_response, grade_code, refine_work (generic), refine_response, refine_code (wrappers), final_audit
- `reasoning` — multi-strategy AI reasoning: direct, decompose, analogical, adversarial + hypothesis engine
- `reflect` — post-action learning: reflect, extract_patterns, update_strategy, improvement_suggestions
- `reviewer` — transcript analysis: review, quick_review (replaces std/agents/reviewer)
- `router` — prompt routing: route, quick_route (replaces std/agents/router)

**pkg/data/** — state management:
- `context` — ContextWindow Class: capacity management with priority eviction
- `knowledge` — KnowledgeBase Class: key-value with metadata and timestamps
- `memory` — MemoryStore Class: tiered memory with fuzzy recall, promote/demote, consolidation + single-store helpers (tiers, thresholds, create, seed, daily, weekly)
- `tasks` — TaskStore Class: task state machine
- `tieredmem` — multi-store composition: working/episodic/semantic with consolidation and decay (single-store helpers moved to memory)
- `trace` — TraceStore Class: span recording, provenance tracking, reputation scoring
- `transcript` — JSONL transcript ingestion: parse, filter, pattern extraction

**pkg/agents/** — agent lifecycle:
- `catalog` — agent registry: create, by_domain, by_name, add (route removed — use pkg/ai/router.quick_route)
- `dialogue` — multi-turn conversation state: init_conversation, add_turn, topic tracking, rapport (pass-through wrappers removed — use std/agent directly)
- `dispatch` — spawn/ask/kill shorthand: run_one, run_many, run_with, run_handler
- `guard` — security scanning: check (with optional patterns), injection detection, loop detection, resource monitoring, severity helpers
- `monitor` — health monitoring: circuit breaker + budget + inspector facade (introspection functions moved to pkg/core/introspect)
- `react` — ReAct loop engine: think→action→observation with circuit breaker

**pkg/infra/** — pipeline tooling:
- `guidance` — stage gates: check_cargo, check_build/tests/clippy, ask_user, gate
- `mcp_session` — DEPRECATED (use pkg/connectors/mcp McpConnector): with_server bracket, make_client factory
- `report` — markdown document builder: write, section, bullets, table
- `testkit` — data-driven test harness: default_grader, run_spec, run_spec_live, load_fixture
- `workflow` — traced session runner: run, run_with_report

**pkg/kit/** — cross-layer compositions:
- `context_manager` — pressure_pct + AI compression on ContextWindow (pressure_level removed — use win.pressure() directly)
- `grading` — agent lifecycle + grader refine loop: grade_draft, grade_run
- `investigate` — AI-powered codebase investigation with tool access
- `security_scan` — compose guard + transcript into scan pipeline
- `tool_executor` — AI tool selection + retry/circuit-breaker execution + result integration (execute accepts optional dispatch_fn for non-builtin tools)
- Infrastructure: `std/trait`
- Interaction: `std/user` — `confirm`, `choose`, `ask`, `ask_with`, `progress`, `progress_pct`, `status`, `table`, `check` (signal poll). `UserBackend` trait on `RuntimeCtx` — `NoopUserBackend` (default/test), `StdinStdoutUserBackend` (terminal)
- Identity: `std/profile` — persistent agent profiles: `load`, `save`, `learn`, `recall`, `recall_prefix`, `forget`, `preference`, `get_preference`, `history`, `merge`, `age`, `decay`. Strategy helpers: `best_strategy`, `rank_strategies`, `adapt_strategy`. File-backed at `.lx/profiles/{name}.json`
- Visualization: `std/diag`
- Testing: `std/test` (satisfaction testing: `spec`, `scenario`, `run`, `run_scenario`, `report`), `std/describe` (BDD-style describe/it blocks with structured results)
- Flow composition: `std/flow` — `load`, `run`, `pipe`, `parallel`, `branch`, `with_retry`, `with_timeout`, `with_fallback`. Flows as first-class composable values with isolated interpreter execution
- Task graphs: `std/taskgraph` — `create`, `add`, `remove`, `run`, `run_with`, `validate`, `topo`, `status`, `dot`. DAG-aware subtask decomposition with topological execution, dependency result threading (`input_from`), per-task retry/timeout/on_fail policy, wave-based parallel scheduling, DOT export

## Agent Extensions (19 sub-modules of `std/agent`)

- `agent.reconcile` — 6 merge strategies (union, intersection, vote, highest_confidence, max_score, merge_fields) + custom Fn
- `agent.dialogue` — multi-turn stateful sessions with config `{role? context? max_turns?}`. Branching: `dialogue_fork` (N forks sharing parent history), `dialogue_compare` (grade + rank forks), `dialogue_merge` (pick winner, resume parent), `dialogue_branches` (list active forks). Parent suspended while forks active. Recursive nested fork support
- `agent.intercept` — composable message middleware with short-circuit
- `Handoff` Trait + `agent.as_context` — structured context transfer for LLM consumption
- `Capabilities` Trait + `agent.capabilities` + `agent.advertise` — runtime capability discovery
- `GateResult` Trait + `agent.gate` — human-in-the-loop approval gates via yield
- `agent.supervise` — Erlang-style supervision: one_for_one/one_for_all/rest_for_one
- `agent.mock` — mock agents with call tracking for testing
- `agent.dispatch` — pattern-based message routing without LLM
- `agent.negotiate` — N-party iterative consensus with converge function
- `agent.topic` / `agent.subscribe` / `agent.publish` — in-process pub/sub with filtered subscriptions
- `agent.route` / `agent.register` — capability-based routing: register agents with traits/protocols/domains, route by filter with selection strategies (least_busy, round_robin, random, custom), fan-out with reconcile via `route_multi`
- `agent.pipeline` — consumer-driven flow control with backpressure: 11 functions (`pipeline`, `pipeline_send`, `pipeline_collect`, `pipeline_batch`, `pipeline_stats`, `pipeline_on_pressure`, `pipeline_pause`, `pipeline_resume`, `pipeline_drain`, `pipeline_close`, `pipeline_add_worker`). Bounded buffers, 4 overflow policies (block, drop_oldest, drop_newest, sample), tail-first pump for backpressure, round-robin worker dispatch, pressure callbacks with level thresholds, per-stage stats with bottleneck detection
- `agent.emit_stream` / `agent.end_stream` — agent-side streaming API for `~>>?`. `emit_stream` writes `{"type":"stream","value":...}` JSON-line, `end_stream` writes `{"type":"stream_end"}`
- `agent.adapter` / `agent.negotiate_format` / `agent.coerce` — Trait format negotiation: `adapter` creates reusable field-mapping interceptors from source→target Trait with explicit mapping record, `negotiate_format` auto-discovers compatible Trait mappings via agent capabilities (exact/structural/subset matching with Levenshtein heuristics), `coerce` does one-shot message transform with validation. Adapters return `Value::Err` on missing required fields (catchable with `??`)
- `agent.reload` / `agent.evolve` / `agent.update_traits` — Hot-swap agent handlers: `reload` replaces handler externally (returns new agent Record with `__handler_id` referencing global mutable handler store), `evolve` self-updates from within handler (thread-local pending flag applied by interpreter after handler returns, takes effect on NEXT message), `update_traits` adds/removes traits on agent Records. Subprocess agents return `Err` on reload. Interceptors preserved — interceptor `next` dynamically resolves handler via store
- `agent.dialogue_save` / `agent.dialogue_load` / `agent.dialogue_list` / `agent.dialogue_delete` — Dialogue persistence: `dialogue_save` persists session state (config + turn history) to `.lx/dialogues/{id}.json`. `dialogue_load` restores session from file and binds to a (possibly different) agent. `dialogue_list` enumerates saved dialogues with metadata (id, role, turns, created, updated, context_preview). `dialogue_delete` removes saved dialogue. Atomic writes (tmp+rename). JSON serialization via `json_conv`
- `agent.on` / `agent.on_remove` / `agent.startup` / `agent.shutdown` / `agent.signal` / `agent.idle_hooks` — Lifecycle hooks: 6 events (startup, shutdown, error, idle, message, signal). Dynamic hook registration on agents. Multiple hooks per event (fire in registration order). Idle hooks require duration in seconds. Error hooks are curried `(err)(msg)`. `agent.kill` runs shutdown hooks before killing. Global HOOKS DashMap with auto-assigned lifecycle IDs

## Other Extensions

- `ai.prompt_structured` — Trait-validated LLM output with auto-retry
- `ai.prompt_json` — lightweight structured output from inline record shape (no Trait needed)

## Runtime

- **Async interpreter**: `eval`/`exec`/`eval_expr` are `async fn`. Interpreter runs inside `ctx.tokio_runtime.block_on()` at CLI entry. Builtin functions split: `BuiltinKind::Sync` (pure builtins — math, string, collection ops) and `BuiltinKind::Async` (HOFs that invoke callbacks — map, filter, fold, etc., return `BoxFuture`). `call_value` is async; `call_value_sync` bridge for sync stdlib functions (`block_in_place` + `Handle::current().block_on()`). Cron background threads use `ctx.tokio_runtime.block_on()` directly
- All I/O builtins receive `&Arc<RuntimeCtx>` — backend traits (still sync): `AiBackend`, `EmitBackend`, `HttpBackend`, `ShellBackend`, `YieldBackend`, `LogBackend`, `UserBackend`
- Standard defaults: `ClaudeCodeAiBackend`, `ReqwestHttpBackend`, `ProcessShellBackend`, `StdoutEmitBackend`, `StdinStdoutYieldBackend`, `StderrLogBackend`, `NoopUserBackend`
- Embedders construct custom `RuntimeCtx` to swap backends for testing, server deployment, or sandboxing
- Dependencies: `async-recursion` (recursive eval boxing), `futures` (join_all/select_all), `tokio` (runtime), `rayon` (still present but unused by concurrency primitives)

## CLI

`lx run`, `lx test`, `lx check`, `lx agent`, `lx diagram`, `lx list`, `lx install`, `lx update`, `lx init`

- **`lx init [name] [--flow]`**: Project scaffolding. Creates `lx.toml`, `src/main.lx`, `test/main_test.lx`. `--flow` adds `src/agents/`, `test/scenarios/`, and `[test]` section with default threshold/runs

- **Workspace support**: `lx.toml` manifest parsing, workspace discovery (walk up from cwd), `lx test` iterates members, `lx test -m name` filters, `lx list` shows member summary, `lx run member-name` resolves to entry file, `lx check` / `lx check -m name` workspace iteration
- Justfile recipes: `just test`, `just test-all` (workspace), `just test-member <name>`, `just list`, `just diagnose`, `just fmt`, `just build`, `just install`

## Workspace

`lx.toml` manifests — root workspace declares `[workspace].members`, each member has `[package]` (name, version, entry, description, authors, license, lx) and optional `[test]` (dir, pattern, threshold, runs), `[backends]` (ai, shell, http, emit, yield, log, user), `[deps]` + `[deps.dev]`. `version` is required when `[package]` is present. Dev deps installed but filtered from `lx run` (available in `lx test`). Backend preferences wired to RuntimeCtx. Test threshold/runs propagated via `RuntimeCtx.test_threshold`/`test_runs`. This repo is the first workspace: tests, brain, workgen, flows. Module resolver checks workspace members between relative and stdlib paths (`use member/path` → member's root directory).

## Test Coverage

98 test suites (96 .lx files + 87_export_shadow dir + 11_modules dir) in `tests/`. Fixtures in `tests/fixtures/`. 98/98 passing. `lx init` scaffolding verified via `just diagnose`.


# Reference
-- Memory: ROM. Codebase layout and how-to guides for implementation work.
-- Update when file structure changes or new how-to patterns emerge.

# Reference

## Codebase Layout

```
crates/lx/src/
  ast/         AST node definitions + type annotation AST
  backends/    RuntimeCtx struct, backend traits (Ai/Emit/Http/Shell/Yield/Log/User), default impls
  lexer/       Tokenizer — mod, numbers, strings, keywords, helpers
  parser/      Recursive descent — mod + split files per feature (func, infix, prefix, pattern, statements, etc.)
  checker/     Bidirectional type checker — mod, stmts, synth, synth_helpers, exhaust, capture, types
  interpreter/ Tree-walking evaluator — mod + split files (agents, apply, eval, modules, patterns, etc.)
  builtins/    Built-in functions — mod, call, str, coll, hof, convert, register, etc.
  visitor/     AST visitor/walker infrastructure
  stdlib/      35 registered Rust modules + 5 standard agents across ~103 .rs files (use `std_module_exists` in mod.rs as source of truth)
  token.rs, value.rs, value_display.rs, value_impls.rs, ast_display.rs, env.rs, error.rs, span.rs, lib.rs
crates/lx-cli/src/  main.rs, manifest.rs, testing.rs, listing.rs, run.rs, agent_cmd.rs, init.rs, install.rs, install_ops.rs, lockfile.rs, check.rs
doc/           35 quick-reference docs
spec/          51 spec files
agent/         Context files (this folder)
pkg/           42 lx packages in 7 clusters:
  core/        circuit, collection, connector, contracts, introspect, pool, prompt, score
  connectors/  mcp (McpConnector), cli (CliConnector), catalog (connector instances)
  ai/          ai_agent, agent_factory, perception, planner, quality, reasoning, reflect, reviewer, router
  data/        context, knowledge, memory, tasks, tieredmem, trace, transcript
  agents/      catalog, dialogue, dispatch, guard, monitor, react
  infra/       guidance, mcp_session (deprecated), report, testkit, workflow
  kit/         context_manager, grading, investigate, security_scan, tool_executor
tests/         98 test suites
  fixtures/    Test helpers
brain/
  lib/         4 brain-specific files (cognitive_saga, context_mgr, identity, tools)
  agents/      6 brain specialist agents
flows/
  agents/      19 spawnable agent scripts (14 collapsed to factory pattern via agent_catalog)
  examples/    14 .lx programs
  lib/         4 files (specialists.lx, github.lx, training.lx, agent_catalog.lx)
  tests/       Flow satisfaction test suites
  prompts/     3 prompt template files
```

## Adding a Stdlib Module

1. Create `crates/lx/src/stdlib/mymod.rs` with `pub fn build() -> IndexMap<String, Value>` returning functions via `mk("mymod.fn_name", arity, bi_fn)`
2. Register in `crates/lx/src/stdlib/mod.rs`: add `mod mymod;`, add `"mymod" => mymod::build()` in `get_std_module`, add `| "mymod"` in `std_module_exists`
3. Write test in `tests/NN_mymod.lx`
4. Sync builtins calling lx functions use `crate::builtins::call_value_sync(f, arg, span, ctx)` (blocking bridge). Async builtins (HOFs) use `crate::builtins::call_value(f, arg, span, ctx).await`. See `builtins/hof.rs` for async pattern (`mk_async` + `BoxFuture`), `builtins/call.rs` for implementation. **Exception**: background `std::thread::spawn` (e.g., cron) must use `ctx.tokio_runtime.block_on(call_value(...))` — not `call_value_sync` (no tokio Handle on bare threads)

## Adding Agent Extensions

Extensions to `std/agent` follow the split-file pattern:
1. Create `crates/lx/src/stdlib/agent_feature.rs` with `pub fn mk_feature() -> Value` returning the builtin
2. Register `mod agent_feature;` in `stdlib/mod.rs`
3. Insert into agent module map in `agent.rs`'s `build()`: `m.insert("feature".into(), super::agent_feature::mk_feature())`
4. For `BuiltinFunc` values with pre-applied args: use `kind: BuiltinKind::Sync(fn_ptr)` (not `func:`), set `arity` = total args (pre-applied + user-supplied)
5. Traits exposed as uppercase keys (e.g., `"Handoff"`) require selective import: `use std/agent {Handoff}`

## Class/Agent Implementation

Class and Agent share the same runtime representation: `Value::Class { name, traits, defaults, methods }`. No `ClassKind` enum. Agent is a Trait defined in `pkg/agent.lx` — the `Agent` keyword auto-imports it and auto-adds "Agent" to the traits list. `Class Worker : [Agent] = { ... }` also works.
- Token: `ClassKw`/`AgentKw` in `token.rs`
- Lexer: `"Class"`/`"Agent"` in `lexer/keywords.rs`
- AST: `Stmt::ClassDecl`/`Stmt::AgentDecl` + `ClassField` struct in `ast/mod.rs` + `ast/types.rs`
- Parser: `parser/stmt_class.rs` (Class), `parser/stmt_agent.rs` (Agent) — fields (`:`) vs methods (`=`)
- Value: `Value::Class { name, traits, defaults, methods }` + `Value::Object` (instance with u64 STORES-backed handle) + `Value::Store { id }` (first-class k/v store) in `value.rs`
- Store backing: STORES DashMap in `stdlib/store.rs` + `store_dispatch.rs` (dot-access method dispatch). No separate OBJECTS DashMap — Object fields live in STORES.
- Interpreter: `exec_stmt.rs` (ClassDecl/AgentDecl eval + Object FieldUpdate), `apply.rs` (Class/Agent constructor with Store cloning), `apply_helpers.rs` (Object/Store field access with `inject_self`)
- Trait injection: `interpreter/traits.rs` — `inject_traits` helper shared between Class and Agent. Defaults from `Value::Trait` (including the Agent Trait from `pkg/agent.lx`) injected at definition time. Agent Trait provides: init, perceive, reason, act, reflect, handle, run, think/think_with/think_structured, use_tool/tools, describe, ask/tell
- Agent Trait dispatch: `handle` auto-dispatches by `msg.action` via `method_of` builtin. `describe` uses `methods_of` builtin for self-description
- Trait: `Value::Trait` with non-empty `fields` acts as Trait (callable as constructor, runtime validation). No separate `Value::Trait`.
- Display: checks traits list for "Agent" → `<Agent X>` if present, `<Class X>` otherwise. `<Trait X>` for Traits-with-fields, `<Trait X>` for behavioral Traits

## Adding Language-Level Features (keywords, AST nodes)

For new keywords like `Agent`, `Trait`, `Trait`, `Class`, `with ... as`:
1. **Token**: add variant to `token.rs`'s `TokenKind` enum
2. **Lexer**: add keyword recognition in `lexer/mod.rs` (lowercase → keyword table at ~line 330; uppercase → TypeName special-case at ~line 345)
3. **AST**: add node to `ast.rs`'s `Expr` or `Stmt` enum
4. **Parser**: handle in `parser/prefix.rs` (expressions) or `parser/statements.rs` (declarations) + add to `parse_stmt` dispatch in `parser/mod.rs`
5. **Interpreter**: add eval case in `interpreter/mod.rs` (or `eval.rs` / `agents.rs` for method impls)
6. **Checker**: add synth case in `checker/synth.rs` and stmt case in `checker/mod.rs`
7. **Diag walker**: add walk case in `stdlib/diag_walk.rs`
8. **Module exports**: add export case in `interpreter/modules.rs`
9. **Value** (if runtime representation needed): add variant to `value.rs`, update `structural_eq`, `hash_value`, `value_display.rs`

## Module Resolution

`interpreter/modules.rs` handles all `use` statements. Resolution order in `eval_use`:

1. **Stdlib** — `std_module_exists(&path)` checks if it's a built-in module
2. **Workspace member** — `resolve_workspace_module(&path)` checks if `path[0]` matches a workspace member name (requires `path.len() >= 2`). Resolves rest of path from member's root dir. Member map lives on `RuntimeCtx.workspace_members` (populated by CLI).
3. **Relative** — `resolve_module_path(source_dir, &path)` handles `./` and `../` prefixes

Key functions: `eval_use` (dispatch), `load_module` (parse + execute + cache), `collect_exports` (extract `+` bindings). Module cache keyed by canonical path prevents double-loading. `loading` set detects circular imports.

## Modifying the CLI

CLI lives in `crates/lx-cli/src/`. `main.rs` has the clap `Command` enum and dispatch.

1. **Add subcommand**: add variant to `Command` enum in `main.rs`, add match arm in `main()`
2. **Add flag to existing command**: add `#[arg]` field to the variant struct
3. **Workspace-aware commands**: use `manifest::find_workspace_root` + `manifest::load_workspace` to discover members. For member filtering: accept `-m`/`--member` flag, filter `ws.members` by name.
4. **Populate RuntimeCtx for workspace imports**: call `manifest::try_load_workspace_members()` and set `ctx.workspace_members` before running any lx code. Without this, `use member/path` won't resolve.

## Error Messages

When adding errors, follow these rules:

- Show actual value and type: `format!("expected Bool, got {} `{}`", val.type_name(), val.short_display())`
- Use `val.short_display()` (80 char cap), never raw `{val}` in errors
- Undefined variable hints: `keyword_hint()` in `interpreter/mod.rs` maps 30+ cross-language keywords to lx equivalents
- Binding pattern hints: `binding_pattern_hint()` detects `mut`/`let`/`var` and suggests `:=`
- Pattern display: `Pattern` impl Display in `ast.rs` for readable error output

## std/diag Architecture

Four files: `diag.rs` (API + mermaid render), `diag_walk.rs` (walker, pre-registration),
`diag_walk_expr.rs` (expression handler with uncurry/classify/handle), `diag_helpers.rs`
(pure helpers). Utility modules (prompt, json, math, etc.) excluded from diagrams to reduce
noise. Pre-registration pass solves forward references. Resource args scanned for tracked
variables across all curried positions.

## Running Flows

`flows/examples/*.lx` are lx translations of real agentic architectures from `~/repos/mcp-toolbelt/packages/arch_diagrams/`. Each has a matching spec in `flows/specs/`. `flows/lib/*.lx` are reusable library modules imported by the examples. Run with `just run flows/examples/research.lx`. Most require actual agent subprocesses or MCP servers — they're structural demonstrations, not standalone tests.

## Flow → Module Mapping

| Flow (examples/)    | Uses                                                                             |
| ------------------- | -------------------------------------------------------------------------------- |
| agentic_loop        | std/ai, pkg/circuit, pkg/tasks, std/agents/auditor                               |
| agent_lifecycle     | std/ai, pkg/memory, std/agents/reviewer, std/cron                                |
| fine_tuning         | std/ai, pkg/trace, MCP Embeddings                                                |
| full_pipeline       | std/ai, pkg/tasks, std/agents/grader, std/agents/planner, std/agents/monitor     |
| security_audit      | std/agents/monitor, pkg/circuit                                                  |
| research            | std/ai, std/agents/router, pkg/tasks                                             |
| perf_analysis       | std/ai, std/agents/router, pkg/tasks                                             |
| project_setup       | pkg/tasks, MCP Workflow                                                          |
| post_hoc_review     | std/ai, std/agents/reviewer, pkg/memory, pkg/trace                               |
| discovery_system    | std/ai, pkg/tasks, pkg/trace, MCP Embeddings                                     |
| tool_generation     | std/ai, pkg/tasks, std/agents/auditor                                            |
| defense_layers      | std/agents/monitor, pkg/circuit, pkg/trace, capability attenuation               |
| mcp_tool_audit      | pkg/tasks, std/audit                                                             |
| software_diffusion  | std/ai, pkg/tasks, std/agents/planner                                            |
| (any flow)          | std/diag (visualize any flow's structure)                                    |

| Library (lib/)      | Purpose                                                                      |
| ------------------- | ---------------------------------------------------------------------------- |
| github              | GitHub API: search_repos, search_axes, scale_stars (moved from pkg/infra/)   |
| specialists         | Specialist agent catalog + keyword map                                       |
| training            | Training data pipeline: harvest, enhance, write_jsonl (moved from pkg/data/) |


# Stdlib
-- Memory: ISA manual (stdlib). Standard library modules and built-in functions.
-- Update when stdlib modules are added or changed. See also LANGUAGE.md and AGENTS.md.

# lx Standard Library

**Note:** 11 packages in `pkg/`. Import the Class/Trait name:
`use pkg/agent {Agent}`, `use pkg/collection {Collection}`, `use pkg/knowledge {KnowledgeBase}`,
`use pkg/tasks {TaskStore}`, `use pkg/trace {TraceStore}`, `use pkg/memory {MemoryStore}`,
`use pkg/context {ContextWindow}`, `use pkg/circuit {CircuitBreaker}`, `use pkg/introspect {Inspector}`,
`use pkg/pool {Pool}`.
`pkg/prompt` remains a pure record builder. 5 collection packages (knowledge, tasks, memory, trace, context)
use `entries: Store ()` + Collection Trait for generic operations. Construct with `ClassName {field: val}`
or `ClassName ()`. Methods via `instance.method args`.

## AI (std/ai)

```lx
use std/ai
resp = ai.prompt "Summarize this code" ^
resp = ai.prompt_with {
  prompt: "Analyze..."
  append_system: "You are a code reviewer."
  tools: ["Read" "Grep" "Bash"]
  max_turns: 10
} ^
resp.text                    -- the response text

result = ai.prompt_structured ScoreProtocol "Rate this" ^
result = ai.prompt_json "Classify this intent" {intent: "" findings: [""]} ^
```

## Store (first-class Value)

`Store` is a first-class value type (`Value::Store { id }`) with dot-access methods:

```lx
s = Store ()
s.set "key" value              s.get "key"
s.keys ()                      s.values ()
s.entries ()                   s.has "key"
s.len ()                       s.remove "key"
s.clear ()                     s.update "key" (v) v + 1
s.filter (k v) condition       s.query {field: "value"}
s.map (k v) transform          s.merge other_store_or_record
s.save "path.json"             s.load "path.json"
s.persist "path.json"          s.reload "path.json"
```

Reference semantics: `a = b` shares the same Store. Store cloning in Class constructors ensures each instance gets its own copy.

## Agent Trait (pkg/agent)

`Trait Agent` — base behavioral contract for all agents. `Agent` keyword auto-imports `pkg/agent {Agent}` and auto-adds "Agent" to traits list. `Class Worker : [Agent] = { ... }` also works. Defaults:

- **Cognitive pipeline:** `init`, `perceive`, `reason`, `act`, `reflect` — override the phases you need
- **Dispatch:** `handle(msg)` — auto-dispatches by `msg.action` via `method_of`, falls back to perceive→reason→act→reflect
- **Message loop:** `run()` — init, then yield/loop: yield ready, handle msg, yield result
- **AI:** `think(prompt)`, `think_with(config)`, `think_structured(schema, prompt)`
- **Tools:** `use_tool(name, input)`, `tools()` — override to wire tool execution
- **Communication:** `ask(agent, msg)` wraps `~>?`, `tell(agent, msg)` wraps `~>`
- **Self-description:** `describe()` — returns `{name, actions: methods_of self, tools}` via `methods_of` builtin

## Collection Trait (pkg/collection)

Generic operations for any Class with `entries: Store ()`. Provides 9 default methods: `get`, `keys`, `values`, `remove`, `query`, `len`, `has`, `save`, `load` — all delegating to `self.entries`. Any conforming Class gets these for free; domain-only methods remain on the Class. Used by: KnowledgeBase, TaskStore, TraceStore, MemoryStore, ContextWindow.

## Prompt Assembly (pkg/prompt)

```lx
use pkg/prompt
p = prompt.create ()
  | prompt.system "You are a code auditor"
  | prompt.section "Checklist" audit_text
  | prompt.instruction "Produce a findings report"
  | prompt.constraint "Only report problems"
  | prompt.example "Example finding: ..."
rendered = prompt.render p
```

## Tracing (pkg/trace)

```lx
use pkg/trace {TraceStore}
t = TraceStore ()
t.record {name: "step1" input: x output: y agent: "researcher"} ^
sum = t.summary ()
rate = t.improvement_rate 3
stop = t.should_stop {min_delta: 2.0 window: 3}
by_agent = t.query {agent: "researcher"}
```

Provenance (message flow tracking):
```lx
t.enable_provenance () ^
t.record_hop {msg_id: "req-1" from: "user" to: "researcher" action: "search"} ^
t.record_hop {msg_id: "req-1" from: "researcher" to: "analyst"} ^
path = t.message_path "req-1"
hops = t.message_hops "req-1"
```

Reputation (agent scoring from trace data):
```lx
score = t.agent_score "researcher"
ranking = t.agent_rank ()
```

## Grading and Auditing

```lx
use std/audit
use std/agents/grader
use std/agents/auditor

rubric = audit.rubric [
  {name: "coverage" description: "covers all items" weight: 50}
  {name: "quality" description: "clear and actionable" weight: 50}
]

grade = grader.grade {work: draft  task: "review doc"  rubric: rubric  threshold: 75}
check = audit.quick_check {output: text  task: "documentation"}
full = auditor.audit {output: text  task: "documentation"}
```

## Deadline (Time Propagation)

```lx
use std/deadline

dl = deadline.create 5000 ^
body = () {
  remaining = deadline.remaining () ^
  expired = deadline.expired () ^
  deadline.check () ^
  sub = deadline.slice 0.3 ^
  remaining
}
result = deadline.scope dl body ^

dl2 = deadline.create_at (time.now().ms + 10000) ^
deadline.extend dl2 5000 ^
```

`deadline.scope` establishes a deadline context. `remaining`, `expired`, `check`, and `slice` read the current scope (thread-local stack). `scope` returns `Result Any Str`. When `~>?`/`~>` is called inside a scope, `_deadline_ms` is auto-injected into Record messages.

## Budget (Cost Tracking)

```lx
use std/budget
b = budget.create {total: 10.0 unit: "dollars"}
b = budget.spend 2.5 b ^
budget.remaining b    -- 7.5
budget.used_pct b     -- 25.0
sub = budget.slice 0.3 b ^  -- sub-budget (30% of remaining)
```

## Retry with Backoff

```lx
use std/retry
result = retry.retry flaky_fn
result = retry.retry_with {max_attempts: 5  base_delay_ms: 200} flaky_fn
```

## Durable Workflows (std/durable)

```lx
use std/durable
wf = durable.workflow "my-flow" {storage_dir: ".lx/durable/"} (ctx) {
  a = durable.step ctx "fetch" () { http.get url ^ }
  b = durable.step ctx "transform" () { process a }
  b
}
result = durable.run wf {url: "..."} ^
durable.status result.workflow_id
durable.list ()
```

8 functions: `workflow`, `run`, `step` (idempotent — cached on replay), `sleep`, `signal`, `send_signal`, `status`, `list`. File-backed at `<storage_dir>/<name>/<run-id>/`.

## Pipeline (std/pipeline)

```lx
use std/pipeline

pipe = pipeline.create "my-pipeline" {storage: ".lx/pipelines/"} ^

result1 = pipeline.stage pipe "step1" input_data (input) {
  process input ^
} ^

result2 = pipeline.stage pipe "step2" result1 (input) {
  transform input ^
} ^

pipeline.complete pipe ^

st = pipeline.status pipe
pipeline.invalidate pipe "step1"
pipeline.clean pipe
all = pipeline.list ()
```

`pipeline.stage` caches completed stage outputs. On re-run with the same input, cached results are returned without re-executing the body. If input changes (hash mismatch), the stage re-executes. `invalidate`/`invalidate_from` remove a stage's cache plus all downstream stages.

## Other Stdlib Modules

| Module            | Purpose                                                              |
|-------------------|----------------------------------------------------------------------|
| `std/json`        | `parse`, `encode`, `encode_pretty`                                   |
| `std/fs`          | `read`, `write`, `append`, `exists`, `stat`, `mkdir`, `ls`, `remove` |
| `std/env`         | `get`, `vars`, `args`, `cwd`, `home`                                 |
| `std/http`        | `get`, `post`, `put`, `delete`                                       |
| `std/re`          | `is_match`, `match`, `find_all`, `replace`, `split`                  |
| `std/md`          | `parse`, `sections`, `code_blocks`, `headings`, `render`, builders   |
| `std/math`        | `abs`, `ceil`, `floor`, `round`, `pow`, `sqrt`, `min`, `max`         |
| `std/time`        | `now`, `sleep`, `format`, `parse`                                    |
| `std/git`         | 36 functions: status, log, diff, blame, grep, commit, branch, etc.   |
| `std/ctx`         | **DEPRECATED** — use `Store()` with dot methods instead               |
| `std/deadline`    | Time budgets: `create`, `create_at`, `scope`, `remaining`, `expired`, `check`, `slice`, `extend` |
| `std/pipeline`    | Stage caching: `create`, `stage`, `complete`, `status`, `invalidate`, `clean`, `list` |
| `std/plan`        | Plan execution: `run` with `on_step` callback, `replan`, `skip`      |
| `std/saga`        | Compensating transactions: `run`, `define`, `execute`                |
| `std/cron`        | Scheduling: `every`, `after`, `at`, `schedule`, `run`                |
| `std/user`        | Interactive: `confirm`, `choose`, `ask`, `progress`, `table`         |
| `std/profile`     | Persistent identity: `load`, `save`, `learn`, `recall`, `preference` |
| `std/diag`        | Visualization: `extract`, `to_mermaid`                               |
| `std/introspect`  | Live observation: `system`, `agents`, `agent`, `messages`, `bottleneck` |
| `std/flow`        | Flow composition: `load`, `run`, `pipe`, `parallel`, `branch`, `with_retry`, `with_timeout`, `with_fallback` |
| `std/taskgraph`   | DAG execution: `create`, `add`, `remove`, `run`, `run_with`, `validate`, `topo`, `status`, `dot` |
| `std/workspace`   | Collaborative editing: `create`, `claim`, `claim_pattern`, `edit`, `append`, `release`, `snapshot`, `regions`, `conflicts`, `resolve`, `history`, `watch`. Line-based region claiming with overlap detection, auto-bound adjustment, regex pattern claiming, watchers. DashMap-backed for `par`/`pmap` safety |
| `std/registry`    | Cross-process discovery: `start`, `stop`, `connect`, `register`, `deregister`, `find`, `find_one`, `health`, `load`, `watch`. In-memory registry with trait/trait/domain filtering, selection strategies (first, least_loaded, round_robin, random), health/load tracking, watcher callbacks |
| `std/yield`       | Typed yield Traits: `YieldApproval`, `YieldReflection`, `YieldInformation`, `YieldDelegation`, `YieldProgress`. Trait-only module (no functions). Auto-injected `kind` field for orchestrator dispatch |

## Flow Composition (std/flow)

```lx
use std/flow

f = flow.load "review.lx" ^
result = flow.run f {task: "review"} ^

pipeline = flow.pipe [
  flow.load "extract.lx" ^
  flow.load "transform.lx" ^
]
result = flow.run pipeline input ^

ensemble = flow.parallel [
  flow.load "reviewer1.lx" ^
  flow.load "reviewer2.lx" ^
]
results = flow.run ensemble input ^

resilient = flow.load "flaky.lx" ^
  | flow.with_timeout 300
  | flow.with_retry {max: 3}
  | flow.with_fallback (flow.load "safe.lx" ^)
result = flow.run resilient input ^
```

`flow.load` reads and parses a .lx file, returning a Flow record. `flow.run` executes in an isolated interpreter with shared RuntimeCtx. Flows must export `+run` or `+main`. `flow.branch` takes a router function that receives input and returns a Flow.

## Task Graphs (std/taskgraph)

```lx
use std/taskgraph

g = taskgraph.create "code-review" ^
taskgraph.add g "parse" {handler: parse_fn  input: {files: changed}} ^
taskgraph.add g "lint" {handler: lint_fn  input: {files: changed}} ^
taskgraph.add g "review" {
  depends: ["parse" "lint"]
  input_from: (results) {ast: results.parse.ast  warnings: results.lint.issues}
  handler: review_fn
} ^
results = taskgraph.run g ^
```

Task options: `handler` (function), `input` (static), `depends` (task ID list), `input_from` (transform dep results), `timeout` (ms), `retry` (count), `on_fail` ("fail"|"skip"). `taskgraph.validate` checks cycles + unknown deps. `taskgraph.topo` returns topological order. `taskgraph.dot` exports DOT graph. `taskgraph.run_with` adds `on_complete`/`on_fail` callbacks and `max_parallel`.

## Standard Agents

Two Rust-backed agents under `std/agents/` (require internal APIs):

| Module                | Functions                | Use for                          |
|-----------------------|--------------------------|----------------------------------|
| `std/agents/auditor`  | `quick_audit`, `audit`   | Output quality checking          |
| `std/agents/grader`   | `quick_grade`, `grade`   | Rubric-based scoring             |

Three deprecated agents (use pkg/ai/ equivalents instead):
- `std/agents/planner` → use `pkg/ai/planner`
- `std/agents/router` → use `pkg/ai/router`
- `std/agents/reviewer` → use `pkg/ai/reviewer`

Removed: `std/agents/monitor` — use `pkg/agents/guard` instead.

## Built-in Functions (No Import Needed)

**Transform:** `map`, `flat_map`, `scan`, `fold`, `sum`, `product`
**Filter:** `filter`, `take`, `drop`, `take_while`, `drop_while`
**Search:** `find`, `find_index`, `first`, `last`, `get`
**Predicates:** `any?`, `all?`, `none?`, `count`, `empty?`, `contains?`, `has_key?`, `sorted?`
**Sort:** `sort`, `sort_by`, `rev`, `min`, `max`, `min_by`, `max_by`
**Reshape:** `chunks`, `windows`, `partition`, `group_by`, `zip`, `enumerate`
**Flatten:** `flatten`, `intersperse`, `uniq`
**Convert:** `to_list`, `to_map`, `to_record`, `keys`, `values`, `entries`, `merge`, `remove`
**String:** `len`, `chars`, `lines`, `split`, `join`, `trim`, `upper`, `lower`, `replace`, `starts?`, `ends?`, `pad_left`, `pad_right`, `repeat`
**Numeric:** `even?`, `odd?`, `parse_int`, `parse_float`, `to_int`, `to_float`
**Type:** `type_of`, `to_str`, `ok?`, `err?`, `some?`
**Effects:** `each` (returns unit), `tap` (returns original), `dbg` (debug print), `print` (stdout)
**Control:** `identity`, `not`, `require`, `timeout`, `step`, `collect`
**Reflection:** `method_of(obj, name)` — returns a method/field by name or None; `methods_of(obj)` — returns list of method names from Class/Object/Record
**Logging:** `log.info`, `log.warn`, `log.err`, `log.debug`
**Streaming:** `collect` materializes Stream→List; HOFs (`map`, `filter`, `each`, `take`, `fold`, `flat_map`) work on streams transparently
**Ambient context:** `context.current ()` — returns ambient context record (empty `{}` outside any scope); `context.get "key"` — returns `Some val` or `None`. Field access: `context.field` inside `with context`. See AGENTS.md for `with context` syntax

## Idioms

**Pipe-first design:**
```lx
audit_items
  | map (item) investigate item root ^
  | filter (.severity == "critical")
  | sort_by (.confidence) | rev
  | take 10
```

**Sections over lambdas:** `items | map (.name) | filter (starts? "test") | sort`

**`^` early in pipes:** `url | fetch ^ | (.body) | json.parse ^ | (.data) | map process`

**Prompt composition:** Use `pkg/prompt` builder, not string concatenation.

**Scoped resources:** Always `with ... as` for connections needing cleanup.

**Fan-out + reconcile:** `results = par { a ~>? msg ^; b ~>? msg ^ }` then `agent.reconcile results {strategy: "vote"}`


Work Item:
# Goal

Add `std/sandbox` — capability-based sandboxing for agent spawns, shell commands, and scoped execution. Deny-by-default policies restrict what code can do at the lx runtime level (RuntimeCtx backend restriction) and optionally at the OS level (Landlock + seccomp). Full spec: `spec/stdlib-sandbox.md`.

# Why

- Every serious agentic tool sandboxes execution: Codex CLI (Landlock+seccomp), Cursor (container), Devin (VM). lx has `agent.eval_sandbox` for one narrow case but nothing for process-level restriction or scoped capability attenuation.
- LLM-generated workflows with `agent.spawn` hand child processes full system access. Tool-generating agents calling `$` can run arbitrary shell commands. No guardrails.
- The existing RuntimeCtx backend architecture already supports swapping backends — sandbox leverages this directly by wrapping existing backends with deny/restrict variants.

# What Changes

**New file `crates/lx/src/backends/restricted.rs` — Deny and Restricted backend wrappers:**

`DenyShellBackend` — returns `Err("shell access denied by sandbox policy")` for both `exec` and `exec_capture`.
`DenyHttpBackend` — returns `Err("network access denied by sandbox policy")` for `request`.
`DenyAiBackend` — returns `Err("AI access denied by sandbox policy")` for `prompt`.
`DenyPaneBackend` — returns `Err("pane access denied by sandbox policy")` for all methods.
`DenyEmbedBackend` — returns `Err("embedding access denied by sandbox policy")` for `embed`.
`RestrictedShellBackend` — wraps inner `ShellBackend`, checks command against allowlist before delegating.

**New file `crates/lx/src/stdlib/sandbox.rs` — module entry, policy creation, introspection:**

Policy is a Record stored in a global DashMap. Preset policies (`:pure`, `:readonly`, `:local`, `:network`, `:full`). Custom policies from config Records. `sandbox.policy`, `sandbox.describe`, `sandbox.permits`, `sandbox.merge`, `sandbox.attenuate`.

**New file `crates/lx/src/stdlib/sandbox_scope.rs` — scope enforcement:**

`sandbox.scope` pushes a policy onto a thread-local stack, creates a child RuntimeCtx with restricted backends swapped in, evaluates the body, pops the policy on exit. Nested scopes intersect (inner can only narrow, never widen).

**New file `crates/lx/src/stdlib/sandbox_exec.rs` — sandboxed shell and spawn:**

`sandbox.exec` runs a shell command under policy restrictions (Layer 1 only — lx-level restriction). `sandbox.spawn` wraps `agent.spawn` with restricted backends. OS-level enforcement (Landlock+seccomp) deferred to a follow-up — Layer 1 (RuntimeCtx restriction) covers the critical path.

# Files Affected

- `crates/lx/src/backends/restricted.rs` — New file: Deny* and Restricted* backends
- `crates/lx/src/backends/mod.rs` — Add `mod restricted; pub use restricted::*;`
- `crates/lx/src/stdlib/sandbox.rs` — New file: module entry, policy, introspection
- `crates/lx/src/stdlib/sandbox_scope.rs` — New file: scope enforcement
- `crates/lx/src/stdlib/sandbox_exec.rs` — New file: sandboxed exec/spawn
- `crates/lx/src/stdlib/mod.rs` — Register module
- `tests/102_sandbox.lx` — New test file

# Task List

### Task 1: Create restricted backend wrappers

**Subject:** Create Deny and Restricted backend wrappers in backends/restricted.rs

**Description:** Create `crates/lx/src/backends/restricted.rs`.

Imports: `std::sync::Arc`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`, `super::*`.

Implement 5 deny backends — each returns a descriptive `Value::Err`:

`pub struct DenyShellBackend;` — impl `ShellBackend`, both `exec` and `exec_capture` return `Ok(Value::Err(Box::new(Value::Str(Arc::from("shell access denied by sandbox policy")))))`.

`pub struct DenyHttpBackend;` — impl `HttpBackend`, `request` returns network denied error.

`pub struct DenyAiBackend;` — impl `AiBackend`, `prompt` returns AI denied error.

`pub struct DenyPaneBackend;` — impl `PaneBackend`, all 4 methods return pane denied error (open returns Err, update/close return LxError, list returns Err).

`pub struct DenyEmbedBackend;` — impl `EmbedBackend`, `embed` returns embedding denied error.

Implement 1 restricted backend:

`pub struct RestrictedShellBackend { pub inner: Arc<dyn ShellBackend>, pub allowed_cmds: Vec<String> }`. Impl `ShellBackend`: for `exec` and `exec_capture`, extract the first word of the command string (split on whitespace, take first). If it's in `allowed_cmds`, delegate to `self.inner.exec(cmd, span)`. Otherwise return `Ok(Value::Err(Box::new(Value::Str(Arc::from(format!("command '{}' not allowed by sandbox policy", first_word))))))`.

Add `mod restricted;` and `pub use restricted::*;` to `crates/lx/src/backends/mod.rs`.

**ActiveForm:** Creating restricted backend wrappers

---

### Task 2: Create sandbox.rs with policy creation and introspection

**Subject:** Create sandbox.rs with policy data structures, presets, and introspection functions

**Description:** Create `crates/lx/src/stdlib/sandbox.rs`.

Imports: `std::sync::{Arc, LazyLock, atomic::{AtomicU64, Ordering}}`, `dashmap::DashMap`, `indexmap::IndexMap`, `num_bigint::BigInt`, `crate::backends::RuntimeCtx`, `crate::builtins::mk`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`.

Define `pub(super) struct Policy`:
- `fs_read: Vec<String>` — allowed read paths
- `fs_write: Vec<String>` — allowed write paths
- `net_allow: Vec<String>` — allowed network destinations
- `shell: ShellPolicy` — enum `Deny | Allow | AllowList(Vec<String>)`
- `agent: bool`
- `mcp: bool`
- `ai: bool`
- `embed: bool`
- `pane: bool`
- `max_time_ms: u64` — 0 = unlimited

Static: `pub(super) static POLICIES: LazyLock<DashMap<u64, Policy>> = ...;`, `static NEXT_ID: ...;`.

`pub(super) fn policy_id(v: &Value, span: Span) -> Result<u64, LxError>`: extract `__policy_id` from Record.

`fn make_preset(name: &str) -> Policy`: match on name:
- `"pure"` → all deny, no fs, no net, no shell, no agent/mcp/ai/embed/pane
- `"readonly"` → fs_read: `["."]`, rest deny
- `"local"` → fs_read/write: `["."]`, shell: Allow, rest deny
- `"network"` → fs_read/write: `["."]`, net_allow: `["*"]`, ai: true, rest deny
- `"full"` → everything allowed

`fn parse_policy(config: &Value, span: Span) -> Result<Policy, LxError>`: extract fields from a Record config. `fs.read` and `fs.write` as lists of strings. `net.allow` as list. `shell` as bool or Record with `allow` list. `agent`, `mcp`, `ai`, `embed`, `pane` as bools. `max_time_ms` as Int.

`pub fn build() -> IndexMap<String, Value>`: register:
- `"policy"` → `bi_policy` arity 1
- `"describe"` → `bi_describe` arity 1
- `"permits"` → `bi_permits` arity 3
- `"merge"` → `bi_merge` arity 1
- `"attenuate"` → `bi_attenuate` arity 2
- `"scope"` → `super::sandbox_scope::bi_scope` arity 2
- `"exec"` → `super::sandbox_exec::bi_exec` arity 2
- `"spawn"` → `super::sandbox_exec::bi_spawn` arity 2

`bi_policy`: args[0] is either a Symbol (`:pure`, etc.) or a config Record. For Symbol, call `make_preset`. For Record, call `parse_policy`. Store in POLICIES, return handle Record.

`bi_describe`: args[0] is policy handle. Look up policy. Return a descriptive Record with `fs_read`, `fs_write`, `net`, `shell`, `agent`, `mcp`, `ai` fields.

`bi_permits`: args[0] is policy handle, args[1] is capability Symbol (`:fs_read`, `:fs_write`, `:shell`, `:net`, `:ai`, `:agent`, `:mcp`), args[2] is target Str. Check the policy for the given capability against the target. Return `Value::Bool`.

`bi_merge`: args[0] is List of policy handles. Intersection: for each field, take the most restrictive value.

`bi_attenuate`: args[0] is parent policy handle, args[1] is overrides Record. Parse overrides, intersect with parent. Error if overrides try to widen (grant capability parent doesn't have).

**ActiveForm:** Creating sandbox.rs with policy and introspection

---

### Task 3: Create sandbox_scope.rs with scope enforcement

**Subject:** Create sandbox_scope.rs with scoped RuntimeCtx restriction

**Description:** Create `crates/lx/src/stdlib/sandbox_scope.rs`.

Imports: `std::sync::Arc`, `std::cell::RefCell`, `crate::backends::*`, `crate::builtins::call_value_sync`, `crate::error::LxError`, `crate::span::Span`, `crate::value::Value`, `super::sandbox::{POLICIES, Policy, policy_id}`.

Thread-local policy stack: `thread_local! { static POLICY_STACK: RefCell<Vec<u64>> = RefCell::new(Vec::new()); }`.

`pub(super) fn current_policy_id() -> Option<u64>`: peek the stack.

`fn build_restricted_ctx(base: &Arc<RuntimeCtx>, policy: &Policy) -> Arc<RuntimeCtx>`: Create a new RuntimeCtx copying all fields from base, then swap backends based on policy:
- If `!policy.ai` → replace `ai` with `Arc::new(DenyAiBackend)`
- If `!policy.pane` → replace `pane` with `Arc::new(DenyPaneBackend)`
- If `!policy.embed` → replace `embed` with `Arc::new(DenyEmbedBackend)`
- If `policy.net_allow.is_empty()` → replace `http` with `Arc::new(DenyHttpBackend)`
- Match `policy.shell`:
  - `ShellPolicy::Deny` → replace `shell` with `Arc::new(DenyShellBackend)`
  - `ShellPolicy::AllowList(cmds)` → replace `shell` with `Arc::new(RestrictedShellBackend { inner: base.shell.clone(), allowed_cmds: cmds.clone() })`
  - `ShellPolicy::Allow` → keep base shell

Construct the new RuntimeCtx with all fields set. Wrap in `Arc::new`.

`pub fn bi_scope(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is policy handle, args[1] is body function. Get policy_id. Look up policy in POLICIES. Build restricted ctx. Push policy_id onto POLICY_STACK. Call `call_value_sync(&args[1], Value::Unit, span, &restricted_ctx)`. Pop from POLICY_STACK. Return the result (propagate errors).

**ActiveForm:** Creating sandbox_scope.rs with scope enforcement

---

### Task 4: Create sandbox_exec.rs with sandboxed exec and spawn

**Subject:** Create sandbox_exec.rs with sandboxed shell execution and agent spawn

**Description:** Create `crates/lx/src/stdlib/sandbox_exec.rs`.

Imports: `std::sync::Arc`, `crate::backends::*`, `crate::error::LxError`, `crate::span::Span`, `crate::value::Value`, `super::sandbox::{POLICIES, policy_id}`.

`pub fn bi_exec(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is policy handle, args[1] is command string. Get policy. Check `policy.shell`:
- `ShellPolicy::Deny` → return `Ok(Value::Err(...))`
- `ShellPolicy::AllowList(cmds)` → extract first word of command, check against list
- `ShellPolicy::Allow` → proceed

If allowed, delegate to `ctx.shell.exec(cmd, span)`.

`pub fn bi_spawn(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is policy handle, args[1] is spawn config Record (same format as agent.spawn). Get policy. If `!policy.agent`, return `Ok(Value::Err(...))`. Otherwise, this delegates to the existing agent spawn mechanism but with a restricted RuntimeCtx. For now, return `Ok(Value::Err(Box::new(Value::Str(Arc::from("sandbox.spawn: OS-level sandboxing not yet implemented — use sandbox.scope for lx-level restriction")))))`. OS-level enforcement (Landlock+seccomp) is a follow-up.

**ActiveForm:** Creating sandbox_exec.rs with sandboxed execution

---

### Task 5: Register std/sandbox and write tests

**Subject:** Register sandbox module in mod.rs and write integration tests

**Description:** Edit `crates/lx/src/stdlib/mod.rs`:

Add `mod sandbox;`, `mod sandbox_scope;`, `mod sandbox_exec;`.

In `get_std_module`, add: `"sandbox" => sandbox::build(),`.

In `std_module_exists`, add `| "sandbox"`.

Create `tests/102_sandbox.lx`:

```
use std/sandbox

-- Preset policies
pure = sandbox.policy :pure
readonly = sandbox.policy :readonly
local = sandbox.policy :local
full = sandbox.policy :full

-- Describe
desc = sandbox.describe pure
assert (desc.ai == false) "pure denies ai"
assert (desc.shell == false) "pure denies shell"

desc_full = sandbox.describe full
assert (desc_full.ai == true) "full allows ai"

-- Permits
assert (sandbox.permits readonly :fs_read ".") "readonly permits fs_read"
assert (not (sandbox.permits readonly :fs_write ".")) "readonly denies fs_write"
assert (not (sandbox.permits pure :shell "ls")) "pure denies shell"

-- Scope: pure blocks shell
result = sandbox.scope pure () {
  $echo "should not run"
}
assert (type_of result == "Err") "pure scope blocks shell"

-- Scope: local allows shell
result2 = sandbox.scope local () {
  $^echo "hello"
}
assert (result2 | trim == "hello") "local scope allows shell"

-- Custom policy with shell allowlist
custom = sandbox.policy {
  shell: {allow: ["echo" "cat"]}
  ai: false
}
result3 = sandbox.scope custom () {
  $^echo "allowed"
}
assert (result3 | trim == "allowed") "allowlist permits echo"

-- Merge: intersection
merged = sandbox.merge [local full]
desc_merged = sandbox.describe merged
assert (desc_merged.ai == false) "merge intersects — local has no ai"

-- Attenuate: can narrow
narrow = sandbox.attenuate full {ai: false} ^
desc_narrow = sandbox.describe narrow
assert (desc_narrow.ai == false) "attenuate narrows"

log.info "102_sandbox: all passed"
```

Run `just diagnose` to verify compilation. Run `just test` to verify tests pass.

**ActiveForm:** Registering sandbox module and writing tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/STD_SANDBOX.md" })
```

Then call `next_task` to begin.

Constraints:
- Do not skip tasks, reorder tasks, or combine tasks- Do not add code comments or doc strings- No #[allow()] macros
Instructions:
- Execute every task in the Task List section sequentially. For each task, implement the code changes described, then run `just diagnose` to verify compilation. After all tasks, run `just test`.