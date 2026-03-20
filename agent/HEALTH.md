-- Memory: diagnostic register. Honest assessment of what works and what's broken.
-- Rewrite when the assessment changes. Keep it short and honest.

# Design Health

Updated after Session 82 (2026-03-20).

## What Works

**Pipes + `^` + `??` compose beautifully.** `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right. This composability is the language's strongest design choice.

**Boundary validation covers both directions.** `Trait` validates agent-to-agent. `MCP` declarations validate agent-to-tool. `Trait` declarations validate agent behavioral contracts at definition time. No unvalidated boundary.

**Type hierarchy is clean: Store → Class → Agent.** Store is a first-class `Value::Store { id }` with dot-access methods. Class and Agent both produce `Value::Class { name, traits, defaults, methods }` — Agent is a Trait in `pkg/agent.lx`, not a separate kind. The `Agent` keyword auto-imports the Trait and auto-adds "Agent" to traits list. Display checks traits for "Agent" to distinguish. Trait declarations produce `Value::Trait` with non-empty fields. No separate `Value::Agent` or `Value::Trait` — fewer variants, shared trait injection logic. Object fields live in STORES (same backing as Store values), eliminating the separate OBJECTS DashMap.

**Collection Trait proves the composability thesis.** `pkg/collection.lx` provides 9 methods as Trait defaults delegating to `self.entries`. Any Class with `entries: Store ()` conforming to Collection gets get/keys/values/remove/query/len/has/save/load for free. 5 packages (knowledge, tasks, memory, trace, context) rewritten — domain-only methods remain, generic operations come from Collection.

## What's Still Wrong

**`? { }` is always parsed as match block.** `cond ? { ... }` after `?` starts a match block, not a regular block. Record spreads like `{..a, b: c}` inside `? { ... }` fail with "unexpected DotDot in pattern." Workaround: bind the record first (`result = {..a, b: c}; cond ? result : other`). Also affects reassignment statements inside `? { x <- val }` (parsed as match pattern). Session 80: documented 7 new gotchas from this family of ambiguities (see GOTCHAS.md).

**Record shorthand fields followed by keyed fields misparse.** `{steps  task  step_count: steps | len}` — `steps task` is parsed as function application, not two shorthand fields. Workaround: always use explicit keys when mixing shorthand and keyed fields. `{..spread  shorthand}` has the same issue.

**`lx check` residual errors reduced.** Session 82 fixed infinite type on reassignment (resolve_deep before unify), bound pattern variables in match arm scopes, and now distinguishes parse errors from type errors in workspace check output. All `Expr` variants are explicitly handled (no Unknown fallback). Exhaustiveness, mutable capture, import conflict, and Trait field type checks added. `--strict` mode available for CI.

See `agent/PRIORITIES.md` for the full ordered work queue.

## Bottom Line

Session 82: Completed TYPE_CHECKER_COMPLETION work item (14 tasks). Checker gains: exhaustiveness checking, mutable capture detection, import conflict detection, Trait field type validation, `--strict` mode, pattern variable binding in match arms, parse vs type error separation, all Expr variants explicitly handled. Infinite type on reassignment fixed. Checker split into 7 files (all under 300 lines). 98/98 tests, 0 errors, 0 warnings.
