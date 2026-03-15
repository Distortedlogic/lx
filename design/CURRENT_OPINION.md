# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude). Updated after Session 32.

## What Works

**Pipes + `^` + `??`** — genuinely excellent error handling for scripting. `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right.

**Agent syntax earns its keep.** `~>`, `~>?`, and `~>>?` as infix operators compose with everything through normal precedence rules. Streaming (`~>>?`) fills the gap where request-response wasn't enough.

**Boundary validation is complete.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool.

**Shell integration is the right model.** `$` has its own lexer mode.

**The stdlib architecture is clean.** One `.rs` file, one match arm, module exists.

**Context threading is solved.** `with` scoped bindings + record field update. `std/blackboard` solves the multi-agent shared state problem.

**Type annotations + checker.** `(x: Int y: Str) -> Response ^ HttpErr` on params, return types, and bindings. `lx check` runs bidirectional inference with unification and structural subtyping. `lx run` stays dynamic.

**Safety model is specified.** Capability attenuation on `agent.spawn`, `checkpoint`/`rollback` for transactional execution. Agents can be sandboxed without external tooling.

**Agent-to-human output has its own primitive.** `emit` replaces `$echo` for user-facing output. Dedicated AST node, callback-based interception, structured values, Protocol validation. Matches the principle that fundamental operations get first-class syntax — `$` for shell, `~>` for agents, `emit` for humans.

## What's Still Wrong

**Currying** — single biggest source of parser ambiguity. Sections cover 90%. Deferred.

**Concurrency is fake** — `par`/`sel` are sequential. Real async needs `tokio`. Streaming (`~>>?`), pub/sub (`std/events`), and reactive dataflow (`|>>`) all depend on this.

**LLM integration landed.** `std/ai` provides `ai.prompt` (text → text) and `ai.prompt_with` (full options → result record with session_id, cost, turns). Backend is Claude CLI (`claude -p --output-format json`). Session resume via session_id. Standard agents can now be built on top.

## Gap Analysis

Reviewed `mcp-toolbelt/packages/arch_diagrams` — 14 agentic flow architectures. Then self-assessed: what do I (Claude) actually struggle with when operating as an agent?

**What lx covers well:** agent spawning + fanout, message validation, MCP tool invocation, context persistence, scheduled execution, executable plans, grading loops, shell integration, end-to-end type safety, multi-turn dialogue, message interceptors, structured handoff, plan revision, knowledge sharing, introspection.

**Session 33 — Newly specified (agent self-assessment):**

| Feature | Spec | Why I Want It |
|---|---|---|
| Reactive dataflow (`\|>>`) | `spec/concurrency-reactive.md` | `par` is all-at-once, sequential is one-at-a-time. Real research is: search for X, result triggers Y and Z, Z triggers W. Need streaming pipelines where results flow as they arrive. |
| Supervision trees | `spec/agents-supervision.md` | Crashed subprocess = manual restart boilerplate everywhere. Need Erlang-style one-for-one/one-for-all/rest-for-one auto-restart. |
| Ambient context | `spec/agents-ambient.md` | Deadline, budget, trace ID must be threaded manually through every function. Need Go-style context that propagates automatically through agent operations. |
| Structured clarification | `spec/agents-clarify.md` | `yield` goes to orchestrator. Agent B needs to ask agent A "did you mean X?" without going through the top. Need `caller` implicit binding. |
| Approval gates | `spec/agents-gates.md` | `yield` is too generic for human approval. Need structured gate with timeout policy, escalation, audit trail. |
| Capability advertisement | `spec/agents-capability.md` | Can't query agents at runtime for what they can do. Need `Capabilities` protocol + `agent.capabilities` helper. |
| Saga pattern | `spec/agents-saga.md` | `checkpoint`/`rollback` is single-agent. Multi-agent workflows need distributed compensation on failure. |
| Message priority | `spec/agents-priority.md` | All messages are FIFO. Cancel signals sit behind 50 status updates. Need `_priority` field with 4 levels. |
| Context compression | stdlib roadmap | Long-running agents fill context. Need `ai.summarize` for structured history compression. |
| Enhanced retry | stdlib roadmap | `retry_with` needs: per-error-type strategy, jitter, circuit breaker integration. |

**Filled (Session 35):** Standard agents (auditor/router/grader/planner/monitor/reviewer), tiered memory (`std/memory`), observability (`std/trace`), subagent QC (`std/agents/monitor`), learning from experience (`std/agents/reviewer`). **Remaining:** MCP Embeddings, `std/diag` visualization, `std/saga` transactions.

**Session 36 — Second self-assessment (what I keep working around):**

| Feature | Spec | Why I Want It |
|---|---|---|
| Feedback loops (`refine`) | `spec/agents-refine.md` | Try-grade-revise is the #1 pattern in every flow. 15-20 lines of boilerplate each time. Need a first-class `refine` expression that captures the entire loop in 5 lines. |
| Consensus / quorum | `spec/agents-consensus.md` | Multi-reviewer agreement (not just fan-out + collect). Deliberation rounds, weighted voting, configurable quorum policies. Security audits and code review need this. |
| Diminishing returns | `spec/agents-progress.md` | Circuit breakers are walls, stuck detection is binary. Need a gradient — improvement rate over time — so I can stop when effort isn't paying off, not when I hit an arbitrary limit. |
| Result reconciliation | `spec/agents-reconcile.md` | Fan-out gives me N conflicting results. No structured merge: dedup by key, vote, confidence-weighted selection, union with conflict resolution. Hand-rolled every time. |
| Workflow status broadcasting | `spec/agents-broadcast.md` | In `par`, siblings can't see each other. Duplicate work, missed opportunities. Need passive peer visibility — not opt-in pub/sub, automatic status sharing. |
| Goal vs task communication | `spec/agents-goals.md` | All messages are flat. No protocol-level distinction between "achieve this outcome" (goal — agent plans how) and "do this action" (task — execute directly). |
| Deadlock detection | `spec/agents-deadlock.md` | A `~>?` B, B `~>?` A = silent hang. With `caller` for clarification, this gets more likely. Need runtime wait-for graph with cycle detection. |

**Session 36 — Deduplication audit:** Found and fixed 5 code-level overlaps:
1. LLM response parsing (text extraction + fence stripping + JSON parse) duplicated in 5 agent modules → extracted `ai::parse_llm_json` + `ai::extract_llm_text` + `ai::strip_json_fences`
2. Eval result record builders (`build_result`, `make_category`) identical in auditor + grader → extracted `audit::build_eval_result` + `audit::make_eval_category`
3. Keyword overlap logic duplicated in 4 places → extracted `audit::keyword_overlap` + `audit::check_references_task`
4. Auditor reimplemented audit checks → now calls `audit::check_empty`, `audit::check_refusal`, `audit::check_hedging`, `audit::check_references_task`
5. Circuit and introspect both track actions/turns independently — kept separate because circuit is per-breaker-instance scoped while introspect is global. Unification deferred to when real async lands and we can share a per-agent (not global) state. Spec notes added.

Also fixed 3 planned-feature overlaps at spec level: `workflow.peers` now specified as convenience layer on `std/blackboard` (not a separate DashMap); consensus and reconcile will share vote-tallying logic; progress tracking in introspect will be readable by circuit via pub(crate) accessors.

## Bottom Line

27 stdlib modules (incl. 6 standard agents). Communication/orchestration layer is solid. Agentic infrastructure layer (tasks, audit, circuit, plan, knowledge, introspect) is implemented. Type annotations + checker working. Session 33 addressed "agents can reliably collaborate." Session 36 addresses "agents can collaborate *efficiently*" — the patterns I keep hand-rolling (refine, reconcile, consensus) and the runtime safety I keep wishing for (deadlock detection, diminishing returns, peer visibility). Code architecture cleaned: shared LLM parsing, shared eval records, shared keyword matching, auditor delegates to audit. Next: full stdlib buildout + the 7 new features.
