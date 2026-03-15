# Stdlib Roadmap

Complete stdlib for the three use cases: agent communication, workflow orchestration, executable plans.

## Existing (12 modules)

| Module | Layer |
|---|---|
| `std/json`, `std/md`, `std/re`, `std/math`, `std/time` | Data |
| `std/fs`, `std/env`, `std/http` | System |
| `std/agent`, `std/mcp`, `std/ai` | Communication |
| `std/ctx`, `std/cron`, `std/tasks`, `std/audit`, `std/circuit`, `std/plan` | Orchestration |
| `std/knowledge`, `std/introspect` | Intelligence |
| — | — |
| **Newly specified (not yet implemented):** | |
| `std/blackboard` | Coordination |
| `std/events` | Coordination |
| `std/diag` | Visualization |

## Planned Modules (deterministic, Rust)

### std/ai

LLM integration. Generic interface, CLI backend (Claude Code CLI). Two functions: `prompt` (simple text → text) and `prompt_with` (full options record → result record with session_id, cost, turns). Supports system prompts, model selection, tool access, structured output via JSON Schema, session resume, budget limits. Backend is `claude -p --output-format json` via `std::process::Command`.

This is the foundation for all standard agents. std/agents/auditor, std/agents/grader, std/agents/router all use std/ai for LLM judgment internally.

### std/tasks

Task state machine with hierarchical subtasks and auto-persist. Status lifecycle: todo → in_progress → submitted → pending_audit → passed/failed → complete. Every multi-step flow needs this.

Design doc: `std_tasks.md`

### std/audit

Structural quality checks that don't need an LLM. Fast pre-filter for the auditor agent. Checks: is_empty, is_too_short, is_repetitive, is_hedging, is_refusal, references_task, files_exist, has_diff.

Design doc: `std_audit.md`

### std/memory

Tiered memory inspired by LSM-trees. L0 episodic (raw transcripts, short retention). L1 working (pattern-action pairs, confidence 0.0-0.7). L2 consolidated (verified patterns, 0.7-0.95). L3 procedural (core rules, always loaded). Promotion on confirmation, demotion on contradiction. Consolidation reviews triggered by std/cron.

### std/trace

Trace collection for observability and fine-tuning. Records input/output/timing/score per agent interaction. Exports datasets for training pipelines. Wraps langfuse MCP or stores locally as JSONL.

### std/circuit

Circuit breakers for agentic loops. Turn counter (hard stop at N turns). Wall-clock timeout. Token budget tracking. Action repetition detection (last N actions compared for similarity). Returns structured reason when breaker fires. Not an agent — a mechanism that agents use.

### std/diag

Program visualization. Two entry points: `lx diagram` CLI subcommand (user-facing) and `std/diag` library (programmatic). Parses lx source, walks the AST to extract a workflow graph (agents as nodes, `~>`/`~>?`/`~>>?` as edges, `par`/`sel` as structural groups, pipes as sequential flow, match arms as decision points). Emits Mermaid flowchart text. No external dependencies — Mermaid text renders in GitHub, VS Code, any markdown viewer. Uses the existing lexer/parser internally. Spec: `spec/stdlib-diag.md`.

### std/knowledge

Shared discovery cache for cross-agent collaboration. File-backed JSON with provenance metadata (source, confidence, tags) and query support. Any agent with the path can read/write. Prevents duplicate tool calls when multiple agents work on the same problem. Functions: `create`, `store`, `get`, `query`, `keys`, `remove`, `merge`, `expire`. File-level locking for concurrent access.

Spec: `spec/stdlib-knowledge.md`

### std/introspect

Agent self-awareness. Runtime metadata about identity, capabilities, budget consumption, action history, and stuck detection. Interpreter collects action log as side effect of evaluation (bounded to last 1000). Functions: `self`, `parent`, `capabilities`, `budget`, `elapsed`, `turn_count`, `actions`, `actions_since`, `mark`, `is_stuck`, `strategy_shift`, `similar_actions`.

Spec: `spec/stdlib-introspect.md`

### std/plan

Dynamic plan execution with revision. Plans are lists of step records with dependencies. `plan.run` executes in topological order, calling `on_step` callback after each step. Callback returns `PlanAction`: `continue`, `replan` (replace remaining steps), `insert_after` (add steps), `skip`, `abort`. Complements `yield` (single-point pause) and `checkpoint`/`rollback` (undo).

Spec: `spec/agents-plans.md`

### std/blackboard

Concurrent shared workspace for multi-agent collaboration within `par` blocks. Unlike `ctx` (single-owner, immutable), a blackboard supports concurrent reads and writes from multiple agents. Last-write-wins conflict resolution. Functions: `create`, `read`, `write`, `watch`, `unwatch`, `keys`, `snapshot`. Backed by a concurrent map (dashmap or similar).

### std/events

Topic-based pub/sub event bus. Decouples producers from consumers. Functions: `create`, `publish`, `subscribe`, `unsubscribe`, `topics`. Handlers invoked synchronously in subscription order. Enables reactive multi-agent systems where agents respond to environmental changes, not just direct messages.

## Planned Standard Agents (lx programs, LLM judgment)

### std/agents/auditor

Quality gate. Takes output + task + context, evaluates whether the response is acceptable. Uses std/audit structural checks as fast pre-filter. Then LLM judgment: does it address the task? does it use context instead of assumptions? is it complete? does it hallucinate?

Design doc: `standard_agents.md`

### std/agents/grader

Rubric scoring. Takes work + task + rubric (list of categories with weights). Scores each category. Supports incremental re-grading (only re-evaluate failed categories on revision). Returns per-category scores, overall score, pass/fail, structured feedback.

Design doc: `standard_agents.md`

### std/agents/router

Prompt classification. Takes a prompt + catalog of specialists. Uses LLM judgment to match prompt to the best domain. Returns domain, agent name, confidence, terminal flag.

Design doc: `standard_agents.md`

### std/agents/planner

Task decomposition. Takes a complex task description and breaks it into ordered subtasks. Each subtask has: title, description, dependencies (which subtasks must complete first), estimated complexity. Creates subtasks in std/tasks. The orchestrating flow executes them in dependency order.

### std/agents/monitor

QC sampling. Watches running subagent transcripts for anomalies. Three detection modes: stuck loops (repeated action sequences), injection (suspicious patterns in tool results), resource abuse (excessive tool calls, reading entire directories). Can interrupt or kill subagents. Runs via std/cron on a sampling interval.

### std/agents/reviewer

Post-hoc transcript review. Reads a completed session transcript. Extracts: patterns that worked (confirmed N times), mistakes to avoid (with lessons), environment facts discovered. Writes to std/memory (L0 → L1 entries). Creates training dataset items via std/trace. The compounding effect: more reviews → richer memory → better future performance.

### std/saga

Multi-agent transactional operations with compensating actions. `saga.run` executes steps in order; on failure, compensating `undo` functions run in reverse for all completed steps. Steps can declare dependencies for concurrent execution. Undo failures are recorded and reported (compensation chain never aborts). `saga.define` creates reusable saga definitions.

Spec: `spec/agents-saga.md`

### Enhanced retry_with

Extend `retry_with` beyond simple count+delay. Add: `retry_if` predicate (only retry on matching errors), `:exponential` backoff strategy, `jitter: true` for randomized delays, `on_exhaust` policy (`:error` or `:fallback`), `fallback` function. Different error types need different strategies — rate limits need backoff, auth errors should fail immediately.

### ai.summarize (extension to std/ai)

Structured context compression for long-running agents. `ai.summarize history {keep: [...] drop: [...] max_tokens: N}`. Understands agent conversation structure — preserves decisions, errors, key findings; drops raw tool output and search results. Combined with `std/introspect.actions`, enables auto-compression of agent history.

## Planned MCP Declarations (typed external service interfaces)

### MCP Embeddings

```
MCP Embeddings = {
  embed { text: Str } -> {embedding: List}
  batch_embed { texts: List } -> {embeddings: List}
  similarity { a: List  b: List } -> {score: Float}
}
```

Wraps any embedding service (local vLLM, OpenAI API, etc.). Used by std/memory for relevance scoring when loading context. Used by std/circuit for action similarity detection. Used by std/agents/monitor for behavioral drift.

### MCP Workflow

```
MCP Workflow = {
  create_task { title: Str  description: Str } -> {id: Str}
  update_task { id: Str  status: Str  notes: Str } -> {ok: Bool}
  list_tasks { status: Str } -> {tasks: List}
  get_task { id: Str } -> Task
}
```

Typed interface to external workflow systems (Linear, Plane, etc.). Distinct from std/tasks which is in-process. This connects to existing project management tools.

## Build Order

The dependency chain determines build order:

1. ~~Type annotations + type checker~~ (DONE — bidirectional inference, unification, `lx check`)
2. ~~Regex literals~~ (DONE — `r/\d+/flags`, first-class Regex values)
3. ~~std/ai~~ (DONE — `ai.prompt` + `ai.prompt_with`, Claude CLI backend)
4. ~~std/tasks~~ (DONE — task state machine, auto-persist, hierarchical subtasks)
5. ~~std/audit~~ (DONE — structural quality checks, rubric evaluate, quick_check)
6. std/agents/auditor (depends on std/audit, std/ai)
7. std/agents/router (depends on std/ai)
8. std/agents/grader (depends on std/tasks, std/ai)
9. std/agents/planner (depends on std/tasks, std/ai)
10. ~~std/circuit~~ (DONE — turn/time/action limits, repetition detection)
11. ~~std/introspect~~ (DONE — identity, elapsed, actions, stuck detection, strategy shift)
12. ~~std/knowledge~~ (DONE — file-backed, provenance, query, merge, expire)
13. ~~std/plan~~ (DONE — dependency-ordered execution, replan/insert_after/skip/abort)
14. std/blackboard (no dependencies, enables parallel agent coordination)
15. std/events (no dependencies, enables reactive agent patterns)
16. std/memory (benefits from MCP Embeddings but works without)
17. std/trace (no dependencies)
18. std/agents/monitor (depends on std/circuit, std/trace)
19. std/agents/reviewer (depends on std/memory, std/trace, std/ai)
20. MCP Embeddings (external service, can be added anytime)
21. MCP Workflow (external service, can be added anytime)
22. std/diag (no dependencies, uses existing lexer/parser, can be added anytime)
23. std/saga (no dependencies, can be added anytime)
24. `|>>` streaming pipe operator (parser + interpreter, depends on lazy sequence infra)
25. `with context` ambient propagation (parser + interpreter extension to `with`)
26. `agent.supervise` + `agent.gate` + `agent.capabilities` (extensions to std/agent)
27. `caller` implicit binding + `_priority` field (interpreter-level)
28. Enhanced `retry_with` (built-in extension)
29. `ai.summarize` (extension to std/ai)

Note: `agent.dialogue`, `agent.intercept`, `agent.handoff`, `agent.supervise`, `agent.gate`, and `agent.capabilities` are extensions to `std/agent`, not separate modules. They're implemented as additional functions in `stdlib/agent.rs`.

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
| multi_agent_coordination | std/blackboard, std/events, `~>>?` streaming, std/knowledge, agent.dialogue |
| safe_delegation | capability attenuation, checkpoint/rollback, agent.handoff, std/saga |
| (any flow) | std/diag (visualize any flow's structure as a diagram) |
| (any multi-step flow) | std/plan (dynamic plan revision), std/introspect (adaptive strategy) |
| (any multi-agent flow) | agent.intercept (tracing/rate-limiting), agent.handoff (context transfer) |
| (any pipeline flow) | `\|>>` (reactive dataflow), `with context` (deadline/budget propagation) |
| (any supervised flow) | agent.supervise (crash recovery), agent.gate (human approval) |
| (any routed flow) | agent.capabilities (dynamic discovery), `_priority` (urgency routing) |
