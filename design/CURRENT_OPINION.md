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

**Critical gaps (implementation):** Standard agents (auditor/router/grader/planner — lx programs using std/ai), tiered memory, observability/trace, subagent QC, learning from experience, embeddings. See `NEXT_PROMPT.md` for the prioritized roadmap.

## Bottom Line

19 stdlib modules. Communication/orchestration layer is solid. Agentic infrastructure layer (tasks, audit, circuit, plan, knowledge, introspect) is implemented. Type annotations + checker working. Regex literals implemented. Session 33 addressed the biggest gaps I hit as an agent: reactive dataflow, supervision, ambient context, clarification, gates, capability discovery, sagas, and priority. These are the features that separate "agents can talk" from "agents can reliably collaborate on complex tasks." Next: full stdlib buildout — 13 new modules, 6 standard agents, 1 MCP declaration.
