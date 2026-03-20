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
- par/sel/pmap sequential — FIXED Session 71d (async interpreter, futures::join_all/select_all)
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
- [x] Type hierarchy refactor: OBJECTS DashMap eliminated, `Value::Agent` removed, `Value::Trait` → `Value::Trait` with fields
- [x] Agent is now a Trait in `pkg/agent.lx` — `Agent` keyword auto-imports it, auto-adds "Agent" to traits list. `Value::Class` has 4 fields: name, traits, defaults, methods. No `ClassKind` enum. Two new builtins: `method_of(obj, name)`, `methods_of(obj)`
- [x] `is_func_def()` parser bug fix — `application_depth` tracking for `(expr) {record}` disambiguation
- [x] Boxed `LxFunc`/`FieldDecl`, visitor context structs, type alias for pubsub return

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

9. ~~**`agent.route`/`register` capability routing**~~ — SHIPPED Session 60. 5 functions: `register`, `unregister`, `registered`, `route`, `route_multi`. Trait/trait/domain filtering, selection strategies (least_busy, round_robin, random, custom), load tracking, reconcile integration.

10. ~~**`introspect.system` live observation**~~ — SHIPPED Session 65. `std/introspect` module: 5 functions (`system`, `agents`, `agent`, `messages`, `bottleneck`). Aggregates from REGISTRY, SESSIONS, SUPERVISORS, TOPICS, ROUTE_TABLE. `introspect.watch` deferred (needs async).

11. ~~**`agent.pipeline`**~~ — SHIPPED Session 66. 11 functions: `pipeline`, `pipeline_send`, `pipeline_collect`, `pipeline_batch`, `pipeline_stats`, `pipeline_on_pressure`, `pipeline_pause`, `pipeline_resume`, `pipeline_drain`, `pipeline_close`, `pipeline_add_worker`. Bounded buffers, 4 overflow policies, pressure callbacks, round-robin workers.

12. ~~**`~>>?` streaming ask**~~ — SHIPPED Session 67. `Value::Stream` (mpsc channel), `Expr::StreamAsk`, `TildeArrowArrowQ` token. Local + subprocess streaming. HOFs work on streams. `agent.emit_stream`/`agent.end_stream`. 1 Rust file (agent_stream.rs).

## Tier 3 — Multi-agent infrastructure, adaptive intelligence

13. ~~**`pkg/trace` extensions**~~ — SHIPPED Session 68. Provenance: `enable_provenance`, `record_hop`, `message_path`, `message_hops`. Reputation: `agent_score`, `agent_rank`. New fields: `provenance: Store ()`, `provenance_enabled`. `record` captures `agent` field, `query` supports `agent` filter. Pure lx.

14. ~~**`std/workspace` collaborative editing**~~ — SHIPPED Session 69. 12 functions: `create`, `claim`, `claim_pattern`, `edit`, `append`, `release`, `snapshot`, `regions`, `conflicts`, `resolve`, `history`, `watch`. Line-based region claiming, overlap detection, bound auto-adjustment, regex pattern claiming, watcher callbacks. 2 Rust files (workspace.rs + workspace_edit.rs).

15. ~~**`std/registry` cross-process discovery**~~ — SHIPPED Session 70. 10 functions: `start`, `stop`, `connect`, `register`, `deregister`, `find`, `find_one`, `health`, `load`, `watch`. In-memory registry with trait/trait/domain filtering, 4 selection strategies (first, least_loaded, round_robin, random), health/load tracking, watcher callbacks. 3 Rust files (registry.rs + registry_query.rs + registry_store.rs).

16. ~~**`agent.dialogue_fork`/`compare`/`merge`**~~ — SHIPPED Session 71. 4 functions: `dialogue_fork`, `dialogue_compare`, `dialogue_merge`, `dialogue_branches`. Fork shares parent history, parent suspended while forks active, compare grades via user function, merge picks winner and resumes parent. Recursive fork tree cleanup. 1 Rust file (agent_dialogue_branch.rs).

17. ~~**`agent.adapter`/`negotiate_format`**~~ — SHIPPED Session 72. 3 functions: `adapter` (static field mapping), `negotiate_format` (runtime negotiation via capabilities), `coerce` (one-shot transform). Levenshtein heuristic for fuzzy field matching. 2 Rust files (agent_adapter.rs + agent_negotiate_fmt.rs).

18. ~~**`agent.reload`/`evolve`**~~ — SHIPPED Session 73. 3 functions: `reload` (external handler replacement via ID-based mutable store), `evolve` (self-update from within handler via thread-local pending flag), `update_traits` (add/remove traits). Subprocess agents return Err. Interceptors preserved. 1 Rust file (agent_reload.rs).

19. ~~**`agent.dialogue_save/load`**~~ — SHIPPED Session 74. 4 functions: `dialogue_save`, `dialogue_load`, `dialogue_list`, `dialogue_delete`. File-backed persistence at `.lx/dialogues/{id}.json` with atomic writes. JSON serialization via `json_conv`. 1 Rust file (agent_dialogue_persist.rs).

20. ~~**`with context` ambient propagation**~~ — SHIPPED Session 75. `Expr::WithContext` AST node, `with context key: val { body }` syntax, thread-local ambient snapshot, `context` global binding (current/get + field access). Nesting merges/overrides/restores. 1 Rust file (interpreter/ambient.rs). Cross-process propagation at `agent.spawn` boundaries deferred (requires async subprocess init protocol).

21. ~~**`lx install/update`**~~ — SHIPPED Session 71b. `lx install`/`lx update` with git+path deps, `lx.lock`, `.lx/deps/` module resolution. 4 new CLI files (install.rs, install_ops.rs, lockfile.rs, check.rs).

22. **`meta` block** (`spec/agents-meta.md`) — Strategy-level iteration. `refine` iterates within one approach; `meta` tries fundamentally different approaches.

23. **Typed yield variants** (`spec/agents-yield-typed.md`) — Structured orchestrator communication.

## Tier 4 — Remaining

24. **`agent.on` lifecycle hooks** (`spec/agents-lifecycle.md`) — Dynamic hook registration for standalone agents (Agent declarations have `on:` for static hooks). Now includes `:signal` event for reactive interrupt handling.

25. ~~**`std/durable`**~~ — SHIPPED Session 71b. 8 functions: `workflow`, `run`, `step`, `sleep`, `signal`, `send_signal`, `status`, `list`. File-backed persistence with atomic writes. 3 Rust files (durable.rs + durable_run.rs + durable_io.rs).

## Tier 5 — Architectural

26. ~~**Async interpreter**~~ — SHIPPED Session 71d. `async fn eval()` with `#[async_recursion(?Send)]`. `par`/`sel`/`pmap` → `futures::join_all`/`select_all`. `BuiltinKind::Sync`/`Async` split. `call_value_sync` bridge for 31 stdlib files. Follow-up: remove rayon dep, convert backend traits to async, migrate remaining sync stdlib callers to async builtins.

## Tier 6 — Parser-heavy, speculative

- `|>>` streaming pipe (`spec/concurrency-reactive.md`)
- `caller` implicit binding (`spec/agents-clarify.md`)
- Deadlock detection (`spec/agents-deadlock.md`)
