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

## Stdlib (30 Rust modules + 6 standard agents + 11 lx packages)

- Data: `std/json`, `std/md`, `std/re`, `std/math`, `std/time`
- System: `std/fs`, `std/env`, `std/http`
- Git: `std/git` — 36 functions
- State primitive: `std/store` — backing implementation for `Value::Store`. `Store ()` constructor creates a first-class Store value with dot-access methods (set, get, keys, values, entries, remove, len, has, clear, filter, query, map, merge, update, save, load, persist, reload, to_record)
- Resilience: `std/deadline`
- Communication: `std/agent`, `std/mcp`, `std/ai`
- Observation: `std/introspect` — system-wide live introspection: `system` (full snapshot), `agents` (agent list), `agent` (deep single-agent info), `messages` (in-flight), `bottleneck` (busiest agent). Aggregates from REGISTRY, SESSIONS, SUPERVISORS, TOPICS, ROUTE_TABLE
- Scheduling: `std/cron`
- Orchestration: `std/ctx` (deprecated — use Store), `std/pipeline`, `std/taskgraph`, `std/workspace`
- Collaboration: `std/workspace` — concurrent multi-agent editing
- Discovery: `std/registry` — cross-process agent discovery
- Persistence: `std/durable` — Temporal-style workflow persistence
- Yield types: `std/yield` — 5 Trait-only exports
- Standard agents: `std/agents/auditor`, `std/agents/grader` (Rust-backed, use internal APIs). `std/agents/planner`, `std/agents/router`, `std/agents/reviewer` (deprecated — use pkg/ai/ equivalents)
- Builtins: `try` (catch propagated errors), `resolve_handler` (look up hot-reloaded agent handlers)

## lx Packages (pkg/ workspace member — 53 packages in 7 clusters)

**pkg/core/** — foundational primitives:
- `adapter` — Trait format adaptation: field mapping between source/target Traits (hoisted from std/agent)
- `agent_errors` — AgentErr union type: 11 structured error variants (hoisted from std/agent)
- `audit` — text quality checks: is_empty, is_hedging, is_refusal, references_task, evaluate, quick_check (hoisted from std/audit)
- `budget` — Budget tracking: multi-dimensional cost tracking with thresholds and sub-budgets (hoisted from std/budget)
- `capability` — Capabilities Trait + advertise/lookup for runtime capability discovery (hoisted from std/agent)
- `circuit` — CircuitBreaker Class: turn/action/time/repetition trip conditions
- `collection` — Collection Trait: get, keys, values, remove, query, len, has, save, load defaults
- `contracts` — shared Trait definitions: ToolRequest, ToolResult, ActionPlan, ActionResult, AgentTask, AgentResult, ContextItem
- `handoff` — Handoff Trait + as_context formatter (hoisted from std/agent)
- `introspect` — Inspector Class + 8 introspection functions
- `negotiate_fmt` — Trait format negotiation with structural matching (hoisted from std/agent)
- `plan` — dependency-aware step execution with revision actions (hoisted from std/plan)
- `pool` — Pool Class: worker pools with round-robin dispatch
- `prompt` — composable prompt assembly
- `reconcile` — 6 reconciliation strategies: union, intersection, vote, highest_confidence, max_score, merge_fields (hoisted from std/agent)
- `retry` — backoff computation and retry loop: exponential/linear/constant (hoisted from std/retry)
- `saga` — compensating transactions: try/undo loop with retry and dependency ordering (hoisted from std/saga)
- `connector` — Connector Trait: connect, disconnect, call, tools
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
- `catalog` — agent registry: create, by_domain, by_name, add
- `dialogue` — multi-turn conversation state
- `dialogue_persist` — dialogue save/load/list/delete via std/fs + std/json (hoisted from std/agent)
- `dispatch` — spawn/ask/kill shorthand: run_one, run_many, run_with, run_handler
- `dispatch_rules` — pattern-based message dispatch: dispatch, dispatch_multi (hoisted from std/agent)
- `guard` — security scanning
- `intercept` — composable message middleware with short-circuit (hoisted from std/agent)
- `mock` — mock agents with Store-backed call recording (hoisted from std/agent)
- `monitor` — health monitoring: circuit breaker + budget + inspector facade
- `negotiate` — N-party iterative consensus with convergence function (hoisted from std/agent)
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

## Agent Extensions (8 remaining Rust sub-modules of `std/agent`)

- `agent.dialogue` — multi-turn stateful sessions with config `{role? context? max_turns?}`. Branching: `dialogue_fork`, `dialogue_compare`, `dialogue_merge`, `dialogue_branches`
- `GateResult` Trait + `agent.gate` — human-in-the-loop approval gates via yield
- `agent.supervise` — Erlang-style supervision: one_for_one/one_for_all/rest_for_one
- `agent.topic` / `agent.subscribe` / `agent.publish` — in-process pub/sub with filtered subscriptions
- `agent.route` / `agent.register` — capability-based routing with selection strategies
- `agent.pipeline` — consumer-driven flow control with backpressure
- `agent.emit_stream` / `agent.end_stream` — agent-side streaming API for `~>>?`
- `agent.reload` / `agent.evolve` / `agent.update_traits` — hot-swap agent handlers
- `agent.on` / lifecycle hooks — 6 events (startup, shutdown, error, idle, message, signal)

**Hoisted to lx packages:** reconcile → `pkg/core/reconcile`, intercept → `pkg/agents/intercept`, Handoff → `pkg/core/handoff`, Capabilities → `pkg/core/capability`, mock → `pkg/agents/mock`, dispatch → `pkg/agents/dispatch_rules`, negotiate → `pkg/agents/negotiate`, adapter/coerce → `pkg/core/adapter`, negotiate_format → `pkg/core/negotiate_fmt`, dialogue_persist → `pkg/agents/dialogue_persist`, agent_errors → `pkg/core/agent_errors`

## Other Extensions

- `ai.prompt_structured` — Trait-validated LLM output with auto-retry
- `ai.prompt_json` — lightweight structured output from inline record shape (no Trait needed)

## Runtime

- **Async interpreter**: `eval`/`exec`/`eval_expr` are `async fn`. Interpreter runs inside `ctx.tokio_runtime.block_on()` at CLI entry. Builtin functions split: `BuiltinKind::Sync` (pure builtins — math, string, collection ops) and `BuiltinKind::Async` (HOFs that invoke callbacks — map, filter, fold, etc., return `BoxFuture`). `call_value` is async; `call_value_sync` bridge for sync stdlib functions (`block_in_place` + `Handle::current().block_on()`). Cron background threads use `ctx.tokio_runtime.block_on()` directly
- All I/O builtins receive `&Arc<RuntimeCtx>` — backend traits (still sync): `AiBackend`, `EmitBackend`, `HttpBackend`, `ShellBackend`, `YieldBackend`, `LogBackend`, `UserBackend`, `PaneBackend`, `EmbedBackend`
- Standard defaults: `ClaudeCodeAiBackend`, `ReqwestHttpBackend`, `ProcessShellBackend`, `StdoutEmitBackend`, `StdinStdoutYieldBackend`, `StderrLogBackend`, `NoopUserBackend`, `YieldPaneBackend`, `VoyageEmbedBackend`
- `AiOpts` fields: `prompt`, `tools` (Vec, empty = no tools), `max_turns`, `json_schema`. `ClaudeCodeAiBackend` maps `structured_output` JSON field to `result.text` when `json_schema` is set
- Deny backends for sandboxing: `DenyShellBackend`, `DenyHttpBackend`, `DenyAiBackend`, `DenyPaneBackend`, `DenyEmbedBackend`, `RestrictedShellBackend` (command allowlist). In `backends/restricted.rs`
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
