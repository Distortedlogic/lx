# Roadmap

Future stdlib modules, extensions, and language features. For what's already implemented, see `doc/`.

## Planned Modules

| Module | Spec | Purpose |
|---|---|---|
| `std/budget` | `spec/agents-budget.md` | Cost tracking, projection, sub-budgets (absorbs std/circuit) |
| `std/reputation` | `spec/agents-reputation.md` | Agent quality tracking, EWMA scoring |
| `std/context` | `spec/agents-context-capacity.md` | Context capacity management: tracking, eviction, compaction |
| `std/prompt` | `spec/agents-prompt.md` | Typed composable prompt assembly, budget-aware rendering |
| `std/strategy` | `spec/agents-strategy.md` | Strategy memory: approach outcomes, adaptive selection |
| `std/skill` | `spec/agents-skill.md` | Runtime registry for Skill declarations |
| `std/durable` | `spec/agents-durable.md` | Workflow persistence, cross-process resumption |
| `std/pool` | `spec/agents-pool.md` | Agent worker pools: fan_out, map, load balancing |
| `std/blackboard` | — | Multi-agent shared state |
| `std/events` | — | In-process event pub/sub |

## Planned Extensions

| Feature | Spec | Purpose |
|---|---|---|
| `plan.run_incremental` | `spec/agents-incremental.md` | Memoized plan execution with input-hash invalidation |
| `agent.negotiate` | `spec/agents-negotiate.md` | Iterative multi-agent consensus |
| `agent.topic` / `agent.subscribe` | `spec/agents-pubsub.md` | Agent-level pub/sub for broadcast |
| `agent.pipeline` | `spec/agents-pipeline.md` | Consumer-driven flow control with backpressure |
| `agent.on` | `spec/agents-lifecycle.md` | Internal agent lifecycle hooks |
| `workflow.peers` / `workflow.share` | `spec/agents-broadcast.md` | Passive sibling visibility in `par` |
| `Goal`/`Task` Protocols | `spec/agents-goals.md` | Convention Protocols, no wrapper functions |
| Causal spans in `std/trace` | — | Parent-child span trees, `trace.chain` |

## Planned Language Changes

| Feature | Spec | Purpose |
|---|---|---|
| `Trait` declarations | `spec/agents-trait.md` | Behavioral contracts: `handles` + `provides` |
| `with ... as` scoped resources | `spec/scoped-resources.md` | Auto-cleanup on scope exit |
| `\|>>` streaming pipe | `spec/concurrency-reactive.md` | Reactive dataflow, lazy until consumed |
| `with context` | `spec/agents-ambient.md` | Ambient deadline/budget propagation |
| `caller` implicit binding | `spec/agents-clarify.md` | Agents ask back without orchestrator |
| `_priority` field | `spec/agents-priority.md` | Binary `:critical` or default |
| `Skill` declarations | `spec/agents-skill.md` | Self-describing capability units with typed I/O |
| `durable` expression | `spec/agents-durable.md` | Automatic workflow state persistence |
| Deadlock detection | `spec/agents-deadlock.md` | Runtime wait-for graph, cycle detection |
| `emit` keyword | `spec/agents-advanced.md` | Agent-to-human fire-and-forget output |

## Eliminated by Merges

| Former Feature | Absorbed Into |
|---|---|
| `consensus` keyword | `agent.reconcile` `:vote` strategy |
| `speculate` keyword | `agent.reconcile` `:max_score` strategy |
| `agent.escalate` | Fold + handoff pattern |
| `agent.negotiate` (2-party) | `agent.dialogue` with Proposal/Counter/Contract protocols |
| `std/decide` module | Decision metadata on trace spans |
| `std/causal` module | Parent-child spans in `std/trace` |
| `agent.handoff` function | `Handoff` Protocol + `~>?` |
| `agent.send_goal`/`agent.send_task` | `Goal`/`Task` Protocols + `~>?` |
| 4-level priority | Binary `:critical`/default |
| `std/agent_test` module | `agent.mock` helpers in `std/agent` |

## Flow → Module Mapping

| Flow | Uses |
|---|---|
| agentic_loop | std/ai, std/circuit, std/tasks, std/agents/auditor |
| agent_lifecycle | std/ai, std/memory, std/agents/reviewer, std/cron |
| subagent_lifecycle | std/ai, std/agents/router, std/circuit |
| subagent_fine_tuning | std/ai, std/trace, MCP Embeddings |
| flow_full_pipeline | std/ai, std/tasks, std/agents/grader, std/agents/planner, std/agents/monitor |
| scenario_security_audit | std/agents/monitor, std/circuit |
| scenario_research | std/ai, std/agents/router, std/tasks |
| scenario_perf_analysis | std/ai, std/agents/router, std/tasks |
| scenario_project_setup | std/tasks, MCP Workflow |
| scenario_post_hoc_review | std/ai, std/agents/reviewer, std/memory, std/trace |
| discovery_system | std/ai, std/tasks, std/trace, MCP Embeddings |
| tool_generation | std/ai, std/tasks, std/agents/auditor |
| defense_layers | std/agents/monitor, std/circuit, std/trace, capability attenuation |
| mcp_tool_audit | std/tasks, std/audit |
| (any flow) | std/diag (visualize any flow's structure) |
