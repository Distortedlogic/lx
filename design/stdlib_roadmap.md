# Stdlib Roadmap

Complete stdlib for the three use cases: agent communication, workflow orchestration, executable plans.

## Existing (12 modules)

| Module | Layer |
|---|---|
| `std/json`, `std/md`, `std/re`, `std/math`, `std/time` | Data |
| `std/fs`, `std/env`, `std/http` | System |
| `std/agent`, `std/mcp` | Communication |
| `std/ctx`, `std/cron` | Orchestration |

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

1. Type annotations + type checker (foundational — everything benefits)
2. Regex literals (foundational — generation ergonomics)
3. std/ai (foundational — all standard agents depend on this for LLM reasoning)
4. std/tasks (no dependencies, enables grading loops)
5. std/audit (no dependencies, enables auditor agent)
6. std/agents/auditor (depends on std/audit, std/ai)
7. std/agents/router (depends on std/ai)
8. std/agents/grader (depends on std/tasks, std/ai)
9. std/agents/planner (depends on std/tasks, std/ai)
10. std/circuit (no dependencies, enables monitor)
11. std/memory (benefits from MCP Embeddings but works without)
12. std/trace (no dependencies)
13. std/agents/monitor (depends on std/circuit, std/trace)
14. std/agents/reviewer (depends on std/memory, std/trace, std/ai)
15. MCP Embeddings (external service, can be added anytime)
16. MCP Workflow (external service, can be added anytime)

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
| defense_layers | std/agents/monitor, std/circuit, std/trace |
| mcp_tool_audit | std/tasks, std/audit |
