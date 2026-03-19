-- Memory: capability register. Everything lx can do right now.
-- Add entries when features ship. Keep current — a stale inventory misleads the next you.

# Implemented Feature Inventory

## Core Language

- Arithmetic, bindings, strings, interpolation, regex literals (`r/\d+/flags`), collections (lists, records, maps, tuples), pattern matching
- Functions, closures, currying, default params, pipes, sections, slicing, named args
- Type definitions with tagged values and pattern matching
- Type annotations: `(x: Int y: Str) -> Result Int Str { ... }` on params, return types, bindings
- Type checker: `lx check` — bidirectional inference, unification, structural subtyping
- Concurrency: `par`, `sel`, `pmap`, `pmap_n`, `timeout` (sequential impl — real async needs tokio)
- Shell: `$cmd`, `$^cmd`, `${...}` with interpolation
- Error handling: `^` propagation, `??` coalescing, `(?? default)` sections. Structured error tags: `Err Timeout "msg"` with pattern matching. Uniform `None` on miss for Record, Map, and Agent field access. `AgentErr` structured errors: 11 tagged variants (Timeout, RateLimited, BudgetExhausted, ContextOverflow, Incompetent, Upstream, PermissionDenied, ProtocolViolation, Unavailable, Cancelled, Internal) via `use std/agent {Timeout ...}`
- Arithmetic: `/` always returns Float (Python 3 semantics), `//` for integer division, mixed Int/Float auto-promotion
- Modules: `use ./path`, aliasing, selective imports, `+` exports, workspace member resolution (`use brain/protocols`)
- **`Class` keyword** — generic stateful objects with `self` method dispatch. `Class Name : [Traits] = { field: default; method = (params) { body } }`. Constructor: `Name {field_overrides}`. Interior mutability: `self.field <- val` mutates via global STORES (no reassign needed). Reference semantics: `a = b` shares same object. `Class Worker : [Agent] = { ... }` also works — explicitly adding Agent to traits list
- **`Store` as first-class Value** — `Value::Store { id }` with dot-access methods: set, get, keys, values, entries, remove, len, has, clear, filter, query, map, update, save, load, persist, reload. `Store ()` constructor. Reference semantics. Store cloning in Class constructors.
- **Trait default methods** — `Trait Name = { required: Sig -> Ret; default_method = (params) { body } }`. Default methods injected into conforming Class/Agent if not overridden.

## Agent System

- `Agent Name: TraitList = { methods }` — first-class agent declarations. `Agent` keyword auto-imports `pkg/agent {Agent}` Trait and auto-adds "Agent" to traits list. Runtime representation: `Value::Class { name, traits, defaults, methods }`. Agent Trait (`pkg/agent.lx`) provides defaults: init, perceive, reason, act, reflect, handle, run, think/think_with/think_structured, use_tool/tools, describe, ask/tell. Method access via `.`
- `receive { action -> handler }` — agent message loop sugar, desugars to yield/loop/match
- `~>` send, `~>?` ask — infix operators, subprocess-transparent
- `Protocol Name = {field: Type}` — message contracts with runtime validation (returns `Err` on validation failure, catchable with `??`)
- Protocol composition (`{..Base extra: Str}`), unions (`A | B | C` with `_variant`), field constraints (`where`)
- `Trait Name = { method: {input} -> output }` — agent behavioral contracts with default method implementations. Traits with non-empty `fields` act as Protocols (callable as constructor, runtime validation). Behavioral Traits have empty `fields`.
- `agent.implements` — runtime trait checking for routing/filtering (works for Class/Agent, Object, Record). Checks traits list for "Agent" to distinguish Agents from plain Classes
- Two new builtins: `method_of(obj, name)` — returns a method by name or None; `methods_of(obj)` — returns list of method names
- `MCP` declarations — typed tool contracts, input/output validation, wrapper generation
- `with expr as name { body }` — scoped resources with auto-cleanup (LIFO close, cleanup on error)
- `yield` — callback-based coroutine, JSON-line orchestrator protocol
- `refine` — first-class feedback loop: try/grade/revise with threshold + max_rounds
- `emit` — agent-to-human fire-and-forget output via EmitBackend
- `with name = expr { body }` — scoped bindings + record field update (`name.field <- value`)

## Stdlib (31 Rust modules + 6 standard agents + 11 lx packages)

- Data: `std/json`, `std/md`, `std/re`, `std/math`, `std/time`
- System: `std/fs`, `std/env`, `std/http`
- Git: `std/git` — 36 functions
- State primitive: `std/store` — backing implementation for `Value::Store`. `Store ()` constructor creates a first-class Store value with dot-access methods (set, get, keys, values, entries, remove, len, has, clear, filter, query, map, update, save, load, persist, reload)
- Resilience: `std/retry`, `std/deadline`
- Communication: `std/agent`, `std/mcp`, `std/ai`
- Observation: `std/introspect` — system-wide live introspection: `system` (full snapshot), `agents` (agent list), `agent` (deep single-agent info), `messages` (in-flight), `bottleneck` (busiest agent). Aggregates from REGISTRY, SESSIONS, SUPERVISORS, TOPICS, ROUTE_TABLE
- Scheduling: `std/cron`
- Orchestration: `std/ctx`, `std/audit`, `std/plan`, `std/saga`, `std/pipeline`, `std/taskgraph`
- Cost management: `std/budget`
- Standard agents: `std/agents/auditor`, `std/agents/router`, `std/agents/grader`, `std/agents/planner`, `std/agents/monitor`, `std/agents/reviewer`

## lx Packages (pkg/ workspace member)

Class-based packages using `entries: Store ()` + Collection Trait:
- `pkg/collection` — `Collection` Trait: get, keys, values, remove, query, len, has, save, load as defaults delegating to `self.entries`
- `pkg/knowledge` — `KnowledgeBase` class: key-value knowledge base with file persistence. Conforms to Collection.
- `pkg/tasks` — `TaskStore` class: task state machine (todo→in_progress→submitted→pending_audit→passed→complete). Conforms to Collection.
- `pkg/trace` — `TraceStore` class: trace collection, scoring, filtering, progress analysis, JSONL export. Conforms to Collection.
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

## Agent Extensions (12 sub-modules of `std/agent`)

- `agent.reconcile` — 6 merge strategies (union, intersection, vote, highest_confidence, max_score, merge_fields) + custom Fn
- `agent.dialogue` — multi-turn stateful sessions with config `{role? context? max_turns?}`
- `agent.intercept` — composable message middleware with short-circuit
- `Handoff` Protocol + `agent.as_context` — structured context transfer for LLM consumption
- `Capabilities` Protocol + `agent.capabilities` + `agent.advertise` — runtime capability discovery
- `GateResult` Protocol + `agent.gate` — human-in-the-loop approval gates via yield
- `agent.supervise` — Erlang-style supervision: one_for_one/one_for_all/rest_for_one
- `agent.mock` — mock agents with call tracking for testing
- `agent.dispatch` — pattern-based message routing without LLM
- `agent.negotiate` — N-party iterative consensus with converge function
- `agent.topic` / `agent.subscribe` / `agent.publish` — in-process pub/sub with filtered subscriptions
- `agent.route` / `agent.register` — capability-based routing: register agents with traits/protocols/domains, route by filter with selection strategies (least_busy, round_robin, random, custom), fan-out with reconcile via `route_multi`

## Other Extensions

- `ai.prompt_structured` — Protocol-validated LLM output with auto-retry
- `ai.prompt_json` — lightweight structured output from inline record shape (no Protocol needed)

## Runtime

- All I/O builtins receive `&Arc<RuntimeCtx>` — backend traits: `AiBackend`, `EmitBackend`, `HttpBackend`, `ShellBackend`, `YieldBackend`, `LogBackend`, `UserBackend`
- Standard defaults: `ClaudeCodeAiBackend`, `ReqwestHttpBackend`, `ProcessShellBackend`, `StdoutEmitBackend`, `StdinStdoutYieldBackend`, `StderrLogBackend`, `NoopUserBackend`
- Embedders construct custom `RuntimeCtx` to swap backends for testing, server deployment, or sandboxing

## CLI

`lx run`, `lx test`, `lx check`, `lx agent`, `lx diagram`, `lx list`

- **Workspace support**: `lx.toml` manifest parsing, workspace discovery (walk up from cwd), `lx test` iterates members, `lx test -m name` filters, `lx list` shows member summary, `lx run member-name` resolves to entry file, `lx check` / `lx check -m name` workspace iteration
- Justfile recipes: `just test`, `just test-all` (workspace), `just test-member <name>`, `just list`, `just diagnose`, `just fmt`, `just build`, `just install`

## Workspace

`lx.toml` manifests — root workspace declares `[workspace].members`, each member has `[package]` (name, version, entry, description) and optional `[test]` (dir, pattern, runner). This repo is the first workspace: tests, brain, workgen, flows. Module resolver checks workspace members between relative and stdlib paths (`use member/path` → member's root directory).

## Test Coverage

81 test suites (80 .lx files + 11_modules dir) in `tests/`. Fixtures in `tests/fixtures/`. 81/81 passing.
