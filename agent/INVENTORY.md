-- Memory: capability register. Everything lx can do right now.
-- Add entries when features ship. Keep current — a stale inventory misleads the next you.

# Implemented Feature Inventory

## Core Language

- Arithmetic, bindings, strings, interpolation, regex literals (`r/\d+/flags`), collections (lists, records, maps, tuples), pattern matching
- Functions, closures, currying, default params, pipes, sections, slicing, named args
- Type definitions with tagged values and pattern matching
- Type annotations: `(x: Int y: Str) -> Result Int Str { ... }` on params, return types, bindings
- Type checker: `lx check` — bidirectional inference, unification, structural subtyping, import resolution (imported names bound as Unknown)
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
- `yield` — callback-based coroutine, JSON-line orchestrator protocol
- `refine` — first-class feedback loop: try/grade/revise with threshold + max_rounds
- `emit` — agent-to-human fire-and-forget output via EmitBackend
- `with name = expr { body }` — scoped bindings + record field update (`name.field <- value`)
- `with context key: val { body }` — ambient context propagation. Scoped state flows through call chains without explicit parameter threading. `context.field` dot-access, `context.current ()` returns full context record, `context.get key` returns Some/None. Nesting merges with outer context; inner values override; outer restored on scope exit. `context` globally available (returns `{}` outside any scope)

## Stdlib (40 Rust modules + 6 standard agents + 11 lx packages)

- Data: `std/json`, `std/md`, `std/re`, `std/math`, `std/time`
- System: `std/fs`, `std/env`, `std/http`
- Git: `std/git` — 36 functions
- State primitive: `std/store` — backing implementation for `Value::Store`. `Store ()` constructor creates a first-class Store value with dot-access methods (set, get, keys, values, entries, remove, len, has, clear, filter, query, map, update, save, load, persist, reload)
- Resilience: `std/retry`, `std/deadline`
- Communication: `std/agent`, `std/mcp`, `std/ai`
- Observation: `std/introspect` — system-wide live introspection: `system` (full snapshot), `agents` (agent list), `agent` (deep single-agent info), `messages` (in-flight), `bottleneck` (busiest agent). Aggregates from REGISTRY, SESSIONS, SUPERVISORS, TOPICS, ROUTE_TABLE
- Scheduling: `std/cron`
- Orchestration: `std/ctx`, `std/audit`, `std/plan`, `std/saga`, `std/pipeline`, `std/taskgraph`, `std/workspace`
- Collaboration: `std/workspace` — concurrent multi-agent editing: `create`, `claim`, `claim_pattern`, `edit`, `append`, `release`, `snapshot`, `regions`, `conflicts`, `resolve`, `history`, `watch`. Line-based region claiming with overlap detection, auto-bound adjustment, regex pattern claiming, watcher callbacks
- Discovery: `std/registry` — cross-process agent discovery: `start`, `stop`, `connect`, `register`, `deregister`, `find`, `find_one`, `health`, `load`, `watch`. In-memory registry with trait/trait/domain filtering, 4 selection strategies (first, least_loaded, round_robin, random), health/load tracking, watcher callbacks for join/leave events
- Persistence: `std/durable` — Temporal-style workflow persistence: `workflow`, `run`, `step` (idempotent with caching), `sleep`, `signal`, `send_signal`, `status`, `list`. File-backed at `.lx/durable/`
- Cost management: `std/budget`
- Standard agents: `std/agents/auditor`, `std/agents/router`, `std/agents/grader`, `std/agents/planner`, `std/agents/monitor`, `std/agents/reviewer`

## lx Packages (pkg/ workspace member)

Class-based packages using `entries: Store ()` + Collection Trait:
- `pkg/collection` — `Collection` Trait: get, keys, values, remove, query, len, has, save, load as defaults delegating to `self.entries`
- `pkg/knowledge` — `KnowledgeBase` class: key-value knowledge base with file persistence. Conforms to Collection.
- `pkg/tasks` — `TaskStore` class: task state machine (todo→in_progress→submitted→pending_audit→passed→complete). Conforms to Collection.
- `pkg/trace` — `TraceStore` class: trace collection, scoring, filtering, progress analysis, JSONL export. Provenance tracking: `enable_provenance`, `record_hop`, `message_path`, `message_hops`. Reputation scoring: `agent_score`, `agent_rank`. Agent attribution on spans and queries. Conforms to Collection.
- `pkg/memory` — `MemoryStore` class: tiered memory with fuzzy keyword recall, promote/demote, consolidation. Conforms to Collection.
- `pkg/context` — `ContextWindow` class: context window capacity management with priority eviction. Conforms to Collection.
- `pkg/circuit` — `CircuitBreaker` class: turn/action/time/repetition trip conditions
- `pkg/introspect` — `Inspector` class: agent self-monitoring: actions, markers, stuck detection, strategy shifts
- `pkg/pool` — `Pool` class: worker pools with round-robin dispatch
- `pkg/prompt` — composable prompt assembly (pure record builder, not a Class)
- Infrastructure: `std/trait`
- Interaction: `std/user` — `confirm`, `choose`, `ask`, `ask_with`, `progress`, `progress_pct`, `status`, `table`, `check` (signal poll). `UserBackend` trait on `RuntimeCtx` — `NoopUserBackend` (default/test), `StdinStdoutUserBackend` (terminal)
- Identity: `std/profile` — persistent agent profiles: `load`, `save`, `learn`, `recall`, `recall_prefix`, `forget`, `preference`, `get_preference`, `history`, `merge`, `age`, `decay`. Strategy helpers: `best_strategy`, `rank_strategies`, `adapt_strategy`. File-backed at `.lx/profiles/{name}.json`
- Visualization: `std/diag`
- Testing: `std/test` (satisfaction testing: `spec`, `scenario`, `run`, `run_scenario`, `report`), `std/describe` (BDD-style describe/it blocks with structured results)
- Flow composition: `std/flow` — `load`, `run`, `pipe`, `parallel`, `branch`, `with_retry`, `with_timeout`, `with_fallback`. Flows as first-class composable values with isolated interpreter execution
- Task graphs: `std/taskgraph` — `create`, `add`, `remove`, `run`, `run_with`, `validate`, `topo`, `status`, `dot`. DAG-aware subtask decomposition with topological execution, dependency result threading (`input_from`), per-task retry/timeout/on_fail policy, wave-based parallel scheduling, DOT export

## Agent Extensions (17 sub-modules of `std/agent`)

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

`lx run`, `lx test`, `lx check`, `lx agent`, `lx diagram`, `lx list`, `lx install`, `lx update`

- **Workspace support**: `lx.toml` manifest parsing, workspace discovery (walk up from cwd), `lx test` iterates members, `lx test -m name` filters, `lx list` shows member summary, `lx run member-name` resolves to entry file, `lx check` / `lx check -m name` workspace iteration
- Justfile recipes: `just test`, `just test-all` (workspace), `just test-member <name>`, `just list`, `just diagnose`, `just fmt`, `just build`, `just install`

## Workspace

`lx.toml` manifests — root workspace declares `[workspace].members`, each member has `[package]` (name, version, entry, description) and optional `[test]` (dir, pattern, runner). This repo is the first workspace: tests, brain, workgen, flows. Module resolver checks workspace members between relative and stdlib paths (`use member/path` → member's root directory).

## Test Coverage

94 test suites (92 .lx files + 87_export_shadow dir + 11_modules dir) in `tests/`. Fixtures in `tests/fixtures/`. 94/94 passing.
