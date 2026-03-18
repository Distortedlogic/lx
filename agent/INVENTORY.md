# Implemented Feature Inventory

What lx can do right now. For project health/status, see the "State" section of `TICK.md`.

## Core Language

- Arithmetic, bindings, strings, interpolation, regex literals (`r/\d+/flags`), collections (lists, records, maps, tuples), pattern matching
- Functions, closures, currying, default params, pipes, sections, slicing, named args
- Type definitions with tagged values and pattern matching
- Type annotations: `(x: Int y: Str) -> Result Int Str { ... }` on params, return types, bindings
- Type checker: `lx check` — bidirectional inference, unification, structural subtyping
- Concurrency: `par`, `sel`, `pmap`, `pmap_n`, `timeout` (sequential impl — real async needs tokio)
- Shell: `$cmd`, `$^cmd`, `${...}` with interpolation
- Error handling: `^` propagation, `??` coalescing, `(?? default)` sections. Structured error tags: `Err Timeout "msg"` with pattern matching. Uniform `None` on miss for Record, Map, and Agent field access
- Arithmetic: `/` always returns Float (Python 3 semantics), `//` for integer division, mixed Int/Float auto-promotion
- Modules: `use ./path`, aliasing, selective imports, `+` exports

## Agent System

- `Agent Name: TraitList = { methods }` — first-class agent declarations with trait conformance, method access via `.`, `uses`/`init`/`on` reserved fields (all wired to runtime), `Value::Agent` runtime representation
- `receive { action -> handler }` — agent message loop sugar, desugars to yield/loop/match
- `~>` send, `~>?` ask — infix operators, subprocess-transparent
- `Protocol Name = {field: Type}` — message contracts with runtime validation (returns `Err` on validation failure, catchable with `??`)
- Protocol composition (`{..Base extra: Str}`), unions (`A | B | C` with `_variant`), field constraints (`where`)
- `Trait Name = { handles: [...] provides: [...] requires: [...] }` — agent behavioral contracts
- `agent.implements` — runtime trait checking for routing/filtering
- `MCP` declarations — typed tool contracts, input/output validation, wrapper generation
- `with expr as name { body }` — scoped resources with auto-cleanup (LIFO close, cleanup on error)
- `yield` — callback-based coroutine, JSON-line orchestrator protocol
- `refine` — first-class feedback loop: try/grade/revise with threshold + max_rounds
- `emit` — agent-to-human fire-and-forget output via EmitBackend
- `with name = expr { body }` — scoped bindings + record field update (`name.field <- value`)

## Stdlib (40 modules)

- Data: `std/json`, `std/md`, `std/re`, `std/math`, `std/time`
- System: `std/fs`, `std/env`, `std/http`
- Git: `std/git` — 36 functions: `status`, `branch`, `root`, `is_repo`, `branches`, `remotes`, `log`, `show`, `blame`, `blame_range`, `diff`, `diff_stat`, `grep`, `add`, `commit`, `commit_with`, `tag`, `tag_with`, `create_branch`, `create_branch_at`, `delete_branch`, `checkout`, `checkout_create`, `merge`, `stash`/`stash_with`/`stash_pop`/`stash_list`/`stash_drop`, `fetch`, `pull`, `push`, `push_with`
- Resilience: `std/retry` — `retry` (default 3 attempts, exponential backoff), `retry_with` (configurable). Returns `Ok value` on success, `Err Exhausted {attempts last_error elapsed_ms}` on exhaustion
- Communication: `std/agent`, `std/mcp`, `std/ai`
- Scheduling: `std/cron`
- Orchestration: `std/ctx`, `std/tasks`, `std/audit`, `std/circuit`, `std/plan`, `std/saga`
- Concurrency: `std/pool` — worker pools: `create`, `fan_out`, `map`, `submit`, `status`, `shutdown`
- Cost management: `std/budget` — `create`, `spend`, `remaining`, `used`, `used_pct`, `project`, `status`, `slice` (sub-budgets)
- Prompt assembly: `std/prompt` — `create`, `system`, `section`, `constraint`, `instruction`, `example`, `compose`, `render`, `render_within`, `estimate`, `sections`, `without`
- Context windows: `std/context` — `create`, `add`, `usage`, `pressure`, `estimate`, `pin`/`unpin`, `evict`, `evict_until`, `items`, `get`, `remove`, `clear`
- Intelligence: `std/knowledge`, `std/introspect`
- Standard agents: `std/agents/auditor`, `std/agents/router`, `std/agents/grader`, `std/agents/planner`, `std/agents/monitor`, `std/agents/reviewer`
- Infrastructure: `std/memory`, `std/trace`, `std/trait`
- Interaction: `std/user` — `confirm`, `choose`, `ask`, `ask_with`, `progress`, `progress_pct`, `status`, `table`, `check` (signal poll). `UserBackend` trait on `RuntimeCtx` — `NoopUserBackend` (default/test), `StdinStdoutUserBackend` (terminal)
- Identity: `std/profile` — persistent agent profiles: `load`, `save`, `learn`, `recall`, `recall_prefix`, `forget`, `preference`, `get_preference`, `history`, `merge`, `age`, `decay`. Strategy helpers: `best_strategy`, `rank_strategies`, `adapt_strategy`. File-backed at `.lx/profiles/{name}.json`
- Visualization: `std/diag`
- Testing: `std/test` (test runner infrastructure, test/describe blocks), `std/describe` (BDD-style describe/it blocks with structured results)

## Agent Extensions (11 sub-modules of `std/agent`)

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

## Other Extensions

- `ai.prompt_structured` — Protocol-validated LLM output with auto-retry
- `ai.prompt_json` — lightweight structured output from inline record shape (no Protocol needed)
- `trace.improvement_rate` + `trace.should_stop` — diminishing returns detection

## Runtime

- All I/O builtins receive `&Arc<RuntimeCtx>` — backend traits: `AiBackend`, `EmitBackend`, `HttpBackend`, `ShellBackend`, `YieldBackend`, `LogBackend`, `UserBackend`
- Standard defaults: `ClaudeCodeAiBackend`, `ReqwestHttpBackend`, `ProcessShellBackend`, `StdoutEmitBackend`, `StdinStdoutYieldBackend`, `StderrLogBackend`, `NoopUserBackend`
- Embedders construct custom `RuntimeCtx` to swap backends for testing, server deployment, or sandboxing

## CLI

`lx run`, `lx test`, `lx check`, `lx agent`, `lx diagram`

## Test Coverage

71 test suites (70 .lx files + 11_modules dir) in `tests/`. Fixtures in `tests/fixtures/`.
