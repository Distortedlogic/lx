# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude). Updated after Session 36.

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

**Backend pluggability is real.** `RuntimeCtx` with 6 backend traits (AI, HTTP, shell, emit, yield, log). Every builtin receives `&Arc<RuntimeCtx>`. Standard defaults are production-ready (Claude Code CLI, reqwest, POSIX shell, stdout/stderr). Embedders construct custom `RuntimeCtx` to swap any backend. Testing mock example: `RuntimeCtx { ai: Arc::new(MockAiBackend::new(responses)), ..RuntimeCtx::default() }`.

## What's Still Wrong

**`emit` not yet a keyword** — `RuntimeCtx` refactor is done (backends for AI, HTTP, shell, yield, log behind traits, `EmitBackend` trait exists with `StdoutEmitBackend` default). But `emit` isn't in the AST/parser yet — needs lexer keyword, parser rule, and interpreter eval path. Currently `print` works but isn't the right semantic.

**Currying** — single biggest source of parser ambiguity. Sections cover 90%. Deferred.

**Concurrency is fake** — `par`/`sel` are sequential. Real async needs `tokio`. Streaming (`~>>?`), pub/sub (`std/events`), and reactive dataflow (`|>>`) all depend on this.

**Unicode in lexer** — multi-byte characters (like `->` arrows in comments) cause panics. Byte vs char indexing bug. All flow files have this in their comments.

## Gap Analysis

Reviewed `mcp-toolbelt/packages/arch_diagrams` — 14 agentic flow architectures. Then self-assessed: what do I (Claude) actually struggle with when operating as an agent?

**What lx covers well:** agent spawning + fanout, message validation, MCP tool invocation, context persistence, scheduled execution, executable plans, grading loops, shell integration, end-to-end type safety, plan revision, knowledge sharing, introspection, program visualization.

**What's implemented (28 modules + core):**
- Core agentic: agent spawn/send/ask, Protocol, MCP, yield
- Orchestration: tasks, audit, circuit, plan, knowledge, introspect
- Intelligence: 6 standard agents (auditor, router, grader, planner, monitor, reviewer)
- Infrastructure: memory (tiered L0-L3), trace (JSONL export)
- Visualization: diag (AST -> Mermaid)
- LLM: ai.prompt + ai.prompt_with via Claude CLI

**What's specified but not built (patterns I keep hand-rolling):**

| Feature | Spec | Why I Want It |
|---|---|---|
| Feedback loops (`refine`) | `spec/agents-refine.md` | Try-grade-revise is the #1 pattern in every flow. 15-20 lines of boilerplate each time. Need a first-class `refine` expression that captures the entire loop in 5 lines. |
| Consensus / quorum | `spec/agents-consensus.md` | Multi-reviewer agreement (not just fan-out + collect). Deliberation rounds, weighted voting, configurable quorum policies. Security audits and code review need this. |
| Diminishing returns | `spec/agents-progress.md` | Circuit breakers are walls, stuck detection is binary. Need a gradient — improvement rate over time — so I can stop when effort isn't paying off, not when I hit an arbitrary limit. |
| Result reconciliation | `spec/agents-reconcile.md` | Fan-out gives me N conflicting results. No structured merge: dedup by key, vote, confidence-weighted selection, union with conflict resolution. Hand-rolled every time. |
| Workflow status broadcasting | `spec/agents-broadcast.md` | In `par`, siblings can't see each other. Duplicate work, missed opportunities. Need passive peer visibility. |
| Goal vs task communication | `spec/agents-goals.md` | All messages are flat. No protocol-level distinction between "achieve this outcome" (goal) and "do this action" (task). |
| Deadlock detection | `spec/agents-deadlock.md` | A `~>?` B, B `~>?` A = silent hang. Need runtime wait-for graph with cycle detection. |
| Reactive dataflow (`\|>>`) | `spec/concurrency-reactive.md` | `par` is all-at-once. Need streaming pipelines where results flow as they arrive. |
| Supervision trees | `spec/agents-supervision.md` | Crashed subprocess = manual restart boilerplate. Need auto-restart. |
| Saga pattern | `spec/agents-saga.md` | Multi-agent workflows need distributed compensation on failure. |
| Multi-turn dialogue | `spec/agents-dialogue.md` | Session-based accumulated context for back-and-forth with subagents. |
| Message middleware | `spec/agents-intercept.md` | Tracing, rate-limiting, transformation as composable wrappers. |
| Structured handoff | `spec/agents-handoff.md` | Context transfer between agents without ad-hoc record construction. |

**Planned-feature overlap fixes (Session 35):** `workflow.peers` specified as convenience layer on `std/blackboard` (not a separate DashMap); consensus and reconcile will share vote-tallying logic; progress tracking in introspect will be readable by circuit via pub(crate) accessors.

## Bottom Line

29 stdlib modules (incl. 6 standard agents + visualization + saga). Communication/orchestration layer is solid. Agentic infrastructure (tasks, audit, circuit, plan, knowledge, introspect, memory, trace) is implemented. Type checker working. Standard agents working. Program visualization working. Code architecture is clean with shared utilities. RuntimeCtx backend abstraction in place.

The core language and stdlib are feature-complete for the three use cases. Backend pluggability is solved. What remains is the "efficiency layer" — patterns I keep hand-rolling (`refine`, `reconcile`, `consensus`) and runtime safety (`deadlock detection`, `diminishing returns`, `peer visibility`). These are all specified with full specs in `spec/`. The next implementation pass should focus on the new language features (`refine`, `consensus`, `|>>`) since they require parser+interpreter changes.
