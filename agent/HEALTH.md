-- Memory: diagnostic register. Honest assessment of what works and what's broken.
-- Rewrite when the assessment changes. Keep it short and honest.

# Design Health

Updated after Session 66 (2026-03-19).

## What Works

**Pipes + `^` + `??` compose beautifully.** `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right. This composability is the language's strongest design choice.

**Boundary validation covers both directions.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool. `Trait` declarations validate agent behavioral contracts at definition time. No unvalidated boundary.

**Type hierarchy is clean: Store → Class → Agent.** Store is a first-class `Value::Store { id }` with dot-access methods. Class and Agent both produce `Value::Class { name, traits, defaults, methods }` — Agent is a Trait in `pkg/agent.lx`, not a separate kind. The `Agent` keyword auto-imports the Trait and auto-adds "Agent" to traits list. Display checks traits for "Agent" to distinguish. Protocol declarations produce `Value::Trait` with non-empty fields. No separate `Value::Agent` or `Value::Protocol` — fewer variants, shared trait injection logic. Object fields live in STORES (same backing as Store values), eliminating the separate OBJECTS DashMap.

**Collection Trait proves the composability thesis.** `pkg/collection.lx` provides 9 methods as Trait defaults delegating to `self.entries`. Any Class with `entries: Store ()` conforming to Collection gets get/keys/values/remove/query/len/has/save/load for free. 5 packages (knowledge, tasks, memory, trace, context) rewritten — domain-only methods remain, generic operations come from Collection.

## What's Still Wrong

**`lx check` is noisy on files with imports.** The type checker doesn't resolve `use` statements — it only sees the parsed AST of a single file. Any file that imports and uses external names produces false "undefined variable" diagnostics. `lx check` on the workspace reports 122 errors, almost all false positives from unresolved imports. Single-file `lx check tests/01_literals.lx` (no imports) works correctly. Fix requires the checker to either resolve imports or suppress diagnostics for imported names.

See `agent/PRIORITIES.md` for the full ordered work queue.

## What's Still Wrong (continued)

**Export names shadow builtins in lx packages.** `+filter` as an export name shadows the builtin `filter` HOF inside the module. Discovered when converting trace — had to rename `trace.filter` → `trace.query`. Any lx package exporting a name that matches a builtin (filter, map, fold, etc.) will hit this. Not a parser bug — it's environment scoping. Workaround: avoid builtin names in exports, or capture the builtin before the export (`keep = filter`).

**`? { }` is always parsed as match block.** `cond ? { ... }` after `?` starts a match block, not a regular block. Record spreads like `{..a, b: c}` inside `? { ... }` fail with "unexpected DotDot in pattern." Workaround: use parens `? ({..a, b: c})` or extract to a function.

## Bottom Line

Session 66: `agent.pipeline` ships consumer-driven flow control with backpressure — 11 functions connecting agents into processing pipelines with bounded buffers, overflow policies, pressure monitoring, and round-robin worker scaling. Tail-first pump algorithm ensures downstream stages drain first, creating natural backpressure. Synchronous implementation (real concurrency needs tokio). The agent system now supports both ad-hoc messaging (`~>?`, pub/sub, routing) and structured pipeline processing. 82/82 tests pass.
