# lx Development Log

Design decisions, technical debt, and session history. Read `TICK.md` first for orientation.

## Key Design Decisions

Non-obvious choices that cause confusion if forgotten:

- **Function body extent** — `map (x) x * 2 | sum` gives map body `x * 2 | sum`. Use blocks or sections.
- **Tuple auto-spread**: N-param function + single N-tuple → auto-destructure.
- **Application requires callable left-side**. `[1 2 3]` is three elements, not application.
- **Collection-mode**: inside `[]`, only TypeConstructor triggers application.
- **Default params reduce effective arity**. 1 required arg + defaults → executes immediately.
- **Tuple variables need commas**. `(b, a)` tuple. `(b a)` is application.
- **`{x: (f 42)}`** — parens for function calls in single-line record field values (multiline records support full expressions).
- **is_func_def ambiguity**: `(a b c) (expr)` with all bare Ident params NOT a func def. Defaults/underscores/type annotations override.
- **`yield` is callback-based**. No handler → runtime error. Subsumed by `RuntimeCtx.yield_` backend.
- **`RuntimeCtx` for backend pluggability**. All I/O builtins receive `&Arc<RuntimeCtx>`. Traits: `AiBackend`, `EmitBackend`, `HttpBackend`, `ShellBackend`, `YieldBackend`, `LogBackend`, `UserBackend`. Defaults in `backends/defaults.rs` + `backends/user.rs`.
- **`with ... as` uses `stop_ident` parser flag**. Expression parsing stops at `Ident("as")` to prevent it being consumed as application argument. Multi-resource separated by `,` (Semi token). Cleanup via `Closeable` convention (record with `.close` field).
- **`MCP` declarations are typed tool contracts**. Callable → record of wrapper functions.
- **Type annotations don't consume lowercase idents as type args**. `(x: Maybe a)` treats `a` as next param, not type var. Write `(x: (Maybe a))`.
- **`{` after type tokens is body, not record type**. `-> Int { body }` — `{` starts body. Record return types need parens: `-> ({x: Int})`.
- **Protocol `where` constraints bind the field name**. `score: Float where score >= 0.0`.
- **Protocol spread overrides**: later fields override same-named spread fields.
- **`Trait` has typed methods**. `Trait Name = { method: {input} -> output }` — methods use MCP tool signature syntax (`{fields} -> type`). Reserved fields: `description` (Str), `requires` ([symbols]), `tags` ([Str]). Conformance validated at Agent definition time — missing method = hard runtime error. `Value::Trait` holds `methods: Vec<TraitMethodDef>` (same shape as `McpToolDef`). `std/trait` module provides `trait.methods` (extract signatures as records) and `trait.match` (keyword matching).
- **`agent.implements` is structural**. Checks method names against Trait's declared methods. Works with both `Value::Agent` and `Value::Record`. Falls back to `__traits` string tags for empty-method traits. No `?` suffix — parses as ternary operator.
- **`std/profile` uses `DashMap` + atomic IDs**. Same pattern as `std/knowledge`. File-backed at `.lx/profiles/{name}.json`. Strategy helpers use `strategy:{problem}:{approach}` domain prefix convention.
- **`Agent` is a keyword**. Lexed like `Protocol`/`Trait` (uppercase → TypeName special-case). Produces `Stmt::AgentDecl` and `Value::Agent`. Methods stored in `IndexMap<String, Value>`. Reserved fields: `uses`, `init`, `on`. Trait conformance validated at definition time (missing method = hard runtime error, not catchable with `??`).

## Technical Debt

Architectural constraints that inform design decisions. Not actionable bugs (those are in `BUGS.md`).

- `par`/`sel`/`pmap`/`std/pool` are sequential — real async needs tokio (architectural)
- Currying removal deferred — requires parser architecture change
- `it` in `sel` blocks — only implicit binding, no explicit name
- Shell line is single-line only — forces `${ }` for multi-line commands
- Named args + default params + currying have complex interaction edge cases

## Session History

| # | Date | Focus |
|---|------|-------|
| 1-5 | 03-13 | Spec authoring, contradiction fixes, test files |
| 6-7 | 03-13 | First Rust impl — lexer, parser, interpreter, type defs |
| 8-14 | 03-14 | Parser fixes, iterators, shell, concurrency, modules, agents, Protocol |
| 15-18 | 03-14 | Stdlib (json/ctx/math/fs/env/re/md/agent/mcp), MCP HTTP transport |
| 19 | 03-14 | Removed 7 features (regex, $$, <>, sets, iterators, types, tuple semis) |
| 20-24 | 03-14 | yield, MCP decls, std/http, std/time, file splits, with/field update |
| 25 | 03-14 | Stale spec cleanup — all 22 spec files updated |
| 26 | 03-14 | std/cron, real-flow gap analysis vs mcp-toolbelt arch_diagrams |
| 27 | 03-15 | Repo reorg (asl/ -> spec/design/tests/flows), 14 flow programs + specs |
| 28 | 03-15 | Design review: types + regex back, full stdlib roadmap |
| 29 | 03-15 | Type annotations + checker: AST, parser, bidirectional inference, `lx check` |
| 30 | 03-15 | Regex literals: `r/\d+/flags`, Value::Regex, std/re accepts both, 25/25 tests |
| 31 | 03-15 | Agentic features: `~>>?` streaming, checkpoint/rollback, capabilities, blackboard, events, negotiation |
| 32 | 03-15 | Agentic layer completion: dialogue, interceptors, handoff, plan revision, introspection, knowledge cache |
| 33 | 03-15 | std/ai + std/tasks + std/audit + std/circuit + std/knowledge + std/plan + std/introspect. 19 stdlib modules, 32/32 tests |
| 34 | 03-15 | Agent self-assessment: 10 missing features identified. 8 new spec files + updates to 10 existing files |
| 35 | 03-15 | Standard agents (auditor/router/grader/planner/monitor/reviewer) + std/memory + std/trace. 40/40 tests |
| 36 | 03-15 | std/diag: program visualization. AST walker, Mermaid output. `lx diagram` CLI. 41/41 tests |
| 37 | 03-15 | RuntimeCtx design + implementation. Backend traits. Refine expression. agent.reconcile. 44/44 tests |
| 38 | 03-15 | trace.improvement_rate + trace.should_stop: diminishing returns detection. 45/45 tests |
| 39 | 03-15 | Agent communication layer: dialogue, intercept, handoff, capabilities, gate, supervise, mock, dispatch, ai.prompt_structured. 54/54 tests |
| 40 | 03-15 | Protocol extensions: composition, unions, field constraints. 56/56 tests |
| 41 | 03-16 | `with ... as` scoped resources, `Trait` declarations + `agent.implements`, `std/pool`. 59/59 tests |
| 42 | 03-16 | Tier 1 stdlib (`std/budget`, `std/prompt`, `std/context`) + Tier 2 agent extensions (`agent.negotiate`, `agent.topic`/`subscribe`/`publish` pub/sub). 64/64 tests |
| 43 | 03-16 | `std/git`: structured git access — 36 functions (status, log, diff, blame, grep, add, commit, branch, stash, remote). 7 Rust files, unified diff parser. 65/65 tests |
| 44 | 03-16 | Gap analysis: 7 new specs for unplanned features (profile, interrupt, constraint propagation, provenance, workspace, teaching, pipeline checkpoint). Priority queue restructured. `std/blackboard`/`std/events` eliminated |
| 45 | 03-16 | `std/retry`: retry-with-backoff (2 functions, `fastrand` dep for jitter). Improved binding pattern-match error messages. 66/66 tests |
| 46 | 03-16 | Spec consolidation: 9 merges applied. Eliminated `std/strategy` (→ `std/profile`), `std/reputation` (→ `std/trace`), `checkpoint`/`on_interrupt` keywords (→ `user.check` + `:signal` lifecycle hook), `plan.run_incremental` (→ `std/pipeline`), `agent.teach` (→ dialogue convention), `workflow.peers` (→ topic convention), constraint propagation spec (→ `with context` ambient), provenance spec (→ `std/trace`), `Goal`/`Task` (→ docs). Agent/Trait declaration specs from Session 45b integrated. Net: 21 planned features (down from ~33) |
| 47 | 03-16 | Error message overhaul: cross-language keyword hints (30+ keywords), value/type in all mismatch errors, Pattern Display impl, Value::short_display(). `cargo install` to host. Quick syntax reference in NEXT_PROMPT for cold-start agents. 66/66 tests |
| 48 | 03-16 | Gap analysis: 7 unplanned features for dynamic multi-agent coordination. New specs: `agents-task-graph` (DAG execution), `agents-capability-routing` (declarative routing), `agents-deadline` (time propagation), `agents-introspect-live` (system observation), `agents-dialogue-branch` (fork/compare/merge), `agents-format-negotiate` (Protocol adapters), `agents-hot-reload` (handler swap). Updated ROADMAP (28 features), PRIORITIES (28 items), OPINION (8 new gaps). No code changes |
| 49 | 03-16 | `std/user`: 9 functions, `UserBackend` trait. `std/profile`: 15 functions, persistent identity + strategy helpers. `Agent` declarations: new `AgentKw` token, `Stmt::AgentDecl` AST node, `Value::Agent` variant. Parser handles `Agent Name: TraitList = { body }` with `uses`/`init`/`on` reserved fields. Trait conformance validated at definition time. Method access via `.`. 69/69 tests |
| 50+ | 03-17 | Flow testing infrastructure: `std/test` (test runner, test/describe blocks), `std/describe` (BDD-style describe/it with structured results). Flow satisfaction test suites for 14 flow specs — 11 deterministic suites (35 scenarios) + 3 live-only stubs. Discovered and documented 16 findings in FLOW_TESTING_FINDINGS.md. Fixed: `Protocol +Name` syntax, `refine` initial expression parsing, `trace.record` Int score handling. 71/71 tests |
| 51 | 03-17 | Enforced Trait methods: `Trait Name = { method: {input} -> output }` with typed MCP-style signatures. `TraitMethodDecl` AST node, `TraitMethodDef` runtime value. Agent conformance checks method existence at definition time. `agent.implements` now structural (checks methods, not string tags). `std/trait` module: `trait.methods` (extract signatures) + `trait.match` (keyword matching). Protocol-named inputs supported (`method: ProtoName -> output`). Old `handles`/`provides` syntax removed. 71/71 tests |
| 52 | 03-18 | Brain-driven language improvements (10 fixes): `/` returns Float for Int/Int (Python 3), `//` for integer division. Map/Agent field miss → `None` (uniform with Record). Protocol validation → `Err` values (catchable). Record spread allows fn calls (`{..mk () ...}`). Agent `uses`/`on` wired to runtime (`Value::Agent` gains fields). `receive` keyword for agent msg loops. `ai.prompt_json` lightweight structured output. Brain sweep: removed 14 `to_float`, converted 5 agents to `receive`. 71/71 tests |
