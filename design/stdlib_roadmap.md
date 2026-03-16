# Stdlib Roadmap

Complete stdlib for the three use cases: agent communication, workflow orchestration, executable plans.

## Implemented Modules (29)

| Module | Layer | Status |
|---|---|---|
| `std/json`, `std/md`, `std/re`, `std/math`, `std/time` | Data | DONE |
| `std/fs`, `std/env`, `std/http` | System | DONE |
| `std/agent`, `std/mcp`, `std/ai` | Communication | DONE |
| `std/ctx`, `std/cron`, `std/tasks`, `std/audit`, `std/circuit`, `std/plan` | Orchestration | DONE |
| `std/knowledge`, `std/introspect` | Intelligence | DONE |
| `std/agents/auditor`, `std/agents/router`, `std/agents/grader` | Standard Agents | DONE |
| `std/agents/planner`, `std/agents/monitor`, `std/agents/reviewer` | Standard Agents | DONE |
| `std/memory`, `std/trace` (incl. improvement_rate, should_stop) | Infrastructure | DONE |
| `std/diag` | Visualization | DONE |
| `std/saga` | Transactions | DONE |

## Specified but Not Yet Implemented (Modules)

| Module | Spec | Purpose |
|---|---|---|
| `std/context` | `spec/agents-context-capacity.md` | Context capacity management: tracking, eviction, compaction |
| `std/prompt` | `spec/agents-prompt.md` | Typed composable prompt assembly, budget-aware rendering |
| `std/strategy` | `spec/agents-strategy.md` | Strategy memory: approach outcomes, adaptive selection |
| `std/budget` | `spec/agents-budget.md` | Cost tracking, projection, sub-budgets (absorbs std/circuit) |
| `std/reputation` | `spec/agents-reputation.md` | Agent quality tracking, EWMA scoring |
| `std/skill` | `spec/agents-skill.md` | Runtime registry for Skill declarations |
| `std/durable` | `spec/agents-durable.md` | Workflow persistence, cross-process resumption |
| `std/blackboard` | — | Multi-agent shared state |
| `std/events` | — | In-process event pub/sub |

## Eliminated by Merges (Session 37)

| Former Feature | Absorbed Into |
|---|---|
| `consensus` keyword | `agent.reconcile` `:vote` strategy |
| `speculate` keyword | `agent.reconcile` `:max_score` strategy |
| `agent.escalate` | Fold + handoff pattern |
| `agent.negotiate` | `agent.dialogue` with Proposal/Counter/Contract protocols |
| `std/decide` module | Decision metadata on trace spans |
| `std/causal` module | Parent-child spans in `std/trace` |
| `agent.handoff` function | `Handoff` Protocol + `~>?` |
| `agent.send_goal`/`agent.send_task` | `Goal`/`Task` Protocols + `~>?` |
| 4-level priority | Binary `:critical`/default |
| `std/agent_test` module | `agent.mock` helpers in `std/agent` |

## Mapping to Arch Diagram Flows

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
