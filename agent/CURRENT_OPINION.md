# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude). Updated after Session 40.

## What Works

**Pipes + `^` + `??`** — genuinely excellent error handling for scripting. `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right.

**Agent syntax earns its keep.** `~>`, `~>?`, and `~>>?` as infix operators compose with everything through normal precedence rules.

**Boundary validation is complete.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool.

**Shell integration is the right model.** `$` has its own lexer mode.

**The stdlib architecture is clean.** One `.rs` file, one match arm, module exists. 29 modules in this pattern. Agent extensions use split-file pattern (agent_dialogue.rs, agent_intercept.rs, etc.) registered through agent.rs's build().

**Type annotations + checker.** `lx check` runs bidirectional inference with unification and structural subtyping. `lx run` stays dynamic.

**Program visualization landed.** `std/diag` + `lx diagram` CLI. Graph IR is plain lx records.

**LLM integration is solid.** `std/ai` provides `ai.prompt`, `ai.prompt_with`, and now `ai.prompt_structured` for Protocol-validated output with auto-retry. Backend is Claude CLI. All 6 standard agents use it.

**Backend pluggability is real.** `RuntimeCtx` with 6 backend traits. Every builtin receives `&Arc<RuntimeCtx>`. Embedders swap any backend.

**Agent communication layer is comprehensive.** Beyond basic `~>`/`~>?`, we now have:
- Multi-turn dialogue sessions with context accumulation
- Composable message middleware (intercept)
- Pattern-based deterministic routing (dispatch) complementing LLM routing
- Structured handoff with Markdown formatting for LLM consumption
- Runtime capability discovery
- Human-in-the-loop approval gates
- Erlang-style supervision with lazy restart
- Mock agents with call tracking for testing
- Protocol-validated LLM output with schema injection

**`refine` + `agent.reconcile` are the right abstractions.** `refine` captures the try/grade/revise loop as a single expression. `agent.reconcile` handles the parallel-results-merging problem with 6 strategies + custom.

**Protocol extensions landed.** Composition (`{..Base extra: Str}`), unions (`Protocol Msg = A | B | C` with `_variant` injection), and field constraints (`where` predicates). Protocols can now DRY shared fields, type-safe message dispatch on union variants, and validate values beyond types.

**Testing story is good.** `agent.mock` with call tracking means agent interactions are fully testable without subprocesses. 56 test suites covering every feature.

## What's Still Wrong

**`emit` not yet a keyword** — `EmitBackend` trait exists but `emit` isn't in the AST/parser yet.

**Currying** — single biggest source of parser ambiguity. Sections cover 90%. Deferred.

**Concurrency is fake** — `par`/`sel` are sequential. Real async needs `tokio`.

**Unicode in lexer** — multi-byte characters in comments cause panics. Byte vs char indexing bug.

**Several stdlib files over 300-line limit** — agents_grader.rs (324), audit.rs, diag_walk.rs, tasks.rs, memory.rs, ast.rs. Existing debt.

**No workflow persistence** — `durable` expression not implemented. Workflows can't survive process restarts.

**No streaming** — `|>>` not implemented. All data flows are eager/synchronous.

**No cost tracking** — `std/budget` not implemented. Agents can't track or project resource consumption.

**No agent behavioral contracts** — `agent.capabilities` is runtime discovery, but there's no way to declare "agents of this kind MUST handle these messages." Routers rely on ad-hoc domain tags or LLM guessing. No spawn-time validation.

**No agent pools** — The most common multi-agent pattern (spawn N workers, fan out, collect) requires manual ceremony: tuple destructuring, per-agent lifecycle management, fragile ordering.

**No resource cleanup** — MCP connect/close ceremony appears 14+ times across flows. No `with ... as` scoped blocks. If `^` propagates mid-block, cleanup never runs.

**No multi-agent negotiation** — `agent.reconcile` merges post-hoc. `agent.dialogue` is 2-party. No primitive for N agents to iteratively converge on a shared decision where they see each other's positions.

**No strategy-level iteration** — `refine` iterates on output quality within one approach. No `meta`-level primitive for "this approach isn't viable, try a fundamentally different one." Agents do this manually with imperative loops.

**No dynamic tool creation** — agents can't generate and execute code at runtime without spawning a subprocess. No sandboxed eval for creating data transformers, custom scorers, or ad-hoc tools from LLM output.

**Yield protocol is untyped** — all yields are opaque JSON blobs. Orchestrators must inspect payload content to decide what the agent needs (approval? information? reflection?). No standard Protocols for yield communication.

## What's Next (Opinion)

The most impactful remaining features, ordered by value:

**Agent type system + resource management (highest leverage):**

1. **`with ... as` scoped resources** — auto-cleanup for MCP, agents, handles. Eliminates the connect/close ceremony that appears 14+ times. Small interpreter change.

2. **`Trait` declarations** — agent behavioral contracts. Enables typed routing, spawn-time validation, interchangeable pool workers. The piece that turns ad-hoc multi-agent wiring into architecture.

3. **`std/pool`** — first-class worker groups with fan-out, load balancing, auto-restart. Most common pattern deserves a first-class abstraction.

4. **`agent.negotiate`** — iterative multi-agent consensus. Fills the gap between post-hoc reconciliation and 2-party dialogue.

**Adaptive intelligence (agent self-improvement):**

5. **`meta` block** — strategy-level iteration. Agents try fundamentally different approaches, not just revise output. Composes with `refine` (output quality) and `std/strategy` (cross-session learning).

6. **`agent.eval_sandbox`** — sandboxed dynamic code execution. Agents create tools at runtime from LLM output without subprocess overhead. Permission-restricted.

7. **Typed yield variants** — `std/yield` Protocols for structured orchestrator communication. Orchestrators dispatch on `kind` instead of inspecting arbitrary payloads.

**Stdlib modules (still important):**

5. **`std/budget`** — cost-awareness for real workflows.
6. **`std/prompt`** — typed prompt assembly, budget-aware rendering.
7. **`std/context`** — context capacity management with eviction policies.

Parser-level features (`|>>`, `with context`, `Skill`, `durable`) are individually valuable but each requires parser/interpreter changes — heavier lift per feature.

## Bottom Line

29 stdlib modules. 56/56 tests. Agent communication layer is comprehensive — dialogue, middleware, routing, supervision, mocking, handoff, capabilities, gates, structured LLM output. Protocol system has composition, unions, and field constraints. The core language and stdlib cover the three use cases.

Two frontiers remain: **agent architecture** (Traits, pools, scoped resources, negotiation) turns ad-hoc wiring into type-safe systems. **Adaptive intelligence** (`meta` blocks, sandboxed eval, typed yields) gives agents the ability to change strategy, create tools, and communicate structured needs to orchestrators. Specs: `spec/agents-trait.md`, `spec/agents-pool.md`, `spec/agents-negotiate.md`, `spec/scoped-resources.md`, `spec/agents-meta.md`, `spec/agents-eval-sandbox.md`, `spec/agents-yield-typed.md`.
