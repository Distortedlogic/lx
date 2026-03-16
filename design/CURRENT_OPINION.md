# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude). Updated after Session 38.

## What Works

**Pipes + `^` + `??`** — genuinely excellent error handling for scripting. `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right.

**Agent syntax earns its keep.** `~>`, `~>?`, and `~>>?` as infix operators compose with everything through normal precedence rules.

**Boundary validation is complete.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool.

**Shell integration is the right model.** `$` has its own lexer mode.

**The stdlib architecture is clean.** One `.rs` file, one match arm, module exists. 29 modules in this pattern. Larger modules split into multiple Rust files (e.g., trace → trace.rs + trace_query.rs + trace_progress.rs) while keeping one `build()` entry point.

**Type annotations + checker.** `lx check` runs bidirectional inference with unification and structural subtyping. `lx run` stays dynamic.

**Program visualization landed.** `std/diag` + `lx diagram` CLI. Graph IR is plain lx records.

**LLM integration is solid.** `std/ai` provides `ai.prompt` and `ai.prompt_with`. Backend is Claude CLI. All 6 standard agents use it.

**Backend pluggability is real.** `RuntimeCtx` with 6 backend traits. Every builtin receives `&Arc<RuntimeCtx>`. Embedders swap any backend.

**Diminishing returns detection works.** `trace.improvement_rate` and `trace.should_stop` give agents gradient-based stopping signals — distinct from circuit breakers (hard limits) and stuck detection (binary). Agents can now detect plateaus and shift strategies adaptively.

**`refine` + `agent.reconcile` are the right abstractions.** `refine` captures the try/grade/revise loop as a single expression. `agent.reconcile` handles the parallel-results-merging problem with 6 strategies + custom. These are the patterns that agents actually need.

## What's Still Wrong

**`emit` not yet a keyword** — `EmitBackend` trait exists but `emit` isn't in the AST/parser yet.

**Currying** — single biggest source of parser ambiguity. Sections cover 90%. Deferred.

**Concurrency is fake** — `par`/`sel` are sequential. Real async needs `tokio`.

**Unicode in lexer** — multi-byte characters in comments cause panics. Byte vs char indexing bug.

**Several stdlib files over 300-line limit** — audit.rs (350), diag_walk.rs (351), tasks.rs (379), memory.rs (417), ast.rs (386). Existing debt.

## Bottom Line

29 stdlib modules. 45/45 tests. Communication/orchestration layer is solid. Type checker working. Standard agents working. Program visualization working. RuntimeCtx backend abstraction in place. Progress tracking (improvement_rate/should_stop) in place.

The core language and stdlib are feature-complete for the three use cases. Remaining work is tracked in `NEXT_PROMPT.md` (priorities 23-47) and `stdlib_roadmap.md` (planned modules/extensions). Next up: agent dialogue, interceptors, handoff — the multi-turn session layer.
