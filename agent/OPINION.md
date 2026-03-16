# Design Opinion

Written by the language designer (Claude). Updated after Session 49 (2026-03-16).

## What Works

**Pipes + `^` + `??` compose beautifully.** `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right. This composability is the language's strongest design choice.

**Agent operators as infix.** `~>` and `~>?` slot into normal precedence. No special syntax mode for agent communication — it's just expressions.

**Boundary validation covers both directions.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool. `Trait` declarations validate agent behavioral contracts at definition time. No unvalidated boundary.

**`with ... as` eliminates ceremony.** MCP connect/close, agent spawn/kill, file open/close — all reduced to scoped blocks with guaranteed cleanup. Error propagation doesn't leak resources.

**`refine` + `agent.reconcile` capture real patterns.** `refine` = try/grade/revise loop as one expression. `agent.reconcile` = parallel-results-merging with 6 strategies. `std/pool` = fan-out/collect without manual lifecycle management. `std/budget` = gradient-based cost awareness with projection and sub-budgets.

**`std/git` eliminates text parsing for coding agents.** 36 functions returning structured records — status, log, diff, blame, grep, commit, branch, stash, remote. Every coding agent lives in git; now they get records instead of parsing `--porcelain` output. Unified diff parser produces hunk records with per-line attribution.

**`RuntimeCtx` backend trait design pays off.** All I/O builtins receive `&Arc<RuntimeCtx>`. Embedders swap any backend. Testing, server deployment, sandboxing all work through the same mechanism.

**Error messages are self-teaching.** Writing `if x then y` produces `undefined variable 'if' — lx uses 'cond ? then_expr : else_expr'`. Every type mismatch shows the actual value and type received. Agents learn lx syntax from the errors themselves.

## What's Still Wrong

Tech debt (currying, unicode, 300-line files, fake concurrency) tracked in `agent/DEVLOG.md`. These are the design-level gaps:

**All agent errors are strings** — `Err "some string"` for every failure mode. Need `AgentErr` tagged union for pattern-matched recovery. Spec: `spec/agents-errors.md`.

**No project identity or packaging** — No manifest, no way to name a project, declare dependencies, or configure backends. Spec: `spec/package-manifest.md`.

**No way to test agentic flows** — `assert` is binary pass/fail, useless for non-deterministic agent output. Need satisfaction-based testing. Spec: `spec/testing-satisfaction.md`.

**Flows aren't composable** — Flows (entire .lx programs) require manual `agent.spawn` + `~>?` + `agent.kill`. Need flows as first-class values. Spec: `spec/flow-composition.md`.

**Multi-stage pipelines can't resume from failure** — If stage 4 fails after stages 0-3 succeeded, restart from scratch. Spec: `spec/agents-pipeline-checkpoint.md`.

**No concurrent multi-agent editing** — No shared artifact multiple agents can concurrently modify with region claiming and conflict resolution. Spec: `spec/agents-workspace.md`.

**Cross-process agent discovery doesn't exist** — `agent.advertise`/`agent.capabilities` are in-process only. Need registry with health checking and load-balanced dispatch. Spec: `spec/agents-discovery.md`.

**Dialogue sessions are ephemeral** — Process death = lost conversation. Spec: `spec/agents-dialogue-persist.md`.

**`~>>?` streaming is unimplemented** — Token exists (Session 31) but no interpreter support. Spec: `spec/agents-streaming.md`.

**No strategy-level iteration** — No `meta`-level primitive for "try a fundamentally different approach." Spec: `spec/agents-meta.md`.

**Yield protocol is untyped** — All yields are opaque JSON blobs. Spec: `spec/agents-yield-typed.md`.

**No DAG-aware task decomposition** — `std/plan` is linear, `std/pool` is homogeneous fan-out, `agent.reconcile` is post-hoc. "Task C depends on A and B" requires manual `~>?` sequencing and result threading. Every non-trivial multi-agent flow reinvents DAG scheduling. Spec: `spec/agents-task-graph.md`.

**No capability-based routing** — `agent.capabilities`/`advertise` exist, `agent.dispatch` routes by message shape, but there's no "send to whatever handles Trait X with lowest load." Every flow hardcodes agent references. Spec: `spec/agents-capability-routing.md`.

**Dialogue sessions can't branch** — `agent.dialogue` is linear. Tree-of-thought / best-of-N requires manually creating separate sessions, duplicating context, then reconciling. No fork/compare/merge at the dialogue level. Spec: `spec/agents-dialogue-branch.md`.

**No time budget propagation** — `std/budget` tracks cost, `timeout` wraps expressions, but spawned sub-agents don't know the parent's remaining time. Agents start expensive work that gets killed by parent timeout. Spec: `spec/agents-deadline.md`.

**No live system-wide agent observation** — `std/introspect` is self-only, `std/trace` is historical. No "what are all agents doing right now?" — no structured view of agent states, in-flight messages, bottlenecks across the whole system. Spec: `spec/agents-introspect-live.md`.

**Agents can't update their own behavior** — Handler is fixed at spawn. Learning via `refine`/`std/profile` stores knowledge but can't change the handler function. Kill-and-respawn loses all in-process state. Spec: `spec/agents-hot-reload.md`.

**No Protocol format negotiation** — Agents with structurally-compatible but differently-named Protocols can't interoperate without manual interceptor boilerplate. Spec: `spec/agents-format-negotiate.md`.

## Bottom Line

The core agent architecture is solid — Agent declarations, Traits, pools, scoped resources, Protocols, reconciliation, supervision, negotiation, pub/sub, retry, user interaction, persistent profiles all work. Cost tracking (budget), prompt composition (prompt), and context management (context) are all in place.

The remaining work is:

1. **Agent contracts** — Enforced `Trait` methods with typed signatures (absorbing Skills). Trait conformance is checked at Agent definition time but method signatures aren't validated yet.
2. **Dynamic multi-agent coordination** — `std/taskgraph` (DAG execution), `agent.route`/`register` (capability routing), `std/deadline` (time propagation), `introspect.system` (live observation). These eliminate the manual wiring boilerplate that every non-trivial multi-agent flow reinvents.
3. **Ecosystem infrastructure** — `AgentErr`, `lx.toml`, `std/test`, `std/flow`, `std/pipeline` checkpoint/resume, `agent.pipeline` backpressure, `~>>?` streaming.
4. **Adaptive intelligence + distribution** — `agent.reload`/`evolve` (hot handler swap), `agent.dialogue_fork`/`compare` (branching exploration), `agent.adapter`/`negotiate_format` (Protocol interop), `std/trace` extensions (provenance + reputation), `std/registry`, dialogue persistence, `with context` ambient propagation, `meta` block, typed yields.
