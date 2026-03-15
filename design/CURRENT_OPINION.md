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

**Concurrency is fake** — `par`/`sel` are sequential. Real async needs `tokio`. Streaming (`~>>?`) and pub/sub (`std/events`) both depend on this.

**LLM integration landed.** `std/ai` provides `ai.prompt` (text → text) and `ai.prompt_with` (full options → result record with session_id, cost, turns). Backend is Claude CLI (`claude -p --output-format json`). Session resume via session_id. Standard agents can now be built on top.

## Gap Analysis

Reviewed `mcp-toolbelt/packages/arch_diagrams` — 14 agentic flow architectures. These are the ACTUAL flows lx was designed to express.

**What lx covers well:** agent spawning + fanout, message validation, MCP tool invocation, context persistence, scheduled execution, executable plans, grading loops, shell integration, end-to-end type safety.

**Newly specified:** Agent streaming (`~>>?`), capability attenuation, checkpoint/rollback, shared blackboard, pub/sub events, negotiation patterns, program visualization (`std/diag`), multi-turn dialogue, message interceptors, structured handoff, dynamic plan revision, agent introspection, shared knowledge cache.

**Newly closed specification gaps:** Multi-turn agent conversation (dialogue sessions), cross-agent knowledge sharing (std/knowledge), plan revision mid-execution (std/plan), agent self-awareness (std/introspect), structured context transfer between agents (handoff), communication middleware (interceptors). These were the missing primitives that standard agents would all benefit from.

**Critical gaps (implementation):** Task tracking, quality gates, prompt routing, task decomposition, circuit breakers, tiered memory, observability, subagent QC, learning from experience, embeddings. See `NEXT_PROMPT.md` for the prioritized roadmap.

## Bottom Line

13 stdlib modules. Communication/orchestration layer is solid, now with LLM integration. Type annotations + checker working. Regex literals implemented. Agentic safety layer specified (capabilities, checkpoint/rollback). Multi-agent coordination specified (blackboard, events, streaming, negotiation, dialogue, interceptors, handoff). Agent intelligence layer specified (introspection, knowledge sharing, dynamic plan revision). Next: full stdlib buildout — 11 new modules, 6 standard agents, 1 MCP declaration. An agent language's stdlib includes agents.
