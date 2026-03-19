-- Memory: program queue. Ordered feature work — what to build next.
-- Reorder when priorities shift. Remove when shipped. Add when specced.

# Priorities

Top item = next thing to implement. Each entry explains WHY it's at this position.

## Tier 0 — Bug fixes (foundation before features)

BUGS.md says "fix bugs before new features." 60 sessions of feature work have accumulated known parser bugs, gotchas, and code quality violations. Every new feature built on a buggy parser inherits those bugs. Fix the foundation before adding more surface area.

**Session 61: Parser bug fixes + code quality** ✓ DONE

- [x] List spread bp — changed `parse_expr(32)` to `parse_expr(0)` (consistent with non-spread element parsing)
- [x] Module `../../` multi-level — already had loop; confirmed working, removed stale BUGS.md entry
- [x] Single-line multi-field records — already working; confirmed with tests, removed stale BUGS.md entry
- [x] Agent body uppercase tokens — accept `TypeName` in Agent body + dot access
- [x] Split all 7 over-300-line files (7 new files: stmt_trait, agent_reconcile_score, cron_helpers, str_extra, diag_types, walk_helpers, tasks_transition)
- [ ] Wire Agent `uses` — deferred (not critical, workaround exists)

**Not fixing (by design / architectural):**
- Named-arg `:` vs ternary `:` — inherent parser ambiguity, workaround (parenthesize) is fine
- `is_func_def` heuristic ambiguity — fixed in Session 64 (`application_depth` tracking)
- Keyword field names — fundamental to how keywords work, `flow.parallel` pattern is fine
- par/sel/pmap sequential — needs tokio, architectural change
- Trait conformance uncatchable — by design (hard error at definition time)

**Session 62: Feature consolidation audit (collaborative with user)** ✓ DONE

Full code review of 44 stdlib modules + 12 agent extensions. Mechanical deduplication (3 shared helpers),
then identified `std/store` as the missing primitive enabling module→package migration. Converted 5 modules:
- [x] Retry/step_deps/deadline delegation (internal Rust helpers)
- [x] `std/store` new primitive (242 lines Rust, 12 functions)
- [x] knowledge, circuit, prompt, tasks, trace, memory, context, introspect, pool → pkg/*.lx
- [ ] budget, profile, pipeline — stay Rust (lx lacks dynamic record field access, randomness, hashing)
- [ ] Agent sub-modules (dispatch, route, etc.) — can't extract from `agent.X` namespace
- [x] **`Class` keyword** — generic stateful objects (Agent minus messaging). DashMap-backed, Trait defaults. 8 packages converted. Session 63

**Session 64: Store promotion, Collection Trait, type hierarchy refactor** ✓ DONE

- [x] `Value::Store { id }` first-class type (dot-access methods, constructor, reference semantics)
- [x] `Collection` Trait (`pkg/collection.lx`) — get/keys/values/remove/query/len/has/save/load defaults
- [x] 5 collection packages rewritten (knowledge, tasks, memory, trace, context) using `entries: Store ()` + Collection
- [x] Type hierarchy refactor: OBJECTS DashMap eliminated, `Value::Agent` → `Value::Class { kind: Agent }`, `Value::Protocol` → `Value::Trait` with fields
- [x] `is_func_def()` parser bug fix — `application_depth` tracking for `(expr) {record}` disambiguation
- [x] Boxed `LxFunc`/`ProtocolField`, visitor context structs, type alias for pubsub return

## Tier 1 — Infrastructure (multiplicative improvement to every tick)

1. ~~**Workspace Phase 2**~~ — SHIPPED Session 54. Module resolver workspace step, `lx run member-name`, `lx check` workspace iteration.

## Tier 2 — Agent features

Tier 1 completed: `std/retry` (Session 44), `std/user` + `std/profile` (Session 49), `Agent` declarations (Session 49). Enforced `Trait` methods (Session 51). Brain-driven improvements (Session 52).

3. ~~**`std/pipeline` checkpoint/resume**~~ — SHIPPED Session 55. 8 functions: `create`, `stage`, `complete`, `status`, `invalidate`, `invalidate_from`, `clean`, `list`.

4. ~~**`AgentErr` structured errors**~~ — SHIPPED Session 56. 11 tagged error variants with constructors, stdlib migration (budget, agent, mcp, pool, http).

5. ~~**`std/test` satisfaction testing**~~ — SHIPPED Session 50+. `test.spec`, `test.scenario`, `test.run`, `test.run_scenario`, `test.report`.

6. ~~**`std/flow` composition**~~ — SHIPPED Session 57. 8 functions: `load`, `run`, `pipe`, `parallel`, `branch`, `with_retry`, `with_timeout`, `with_fallback`.

7. ~~**`std/taskgraph` DAG execution**~~ — SHIPPED Session 58. 9 functions: `create`, `add`, `remove`, `run`, `run_with`, `validate`, `topo`, `status`, `dot`. Kahn's algorithm, wave-based execution, `input_from` result threading, per-task retry/timeout, `on_fail` policy, DOT export.

8. ~~**`std/deadline` time propagation**~~ — SHIPPED Session 59. 8 functions: `create`, `create_at`, `scope`, `remaining`, `expired`, `check`, `slice`, `extend`. Thread-local scope stack, auto `_deadline_ms` injection on `~>?`/`~>`.

9. ~~**`agent.route`/`register` capability routing**~~ — SHIPPED Session 60. 5 functions: `register`, `unregister`, `registered`, `route`, `route_multi`. Trait/protocol/domain filtering, selection strategies (least_busy, round_robin, random, custom), load tracking, reconcile integration.

10. **`introspect.system` live observation** (`spec/agents-introspect-live.md`) — "What are all agents doing right now?" Structured system snapshot: agent states, in-flight messages, active dialogues, pool status, bottleneck detection. Extensions to existing `pkg/introspect`.

11. **`agent.pipeline`** (`spec/agents-pipeline.md`) — Consumer-driven flow control with backpressure.

12. **`~>>?` streaming ask** (`spec/agents-streaming.md`) — Stream partial results from long-running agents. Token already lexed (Session 31).

## Tier 3 — Multi-agent infrastructure, adaptive intelligence

13. **`pkg/trace` extensions** — Provenance (message flow tracking as trace spans: `trace.enable_provenance`, `trace.message_path`, `trace.message_hops`) + reputation (agent scoring from trace data: `trace.agent_score`, `trace.agent_rank`). One observability system instead of three separate modules. Absorbs `spec/agents-provenance.md` and `spec/agents-reputation.md`.

14. **`std/workspace` collaborative editing** (`spec/agents-workspace.md`) — Multiple agents editing the same artifact concurrently with region claiming and conflict resolution.

15. **`std/registry` cross-process discovery** (`spec/agents-discovery.md`) — Discovery by trait/protocol/domain, health checking, load-balanced dispatch.

16. **`agent.dialogue_fork`/`compare`/`merge`** (`spec/agents-dialogue-branch.md`) — Fork dialogues for tree-of-thought / best-of-N exploration. Fork shares parent history, branches execute in parallel, compare grades them, merge picks the winner.

17. **`agent.adapter`/`negotiate_format`** (`spec/agents-format-negotiate.md`) — Runtime Protocol format negotiation. Static field mapping adapters, dynamic capability-based format discovery, one-shot coercion. Enables plug-and-play agent composition across Protocol boundaries.

18. **`agent.reload`/`evolve`** (`spec/agents-hot-reload.md`) — Hot-swap agent handlers without restart. `agent.evolve` for self-update from within handler. Preserves dialogues, interceptors, identity. Enables adaptive long-lived agents.

19. **`agent.dialogue_save/load`** (`spec/agents-dialogue-persist.md`) — Persist dialogue sessions across process restarts.

20. **`with context` ambient propagation** (`spec/agents-ambient.md`) — Scoped ambient state flowing through call chains. Now includes cross-process constraint propagation at `agent.spawn` boundaries (absorbs `spec/agents-constraint-propagation.md`).

21. **`lx install/update`** (`spec/package-manifest.md`) — Dependency resolution and lock file management.

22. **`meta` block** (`spec/agents-meta.md`) — Strategy-level iteration. `refine` iterates within one approach; `meta` tries fundamentally different approaches.

23. **Typed yield variants** (`spec/agents-yield-typed.md`) — Structured orchestrator communication.

## Tier 4 — Remaining

24. **`agent.on` lifecycle hooks** (`spec/agents-lifecycle.md`) — Dynamic hook registration for standalone agents (Agent declarations have `on:` for static hooks). Now includes `:signal` event for reactive interrupt handling.

25. **`std/durable`** (`spec/agents-durable.md`) — Full Temporal-style workflow persistence. When this ships, `std/pipeline` becomes a convenience layer on top.

## Tier 5 — Parser-heavy, speculative

- `|>>` streaming pipe (`spec/concurrency-reactive.md`)
- `caller` implicit binding (`spec/agents-clarify.md`)
- `durable` expression (`spec/agents-durable.md`)
- Deadlock detection (`spec/agents-deadlock.md`)
