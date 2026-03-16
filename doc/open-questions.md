# Resolved Questions — Decision Log

## Resolved in v0.1

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
| Multi-turn agent conversation | `agent.dialogue` / `agent.dialogue_turn` / `agent.dialogue_end` — library functions, session accumulates history |
| Message middleware | `agent.intercept agent middleware` — returns wrapped agent, composable by wrapping |
| Structured context transfer | `agent.handoff` + `Handoff` Protocol + `agent.as_context` |
| Dynamic plan revision | `std/plan` — plans as data, `plan.run` with `on_step` callback returns `PlanAction` |
| Shared discovery cache | `std/knowledge` — file-backed JSON with provenance metadata and query support |
| Agent introspection | `std/introspect` — identity, budget, actions, stuck detection |
| Dialogue history size | Bounded by `max_turns` in config. Default unlimited, capped at session lifetime. |
| Interceptor ordering | Outside-in execution, inside-out response |
| Knowledge eviction | `knowledge.expire before_time kb` — explicit eviction, no auto-TTL |

## Resolved Post-v0.1 (Session 33)

| Question | Decision |
|---|---|
| Reactive dataflow | `\|>>` streaming pipe. Same precedence as `\|`. Lazy, `collect` materializes. |
| Agent crash recovery | `agent.supervise` — Erlang-style strategies (one_for_one, one_for_all, rest_for_one) |
| Context propagation | `with context key: val { }` — ambient context auto-propagates to agent ops |
| Agent back-channel | `caller` implicit binding in handler (like `it` in `sel`) |
| Human-in-the-loop | `agent.gate` — structured approval with timeout policy |
| Agent capability query | `Capabilities` protocol + `agent.capabilities` helper |
| Multi-agent transactions | `std/saga` — compensating actions in reverse order |
| Message urgency | `_priority` field (`:critical`/`:high`/`:normal`/`:low`). Runtime priority queue. |
| Context compression | `ai.summarize` in `std/ai` — structured compression with keep/drop policies |
| Retry strategies | `retry_with` — per-error-type strategy, exponential backoff, jitter, circuit breaker |
