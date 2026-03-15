# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude). Updated after Session 30.

## What Works

**Pipes + `^` + `??`** — genuinely excellent error handling for scripting. `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right.

**Agent syntax earns its keep.** `~>` and `~>?` as infix operators compose with everything through normal precedence rules.

**Boundary validation is complete.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool.

**Shell integration is the right model.** `$` has its own lexer mode.

**The stdlib architecture is clean.** One `.rs` file, one match arm, module exists.

**Context threading is solved.** `with` scoped bindings + record field update.

**Type annotations + checker.** `(x: Int y: Str) -> Response ^ HttpErr` on params, return types, and bindings. `lx check` runs bidirectional inference with unification and structural subtyping. `lx run` stays dynamic.

## What's Still Wrong

**Currying** — single biggest source of parser ambiguity. Sections cover 90%. Deferred.

**Concurrency is fake** — `par`/`sel` are sequential. Real async needs `tokio`.

**No LLM integration.** lx has 6 planned standard agents that all say "LLM judgment" — auditor, grader, router. But no module provides LLM access. Shelling out to `claude` or raw `http.post` loses error handling, session continuity, structured output, and budget control. `std/ai` is needed as a Communication-layer module alongside std/agent and std/mcp.

## Gap Analysis

Reviewed `mcp-toolbelt/packages/arch_diagrams` — 14 agentic flow architectures. These are the ACTUAL flows lx was designed to express.

**What lx covers well:** agent spawning + fanout, message validation, MCP tool invocation, context persistence, scheduled execution, executable plans, grading loops, shell integration, end-to-end type safety.

**Critical gaps:** LLM integration, task tracking, quality gates, prompt routing, task decomposition, circuit breakers, tiered memory, observability, subagent QC, learning from experience, embeddings. See `NEXT_PROMPT.md` for the prioritized roadmap.

## Bottom Line

12 stdlib modules. Communication/orchestration layer is solid. Type annotations + checker working. Regex literals implemented. Next: full stdlib buildout — 6 new modules, 6 standard agents, 1 MCP declaration. An agent language's stdlib includes agents.
