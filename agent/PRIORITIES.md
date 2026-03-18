-- Memory: program queue. Ordered feature work — what to build next.
-- Reorder when priorities shift. Remove when shipped. Add when specced.

# Priorities

Top item = next thing to implement. Each entry explains WHY it's at this position. Items 1, 22, 23, and Tier 5 require parser/interpreter changes; the rest are stdlib-only.

## Tier 1 — Infrastructure (multiplicative improvement to every tick)

1. ~~**Workspace Phase 2**~~ — SHIPPED Session 54. Module resolver workspace step, `lx run member-name`, `lx check` workspace iteration.

## Tier 2 — Agent features

Tier 1 completed: `std/retry` (Session 44), `std/user` + `std/profile` (Session 49), `Agent` declarations (Session 49). Enforced `Trait` methods (Session 51). Brain-driven improvements (Session 52).

3. ~~**`std/pipeline` checkpoint/resume**~~ — SHIPPED Session 55. 8 functions: `create`, `stage`, `complete`, `status`, `invalidate`, `invalidate_from`, `clean`, `list`.

4. **`AgentErr` structured errors** (`spec/agents-errors.md`) — Tagged error mechanism works (`Err Timeout "msg"`). Remaining: define 11 standard agent error variants as convention and update stdlib.

5. **`std/test` satisfaction testing** (`spec/testing-satisfaction.md`) — Spec + scenarios + grader + threshold scoring for non-deterministic agentic flows.

6. **`std/flow` composition** (`spec/flow-composition.md`) — Flows as first-class values: `flow.load`/`flow.run`/`flow.pipe`/`flow.par`.

7. **`std/taskgraph` DAG execution** (`spec/agents-task-graph.md`) — Dependency-ordered subtask decomposition. Declare tasks + dependencies + agents, runtime executes in topological order with max parallelism. Eliminates manual DAG scheduling boilerplate in every non-trivial multi-agent flow.

8. **`std/deadline` time propagation** (`spec/agents-deadline.md`) — Time budgets that propagate across `~>?` boundaries. Sub-agents know remaining time, can degrade gracefully. `deadline.scope`, `deadline.remaining`, `deadline.slice`. Orthogonal to `std/budget` (cost).

9. **`agent.route`/`register` capability routing** (`spec/agents-capability-routing.md`) — Declarative routing: `agent.route msg {trait: "Reviewer"}` finds the best available agent by trait/protocol/domain with load-awareness. `agent.route_multi` fans out to all matching + reconcile. Stepping stone to `std/registry`.

10. **`introspect.system` live observation** (`spec/agents-introspect-live.md`) — "What are all agents doing right now?" Structured system snapshot: agent states, in-flight messages, active dialogues, pool status, bottleneck detection. Extensions to existing `std/introspect`.

11. **`agent.pipeline`** (`spec/agents-pipeline.md`) — Consumer-driven flow control with backpressure.

12. **`~>>?` streaming ask** (`spec/agents-streaming.md`) — Stream partial results from long-running agents. Token already lexed (Session 31).

## Tier 3 — Multi-agent infrastructure, adaptive intelligence

13. **`std/trace` extensions** — Provenance (message flow tracking as trace spans: `trace.enable_provenance`, `trace.message_path`, `trace.message_hops`) + reputation (agent scoring from trace data: `trace.agent_score`, `trace.agent_rank`). One observability system instead of three separate modules. Absorbs `spec/agents-provenance.md` and `spec/agents-reputation.md`.

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
