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
| `RuntimeCtx` backend refactor | Infrastructure | `spec/runtime-backends.md` |
| `std/blackboard` | Coordination | — |
| `std/events` | Coordination | — |

## Module Descriptions (Implemented)

### std/ai (DONE)

LLM integration. `prompt` (text -> text) and `prompt_with` (full options -> result record with session_id, cost, turns). Standard backend: Claude Code CLI (`claude -p --output-format json`) — handles auth, model routing, tool permissions, sessions, cost tracking. Will be behind `AiBackend` trait after `RuntimeCtx` refactor. Shared utilities: `ai::parse_llm_json`, `ai::extract_llm_text`, `ai::strip_json_fences`.

### std/tasks (DONE)

Task state machine with hierarchical subtasks and auto-persist. Status lifecycle: todo -> in_progress -> submitted -> pending_audit -> passed/failed -> complete.

### std/audit (DONE)

Structural quality checks. Fast pre-filter for auditor agent. Checks: is_empty, is_hedging, is_refusal, references_task, has_diff. Shared utilities: `build_eval_result`, `make_eval_category`, `keyword_overlap`.

### std/circuit (DONE)

Circuit breakers. Turn counter, wall-clock timeout, action repetition detection. Returns structured reason when breaker fires.

### std/knowledge (DONE)

File-backed shared discovery cache. Provenance metadata. Query with filter functions. Merge, expire.

### std/plan (DONE)

Dynamic plan execution with revision. Topological order. `PlanAction`: continue, replan, insert_after, skip, abort.

### std/introspect (DONE)

Agent self-awareness. Identity, elapsed, turn count, action log (bounded to 1000), markers, stuck detection, strategy shift.

### std/memory (DONE)

Tiered memory (L0-L3). Promotion on confirmation, demotion on contradiction. Confidence scoring.

### std/trace (DONE)

Trace collection. Input/output/timing/score per interaction. JSONL export.

### std/diag (DONE)

Program visualization. AST walker extracts workflow graph (agents as nodes, `~>`/`~>?` as edges, `par`/`sel` as structural groups, match as decision points). Emits Mermaid flowchart. `lx diagram` CLI subcommand + `diag.extract`/`diag.to_mermaid` library API. Graph IR is plain lx records.

### std/saga (DONE)

Multi-agent transactional operations with compensating actions. `saga.run` executes steps in order; on failure, undo functions run in reverse. `saga.run_with` adds options (timeout, max_retries, on_compensate). `saga.define`/`saga.execute` for reusable definitions with initial context. Supports dependency ordering.

## Module Descriptions (Not Yet Implemented)

### std/blackboard

Concurrent shared workspace for multi-agent collaboration within `par` blocks. Last-write-wins. Functions: `create`, `read`, `write`, `watch`, `unwatch`, `keys`, `snapshot`.

### std/events

Topic-based pub/sub event bus. Functions: `create`, `publish`, `subscribe`, `unsubscribe`, `topics`. Synchronous handlers.

### std/saga

Multi-agent transactional operations with compensating actions. `saga.run` executes steps in order; on failure, undo functions run in reverse. Spec: `spec/agents-saga.md`.

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

### `refine` expression (new keyword)

First-class feedback loop: try -> grade -> revise -> re-grade with threshold and max_rounds. Spec: `spec/agents-refine.md`.

### `consensus` expression (new keyword)

Multi-agent voting with quorum policies (`:unanimous`, `:majority`, `:any`, `(n K)`). Optional deliberation. Spec: `spec/agents-consensus.md`.

### `|>>` streaming pipe (new operator)

Reactive dataflow. Items flow downstream as they complete. Lazy until `collect`/`each`. Spec: `spec/concurrency-reactive.md`.

### `with context` ambient propagation

Extends `with`. Deadline, budget, trace ID propagate automatically. Spec: `spec/agents-ambient.md`.

### `caller` implicit binding

Handler-scoped. Agents ask back without going through orchestrator. Spec: `spec/agents-clarify.md`.

### `_priority` message field

4 levels: `:critical`/`:high`/`:normal`/`:low`. Stripped before handler delivery. Spec: `spec/agents-priority.md`.

## Planned Extensions to Existing Modules

| Extension | Target | Spec |
|---|---|---|
| `introspect.progress` / `improvement_rate` / `should_stop` | std/introspect | `spec/agents-progress.md` |
| `agent.reconcile` | std/agent | `spec/agents-reconcile.md` |
| `agent.dialogue` / `agent.dialogue_turn` | std/agent | `spec/agents-dialogue.md` |
| `agent.intercept` | std/agent | `spec/agents-intercept.md` |
| `agent.handoff` / `agent.as_context` | std/agent | `spec/agents-handoff.md` |
| `agent.supervise` | std/agent | `spec/agents-supervision.md` |
| `agent.gate` | std/agent | `spec/agents-gates.md` |
| `agent.capabilities` | std/agent | `spec/agents-capability.md` |
| `workflow.peers` / `workflow.share` | std/agent | `spec/agents-broadcast.md` |
| Goal/Task protocols + `agent.send_goal`/`agent.send_task` | std/agent | `spec/agents-goals.md` |
| Deadlock detection | interpreter | `spec/agents-deadlock.md` |
| `RuntimeCtx` backends | interpreter + all stdlib | `spec/runtime-backends.md` |
| Enhanced `retry_with` | built-in | stdlib roadmap |
| `ai.summarize` | std/ai | stdlib roadmap |

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
