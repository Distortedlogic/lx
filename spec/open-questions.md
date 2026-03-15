# Open Questions

All v0.1 design questions have been resolved. Decisions and rationale are in [design.md](design.md). This file tracks considerations for future versions.

## Resolved in v0.1 (Summary)

| Question | Decision |
|---|---|
| User-defined generators | Removed — use `loop`/`break` for custom iteration |
| Duration literals | Stdlib functions: `time.sec 5`, `time.ms 100` |
| Retry combinator | Stdlib: `retry n f` with optional backoff config |
| Interactive/notebook mode | `lx notebook` — shared environment, `---` separated blocks |
| Shell string escapes | `$` interpolates `{expr}` |
| Error code taxonomy | Flat codes in v1: `error[type]`, `error[parse]` |
| Bytes/Str boundary | Three functions: `fs.read`, `fs.read_bytes`, `fs.read_lossy` |
| Signal handling | `defer` built-in: `defer () cleanup`. LIFO on scope exit |
| Map literal key types | Expression keys: `%{expr: val}` always evaluates `expr` |
| `^`/`??` vs `\|` precedence | `^` and `??` are lower than `\|` — apply to pipeline results |
| `$^` return type | Returns `Str ^ ShellErr` (stdout extracted), not full record |
| `? {` disambiguation | `?` followed by `{` always starts a match block |
| `assert` semantics | Hard panic, not recoverable via `^`/`??`. Test runner catches. |
| Mutable captures in concurrency | Compile error — prevents data races |
| `defer` scope | Per-block-scope, not per-function |
| Forward references | Top-level bindings visible to each other. Within blocks, sequential. |
| Type inference | Bidirectional with local inference. Per-function checking. |
| Variant uniqueness | Variant constructors must be unique within a module |

## Resolved Post-v0.1

| Question | Decision |
|---|---|
| Division by zero | Runtime panic, not `Result`. Use `math.safe_div` for recoverable. |
| Tuple auto-spread | Function with N params + single N-tuple arg = auto-spread |
| `+main` type | Must be a function, compile error otherwise |
| Import shadowing | Selective imports shadow built-ins with a warning |
| `pmap_n` | Added to v1: `pmap_n limit f xs` for rate-limited concurrency |
| `none?` ambiguity | Exclusively 2-arg collection predicate. Use `!some?` for Maybe. |

## Resolved Post-v0.1 (Session 4)

| Question | Decision |
|---|---|
| Data processing at scale | Deferred — no stdlib modules for dataframes, databases, or ML in v1 |
| MCP transport | Both stdio and HTTP+SSE are implemented |
| Message serialization | JSON (serde_json) for inter-agent messages |

## Resolved Post-v0.1 (Session 31)

| Question | Decision |
|---|---|
| Agent permissions | Capability attenuation on `agent.spawn` — `capabilities` field restricts tools, fs, network, budget |
| Shared state in par | `std/blackboard` — concurrent shared workspace with last-write-wins |
| Agent streaming | `~>>?` operator — returns lazy stream of partial results from agent |
| Transactional execution | `checkpoint`/`rollback` keywords — snapshot and restore mutable state |
| Event-driven agents | `std/events` — topic-based pub/sub event bus |
| Agent negotiation | Pattern using Protocol (Offer/Accept/Reject), not a primitive |

## Resolved Post-v0.1 (Session 32)

| Question | Decision |
|---|---|
| Multi-turn agent conversation | `agent.dialogue` / `agent.dialogue_turn` / `agent.dialogue_end` — library functions in `std/agent`, not keywords. Session accumulates history via JSON-line protocol. |
| Message middleware | `agent.intercept agent middleware` — returns wrapped agent. Middleware takes `(msg next)`, composable by wrapping. |
| Structured context transfer | `agent.handoff` + `Handoff` Protocol + `agent.as_context`. Agent constructs handoff explicitly, no auto-population. |
| Dynamic plan revision | `std/plan` module. Plans are data (step records with deps). `plan.run` with `on_step` callback returns `PlanAction` (continue/replan/skip/abort/insert_after). |
| Shared discovery cache | `std/knowledge` — file-backed JSON with provenance metadata (source, confidence, tags) and query support. Shared via path. |
| Agent introspection | `std/introspect` — separate module. Identity, budget, actions, stuck detection. Interpreter collects action history. |
| Dialogue history size | Bounded by `max_turns` in config. Default unlimited but capped at session lifetime. |
| Interceptor ordering | Outside-in execution, inside-out response. Compose by wrapping: outer interceptor sees message first. |
| Knowledge eviction | `knowledge.expire before_time kb` — explicit eviction by caller. No auto-TTL. |

## Resolved Post-v0.1 (Session 33)

| Question | Decision |
|---|---|
| Reactive dataflow | `\|>>` streaming pipe operator. Same precedence as `\|`. Lazy — items flow downstream as they complete. `collect` materializes. Spec: `spec/concurrency-reactive.md`. |
| Agent crash recovery | `agent.supervise` library function with Erlang-style strategies (one_for_one, one_for_all, rest_for_one). Spec: `spec/agents-supervision.md`. |
| Context propagation | `with context key: val { }` — ambient context that auto-propagates deadline, budget, request_id, trace_id to all agent ops. Spec: `spec/agents-ambient.md`. |
| Agent back-channel | `caller` implicit binding in handler (like `it` in `sel`). Agents can ask their caller questions inline. Spec: `spec/agents-clarify.md`. |
| Human-in-the-loop | `agent.gate` library function. Structured approval with timeout policy (:abort/:approve/:reject/:escalate). Spec: `spec/agents-gates.md`. |
| Agent capability query | `Capabilities` protocol + `agent.capabilities` query helper. Agents self-report protocols, tools, domains, budget, status. Spec: `spec/agents-capability.md`. |
| Multi-agent transactions | `std/saga` module. Saga pattern with compensating actions in reverse order. Spec: `spec/agents-saga.md`. |
| Message urgency | `_priority` field on messages (`:critical`, `:high`, `:normal`, `:low`). Runtime priority queue. Spec: `spec/agents-priority.md`. |
| Context compression | `ai.summarize` function in `std/ai`. Structured compression of agent history with keep/drop policies. |
| Retry strategies | Enhanced `retry_with` with per-error-type strategy, exponential backoff, jitter, circuit breaker integration. |

## Agentic Design Questions (v1)

These are open questions for the agentic layer — to be resolved during implementation:

| Question | Considerations |
|---|---|
| Agent process model | Are agents subprocesses (CLI invocations), API calls, or both? Should `agent.spawn` support both local and remote agents? |
| Agent discovery | How do agents find each other? Registry, well-known names, URIs? |
| Channel backpressure | What happens when a channel sender outpaces the receiver? Buffer, drop, block? |
| Agent lifecycle | What happens to subagents when the parent dies? Orphan cleanup? |
| Dialogue persistence | Should dialogue sessions be serializable/resumable across process restarts? Currently session-scoped. |
| Knowledge consistency | File-level locking for concurrent writes — sufficient for v1? May need advisory locks or WAL for high-contention. |
| Introspection performance | Action logging adds overhead. Should it be opt-in via config, or always-on with bounded buffer? |
| Interceptor + streaming | How do interceptors interact with `~>>?` streams? Intercept each chunk, or only the initial message? |
| Plan step parallelism | Steps with no mutual dependencies could run in parallel. Should `plan.run` do this automatically? |
| Checkpoint scope | Should `checkpoint` track shell commands via compensating actions? Current design: shell/MCP not rolled back. |
| Blackboard consistency | Should `std/blackboard` support CRDTs or transactional multi-key updates beyond last-write-wins? |
| Stream backpressure | When a `~>>?` consumer is slower than the producer, buffer or block? |

| Saga undo failures | Record and continue — don't fail the compensation chain. Return full report of what succeeded/failed. |
| Priority preemption | No mid-handler interruption. Critical messages queued and processed next. Long handlers can poll with `agent.check_critical`. |
| Ambient context custom fields | Pass through as-is, no runtime enforcement. Available via `context.current ()`. |
| `\|>>` ordering | Completion order by default (fastest first). Caller uses `sort_by` for input order if needed. |

## Considerations for v2

These are not blockers for v1 implementation but worth revisiting after real-world usage:

**Dotted error codes** — `error[type.mismatch]` vs `error[type]`. Flat is simpler for v1. If programmatic error filtering becomes a real need, add subcodes.

**Or-patterns in match arms** — `1 | 2 -> ...` conflicts with pipe. Could use `1 , 2 -> ...` or `[1 2] -> ...` as set-of-values syntax. Guards work but are verbose for large literal sets. Revisit if pattern matching on sets of values proves common.

**String interpolation patterns** — matching `"http://{rest}"` in pattern arms. Powerful but complex to implement. Regex handles the same cases. Revisit if regex patterns in match arms prove too verbose.

**~~Concurrency limits on `pmap`~~** — Resolved: `pmap_n limit f xs` added in v1. Rate-limited APIs are too common to defer. See [concurrency.md](concurrency.md).

**~~Streaming/channel primitives~~** — Resolved: `~>>?` for agent streaming, `std/events` for pub/sub. Channels still deferred.

**CLI argument parser** — `std/args` or `std/cli` for declarative argument parsing (flags, options, subcommands). v1 uses `env.args` with pattern matching, which handles simple cases. A structured parser would help for complex CLIs.

**Plugin/extension system** — Loading native (Rust/C) functions as lx modules for performance-critical operations. The FFI boundary would need careful design around error handling and type mapping.

**WASM target** — `lx build --target wasm` for running lx scripts in browsers or edge runtimes. The runtime model (async I/O, work-stealing) needs adaptation.

**Pattern matching on regex** — Using regex directly in match arms instead of guards with `std/re`. Currently requires guards: `s & (re.is_match r/\d+/ s)`.

**`where` clauses for type constraints** — Currently there's no way to express "this generic type must support equality" or "must be sortable." Structural typing handles fields, but behavioral constraints (like "has a `<` operator") are implicit. Revisit if the lack of constraints causes confusing error messages.
