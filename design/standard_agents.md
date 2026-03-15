# Standard Agents (std/agents/*) — PLANNED, NOT IMPLEMENTED

Standard agents are stdlib. They're imported with `use std/agents/auditor`, spawned, and communicated with via `~>?` like any agent. They ship with the language because an agent language's stdlib includes agents — the same way a general-purpose language's stdlib includes string functions.

## Why Agents in Stdlib

lx has three use cases: agent-to-agent communication, agentic workflow programs, and executable agent plans. Every one of those involves agents talking to other agents. The universal agents — auditor, router, grader — appear in every arch_diagram flow. They're as fundamental as `std/fs` or `std/json`.

The distinction between stdlib modules (Rust functions) and stdlib agents (lx programs) is implementation detail, not a user concept. `use std/agents/auditor` gives you an auditor. Whether it's Rust or lx underneath doesn't matter to the caller.

## std/agents/auditor

Evaluates whether an agent's response is good given the task and available context.

### Protocol

```
Protocol AuditRequest = {
  output: Str
  task: Str
  context: Str
  rubric: List = []
}

Protocol AuditResult = {
  score: Int
  passed: Bool
  categories: List
  feedback: Str
  failed: List
}
```

### What It Checks

**Structural (fast, no LLM — uses std/audit internally):**
- Output is non-empty and above minimum length
- Output doesn't hedge ("I think", "maybe", "possibly")
- Output doesn't refuse ("I can't", "I'm unable")
- Output references key terms from the task
- Referenced file paths exist on disk

**Judgment (LLM reasoning):**
- Output actually addresses the task, not a tangent
- Output uses the provided context, not assumptions
- Output is complete (doesn't stop halfway)
- Output doesn't hallucinate facts, files, or APIs

Structural checks run first as fast pre-filter. If they fail, skips expensive LLM evaluation.

### Usage

```
use std/agents/auditor

a = auditor.spawn ^
result = a ~>? AuditRequest {
  output: agent_response
  task: "fix the auth token refresh bug"
  context: "src/auth/token.rs has refresh() that doesn't handle expiry"
}
result.passed ? {
  true -> tasks.pass store task_id
  false -> worker ~>? {action: "revise" feedback: result.feedback} ^
}
auditor.kill a ^
```

## std/agents/router

Classifies a prompt to a specialist domain using a configurable catalog.

### Protocol

```
Protocol RouteRequest = {
  prompt: Str
  catalog: List
}

Protocol RouteResult = {
  domain: Str
  agent: Str
  confidence: Float
  terminal: Bool
}
```

### How It Routes

Reads the catalog (list of `{name, domain, description, terminal}` records). Uses LLM reasoning to match prompt to best domain. Returns match with confidence. No match above threshold returns `{domain: "none" confidence: 0.0}`.

Router is an agent because domain classification is judgment. "Analyze the rate limiting" could be research, performance, or codebase depending on context.

### Usage

```
use std/agents/router

r = router.spawn ^
route = r ~>? RouteRequest {
  prompt: user_prompt
  catalog: specialist_catalog
}
route.confidence > 0.5 ? {
  true -> dispatch route.agent user_prompt
  false -> handle_directly user_prompt
}
router.kill r ^
```

## std/agents/grader

Scores work against a multi-category rubric with incremental re-grading.

### Protocol

```
Protocol GradeRequest = {
  work: Str
  task: Str
  rubric: List
  previous_grades: List = []
}

Protocol GradeResult = {
  score: Int
  passed: Bool
  categories: List
  feedback: Str
  failed: List
}
```

### Incremental Re-grading

When `previous_grades` is provided, only evaluates categories that previously failed. Passing categories keep their previous scores. Avoids re-evaluating the full rubric on each revision.

### Auditor vs Grader

Auditor: binary quality gate (is this response acceptable?). Grader: quantitative rubric evaluation (score per category, threshold, incremental re-grade). Use auditor for simple pass/fail. Use grader when you need per-category scores and a grading loop.

## Implementation

Standard agents are `.lx` files that ship with the lx binary. `use std/agents/auditor` resolves to the bundled agent definition. The module exports `spawn` and `kill` functions that handle the subprocess lifecycle. The agent itself is an lx program that receives messages via `~>?` and uses `std/audit` for structural checks.

Standard agents use `std/ai` internally for LLM reasoning. The auditor calls `ai.prompt_with` with a system prompt for quality evaluation. The grader calls it with the rubric as structured context. The router calls it with the catalog for classification. `std/ai` is the foundation — standard agents are compositions built on top.

Standard agents use `emit` for progress reporting and status updates. Orchestrators can intercept `emit` output to render progress bars, update UIs, or route to logging systems. Agents should prefer `emit` over `$echo` for all user-facing output — `$echo` is for shell-level side effects, `emit` is for communication.

`agent.spawn {name: "auditor"}` also works as shorthand — the runtime resolves standard agent names to their bundled definitions.

## Negotiation Protocol (Standard Pattern)

Before committing to work, agents can negotiate scope, constraints, and cost. This uses existing Protocol syntax — no new primitives needed.

```
Protocol Offer = {task: Str  constraints: Rec  budget: Int}
Protocol Accept = {commitment: Str  estimated_cost: Int}
Protocol Reject = {reason: Str  counter_offer: Any}
```

The router and planner standard agents will adopt this pattern. An agent that supports negotiation responds to `Offer` messages with either `Accept` or `Reject`. The caller can recursive negotiate on `counter_offer`.

Standard agents that support negotiation: router (can reject if catalog doesn't cover the prompt), planner (can reject if task exceeds budget, counter-offer with reduced scope), grader (can reject if rubric has too many categories for budget).

## Handoff Integration

Standard agents participate in structured handoff chains. The auditor, grader, and router all accept `Handoff` context in their request messages and can produce handoff records in their responses.

When an auditor rejects work, it produces a handoff with `tried` (what checks failed), `recommendations` (what to fix), and `uncertainties` (borderline findings). This handoff feeds directly into the worker's revision cycle.

The planner agent generates plans compatible with `std/plan` — step records with dependencies. The orchestrator executes them via `plan.run` with dynamic revision, so the planner's output is directly executable with mid-flight adjustments.

## Dialogue in Standard Agents

Standard agents support both single-shot `~>?` and multi-turn `agent.dialogue` modes. In dialogue mode, the agent maintains conversation context — useful for iterative grading where the grader explains its reasoning and the worker asks clarifying questions, or for router negotiation where the prompt is refined through conversation.

## Future Standard Agents

As flows mature, more agents become candidates for stdlib:
- **std/agents/monitor** — QC sampling agent that audits running subagents (from scenario_security_audit)
- **std/agents/reviewer** — post-hoc transcript review agent (from scenario_post_hoc_review)
- **std/agents/planner** — task decomposition agent (from flow_full_pipeline)
