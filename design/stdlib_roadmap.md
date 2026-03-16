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
| `std/memory`, `std/trace` | Infrastructure | DONE |
| `std/diag` | Visualization | DONE |
| `std/saga` | Transactions | DONE |

## Specified but Not Yet Implemented

| Module / Feature | Layer | Spec |
|---|---|---|
| `std/blackboard` | Coordination | — |
| `std/events` | Coordination | — |

## Module Descriptions (Implemented)

### std/ai (DONE)

LLM integration. `prompt` (text -> text) and `prompt_with` (full options -> result record with session_id, cost, turns). Standard backend: Claude Code CLI. Shared utilities: `ai::parse_llm_json`, `ai::extract_llm_text`, `ai::strip_json_fences`.

### std/tasks (DONE)

Task state machine with hierarchical subtasks and auto-persist. Status lifecycle: todo -> in_progress -> submitted -> pending_audit -> passed/failed -> complete.

### std/audit (DONE)

Structural quality checks. Fast pre-filter for auditor agent. Shared utilities: `build_eval_result`, `make_eval_category`, `keyword_overlap`.

### std/circuit (DONE — will be absorbed by std/budget)

Circuit breakers. Turn counter, wall-clock timeout, action repetition detection. Returns structured reason when breaker fires. When `std/budget` is implemented, circuit's functionality folds into budget (multi-dimension limits with gradient status instead of binary tripped/not-tripped). Repetition detection stays in introspect.

### std/knowledge (DONE)

File-backed shared discovery cache. Provenance metadata. Query with filter functions. Merge, expire.

### std/plan (DONE)

Dynamic plan execution with revision. Topological order. `PlanAction`: continue, replan, insert_after, skip, abort.

### std/introspect (DONE)

Agent self-awareness. Identity, elapsed, turn count, action log (bounded to 1000), markers, stuck detection, strategy shift.

### std/memory (DONE)

Tiered memory (L0-L3). Promotion on confirmation, demotion on contradiction. Confidence scoring.

### std/trace (DONE)

Trace collection. Input/output/timing/score per interaction. JSONL export. Will be extended with parent-child spans (causal chains) and progress query utilities.

### std/diag (DONE)

Program visualization. AST walker extracts workflow graph. Mermaid flowchart. `lx diagram` CLI subcommand + `diag.extract`/`diag.to_mermaid` library API.

### std/saga (DONE)

Multi-agent transactional operations with compensating actions. `saga.run`, `saga.run_with`, `saga.define`/`saga.execute`. Supports dependency ordering.

## Standard Agents (All DONE)

| Agent | Purpose |
|---|---|
| `std/agents/auditor` | Quality gate. Uses std/audit structural checks + std/ai LLM judgment |
| `std/agents/router` | Prompt classification. LLM matches prompt to specialist domain |
| `std/agents/grader` | Rubric scoring. Incremental re-grade on revision |
| `std/agents/planner` | Task decomposition into ordered subtasks |
| `std/agents/monitor` | QC sampling. Stuck loops, injection, resource abuse detection |
| `std/agents/reviewer` | Post-hoc transcript review. Learning extraction to memory+trace |

## Planned Language Features (Not Yet Implemented)

### `|>>` streaming pipe (new operator)

Reactive dataflow. Items flow downstream as they complete. Lazy until `collect`/`each`. Spec: `spec/concurrency-reactive.md`.

### `with context` ambient propagation

Extends `with`. Deadline, budget, trace ID propagate automatically. Spec: `spec/agents-ambient.md`.

### `caller` implicit binding

Handler-scoped. Agents ask back without going through orchestrator. Spec: `spec/agents-clarify.md`.

### `_priority` message field (binary)

`:critical` or default. Stripped before handler delivery. `agent.check_critical` polls for pending critical messages. Spec: `spec/agents-priority.md`.

### `Skill` declarations (new keyword)

Self-describing, discoverable capability units with typed I/O. Completes the trinity: Protocol (messages), MCP (external tools), Skill (internal capabilities). `std/skill` provides registry, discovery, matching, composition. Spec: `spec/agents-skill.md`.

### `durable` expression (new keyword)

Automatic workflow state persistence at suspension points. Cross-process resumption via `lx resume <id>`. `DurableBackend` trait on RuntimeCtx. Default: filesystem JSON. Spec: `spec/agents-durable.md`.

## Planned New Modules (Not Yet Implemented)

### std/budget

Cumulative cost/resource accounting. Track spend across token, API call, and wall-time dimensions. Gradient status (comfortable/tight/critical/exceeded). Sub-budgets. Absorbs `std/circuit` functionality (hard limits are just exceeded budget status). Spec: `spec/agents-budget.md`.

### std/reputation

Cross-interaction agent quality tracking. EWMA scoring per (agent, task_type) pair. Spec: `spec/agents-reputation.md`.

### std/skill

Runtime registry and discovery for Skill declarations. Functions: `registry`, `list`, `match`, `match_semantic`, `run`, `get`, `compose`. Spec: `spec/agents-skill.md`.

### std/durable

Workflow persistence management. Functions: `status`, `resume`, `cancel`, `list`, `cleanup`. Spec: `spec/agents-durable.md`.

## Planned Extensions to Existing Modules

| Extension | Target | Spec |
|---|---|---|
| `trace.improvement_rate` / `trace.should_stop` | std/trace | `spec/agents-progress.md` |
| Parent-child spans + `trace.chain` | std/trace | (causal chain queries) |
| `agent.reconcile` (vote/quorum/deliberation/max_score/early_stop) | std/agent | `spec/agents-reconcile.md` |
| `agent.dialogue` / `agent.dialogue_turn` | std/agent | `spec/agents-dialogue.md` |
| `agent.intercept` | std/agent | `spec/agents-intercept.md` |
| `Handoff` Protocol + `agent.as_context` | std/agent | `spec/agents-handoff.md` |
| `agent.supervise` | std/agent | `spec/agents-supervision.md` |
| `agent.gate` | std/agent | `spec/agents-gates.md` |
| `agent.capabilities` | std/agent | `spec/agents-capability.md` |
| `workflow.peers` / `workflow.share` | std/agent | `spec/agents-broadcast.md` |
| `Goal`/`Task` Protocols | std/agent | `spec/agents-goals.md` |
| `agent.mock` + call tracking | std/agent | `spec/agents-test-harness.md` |
| `agent.dispatch` / `agent.dispatch_multi` | std/agent | `spec/agents-dispatch.md` |
| Deadlock detection | interpreter | `spec/agents-deadlock.md` |
| `RuntimeCtx` backends | interpreter + all stdlib | DONE |
| `ai.prompt_structured` / `ai.prompt_structured_with` | std/ai | `spec/agents-structured-output.md` |
| `plan.run_incremental` | std/plan | `spec/agents-incremental.md` |

## Eliminated by Merges (Session 37)

| Former Feature | Absorbed Into | Rationale |
|---|---|---|
| `consensus` keyword | `agent.reconcile` `:vote` strategy | Voting is a reconciliation strategy, not a separate keyword |
| `speculate` keyword | `agent.reconcile` `:max_score` strategy | Best-of-N is a reconciliation strategy |
| `agent.escalate` | Fold + handoff pattern | Expressible as composition, not a primitive |
| `agent.negotiate` | `agent.dialogue` with Protocols | Negotiation is multi-turn dialogue with Proposal/Counter/Contract protocols |
| `std/decide` module | Decision metadata on trace spans | Decisions are trace spans with structured metadata |
| `std/causal` module | Parent-child spans in `std/trace` | Causal spans are trace spans with `parent_id` |
| `agent.handoff` function | `Handoff` Protocol + `~>?` | Just a Protocol convention, not a function |
| `agent.send_goal`/`agent.send_task` | `Goal`/`Task` Protocols + `~>?` | Just Protocol definitions, not wrapper functions |
| 4-level priority | Binary `:critical`/default | Only critical (stop signals) needs priority |
| `std/agent_test` module | `agent.mock` helpers in `std/agent` | Scenarios are regular test code; only mock_agent needs stdlib support |

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
| (any flow) | std/diag (visualize any flow's structure as a diagram) |
