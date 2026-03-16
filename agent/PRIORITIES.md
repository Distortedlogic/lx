# Priorities

Ordered work queue. Top item = next thing to implement. Each entry explains WHY it's at this position so you can judge whether circumstances have changed.

## Tier 1 — Highest leverage remaining

No parser changes needed. Pure stdlib modules that fill gaps agents hit constantly. `std/retry` completed (Session 44).

1. **`std/user`** (`spec/stdlib-user.md`) — Structured agent-to-user interaction: `confirm`, `choose`, `ask`, `progress`, `status`, `table`, `check` (non-blocking signal poll). Fills the gap between `emit` (fire-and-forget text) and `yield` (heavy orchestrator round-trip). New `UserBackend` trait on `RuntimeCtx` — terminal, yield-bridge, and noop backends. `user.check` absorbs cooperative interrupt checking (no `checkpoint` keyword needed).

2. **`std/profile`** (`spec/agents-profile.md`) — Persistent agent identity across sessions. Agents accumulate knowledge, preferences, and relationship history. File-backed profiles at `.lx/profiles/`. Now also absorbs `std/strategy` — strategy outcomes stored as `strategy:` prefixed domains with `profile.best_strategy`/`rank_strategies`/`adapt_strategy` helpers. One module for all cross-session agent state.

## Tier 2 — Agent identity, communication, testing, packaging

3. **`Agent` declarations** (`spec/agents-declaration.md`) — First-class agent keyword with trait conformance, MCP bindings (`uses`), optional state (`init`), lifecycle hooks (`on` — including `:signal` for reactive interruption). Eliminates dispatch boilerplate. New keyword + AST node.

4. **Enforced `Trait` methods** (`spec/agents-trait.md`) — Trait methods have typed signatures (same `{input} -> output` syntax as MCP declarations). Validated at Agent definition time and spawn time. Absorbs `Skill` declarations — Trait methods ARE skills, with optional description/examples for LLM discovery. `trait.methods`/`trait.match` replace `std/skill`.

5. **`std/pipeline` checkpoint/resume** (`spec/agents-pipeline-checkpoint.md`) — Multi-stage pipelines restart from scratch when a late stage fails. `pipeline.stage` caches completed stage outputs, resumes from last success on re-run. Input hashing for automatic cache invalidation. Also covers the `plan.run_incremental` use case — same mechanism.

6. **`AgentErr` structured errors** (`spec/agents-errors.md`) — Every agent failure is `Err "string"`. Tagged union with 11 variants for pattern-matched recovery.

7. **`lx.toml` package manifest** (`spec/package-manifest.md`) — Project boundary, deps, backend config. Unblocks `std/test` and `std/flow`.

8. **`std/test` satisfaction testing** (`spec/testing-satisfaction.md`) — Spec + scenarios + grader + threshold scoring for non-deterministic agentic flows.

9. **`std/flow` composition** (`spec/flow-composition.md`) — Flows as first-class values: `flow.load`/`flow.run`/`flow.pipe`/`flow.par`.

10. **`agent.pipeline`** (`spec/agents-pipeline.md`) — Consumer-driven flow control with backpressure.

11. **`~>>?` streaming ask** (`spec/agents-streaming.md`) — Stream partial results from long-running agents. Token already lexed (Session 31).

## Tier 3 — Multi-agent infrastructure, adaptive intelligence

12. **`std/trace` extensions** — Provenance (message flow tracking as trace spans: `trace.enable_provenance`, `trace.message_path`, `trace.message_hops`) + reputation (agent scoring from trace data: `trace.agent_score`, `trace.agent_rank`). One observability system instead of three separate modules. Absorbs `spec/agents-provenance.md` and `spec/agents-reputation.md`.

13. **`std/workspace` collaborative editing** (`spec/agents-workspace.md`) — Multiple agents editing the same artifact concurrently with region claiming and conflict resolution.

14. **`std/registry` cross-process discovery** (`spec/agents-discovery.md`) — Discovery by trait/protocol/domain, health checking, load-balanced dispatch.

15. **`agent.dialogue_save/load`** (`spec/agents-dialogue-persist.md`) — Persist dialogue sessions across process restarts.

16. **`with context` ambient propagation** (`spec/agents-ambient.md`) — Scoped ambient state flowing through call chains. Now includes cross-process constraint propagation at `agent.spawn` boundaries (absorbs `spec/agents-constraint-propagation.md`).

17. **`lx install/update`** (`spec/package-manifest.md`) — Dependency resolution and lock file management.

18. **`meta` block** (`spec/agents-meta.md`) — Strategy-level iteration. `refine` iterates within one approach; `meta` tries fundamentally different approaches.

19. **Typed yield variants** (`spec/agents-yield-typed.md`) — Structured orchestrator communication.

## Tier 4 — Remaining

20. **`agent.on` lifecycle hooks** (`spec/agents-lifecycle.md`) — Dynamic hook registration for standalone agents (Agent declarations have `on:` for static hooks). Now includes `:signal` event for reactive interrupt handling.

21. **`std/durable`** (`spec/agents-durable.md`) — Full Temporal-style workflow persistence. When this ships, `std/pipeline` becomes a convenience layer on top.

## Tier 5 — Parser-heavy, speculative

- `|>>` streaming pipe (`spec/concurrency-reactive.md`)
- `caller` implicit binding (`spec/agents-clarify.md`)
- `durable` expression (`spec/agents-durable.md`)
- Deadlock detection (`spec/agents-deadlock.md`)
