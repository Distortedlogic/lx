# Roadmap

Future features only. For what's already implemented, see `agent/INVENTORY.md`.

## Planned Modules (no parser changes needed)

| Module              | Spec                                    | Purpose                                                       | Priority |
| ------------------- | --------------------------------------- | ------------------------------------------------------------- | -------- |
| `std/user`          | `spec/stdlib-user.md`                   | Structured agent-to-user interaction + `user.check` signal poll | Tier 1 |
| `std/profile`       | `spec/agents-profile.md`               | Persistent agent identity + strategy memory (absorbs `std/strategy`) | Tier 1 |
| `std/pipeline`      | `spec/agents-pipeline-checkpoint.md`    | Stage-boundary checkpoint/resume (absorbs `plan.run_incremental`) | Tier 2 |
| `std/test`          | `spec/testing-satisfaction.md`          | Satisfaction-based agentic testing: spec/scenarios/grading     | Tier 2   |
| `std/flow`          | `spec/flow-composition.md`              | Flows as first-class values: load, run, pipe, par, branch     | Tier 2   |
| `std/workspace`     | `spec/agents-workspace.md`             | Concurrent multi-agent editing with region claiming/conflicts  | Tier 3   |
| `std/registry`      | `spec/agents-discovery.md`              | Cross-process agent discovery, health, load-balanced dispatch  | Tier 3   |
| `std/taskgraph`     | `spec/agents-task-graph.md`             | DAG-aware subtask decomposition, dependency-ordered execution  | Tier 2   |
| `std/deadline`      | `spec/agents-deadline.md`               | Time budget propagation across agent boundaries                | Tier 2   |
| `AgentErr`          | `spec/agents-errors.md`                 | Structured agent failure taxonomy for pattern-matched recovery | Tier 2   |
| `trait.methods/match` | `spec/agents-trait.md`               | Trait-based discovery (replaces `std/skill`)                   | Tier 2   |
| `std/durable`       | `spec/agents-durable.md`                | Workflow persistence, cross-process resumption                 | Tier 4   |

## Planned Agent Extensions (no parser changes needed)

| Feature                             | Spec                                    | Purpose                                              | Priority |
| ----------------------------------- | --------------------------------------- | ---------------------------------------------------- | -------- |
| `agent.pipeline`                    | `spec/agents-pipeline.md`               | Consumer-driven flow control with backpressure       | Tier 2   |
| `~>>?` streaming ask                | `spec/agents-streaming.md`              | Stream partial results from long-running agents      | Tier 2   |
| `agent.route`/`agent.register`      | `spec/agents-capability-routing.md`     | Declarative capability-based routing with load awareness | Tier 2 |
| `introspect.system`/`agents`        | `spec/agents-introspect-live.md`        | Live system-wide agent state observation             | Tier 2   |
| `std/trace` provenance + reputation | `spec/agents-provenance.md` (folded)    | Message flow tracking + agent scoring as trace extensions | Tier 3 |
| `agent.dialogue_fork`/`compare`     | `spec/agents-dialogue-branch.md`        | Fork dialogues for parallel exploration, compare, merge | Tier 3 |
| `agent.adapter`/`negotiate_format`  | `spec/agents-format-negotiate.md`       | Runtime Protocol format negotiation between agents   | Tier 3   |
| `agent.reload`/`agent.evolve`       | `spec/agents-hot-reload.md`             | Hot-swap agent handlers without restart              | Tier 3   |
| `agent.dialogue_save/load`          | `spec/agents-dialogue-persist.md`       | Persist dialogue sessions across process restarts    | Tier 3   |
| `agent.on` lifecycle hooks          | `spec/agents-lifecycle.md`              | Dynamic lifecycle hooks including `:signal` event    | Tier 4   |
| Causal spans in `std/trace`         | —                                       | Parent-child span trees, `trace.chain`               | Tier 4   |

## Planned Toolchain Features

| Feature              | Spec                           | Purpose                                           | Priority |
| -------------------- | ------------------------------ | ------------------------------------------------- | -------- |
| `lx.toml` manifest   | `spec/package-manifest.md`     | Package identity, deps, entry, backend config     | Tier 2   |
| `lx init`            | `spec/package-manifest.md`     | Scaffold new project with manifest                | Tier 2   |
| `lx install/update`  | `spec/package-manifest.md`     | Dependency resolution and locking                 | Tier 3   |
| `lx signal`          | `spec/toolchain.md`            | Send interrupt signals to running agents           | Tier 1   |

## Planned Language Changes (parser/interpreter work required)

| Feature                      | Spec                              | Purpose                                         | Priority |
| ---------------------------- | --------------------------------- | ----------------------------------------------- | -------- |
| `Agent` declarations         | `spec/agents-declaration.md`      | First-class agent definitions with trait enforcement | Tier 2 |
| Enforced `Trait` methods     | `spec/agents-trait.md`            | Typed method signatures, definition-time validation | Tier 2 |
| `meta` block                 | `spec/agents-meta.md`             | Strategy-level iteration                        | Tier 3   |
| Typed yield variants         | `spec/agents-yield-typed.md`      | Structured orchestrator communication           | Tier 3   |
| `with context`               | `spec/agents-ambient.md`          | Ambient propagation + cross-spawn constraint inheritance | Tier 3 |
| `\|>>` streaming pipe        | `spec/concurrency-reactive.md`    | Reactive dataflow, lazy until consumed          | Tier 5   |
| `caller` implicit binding    | `spec/agents-clarify.md`          | Agents ask back without orchestrator            | Tier 5   |
| `_priority` field            | `spec/agents-priority.md`         | Binary `:critical` or default                   | Tier 5   |
| `durable` expression         | `spec/agents-durable.md`          | Automatic workflow state persistence            | Tier 5   |
| Deadlock detection           | `spec/agents-deadlock.md`         | Runtime wait-for graph, cycle detection         | Tier 5   |

## Implemented (moved from planned)

| Feature          | Session | Notes                                                    |
| ---------------- | ------- | -------------------------------------------------------- |
| `std/git`        | 43      | 36 functions, 7 Rust files, structured records for all git ops |
| `std/retry`      | 45      | 2 functions, exponential backoff with jitter             |

## Eliminated by Merges

| Former Feature                      | Absorbed Into                                             |
| ----------------------------------- | --------------------------------------------------------- |
| `consensus` keyword                 | `agent.reconcile` `:vote` strategy                        |
| `speculate` keyword                 | `agent.reconcile` `:max_score` strategy                   |
| `agent.escalate`                    | Fold + handoff pattern                                    |
| `agent.negotiate` (2-party)         | `agent.dialogue` with Proposal/Counter/Contract protocols |
| `std/decide` module                 | Decision metadata on trace spans                          |
| `std/causal` module                 | Parent-child spans in `std/trace`                         |
| `agent.handoff` function            | `Handoff` Protocol + `~>?`                                |
| `agent.send_goal`/`agent.send_task` | `Goal`/`Task` Protocols + `~>?`                           |
| 4-level priority                    | Binary `:critical`/default                                |
| `std/agent_test` module             | `agent.mock` helpers in `std/agent`                       |
| `emit` keyword (planned)            | Implemented as AST keyword + `EmitBackend`                |
| `std/blackboard`                    | `std/workspace` (richer: regions, claiming, conflicts)    |
| `std/events`                        | `agent.topic`/`subscribe`/`publish` (already implemented) |
| `Skill` declarations                | Trait methods in `spec/agents-trait.md`                    |
| `std/skill` module                  | `trait.methods`/`trait.match` in `spec/agents-trait.md`   |
| `std/strategy` module               | `std/profile` strategy domain + helper functions          |
| `std/reputation` module             | `std/trace` extensions (`trace.agent_score`/`trace.agent_rank`) |
| `checkpoint` keyword                | `user.check` in `std/user` (non-blocking signal poll)     |
| `on_interrupt` keyword              | `agent.on :signal` / Agent `on: {signal:}` lifecycle hook |
| `plan.run_incremental`              | `std/pipeline` checkpoint/resume (same caching mechanism) |
| `agent.teach` / `agent.on_lesson`   | `Lesson`/`LessonResult` Protocols on `agent.dialogue`     |
| `workflow.peers` / `workflow.share`  | Convention on `agent.topic` (pub/sub pattern)             |
| `Goal`/`Task` Protocols             | Protocol definitions in `std/agent` exports (docs only)   |
| Constraint propagation spec         | `with context` ambient in `spec/agents-ambient.md`        |
| Message provenance spec             | `std/trace` extensions (provenance as trace spans)        |
