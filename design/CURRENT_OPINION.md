# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude). Updated after Session 37.

## What Works

**Pipes + `^` + `??`** — genuinely excellent error handling for scripting. `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right.

**Agent syntax earns its keep.** `~>`, `~>?`, and `~>>?` as infix operators compose with everything through normal precedence rules. Streaming (`~>>?`) fills the gap where request-response wasn't enough.

**Boundary validation is complete.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool.

**Shell integration is the right model.** `$` has its own lexer mode.

**The stdlib architecture is clean.** One `.rs` file, one match arm, module exists. 28 modules in this pattern.

**Context threading is solved.** `with` scoped bindings + record field update. `std/blackboard` (specified) solves the multi-agent shared state problem.

**Type annotations + checker.** `(x: Int y: Str) -> Response ^ HttpErr` on params, return types, and bindings. `lx check` runs bidirectional inference with unification and structural subtyping. `lx run` stays dynamic.

**Safety model is specified.** Capability attenuation on `agent.spawn`, `checkpoint`/`rollback` for transactional execution. Agents can be sandboxed without external tooling.

**Program visualization landed.** `std/diag` + `lx diagram` CLI. AST walker extracts agents, messages, control flow, decisions. Graph IR is plain lx records — agents can inspect and transform before rendering. Mermaid output renders everywhere.

**LLM integration is solid.** `std/ai` provides `ai.prompt` (text -> text) and `ai.prompt_with` (full options -> result record with session_id, cost, turns). Backend is Claude CLI (`claude -p --output-format json`). All 6 standard agents use it. Shared parsing utilities prevent duplication.

**Code architecture is clean.** Shared LLM parsing (`ai::parse_llm_json`, `ai::extract_llm_text`, `ai::strip_json_fences`). Shared eval records (`audit::build_eval_result`, `audit::make_eval_category`). Shared keyword matching (`audit::keyword_overlap`). Auditor delegates to audit module for structural checks.

**Backend pluggability is real.** `RuntimeCtx` with 6 backend traits (AI, HTTP, shell, emit, yield, log). Every builtin receives `&Arc<RuntimeCtx>`. Standard defaults are production-ready (Claude Code CLI, reqwest, POSIX shell, stdout/stderr). Embedders construct custom `RuntimeCtx` to swap any backend.

## What's Still Wrong

**`emit` not yet a keyword** — `EmitBackend` trait exists with `StdoutEmitBackend` default, but `emit` isn't in the AST/parser yet.

**Currying** — single biggest source of parser ambiguity. Sections cover 90%. Deferred.

**Concurrency is fake** — `par`/`sel` are sequential. Real async needs `tokio`.

**Unicode in lexer** — multi-byte characters in comments cause panics. Byte vs char indexing bug.

## Gap Analysis

Reviewed `mcp-toolbelt/packages/arch_diagrams` — 14 agentic flow architectures. Then self-assessed: what do I (Claude) actually struggle with when operating as an agent?

**What lx covers well:** agent spawning + fanout, message validation, MCP tool invocation, context persistence, scheduled execution, executable plans, grading loops, shell integration, end-to-end type safety, plan revision, knowledge sharing, introspection, program visualization.

**What's specified but not built (patterns I keep hand-rolling):**

| Feature | Spec | Why I Want It |
|---|---|---|
| Feedback loops (`refine`) | DONE | First-class `refine` expression implemented. |
| Result reconciliation + voting + best-of-N | `spec/agents-reconcile.md` | Single `agent.reconcile` with strategies: `:vote` (quorum/deliberation), `:max_score` (early_stop), `:union`, `:intersection`. Subsumes former consensus and speculate keywords. |
| Diminishing returns | `spec/agents-progress.md` | `trace.improvement_rate`/`trace.should_stop` — gradient progress via scored trace spans. |
| Workflow status broadcasting | `spec/agents-broadcast.md` | Convenience layer over `std/blackboard`. |
| Goal vs task communication | `spec/agents-goals.md` | `Goal`/`Task` Protocol definitions — convention, no wrapper functions. |
| Deadlock detection | `spec/agents-deadlock.md` | Runtime wait-for graph with cycle detection. |
| Reactive dataflow (`\|>>`) | `spec/concurrency-reactive.md` | Streaming pipelines where results flow as they arrive. |
| Supervision trees | `spec/agents-supervision.md` | Auto-restart on crash. |
| Multi-turn dialogue | `spec/agents-dialogue.md` | Session-based context accumulation. Subsumes negotiation pattern. |
| Message middleware | `spec/agents-intercept.md` | Composable wrappers for tracing, rate-limiting, transformation. |
| Handoff convention | `spec/agents-handoff.md` | `Handoff` Protocol + `agent.as_context` helper (not a function). |

**What's specified but not built (intelligence layer):**

| Feature | Spec | Why I Want It |
|---|---|---|
| Structured AI output (`ai.prompt_structured`) | `spec/agents-structured-output.md` | Protocol-validated LLM output with auto-retry. |
| Skill declarations (`Skill` keyword) | `spec/agents-skill.md` | Self-describing internal capabilities for LLM discovery. |
| Cost budgeting (`std/budget`) | `spec/agents-budget.md` | Gradient resource tracking. Absorbs `std/circuit` on implementation. |
| Agent reputation (`std/reputation`) | `spec/agents-reputation.md` | Cross-interaction quality tracking, learning router feedback. |
| Incremental plans (`plan.run_incremental`) | `spec/agents-incremental.md` | Memoized execution with input-hash cache invalidation. |

**What's specified but not built (operational layer):**

| Feature | Spec | Why I Want It |
|---|---|---|
| Durable execution (`durable`) | `spec/agents-durable.md` | Long-running workflows survive process death. The single biggest gap. |
| Content-addressed dispatch | `spec/agents-dispatch.md` | Deterministic pattern-based routing without LLM calls. |
| Mock agents for testing | `spec/agents-test-harness.md` | `agent.mock` + call tracking. Test scenarios are regular lx code. |
| Causal chain queries | Extension to `std/trace` | Parent-child spans, `trace.chain` for failure lineage. |

**Merges applied (Session 37):** Eliminated 6 redundant features. `consensus` keyword → `:vote` strategy in `agent.reconcile`. `speculate` keyword → `:max_score` strategy in `agent.reconcile`. `agent.escalate` → fold + handoff pattern (documented, not primitive). `agent.negotiate` → dialogue with Proposal/Contract protocols. `std/decide` → decision metadata on trace spans. `std/causal` → parent-child spans in `std/trace`. Also simplified: 4-level priority → binary `:critical`/default. `agent.handoff` function → `Handoff` Protocol convention. `agent.send_goal`/`agent.send_task` → just Protocol definitions. `std/agent_test` module → `agent.mock` helpers in `std/agent`.

## Bottom Line

29 stdlib modules (incl. 6 standard agents + visualization + saga). Communication/orchestration layer is solid. Type checker working. Standard agents working. Program visualization working. RuntimeCtx backend abstraction in place.

The core language and stdlib are feature-complete for the three use cases. What remains:

1. **Efficiency layer** — `reconcile` (subsumes consensus + speculate + best-of-N), diminishing returns, peer visibility, deadlock detection.

2. **Intelligence layer** — structured AI output, skills, reputation, budgets (absorbing circuit), incremental plans.

3. **Operational layer** — durable execution (biggest architectural gap), content-addressed dispatch, mock agents for testing, causal chain queries in trace.

Next implementation: `Skill` + `ai.prompt_structured` (highest daily impact). Then `durable` (biggest gap) and `agent.reconcile` + `agent.dispatch` (most common hand-rolled patterns).
